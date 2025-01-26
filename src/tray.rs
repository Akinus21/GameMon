use libappindicator::AppIndicator;
use libappindicator::AppIndicatorStatus;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, *};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};

pub fn spawn_tray_linux(
    sender: mpsc::Sender<String>,
    title: String,
    icon_path: PathBuf,
    menu: Vec<(String, String)>, // Change &str to String for owned data
) {
    let application = gtk::Application::new(
        Some("com.example.trayapp"),
        gtk::gio::ApplicationFlags::FLAGS_NONE,
    );

    // Wrap the sender in an Arc<Mutex<>> to make it thread-safe and shareable
    let sender = Arc::new(Mutex::new(sender));

    application.connect_activate(move |_| {
        // Build tray application
        let mut indicator = AppIndicator::new("Example", "applications-internet");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_title(&title);
        let icon = icon_path.to_str().unwrap();
        println!("DEBUG: Icon at {:?}", icon);
        indicator.set_icon(icon);

        // Build menu
        let mut new_menu = gtk::Menu::new();
        for item in menu.clone() { // Cloning the owned `menu` vector
            let mi = gtk::MenuItem::with_label(&item.0);
            // Clone the Arc<Mutex<Sender>> for the closure
            let sender_clone = Arc::clone(&sender);
            mi.connect_activate(move |_| {
                if let Ok(sender) = sender_clone.lock() {
                    sender
                        .send(item.1.clone()) // Clone the String to send it
                        .expect("Failed to send message");
                }
            });
            new_menu.append(&mi);
        }

        new_menu.show_all();
        indicator.set_menu(&mut new_menu);
    });

    application.run();
}
