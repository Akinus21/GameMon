use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::{env, fs, io};
use std::path::Path;
use notify_rust::Notification;
use std::process::{exit, Command, Output, ExitStatus};
use reqwest::blocking::Client;
use serde_json;
use native_dialog::{MessageDialog, MessageType};
use serde_json::Value;
use std::process::Command as ProcessCommand;
use zip::read::ZipArchive;
use GameMon::config::{GAMEMON_DIR
    ,GAMEMON_CONFIG_DIR
    ,GAMEMON_CONFIG_FILE
    ,GAMEMON_EXECUTABLE
    ,GAMEMON_GUI_EXECUTABLE
    ,GAMEMON_RESOURCE_DIR
    ,GAMEMON_UPDATER
    ,GAMEMON_ICON
    ,GAMEMON_BIN_DIR
    ,ensure_paths_exist
};

pub fn main() {
    if !GAMEMON_DIR.as_path().exists(){
        let _ = install();
    } else {
        let _ = update();
    }
}

pub fn update() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = ensure_paths_exist() {
        eprintln!("Error ensuring paths exist: {}", e);
    }

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
        #[cfg(unix)]
        {
            if !is_service_running(){
                let _start = match start_game_mon(){
                    Ok(_) => {
                        println!("Restarted GameMon Successfully!")
                    }
                    Err(e) => {
                        println!("Error restarting GameMon: {:?}", e)
                    }
                };
            }
        }
        return Ok(()); // No update needed
    }

    println!("Update available! Current version: {}, Latest version: {}", current_version, latest_version);

    // Detect the current platform (target OS and architecture)
    let target_os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    // Show a Yes/No dialog to the user
    let result = MessageDialog::new()
        .set_title("Update Available")
        .set_type(MessageType::Info)
        .set_text(&format!("A new version ({}) is available. Do you want to update?", latest_version))
        .show_confirm();

    if Some(result.unwrap()) == Some(true) {
        // Determine the correct asset URL for the current platform
        let asset_url = match target_os {
            "linux" => response["assets"]
                .as_array()
                .and_then(|assets| assets.iter().find(|a| a["name"].as_str().unwrap_or("").contains("linux")))
                .and_then(|asset| asset["browser_download_url"].as_str())
                .ok_or("No Linux download URL found")?,
            "windows" => response["assets"]
                .as_array()
                .and_then(|assets| assets.iter().find(|a| a["name"].as_str().unwrap_or("").contains("windows")))
                .and_then(|asset| asset["browser_download_url"].as_str())
                .ok_or("No Windows download URL found")?,
            _ => return Err("Unsupported platform".into()),
        };

        // Proceed to download the correct file based on the asset URL
        println!("Downloading from: {}", asset_url);

        // Get system's temp directory from TMP environment variable
        let tmp_dir = env::var("TMP")
            .unwrap_or_else(|_| env::temp_dir().to_str().unwrap().to_string());

        let tmp_archive_path;
        #[cfg(unix)]
        {
            tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.tar.gz");
        }
        #[cfg(windows)]
        {
            tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.zip");
        }

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
        {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_archive_path, fs::Permissions::from_mode(0o755))?;
        }

        // Extract the downloaded archive
        let tmp_extract_dir;
        // Set permissions on the new file (Linux only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp_archive_path, fs::Permissions::from_mode(0o755))?;
            // Extract the downloaded archive
            tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
            extract_tar_gz(&tmp_archive_path, &tmp_extract_dir)?;
        }

        #[cfg(windows)]
        {
            // Extract the downloaded archive
            tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
            extract_zip(&tmp_archive_path, &tmp_extract_dir)?;
        }

        // Replace the current executable  

        // Stop all instances of GameMon first
        println!("Stopping all GameMon processes...");
        let _stop = stop_game_mon();

        // Then replace

        let (new_exe, new_gui, new_updater, new_icon);

        if cfg!(target_os = "linux") {
            new_exe = tmp_extract_dir.join("GameMon");
            new_gui = tmp_extract_dir.join("GameMon-gui");
            new_updater = tmp_extract_dir.join("GameMon-update");
            new_icon = tmp_extract_dir.join("resources/gamemon.png");
        } else if cfg!(target_os = "windows") {
            new_exe = tmp_extract_dir.join("GameMon.exe");
            new_gui = tmp_extract_dir.join("GameMon-gui.exe");
            new_updater = tmp_extract_dir.join("GameMon-update.exe");
            new_icon = tmp_extract_dir.join("resources\\gamemon.png");
        } else {
            panic!("Unsupported OS");
        }

        let errors = vec![
            replace_binary(&new_gui, GAMEMON_GUI_EXECUTABLE.as_path(), "GUI"),
            replace_binary(&new_exe, GAMEMON_EXECUTABLE.as_path(), "GameMon"),
            replace_binary(&new_updater, GAMEMON_UPDATER.as_path(), "Updater"),
            replace_binary(&new_icon, GAMEMON_ICON.as_path(), "Icon"),
        ];

        // Handle any failures
        let failed_ops: Vec<_> = errors.into_iter().filter(|res| res.is_err()).collect();

        if !failed_ops.is_empty() {
            eprintln!("Some files failed to replace. Check error messages above.");
            std::process::exit(1);
        } else {
            println!("All files replaced successfully!");
            let _start = match start_game_mon(){
                Ok(_) => {
                    println!("Restarted GameMon Successfully!")
                }
                Err(e) => {
                    println!("Error restarting GameMon: {:?}", e)
                }
            };
        }

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

        #[cfg(windows)]
        {
            let _child = std::process::Command::new("cmd")
                .args(&["/C", "move", "GameMon-update_tmp.exe", "GameMon-update.exe"])
                .spawn()?;
        }
        
    } else {
        println!("User chose not to update.");
    }

    #[cfg(unix)]
    {
        if !is_service_running(){
            let _start = match start_game_mon(){
                Ok(_) => {
                    println!("Restarted GameMon Successfully!")
                }
                Err(e) => {
                    println!("Error restarting GameMon: {:?}", e)
                }
            };
        }
    }

    Ok(())
}

