
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tray::Tray;

mod gui;
mod config;
mod tray;
mod util;
mod app;
 
pub fn main() {
    // Create a channel for communication
    let (wtx, wrx) = mpsc::channel(); // For watchdog
    let (ttx, trx) = mpsc::channel(); // For tray events


    // Spawn the watchdog function in its own thread
    thread::spawn(move || {
        let result = app::watchdog();
        // Send the result back to the main thread
        if let Err(e) = wtx.send(result) {
            eprintln!("Failed to send watchdog result to main thread: {}", e);
        }
    });

    // Spawn the tray logic in its own thread
    thread::spawn(move || {
        let tray = Tray::new(ttx);
        tray.spawn();
        gtk::main(); // Keep GTK running in the tray thread
    });

    loop {
        // Handle watchdog messages
        match wrx.try_recv() {
            Ok(Ok(_)) => {
                println!("Watchdog started successfully.");
            }
            Ok(Err(e)) => {
                eprintln!("Watchdog encountered an error: {}", e);
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Watchdog thread disconnected. Exiting...");
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No messages yet, continue running
            }
        }

        // Handle tray events
        if let Ok(event) = trx.try_recv() {
            println!("Tray event recieved: {}", event);
            match event.as_str() {
                "Show GUI" => {
                    println!("Show GUI triggered.");
                    // let result = gui::show_gui();
                    // assert!(result.is_ok());
                }
                "Exit" => {
                    println!("Exit triggered. Exiting application...");
                    break;
                }
                _ => eprintln!("Unknown menu event: {}", event),
            }
        }

        // Keep the main thread alive with a sleep
        thread::sleep(Duration::from_secs(1));
    }

    println!("Main function exiting.");

}