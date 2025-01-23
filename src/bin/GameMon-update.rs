use std::{env, fs, io};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use notify_rust::Notification;
use std::process::{Command, exit};
use reqwest::blocking::Client;
use serde_json;
use native_dialog::{MessageDialog, MessageType};
use serde_json::Value;
use std::process::Command as ProcessCommand;



pub fn main() {
    let _ = update();
}


pub fn update() -> Result<(), Box<dyn std::error::Error>> {
    // GitHub API URL for latest release
    let latest_release_url = "https://api.github.com/repos/Akinus21/GameMon/releases/latest";

    // Create HTTP client
    let client = Client::new();
    let response: Value = client
        .get(latest_release_url)
        .header("User-Agent", "GameMon-Updater")
        .send()?
        .json()?;

    // Extract the version from the GitHub release
    let latest_version = response["tag_name"]
        .as_str()
        .ok_or("No version found in release")?;

    // Get current version from the package's version defined in Cargo.toml
    let current_version = env!("CARGO_PKG_VERSION");

    // Compare versions
    if latest_version == current_version {
        println!("You are already on the latest version: {}", current_version);
        return Ok(()); // No update needed
    }

    println!("Update available! Current version: {}, Latest version: {}", current_version, latest_version);

    // Show a Yes/No dialog to the user
    let result = MessageDialog::new()
        .set_title("Update Available")
        .set_type(MessageType::Info)
        .set_text(&format!("A new version ({}) is available. Do you want to update?", latest_version))
        .show_confirm();

    // If user clicks "Yes", proceed with update
    if Some(result.unwrap()) == Some(true) {
        // Extract download URL for the asset
        let asset_url = response["assets"][0]["browser_download_url"]
            .as_str()
            .ok_or("No download URL found")?;

        // Get system's temp directory from TMP environment variable
        let tmp_dir = env::var("TMP")
            .unwrap_or_else(|_| env::temp_dir().to_str().unwrap().to_string());

        let tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.tar.gz");

        // Download the asset to the temp directory
        let mut resp = client.get(asset_url).send()?;
        let mut file = fs::File::create(&tmp_archive_path)?;
        resp.copy_to(&mut file)?;

        // Notify the user that download is complete
        Notification::new()
            .summary("GameMon Update")
            .body("Download complete! Extracting update...")
            .icon("notification")
            .show()?;

        // Set permissions on the new file (Linux only)
        #[cfg(unix)]
        fs::set_permissions(&tmp_archive_path, fs::Permissions::from_mode(0o755))?;

        // Extract the downloaded archive
        let tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
        extract_tar_gz(&tmp_archive_path, &tmp_extract_dir)?;

        // Replace the current executable  

        // Stop all instances of GameMon first
        println!("Stopping all GameMon processes...");
        let _stop = stop_game_mon();

        // Then replace

        let result = if cfg!(target_os = "linux") {
            let new_exe = tmp_extract_dir.join("GameMon");
            let new_gui= tmp_extract_dir.join("GameMon-gui");
            let new_updater= tmp_extract_dir.join("GameMon-update");
            let curr_exe = Path::new(&env::current_dir().unwrap()).join("GameMon");
            let curr_gui = Path::new(&env::current_dir().unwrap()).join("GameMon-gui");
            let curr_updater = Path::new(&env::current_dir().unwrap()).join("GameMon-update_tmp");
            let new_res = tmp_extract_dir.join("resources");
            let curr_res = Path::new(&env::current_dir().unwrap()).join("resources");
            println!("Replacing GUI binary with new version...");
            fs::copy(new_gui, curr_gui)?;
            println!("Replacing GameMon binary with new version...");
            fs::copy(new_exe, curr_exe)?;
            println!("Replacing GameMon updater binary with new version...");
            fs::copy(new_updater, curr_updater)?;
            println!("Copying resources...");
            fs::copy(new_res, curr_res)

        } else if cfg!(target_os = "windows") {
            let new_exe = tmp_extract_dir.join("GameMon.exe");
            let new_gui= tmp_extract_dir.join("GameMon-gui.exe");
            let new_updater= tmp_extract_dir.join("GameMon-update.exe");
            let curr_exe = Path::new(&env::current_dir().unwrap()).join("GameMon.exe");
            let curr_gui = Path::new(&env::current_dir().unwrap()).join("GameMon-gui.exe");
            let curr_updater = Path::new(&env::current_dir().unwrap()).join("GameMon-update_tmp.exe");
            let new_res = tmp_extract_dir.join("resources");
            let curr_res = Path::new(&env::current_dir().unwrap()).join("resources");            
            println!("Replacing GUI binary with new version...");
            fs::copy(new_gui, curr_gui)?;
            println!("Replacing GameMon binary with new version...");
            fs::copy(new_exe, curr_exe)?;
            println!("Replacing GameMon updater binary with new version...");
            fs::copy(new_updater, curr_updater)?;
            println!("Copying resources...");
            fs::copy(new_res, curr_res)


        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS").into())
        };
    
        let _replace = match result {
            Ok(_) => {
                println!("Executables copied successfully.");
                let _start = match start_game_mon(){
                    Ok(_) => {
                        println!("Restarted GameMon Successfully!")
                    }
                    Err(e) => {
                        println!("Error restarting GameMon: {:?}", e)
                    }
                };
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to copy executable: {}", e);
                Err(e)
            }
        };

        // Notify the user that the update is complete
        let _result = MessageDialog::new()
            .set_title("GameMon Update Complete!")
            .set_type(MessageType::Warning)
            .set_text(&format!("GameMon is updated!  Thank you so much!  Enjoy!"))
            .show_alert();
        #[cfg(unix)]
        {
            let _child = std::process::Command::new("mv")
                .arg("GameMon-update_tmp")
                .arg("GameMon-update")
                .spawn()?;
        }
        
    } else {
        println!("User chose not to update.");
    }

    Ok(())
}

fn extract_tar_gz(tar_gz_path: &Path, extract_to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Open the .tar.gz file
    let tar_gz = fs::File::open(tar_gz_path)?;
    let decompressor = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(decompressor);

    // Create the extraction directory if it doesn't exist
    if !extract_to.exists() {
        fs::create_dir_all(extract_to)?;
    }

    // Extract the tar.gz file
    archive.unpack(extract_to)?;

    println!("Extracted files to {:?}", extract_to);
    Ok(())
}

fn stop_game_mon() -> Result<(), Box<dyn std::error::Error>> {
    const MAX_RETRIES: usize = 3;
    let mut attempts = 0;

    while attempts < MAX_RETRIES {
        #[cfg(unix)]
        {
            // Check if GameMon is running as a service. Let the user know they will have to kill it manually then. 
            for g in ["GameMon", "gamemon"]{
                if check_service(g).unwrap(){
                    if is_active(g).unwrap(){
                        let _result = MessageDialog::new()
                            .set_title("GameMon running as Service!")
                            .set_type(MessageType::Warning)
                            .set_text(&format!("GameMon is running as a service on this user!  You will need to stop the service manually to update."))
                            .show_alert();
                        exit(1)
                    }
                }  
            };

            // Verify if any GameMon processes (not GameMon-gui) are still running
            let check_output = ProcessCommand::new("pgrep")
                .arg("GameMon")
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
                println!("All instances of GameMon (including GameMon-gui) have been stopped.");
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