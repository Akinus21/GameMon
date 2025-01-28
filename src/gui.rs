use std::env;
use GameMon::app::Gui;
use iced::settings::Settings;
use iced::window::settings::Settings as Win_Settings;


pub fn main() -> iced::Result {

    //run updater
    let _child = std::process::Command::new("./GameMon-update")
        .spawn();

    // Start the GUI application
    let gamemon_icon_path = env::current_dir().unwrap().join("resources/gamemon.png");
    let gamemon_icon = iced::window::icon::from_file(&gamemon_icon_path).unwrap();
  
    println!("DEBUG: Icon at {:?}", &gamemon_icon_path);
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