use std::{sync::mpsc::Sender, thread, time::Duration};
use tray_icon::{menu::{Menu, MenuEvent, MenuItem}, TrayIconBuilder, TrayIconEvent};
use crate::util::CustomIcon;

pub struct Tray {
    sender: Sender<String>, // Channel to send menu events
}

impl Tray {
    pub fn new(sender: Sender<String>) -> Self {
        Tray { sender }
    }

    pub fn spawn(&self) {
        // Initialize the GTK library
        gtk::init().unwrap();

        // Create the tray icon
        let icon = CustomIcon::new("resources/tray_icon-green.png").get_icon();

        // Create the tray menu items
        let show_gui_item = MenuItem::new("Show GUI", true, None);
        let exit_item = MenuItem::new("Exit", true, None);

        // Create the tray menu
        let tray_menu = Menu::new();
        tray_menu.append(&show_gui_item).unwrap();
        tray_menu.append(&exit_item).unwrap();

        // Build the tray app
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("GameMon Tray")
            .with_icon(icon)
            .build()
            .unwrap();

        let sender_clone = self.sender.clone();

        loop {
            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                println!("tray event triggered: {:?}", event);
            };
            
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                println!("menu event triggered: {:?}", event);
                if let Err(e) = sender_clone.send(format!("{:?}", event)) {
                    eprintln!("Failed to send event: {:?}", e);
                }
            };

            thread::sleep(Duration::from_secs(1));

        }
    }

}
