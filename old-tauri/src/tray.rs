use tauri::{SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem};
use crate::gui::Gui;

pub struct Tray {
    pub tray: SystemTray,
    gui: Gui,  // Add the Gui instance here
}

impl Tray {
    pub fn new(gui: Gui) -> Self {
        let show_item = CustomMenuItem::new("show".to_string(), "Show");
        let exit_item = CustomMenuItem::new("exit".to_string(), "Exit");

        let tray_menu = SystemTrayMenu::new()
            .add_item(show_item)
            .add_item(exit_item);

        let tray = SystemTray::new().with_menu(tray_menu);

        Self { tray, gui }  // Store Gui instance
    }

    pub fn handle_event(&mut self, event: SystemTrayEvent) {
        match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "show" => {
                        println!("Show the GUI");

                        // Notify the app to show the GUI
                        self.gui.show_gui();  // You need to implement show_gui method
                    }
                    "exit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
