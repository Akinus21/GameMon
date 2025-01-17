use gtk4 as gtk;
use gtk::prelude::*;
use gtk::Application;
use crate::gui::Gui;
use tray_icon::{TrayIconEvent, menu::{MenuEvent}};
use crate::tray::Tray;

mod gui;
mod config;
mod tray;
mod util;

// fn main() -> glib::ExitCode {
//     let gui = Gui::new();
//     gui.show_gui();  // Show the GTK window with the configuration
//     glib::ExitCode::SUCCESS
// }

fn main() {
    Tray::new().spawn();  // Spawn the system tray icon

    if let Ok(event) = TrayIconEvent::receiver().try_recv() {
        println!("tray event: {:?}", event);
    }
    
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        println!("menu event: {:?}", event);
    }
}