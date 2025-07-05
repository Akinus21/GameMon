
use std::process::{exit, Command};
use std::sync::mpsc;
use std::{env, fs, thread};
use std::time::Duration;
use game_mon::config::{check_for_updates,
    Config,
    GAMEMON_BIN_DIR,
    GAMEMON_CONFIG_FILE,
    GAMEMON_DIR,
    GAMEMON_GUI_EXECUTABLE,
    GAMEMON_RESOURCE_DIR
};
use game_mon::service;
use game_mon::tray;

mod logger;

#[cfg(windows)]
use GameMon::config;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

pub fn main() {
    logger::Logger::init().expect("Failed to initialize logger");
    
    log::info!("MAIN FUNCTION ENTRY: Starting GameMon...");
    
    // Check if the gamemon directory exists.  If not, create it.
    if !GAMEMON_DIR.as_path().exists() {
        log::info!("Home directory does not exist. Creating it...");
        if let Err(e) = fs::create_dir_all(GAMEMON_DIR.as_path()) {
            log::error!("Failed to create directory: {}", e);
        } else {
            log::info!("Directory created at {:?}", GAMEMON_DIR.as_path());
        }
    } else {
        // log::info!("Directory already exists at {:?}", dir_path);
    }

    // Set the working directory to the new path
    if let Err(e) = env::set_current_dir(GAMEMON_DIR.as_path()) {
        log::error!("Failed to change directory: {}", e);
    } else {
        log::info!("Current directory changed to {:?}", GAMEMON_DIR.as_path());
    }

    // Check the OS and set things accordingly
    if cfg!(target_os = "linux") {

        if let Ok(uid) = env::var("UID") {
            let runtime_dir = format!("/run/user/{}", uid);
            env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        } else {
            log::error!("Warning: Could not determine UID, XDG_RUNTIME_DIR not set.");
        }

    } else if cfg!(target_os = "windows") {
        log::info!("Running on Windows");
        // Windows-specific actions

        // Add the directory to the PATH environment variable
        let mut path = env::var("PATH").unwrap();
        path.push(';');
        path.push_str(&*GAMEMON_BIN_DIR.to_string_lossy());

        env::set_var("PATH", path);

    } else if cfg!(target_os = "macos") {
        log::info!("Running on macOS");
        // macOS-specific actions
    } else {
        log::info!("Running on an unknown OS");
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
            log::error!("Failed to send watchdog result to main thread: {}", e);
        }
    });

    // Spawn the tray logic in its own thread
    thread::spawn(move || {
        let _ = gtk::init();
        tray::spawn_tray(ttx.clone()
            ,"GameMon - A Gaming Monitor".to_string()
            ,GAMEMON_RESOURCE_DIR.as_path().join("gamemon.png")
            ,vec!(("Show GUI".to_string(), "show_gui".to_string())
                        ,("Check for Updates".to_string(), "updates".to_string() )
                        ,("Quit".to_string(), "quit".to_string())
                    )
        );
        gtk::main(); // Keep GTK running in the tray thread
    });

    loop {
        // Handle watchdog messages
        match wrx.try_recv() {
            Ok(Ok(_)) => {
                log::info!("Watchdog started successfully.");
            }
            Ok(Err(e)) => {
                log::error!("Watchdog encountered an error: {}", e);
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Watchdog thread disconnected. Exiting...");
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
                        log::info!("Received quit message from tray.");
                        break; // Exit the main loop
                    }
                    "show_gui" => {
                        log::info!("Received Show GUI message from tray.");
                            show_gui();
                    }
                    "updates" => {
                        log::info!("Received Check for Updates message from tray.");
                        match check_for_updates() {
                            Ok(_) => log::info!("Check for updates complete!"),
                            Err(e) => log::error!("Error checking for updates: {:?}\n", e),
                        }
                    }
                    msg if msg.starts_with("start:") => {
                        let game_name = msg.trim_start_matches("start:");
                        log::info!("Running start commands for {}", game_name);
                        if let Ok(config) = Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()) {
                            if let Some(entry) = config.entries.iter().find(|e| e.game_name == game_name) {
                                let _ = service::run_commands(&entry.start_commands);
                            }
                        }
                    }
                    msg if msg.starts_with("end:") => {
                        let game_name = msg.trim_start_matches("end:");
                        log::info!("Running end commands for {}", game_name);
                        if let Ok(config) = Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()) {
                            if let Some(entry) = config.entries.iter().find(|e| e.game_name == game_name) {
                                let _ = service::run_commands(&entry.end_commands);
                            }
                        }
                    }
                    other => {
                        log::info!("Received message from tray: {}", other);
                    }
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Tray thread disconnected. Exiting...");
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No tray messages yet, continue running
            }
        }
    
        // Keep the main thread alive with a sleep
        thread::sleep(Duration::from_secs(1));
    }

    log::info!("Main function exiting.");

}

pub fn show_gui() {
    // Ensure the executable path is constructed correctly
    
    let gui_path = GAMEMON_GUI_EXECUTABLE.as_path();

    // Check if the file exists before attempting to execute it
    if !gui_path.exists() {
        log::error!("Error: GameMon-gui not found at {:?}", gui_path);
        exit(1); // Exit or handle the error appropriately
    }

    let gui_path_str = gui_path.to_str().expect("Failed to convert path to string");

    // Attempt to spawn the process and handle any errors
    #[cfg(unix)]{
        match Command::new(gui_path_str).spawn() {
            Ok(_child) => {
                // Optionally, handle child process output, status, etc.
                log::info!("Successfully spawned GameMon-gui.");
            }
            Err(e) => {
                log::error!("Failed to spawn GameMon-gui: {}", e);
                exit(1); // Exit or handle the error appropriately
            }
        }
    }

    #[cfg(windows)]
    {
        match config::run_windows_cmd(&gui_path_str) {
            Ok(_) => log::info!("{:?} executed successfully", &gui_path_str),
            Err(e) => log::error!("Failed to execute command '{}': {}", &gui_path_str, e),
        }
    }
}