pub fn install() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = ensure_paths_exist() {
        eprintln!("Error ensuring paths exist: {}", e);
    }

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

    println!("Installing latest version: {}", latest_version);

    // Show a Yes/No dialog to the user
    let result = MessageDialog::new()
        .set_title("Install?")
        .set_type(MessageType::Info)
        .set_text(&format!("GameMon is not installed on this machine.  Would you like to install? \nLatest version: {}", latest_version))
        .show_confirm();

    // If user clicks "Yes", proceed with update
    if Some(result.unwrap()) == Some(true) {

        // Detect the current platform (target OS and architecture)
        let target_os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unknown"
        };

        // Determine the correct asset URL for the current platform
        let asset_url = match target_os {
            "linux" => response["assets"]
                .as_array()
                .and_then(|assets| assets.iter().find(|a| a["name"].as_str().unwrap_or("").contains("linux")))
                .and_then(|asset| asset["browser_download_url"].as_str())
                .ok_or("No Linux download URL found")?,
            "windows" => response["assets"]
                .as_array()
                .and_then(|assets| assets.iter().find(|a| a["name"].as_str().unwrap_or("").contains("windows")))
                .and_then(|asset| asset["browser_download_url"].as_str())
                .ok_or("No Windows download URL found")?,
            _ => return Err("Unsupported platform".into()),
        };

        // Proceed to download the correct file based on the asset URL
        println!("Downloading from: {}", asset_url);

        // Get system's temp directory from TMP environment variable
        let tmp_dir = env::var("TMP")
            .unwrap_or_else(|_| env::temp_dir().to_str().unwrap().to_string());

        let tmp_archive_path;
        #[cfg(unix)]
        {
            tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.tar.gz");
        }
        #[cfg(windows)]
        {
            tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.zip");
        }

        // Download the asset to the temp directory
        let mut resp = client.get(asset_url).send()?;
        let mut file = fs::File::create(&tmp_archive_path)?;
        resp.copy_to(&mut file)?;

        // Notify the user that download is complete
        Notification::new()
            .summary("GameMon Install")
            .body("Download complete! Extracting update...")
            .icon("notification")
            .show()?;

        let tmp_extract_dir;
        // Set permissions on the new file (Linux only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp_archive_path, fs::Permissions::from_mode(0o755))?;
            // Extract the downloaded archive
            tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
            extract_tar_gz(&tmp_archive_path, &tmp_extract_dir)?;
        }

        #[cfg(windows)]
        {
            // Extract the downloaded archive
            tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
            extract_zip(&tmp_archive_path, &tmp_extract_dir)?;
        }

        // Replace the current executable  

        // Stop all instances of GameMon first
        println!("Stopping all GameMon processes...");
        let _stop = stop_game_mon();

        // Then replace

        let (new_exe, new_gui, new_updater, new_icon, errors, new_bin);

        if cfg!(target_os = "linux") {
            new_exe = tmp_extract_dir.join("GameMon");
            new_gui = tmp_extract_dir.join("GameMon-gui");
            new_updater = tmp_extract_dir.join("GameMon-update");
            new_icon = tmp_extract_dir.join("resources/gamemon.png");

            errors = vec![
                replace_binary(&new_gui, GAMEMON_GUI_EXECUTABLE.as_path(), "GUI"),
                replace_binary(&new_exe, GAMEMON_EXECUTABLE.as_path(), "GameMon"),
                replace_binary(&new_updater, GAMEMON_UPDATER.as_path(), "Updater"),
                replace_binary(&new_icon, GAMEMON_ICON.as_path(), "Icon"),
            ];
        } else if cfg!(target_os = "windows") {
            new_exe = tmp_extract_dir.join("GameMon.exe");
            new_gui = tmp_extract_dir.join("GameMon-gui.exe");
            new_updater = tmp_extract_dir.join("GameMon-update.exe");
            new_icon = tmp_extract_dir.join("resources\\gamemon.png");
            new_bin = tmp_extract_dir.join("resources\\bin");

            errors = vec![
                replace_binary(&new_gui, GAMEMON_GUI_EXECUTABLE.as_path(), "GUI"),
                replace_binary(&new_exe, GAMEMON_EXECUTABLE.as_path(), "GameMon"),
                replace_binary(&new_updater, GAMEMON_UPDATER.as_path(), "Updater"),
                replace_binary(&new_icon, GAMEMON_ICON.as_path(), "Icon"),
                copy_dir_recursive(&new_bin, GAMEMON_DIR.as_path())
            ]

        } else {
            panic!("Unsupported OS");
        }

        // Handle any failures
        let failed_ops: Vec<_> = errors.into_iter().filter(|res| res.is_err()).collect();

        if !failed_ops.is_empty() {
            eprintln!("Some files failed to install. Check error messages above.");
            std::process::exit(1);
        } else {
            println!("All files installed successfully!");
            let _start = match start_game_mon(){
                Ok(_) => {
                    println!("Started GameMon Successfully!")
                }
                Err(e) => {
                    println!("Error starting GameMon: {:?}", e)
                }
            };
        }

        // Notify the user that the update is complete
        let _result = MessageDialog::new()
            .set_title("GameMon Install Complete!")
            .set_type(MessageType::Warning)
            .set_text(&format!("GameMon is installed!  Thank you so much!  Enjoy!"))
            .show_alert();

        #[cfg(unix)]
        {
            let _child = std::process::Command::new("mv")
                .arg("GameMon-update_tmp")
                .arg("GameMon-update")
                .spawn()?;
        }

        #[cfg(windows)]
        {
            let _child = std::process::Command::new("cmd")
                .args(&["/C", "move", "GameMon-update_tmp.exe", "GameMon-update.exe"])
                .spawn()?;
        }
        
    } else {
        println!("User chose not to update.");
    }

    #[cfg(unix)]
    {
        if !is_service_running(){
            let _start = match start_game_mon(){
                Ok(_) => {
                    println!("Started GameMon Successfully!")
                }
                Err(e) => {
                    println!("Error Starting GameMon: {:?}", e)
                }
            };
        }
    }

    Ok(())
}

