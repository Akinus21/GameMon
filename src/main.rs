
use std::path::PathBuf;
use std::process::{exit, Command};
use std::sync::mpsc;
use std::{env, fs, thread};
use std::time::Duration;
use GameMon::service;
use GameMon::tray;

pub fn main() {

    //run updater
    let _child = std::process::Command::new("./GameMon-update")
        .spawn();

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
            // println!("Directory already exists at {:?}", dir_path);
        }
        env::set_current_dir(dir_path ).expect("Failed to change directory");
        env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");


    } else if cfg!(target_os = "windows") {
        println!("Running on Windows");
        // Windows-specific actions

        // Get the %APPDATA% directory
        let appdata = env::var("APPDATA").unwrap_or_else(|_| {
            eprintln!("Failed to get APPDATA environment variable. Using default path.");
            String::from("C:\\Users\\Default\\AppData\\Roaming")
        });

        let dir_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(appdata))
        .join("gamemon");

        // Check if the directory exists
        if !dir_path.exists() {
            println!("Configuration directory does not exist. Creating it...");
            if let Err(e) = fs::create_dir_all(&dir_path) {
                eprintln!("Failed to create directory: {}", e);
            } else {
                println!("Directory created at {:?}", dir_path);
            }
        } else {
            println!("Directory already exists at {:?}", dir_path);
        }

        // Set the working directory to the new path
        if let Err(e) = env::set_current_dir(&dir_path) {
            eprintln!("Failed to change directory: {}", e);
        } else {
            println!("Current directory changed to {:?}", dir_path);
        }

    } else if cfg!(target_os = "macos") {
        println!("Running on macOS");
        // macOS-specific actions
    } else {
        println!("Running on an unknown OS");
        // Fallback actions
    }

    // Create a channel for communication
    let (wtx, wrx) = mpsc::channel(); // For watchdog
    let (ttx, trx) = mpsc::channel(); // For tray


    // Spawn the watchdog function in its own thread
    thread::spawn(move || {
        let result = service::watchdog();
        // Send the result back to the main thread
        if let Err(e) = wtx.send(result) {
            eprintln!("Failed to send watchdog result to main thread: {}", e);
        }
    });

    // Spawn the tray logic in its own thread
    thread::spawn(move || {
        let _ = gtk::init();
        tray::spawn_tray(ttx.clone());
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
    
        // Handle tray messages
        match trx.try_recv() {
            Ok(message) => {
                match message.as_str() {
                    "quit" => {
                        println!("Received quit message from tray.");
                        break; // Exit the main loop
                    }
                    "show_gui" => {
                        println!("Received Show GUI message from tray.");
                            show_gui();
                    }
                    other => {
                        println!("Received message from tray: {}", other);
                    }
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Tray thread disconnected. Exiting...");
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No tray messages yet, continue running
            }
        }
    
        // Keep the main thread alive with a sleep
        thread::sleep(Duration::from_secs(1));
    }

    println!("Main function exiting.");

}

pub fn show_gui() {
    // Ensure the executable path is constructed correctly
    
    let gui_path = match std::env::consts::OS {
        "linux" => {
            env::current_dir().unwrap().join("GameMon-gui")
        }
        "windows" => {
            env::current_dir().unwrap().join("GameMon-gui.exe")
        }
        _ => {
            env::current_dir().unwrap().join("GameMon-gui")
        }
    };

    // Check if the file exists before attempting to execute it
    if !gui_path.exists() {
        eprintln!("Error: GameMon-gui not found at {:?}", gui_path);
        exit(1); // Exit or handle the error appropriately
    }

    let gui_path_str = gui_path.to_str().expect("Failed to convert path to string");

    // Attempt to spawn the process and handle any errors
    match Command::new(gui_path_str).spawn() {
        Ok(_child) => {
            // Optionally, handle child process output, status, etc.
            println!("Successfully spawned GameMon-gui.");
        }
        Err(e) => {
            eprintln!("Failed to spawn GameMon-gui: {}", e);
            exit(1); // Exit or handle the error appropriately
        }
    }
}