use GameMon::app::Gui;


pub fn main() -> iced::Result {

    //run updater
    let _child = std::process::Command::new("./GameMon-update")
        .spawn();

    // Start the GUI application
    iced::application("GameMon", Gui::update, Gui::view).theme(Gui::theme).run()
}