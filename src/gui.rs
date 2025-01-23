use std::process::{Command, exit};
use std::path::{Path, PathBuf};
use std::fs;

pub fn show_gui(original_dir: &PathBuf) {
    // Ensure the executable path is constructed correctly
    
    let gui_path = match std::env::consts::OS {
        "linux" => {
            original_dir.join("GameMon-gui")
        }
        "windows" => {
            original_dir.join("GameMon-gui.exe")
        }
        _ => {
            original_dir.join("GameMon-gui")
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
        Ok(mut child) => {
            // Optionally, handle child process output, status, etc.
            println!("Successfully spawned GameMon-gui.");
        }
        Err(e) => {
            eprintln!("Failed to spawn GameMon-gui: {}", e);
            exit(1); // Exit or handle the error appropriately
        }
    }
}