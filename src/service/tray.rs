use libappindicator::AppIndicator;
use libappindicator::AppIndicatorStatus;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, *};
use std::env;
use std::sync::{mpsc, Arc, Mutex};

pub fn spawn_tray(sender: mpsc::Sender<String>) {
    let application = gtk::Application::new(
        Some("com.example.trayapp"),
        gtk::gio::ApplicationFlags::FLAGS_NONE,
    );

    // Wrap the sender in an Arc<Mutex<>> to make it thread-safe and shareable
    let sender = Arc::new(Mutex::new(sender));

    application.connect_activate(move |_| {
        let mut indicator = AppIndicator::new("GameMon", "applications-internet");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_title("GameMon - Gaming Monitor");
        let icon_path = env::current_dir().unwrap().join("resources/gamemon.png");
        let icon = icon_path.to_str().unwrap();
        println!("DEBUG: Icon at {:?}", &icon);
        indicator.set_icon(icon);

        let mut menu = gtk::Menu::new();
        let show_gui = gtk::MenuItem::with_label("Show Config GUI");

        // Clone the Arc<Mutex<Sender>> for the closure
        let sender_clone = Arc::clone(&sender);
        show_gui.connect_activate(move |_| {
            if let Ok(sender) = sender_clone.lock() {
                sender.send("show_gui".to_string()).expect("Failed to send show_gui message");
            }
        });
        menu.append(&show_gui);

        let quit_item = gtk::MenuItem::with_label("Quit");

        // Clone the Arc<Mutex<Sender>> for the closure
        let sender_clone = Arc::clone(&sender);
        quit_item.connect_activate(move |_| {
            if let Ok(sender) = sender_clone.lock() {
                sender.send("quit".to_string()).expect("Failed to send quit message");
            }
        });
        menu.append(&quit_item);

        menu.show_all();
        indicator.set_menu(&mut menu);
    });

    application.run();
}
