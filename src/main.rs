
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use std::sync::mpsc;
use std::{env, fs, thread};
use std::time::Duration;
use GameMon::config::{GAMEMON_BIN_DIR
    , GAMEMON_CONFIG_DIR
    , GAMEMON_CONFIG_FILE
    , GAMEMON_DIR
    , GAMEMON_EXECUTABLE
    , GAMEMON_GUI_EXECUTABLE
    , GAMEMON_RESOURCE_DIR
    , GAMEMON_UPDATER
};
use GameMon::service;
use GameMon::tray;

pub fn main() {
    
    let _child = std::process::Command::new(GAMEMON_UPDATER.as_path())
        .spawn()
        .expect("Failed to start updater");

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
            ,vec!(("show_gui".to_string(), "show_gui".to_string())
                        ,("quit".to_string(), "quit".to_string())
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

        use std::ptr;
        use winapi::um::shellapi::ShellExecuteW;
        use winapi::um::winuser::SW_HIDE;
        use winapi::um::winnt::LPCWSTR;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        fn to_wide_null(s: &str) -> Vec<u16> {
            OsStr::new(s).encode_wide().chain(Some(0)).collect()
        }

        // let mut cmd = Command::new("cmd")
        //     .arg("/c")
        //     .arg("start")
        //     .arg("")
        //     .arg(&gui_path_str)
        //     .stdout(Stdio::null()) // Suppress console output
        //     .stderr(Stdio::null());

        // match cmd.spawn() {
        //     Ok(_child) => println!("Successfully spawned GameMon-gui."),
        //     Err(e) => {
        //         eprintln!("Failed to spawn GameMon-gui: {}", e);
        //         std::process::exit(1);
        //     }
        // }

        let path = to_wide_null(&gui_path_str);

        unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                to_wide_null("open").as_ptr() as LPCWSTR,
                path.as_ptr() as LPCWSTR,
                ptr::null(),
                ptr::null(),
                SW_HIDE, // Ensures no console window appears
            );
        }
    }
}