use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};
use crate::util::CustomIcon;

pub struct Tray;

impl Tray {
    pub fn new() -> Self {
        Tray
    }

    pub fn spawn(&self) {
        
        // Create the tray icon
        let icon = CustomIcon::new("resources/tray_icon-green.png").get_icon();

        // Create the tray menu items
        let mut custom_menu_items = Vec::new();
        custom_menu_items.push(MenuItem::new(
            "Show GUI",
            true,
            None
        ));
        custom_menu_items.push(MenuItem::new(
            "Exit",
            true,
            None
        ));

        let custom_menu_items_refs: Vec<&dyn tray_icon::menu::IsMenuItem> = custom_menu_items
                                                                            .iter()
                                                                            .map(|item| item as &dyn tray_icon::menu::IsMenuItem)
                                                                            .collect();

        // Create the tray menu
        let tray_menu = Menu::new();
        if let Err(e) = tray_menu.append_items(&custom_menu_items_refs) {
            eprintln!("Failed to append items to tray menu: {}", e);
        }

        let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("system-tray - tray icon library!")
        .with_icon(icon)
        .build()
        .unwrap();  
        
    }
}
