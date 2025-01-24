use std::env;
use std::process::{Command, exit};

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