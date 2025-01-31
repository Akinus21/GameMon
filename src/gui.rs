use GameMon::app::Gui;
use iced::settings::Settings;
use iced::window::settings::Settings as Win_Settings;
use GameMon::config::{GAMEMON_ICON, GAMEMON_UPDATER, GAMEMON_CONFIG_FILE, GAMEMON_LOGO, check_for_updates};


pub fn main() -> iced::Result {

    match check_for_updates() {
        Ok(_) => println!("Check for updates complete!"),
        Err(e) => eprintln!("Error checking for updates: {:?}\n", e),
    }

    // Start the GUI application
    let gamemon_icon = iced::window::icon::from_file(GAMEMON_LOGO.as_path()).unwrap();
  
    println!("DEBUG: Icon at {:?}", gamemon_icon);

    iced::application("GameMon", Gui::update, Gui::view).theme(Gui::theme)
        .settings(Settings {
            id: Some("GameMon".to_string()),
            ..Default::default()
        })
        .window(Win_Settings{
            icon: Some(gamemon_icon),
            ..Default::default()
        })
        .run()
}