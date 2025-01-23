use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{env, error::Error, io::Write};
use image::ImageReader;
use native_dialog::{MessageDialog, MessageType};
use self_update::{backends::github::Update, cargo_crate_version, self_replace, ArchiveKind};
use notify_rust::Notification;
use std::fs;
use std::process::{Command, exit};
use reqwest::blocking::Client;
use serde_json;
use std::process::Command as ProcessCommand;


pub struct CustomIcon{
    file_path: String,
}

impl CustomIcon {
    pub fn new(file_path: &str) -> Self {
        CustomIcon {
            file_path: file_path.to_string(),
        }
    }

    pub fn get_icon(&self) -> tray_icon::Icon {
        let img = ImageReader::open(&self.file_path)
            .expect("Failed to open tray icon file")
            .decode()
            .expect("Failed to decode PNG image");

        // Convert the image to RGBA and create the icon
        let rgba = img.to_rgba8();
        let width = img.width() as u32;
        let height = img.height() as u32;
        tray_icon::Icon::from_rgba(rgba.into_raw(), width, height)
            .expect("Failed to create icon from decoded image")
    }
}

pub fn stop_game_mon() -> Result<(), Box<dyn std::error::Error>> {
    const MAX_RETRIES: usize = 3;
    let mut attempts = 0;

    while attempts < MAX_RETRIES {
        #[cfg(unix)]
        {
            // Verify if any GameMon processes (not GameMon-gui) are still running
            let check_output = ProcessCommand::new("pgrep")
                .arg("GameMon-gui")
                .output()?;

            // Store the result of from_utf8_lossy in a `let` binding to ensure it's not a temporary
            let remaining_processes = String::from_utf8_lossy(&check_output.stdout);

            // Filter out GameMon-gui by excluding it from the list of processes
            let mut remaining_processes: Vec<&str> = remaining_processes
                .lines()
                .collect();

            loop {
                if !remaining_processes.is_empty() {
                    let count = &remaining_processes.len() -1;
                    let p = &remaining_processes[count];
                    println!("Killing {:?}", &remaining_processes[count]);
                    let _check_output = ProcessCommand::new("pkill")
                        .arg(p)
                        .output()?;
                    remaining_processes.remove(count);
    
                } else {
                    break;
                }
            }

            if remaining_processes.is_empty() {
                println!("All instances of GameMon-gui have been stopped.");
                return Ok(());
            } else {
                for p in remaining_processes {
                    let _check_output = ProcessCommand::new("pkill")
                        .arg(p)
                        .output()?;
                }
            }

            eprintln!("Some GameMon instances are still running on attempt {}.", attempts + 1);
        }

        #[cfg(windows)]
        {
            // On Windows, use `taskkill` to stop the processes, excluding GameMon-gui
            let output = ProcessCommand::new("taskkill")
                .arg("/f") // Force terminate
                .arg("/im")
                .arg("GameMon.exe") // Assuming executable is named "GameMon.exe"
                .output()?;

            if !output.status.success() {
                eprintln!("Failed to stop GameMon processes on attempt {}. taskkill output: {:?}", attempts + 1, output);
            }

            // Verify if any GameMon processes (not GameMon-gui) are still running
            let check_output = ProcessCommand::new("tasklist")
                .arg("/fi")
                .arg("imagename eq GameMon.exe")
                .output()?;

            // Store the result of tasklist output to avoid temporary value dropping
            let remaining_processes = String::from_utf8_lossy(&check_output.stdout);

            // Filter out GameMon-gui by checking the task name
            let remaining_processes: Vec<&str> = remaining_processes
                .lines()
                .filter(|&line| !line.contains("GameMon-gui.exe"))
                .collect();

            if remaining_processes.is_empty() {
                println!("All instances of GameMon (excluding GameMon-gui) have been stopped.");
                return Ok(());
            }

            eprintln!("Some GameMon instances are still running on attempt {}.", attempts + 1);
        }

        // Wait before retrying (optional, you can adjust the duration)
        println!("Retrying to stop GameMon... (Attempt {})", attempts + 1);
        std::thread::sleep(std::time::Duration::from_secs(2)); // 2 seconds delay between retries

        attempts += 1;
    }

    // If the loop completes without success, return an error
    eprintln!("Failed to stop all GameMon processes after {} attempts.", MAX_RETRIES);
    Err("Failed to stop all GameMon processes.".into())
}

fn start_game_mon() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        // Check if GameMon is running as a service
        for g in ["GameMon", "gamemon"] {
            if check_service(g).unwrap_or(false) {
                MessageDialog::new()
                    .set_title("GameMon listed as Service!")
                    .set_type(MessageType::Warning)
                    .set_text(&format!(
                        "GameMon is listed as a service! You will need to start the service manually to complete the update."
                    ))
                    .show_alert()?;
                return Ok(()); // Inform the user and exit the function
            }
        }

        // Spawn the GameMon executable as a new process
        let _child = std::process::Command::new("./GameMon")
            .spawn()?; // Spawn the process and detach it
        println!("GameMon started successfully.");
        Ok(())
    }

    #[cfg(windows)]
    {
        // Spawn the GameMon executable as a new process on Windows
        let _child = std::process::Command::new("GameMon.exe")
            .spawn()?; // Spawn the process and detach it
        println!("GameMon started successfully.");
        Ok(())
    }
}

fn check_service(program_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let is_found = if cfg!(target_os = "linux") {
        // On Linux, we can use systemctl to list all services
        let output = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .arg("--quiet")
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        output_str.contains(program_name)
    } else if cfg!(target_os = "windows") {
        // On Windows, use tasklist to list all running processes
        let output = Command::new("tasklist")
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        output_str.to_lowercase().contains(&program_name.to_lowercase())
    } else {
        eprintln!("Unsupported OS");
        exit(1);
    };

    Ok(is_found)
}

fn is_active(program_name: &str) -> Result<bool, Box<dyn std::error::Error>>{
    let active = if cfg!(target_os = "linux") {
        // On Linux, we can use systemctl to list all services
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg(program_name)
            .output().unwrap();

        let output_str = String::from_utf8_lossy(&output.stdout);
        output_str.eq("active")
    } else {
        false
    };

    Ok(active)
}