fn replace_binary(src: &Path, dest: &Path, name: &str) -> io::Result<()> {
    print!("Replacing {} binary... ", name);
    io::stdout().flush().ok(); // Flush output for better user experience

    // Ensure the destination directory exists
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            println!("Destination directory does not exist, creating it...");
            fs::create_dir_all(parent)?;
        }
    }

    // Copy the file from source to destination
    match fs::copy(src, dest) {
        Ok(_) => {
            println!("Success!");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed! Error: {}", e);
            Err(e)
        }
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    if !src.exists() || !src.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Source directory does not exist or is not a directory"));
    }

    // Create destination directory if it does not exist
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    // Iterate over each entry in the source directory
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            // Recursively copy subdirectory
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            // Copy individual file
            fs::copy(&src_path, &dest_path)?;
        }
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

fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), Box<dyn Error>> {
    // Open the ZIP file
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Create the extraction directory if it doesn't exist
    if !extract_to.exists() {
        fs::create_dir_all(extract_to)?;
    }

    // Extract each file in the ZIP archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = extract_to.join(file.name());

        if file.is_dir() {
            // Create directories inside the output folder
            fs::create_dir_all(&out_path)?;
        } else {
            // Create parent directory if needed
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            // Write file contents
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("Extracted ZIP files to {:?}", extract_to);
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

            // Check if the process was terminated successfully
            if !output.status.success() {
                // Convert stderr to a string to check if it contains "not found"
                let stderr_str = String::from_utf8_lossy(&output.stderr);

                if stderr_str.contains("not found") {
                    // If the process was not found, consider it "closed" and break out of the loop
                    println!("GameMon.exe is not running. Proceeding as closed.");
                    break;
                } else {
                    eprintln!("Failed to stop GameMon processes on attempt {}. taskkill output: {:?}", attempts + 1, output);
                }
            } else {
                println!("Successfully stopped GameMon.exe.");
                break;
            }
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
        let _child = std::process::Command::new(&*GAMEMON_EXECUTABLE)
            .spawn()?; // Spawn the process and detach it
        println!("GameMon started successfully.");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn is_active(program_name: &str) -> Result<bool, Box<dyn std::error::Error>>{
    use std::os::unix::process::ExitStatusExt;
    let active = if cfg!(target_os = "linux") {
        // On Linux, we can use systemctl to list all services
        let check_output = match Command::new("systemctl")
            .arg("is-active")
            .arg(program_name)
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to check service status for {}: {}", program_name, e);
                Output {
                    status: ExitStatus::from_raw(if cfg!(unix) { 1 } else { 0 }), // Unix uses 1, Windows default is 0
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                }
            }
        };


        let output_str = String::from_utf8_lossy(&check_output.stdout);
        output_str.eq("active")
    } else {
        false
    };

    Ok(active)
}

#[cfg(target_os = "linux")]
fn is_service_running() -> bool {
    use std::os::unix::process::ExitStatusExt;
    // Verify if any GameMon processes (not GameMon-gui) are still running
    let check_output = match ProcessCommand::new("pgrep").arg("GameMon").output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Failed to execute pgrep: {}", e);
            Output {
                status: ExitStatus::from_raw(if cfg!(unix) { 1 } else { 0 }), // Unix uses 1, Windows default is 0
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        }
    };

    // Store the result of from_utf8_lossy in a `let` binding to ensure it's not a temporary
    let remaining_processes = String::from_utf8_lossy(&check_output.stdout);

    // Filter out GameMon-gui by excluding it from the list of processes
    let remaining_processes: Vec<&str> = remaining_processes
    .lines()
    .filter(|line| !line.contains("GameMon-gui"))
    .collect();

    match remaining_processes.len(){
        0 => false,
        _ => true,
    }
}