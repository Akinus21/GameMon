
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod gui;
mod config;
mod tray;
mod util;
mod app;
 
pub fn main() {
//     // Create a channel for communication
//     let (tx, rx) = mpsc::channel();

//     // Spawn the watchdog function in its own thread
//     thread::spawn(move || {
//         let result = app::watchdog();
//         // Send the result back to the main thread
//         if let Err(e) = tx.send(result) {
//             eprintln!("Failed to send watchdog result to main thread: {}", e);
//         }
//     });

//     loop {
//         match rx.try_recv() {
//             Ok(Ok(_)) => {
//                 println!("Watchdog started successfully.");
//             }
//             Ok(Err(e)) => {
//                 eprintln!("Watchdog encountered an error: {}", e);
//                 break;
//             }
//             Err(mpsc::TryRecvError::Disconnected) => {
//                 eprintln!("Watchdog thread disconnected. Exiting...");
//                 break;
//             }
//             Err(mpsc::TryRecvError::Empty) => {
//                 // No messages yet, continue running
//             }
//         }

//         // Keep the main thread alive with a sleep
//         thread::sleep(Duration::from_secs(1));
//     }

//     println!("Main function exiting.");
    let result = gui::show_gui();
    assert!(result.is_ok());
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

#[test]
fn test_gui() {
    let result = gui::show_gui();
    assert!(result.is_ok());
}