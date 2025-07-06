use game_mon::app::Gui;
use iced::settings::Settings;
use iced::window::settings::Settings as Win_Settings;
use game_mon::config::{GAMEMON_LOGO, check_for_updates, CURRENT_VERSION};
mod logger;


pub fn main() -> iced::Result {

    logger::Logger::init_with_target("GameMon-service").expect("Failed to initialize logger");

    match check_for_updates("".to_string()) {
        Ok(_) => log::info!("Check for updates complete!"),
        Err(e) => log::error!("Error checking for updates: {:?}\n", e),
    }

    // Start the GUI application
    let gamemon_icon = iced::window::icon::from_file(GAMEMON_LOGO.as_path()).unwrap();
  
    log::info!("DEBUG: Icon at {:?}", gamemon_icon);

    let window_title = format!("GameMon v{}", CURRENT_VERSION.to_string());
    let window_title: &'static str = Box::leak(window_title.into_boxed_str());

    iced::application(window_title, Gui::update, Gui::view).theme(Gui::theme)
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