// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Builder, Manager};
use tauri::{SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem};
mod gui;
mod config;

use gui::Gui;

fn main() {
    let gui = Gui::new(); // Initialize your Gui struct

    // Define tray menu items
    let show_item = CustomMenuItem::new("show".to_string(), "Show");
    let exit_item = CustomMenuItem::new("exit".to_string(), "Exit");

    // Create tray menu
    let tray_menu = SystemTrayMenu::new()
        .add_item(show_item)
        .add_item(exit_item);

    // Create the system tray and bind event handling
    let system_tray = SystemTray::new().with_menu(tray_menu);

    // Handle tray events
    Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(move |app, event| {
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "show" => {
                            println!("Show the GUI");
                            gui.show_gui();  // Notify the app to show the GUI
                            app.get_window("main").unwrap().show().unwrap();  // Show the main window (GUI)
                        }
                        "exit" => {
                            std::process::exit(0);  // Exit the application
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
