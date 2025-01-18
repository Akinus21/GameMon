
use std::env;

mod gui;
mod config;
mod tray;
mod util;
mod app;
 
pub fn main(){
    // Set the environment variables
    env::set_var("DISPLAY", ":0");  // Adjust as needed for your system
    env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/run/user/1000/bus");  // Replace 1000 with your user ID

    match app::watchdog() {
        Ok(_) => {
            // Handle success (if needed)
            println!("Watchdog started successfully.");
        },
        Err(e) => {
            // Handle the error
            eprintln!("Failed to start watchdog: {}", e);
            // Optionally, you can exit the program or return a specific error code
            std::process::exit(1);
        }
    }
}

#[test]
fn get_entries() {
    let config = config::Config::load_from_file(&*config::Config::get_config_path().unwrap()).unwrap();
    println!("{:?}", config.entries);
    assert_eq!(config.entries.len(), 2);
}

#[test]
fn notify_test() {
    let notification = notify_rust::Notification::new()
        .summary("GameMon")
        .body("GameMon is running.")
        .icon("dialog-information")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show();
    assert!(notification.is_ok());
}