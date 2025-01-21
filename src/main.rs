
use std::sync::mpsc;
use std::{env, fs, thread};
use std::time::Duration;

use tray::Tray;

mod gui;
mod config;
mod tray;
mod util;
mod app;
 
pub fn main() {

    // Check the OS and set the directory accordingly
    if cfg!(target_os = "linux") {
        let dir_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("gamemon");

        // Check if the directory exists
        if !dir_path.exists() {
            println!("Home directory does not exist. Creating it...");
            if let Err(e) = fs::create_dir_all(&dir_path) {
                eprintln!("Failed to create directory: {}", e);
            } else {
                println!("Directory created at {:?}", dir_path);
            }
        } else {
            println!("Directory already exists at {:?}", dir_path);
        }

        env::set_current_dir(dir_path ).expect("Failed to change directory");
        env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");


    } else if cfg!(target_os = "windows") {
        println!("Running on Windows");
        // Windows-specific actions
    } else if cfg!(target_os = "macos") {
        println!("Running on macOS");
        // macOS-specific actions
    } else {
        println!("Running on an unknown OS");
        // Fallback actions
    }

    // Create a channel for communication
    let (wtx, wrx) = mpsc::channel(); // For watchdog
    // let (ttx, trx) = mpsc::channel(); // For tray events


    // Spawn the watchdog function in its own thread
    thread::spawn(move || {
        let result = app::watchdog();
        // Send the result back to the main thread
        if let Err(e) = wtx.send(result) {
            eprintln!("Failed to send watchdog result to main thread: {}", e);
        }
    });

    // // Spawn the tray logic in its own thread
    // thread::spawn(move || {
    //     let tray = Tray::new(ttx);
    //     tray.spawn();
    //     gtk::main(); // Keep GTK running in the tray thread
    // });

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

        // // Handle tray events
        // if let Ok(event) = trx.try_recv() {
        //     println!("Tray event recieved: {}", event);
        //     match event.as_str() {
        //         "Show GUI" => {
        //             println!("Show GUI triggered.");
        //             // let result = gui::show_gui();
        //             // assert!(result.is_ok());
        //         }
        //         "Exit" => {
        //             println!("Exit triggered. Exiting application...");
        //             break;
        //         }
        //         _ => eprintln!("Unknown menu event: {}", event),
        //     }
        // }

        // Keep the main thread alive with a sleep
        thread::sleep(Duration::from_secs(1));
    }

    println!("Main function exiting.");

}