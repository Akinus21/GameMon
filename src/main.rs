
use std::process::{exit, Command};
use std::sync::mpsc;
use std::{env, fs, thread};
use std::time::Duration;
use game_mon::config::{GAMEMON_BIN_DIR
    , GAMEMON_DIR
    , GAMEMON_GUI_EXECUTABLE
    , GAMEMON_RESOURCE_DIR
    , check_for_updates
};
use game_mon::service;
use game_mon::tray;
#[cfg(windows)]
use GameMon::config;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

pub fn main() {
    
    match check_for_updates() {
        Ok(_) => println!("Check for updates complete!"),
        Err(e) => eprintln!("Error checking for updates: {:?}\n", e),
    }


    // Check if the gamemon directory exists.  If not, create it.
    if !GAMEMON_DIR.as_path().exists() {
        println!("Home directory does not exist. Creating it...");
        if let Err(e) = fs::create_dir_all(GAMEMON_DIR.as_path()) {
            eprintln!("Failed to create directory: {}", e);
        } else {
            println!("Directory created at {:?}", GAMEMON_DIR.as_path());
        }
    } else {
        // println!("Directory already exists at {:?}", dir_path);
    }

    // Set the working directory to the new path
    if let Err(e) = env::set_current_dir(GAMEMON_DIR.as_path()) {
        eprintln!("Failed to change directory: {}", e);
    } else {
        println!("Current directory changed to {:?}", GAMEMON_DIR.as_path());
    }

    // Check the OS and set things accordingly
    if cfg!(target_os = "linux") {

        if let Ok(uid) = env::var("UID") {
            let runtime_dir = format!("/run/user/{}", uid);
            env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        } else {
            eprintln!("Warning: Could not determine UID, XDG_RUNTIME_DIR not set.");
        }

    } else if cfg!(target_os = "windows") {
        println!("Running on Windows");
        // Windows-specific actions

        // Add the directory to the PATH environment variable
        let mut path = env::var("PATH").unwrap();
        path.push(';');
        path.push_str(&*GAMEMON_BIN_DIR.to_string_lossy());

        env::set_var("PATH", path);

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
                    "updates" => {
                        println!("Received Check for Updates message from tray.");
                        match check_for_updates() {
                            Ok(_) => println!("Check for updates complete!"),
                            Err(e) => eprintln!("Error checking for updates: {:?}\n", e),
                        }
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
    
    let gui_path = GAMEMON_GUI_EXECUTABLE.as_path();

    // Check if the file exists before attempting to execute it
    if !gui_path.exists() {
        eprintln!("Error: GameMon-gui not found at {:?}", gui_path);
        exit(1); // Exit or handle the error appropriately
    }

    let gui_path_str = gui_path.to_str().expect("Failed to convert path to string");

    // Attempt to spawn the process and handle any errors
    #[cfg(unix)]{
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

    #[cfg(windows)]
    {
        match config::run_windows_cmd(&gui_path_str) {
            Ok(_) => println!("{:?} executed successfully", &gui_path_str),
            Err(e) => eprintln!("Failed to execute command '{}': {}", &gui_path_str, e),
        }
    }
}