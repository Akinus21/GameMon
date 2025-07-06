use libappindicator::AppIndicator;
use libappindicator::AppIndicatorStatus;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, *};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use crate::config::{Config, GAMEMON_CONFIG_FILE};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use glib::ControlFlow;
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(windows)]
use tray_item::TrayItem;

use crate::config::CURRENT_VERSION;

#[cfg(unix)]
pub fn spawn_tray(
    sender: Sender<String>,
    title: String,
    icon_path: PathBuf,
    menu_var: Vec<(String, String)>,
) {
    let application = gtk::Application::new(
        Some("com.example.trayapp"),
        gtk::gio::ApplicationFlags::FLAGS_NONE,
    );

    let sender = Arc::new(Mutex::new(sender));
    let menu_var = menu_var.clone();
    let icon_path_str = icon_path.to_str().expect("Invalid icon path").to_string();
    let title_clone = title.clone();

    let (glib_tx, glib_rx) = glib::MainContext::channel(glib::Priority::default());
    let glib_rx = Rc::new(RefCell::new(Some(glib_rx)));

    let config_path = GAMEMON_CONFIG_FILE.clone();
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
            .expect("Failed to create watcher");
        watcher
            .watch(config_path.as_path(), RecursiveMode::NonRecursive)
            .expect("Failed to watch config file");

        while let Ok(event) = rx.recv() {
            if let Ok(ev) = event {
                if matches!(ev.kind, EventKind::Modify(_)) {
                    let _ = glib_tx.send(());
                }
            }
        }
    });

    application.connect_activate(move |app| {
        let mut indicator = AppIndicator::new("gamemon-tray", "applications-internet");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_title(&title_clone);
        indicator.set_icon(&icon_path_str);

        let menu = Rc::new(RefCell::new(gtk::Menu::new()));
        let menu_for_build = menu.clone();
        let sender_for_build = sender.clone();
        let menu_var = menu_var.clone();

        let build_menu = move || {
            let menu = menu_for_build.clone();
            let sender = sender_for_build.clone();
            let mut menu_ref = menu.borrow_mut();
            let children: Vec<_> = menu_ref.children().iter().cloned().collect();
            for child in children {
                menu_ref.remove(&child);
            }

            let app_name_item = gtk::MenuItem::with_label(&format!("GameMon v{}", CURRENT_VERSION.to_string()));
            app_name_item.set_sensitive(false);
            menu_ref.append(&app_name_item);
            menu_ref.append(&gtk::SeparatorMenuItem::new());

            for (label, command) in &menu_var {
                let mi = gtk::MenuItem::with_label(label);
                let command = command.clone();
                let sender = sender.clone();
                mi.connect_activate(move |_| {
                    if let Ok(tx) = sender.lock() {
                        let _ = tx.send(command.clone());
                    }
                });
                menu_ref.append(&mi);
            }

            menu_ref.append(&gtk::SeparatorMenuItem::new());

            let bolos_item = gtk::MenuItem::with_label("BOLOs");
            let bolos_menu = gtk::Menu::new();

            if let Ok(config) = Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()) {
                for entry in config.entries {
                    let game_name = entry.game_name.clone();
                    let game_name_start = game_name.clone();
                    let game_name_end = game_name.clone();

                    let item = gtk::MenuItem::with_label(&game_name);
                    let sub = gtk::Menu::new();

                    let sender_start = sender.clone();
                    let sender_end = sender.clone();

                    let start = gtk::MenuItem::with_label("Run Start Commands");
                    let end = gtk::MenuItem::with_label("Run End Commands");

                    start.connect_activate(move |_| {
                        if let Ok(tx) = sender_start.lock() {
                            let _ = tx.send(format!("start:{}", game_name_start));
                        }
                    });

                    end.connect_activate(move |_| {
                        if let Ok(tx) = sender_end.lock() {
                            let _ = tx.send(format!("end:{}", game_name_end));
                        }
                    });

                    sub.append(&start);
                    sub.append(&end);
                    sub.show_all();

                    item.set_submenu(Some(&sub));
                    bolos_menu.append(&item);
                }
            }

            bolos_item.set_submenu(Some(&bolos_menu));
            bolos_menu.show_all();
            menu_ref.append(&bolos_item);

            menu_ref.show_all();
        };

        build_menu();
        indicator.set_menu(&mut menu.borrow_mut());

        let build_menu_clone = build_menu.clone();
        if let Some(glib_rx_real) = glib_rx.borrow_mut().take() {
            glib_rx_real.attach(None, move |_| {
                log::info!("Tray: Config file changed, rebuilding menu...");
                build_menu_clone();
                ControlFlow::Continue
            });
        }

        app.add_window(&gtk::Window::new(gtk::WindowType::Toplevel));
    });

    application.run();
}
