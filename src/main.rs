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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use gtk::glib;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};

#[cfg(windows)]
use GameMon::config;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;



pub fn main() {
    logger::Logger::init().expect("Failed to initialize logger");
    log::info!("MAIN FUNCTION ENTRY: Starting GameMon...");

    // Flag to control graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let signal_flag = running.clone();

    // Signal handling thread
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT, SIGTERM, SIGHUP, SIGQUIT, SIGABRT])
            .expect("Failed to register signals");

        for sig in signals.forever() {
            match sig {
                SIGINT => log::warn!("🔴 Caught SIGINT (Ctrl+C)."),
                SIGTERM => log::warn!("🔴 Caught SIGTERM (Terminate)."),
                SIGHUP => log::warn!("🔴 Caught SIGHUP (Hangup)."),
                SIGQUIT => log::warn!("🔴 Caught SIGQUIT."),
                SIGABRT => log::warn!("🔴 Caught SIGABRT (Abort)."),
                _ => log::warn!("🔴 Caught unknown signal: {}", sig),
            }
            signal_flag.store(false, Ordering::SeqCst);

            // Quit GTK safely
            glib::MainContext::default().invoke(|| {
                gtk::main_quit();
            });

            break;
        }
    });

    // Setup directory
    if !GAMEMON_DIR.as_path().exists() {
        log::info!("Home directory does not exist. Creating it...");
        if let Err(e) = fs::create_dir_all(GAMEMON_DIR.as_path()) {
            log::error!("Failed to create directory: {}", e);
        } else {
            log::info!("Directory created at {:?}", GAMEMON_DIR.as_path());
        }
    }

    if let Err(e) = env::set_current_dir(GAMEMON_DIR.as_path()) {
        log::error!("Failed to change directory: {}", e);
    } else {
        log::info!("Current directory changed to {:?}", GAMEMON_DIR.as_path());
    }

    if cfg!(target_os = "linux") {
        if let Ok(uid) = env::var("UID") {
            let runtime_dir = format!("/run/user/{}", uid);
            env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        } else {
            log::error!("Warning: Could not determine UID, XDG_RUNTIME_DIR not set.");
        }
    } else if cfg!(target_os = "windows") {
        log::info!("Running on Windows");
        let mut path = env::var("PATH").unwrap_or_default();
        path.push(';');
        path.push_str(&*GAMEMON_BIN_DIR.to_string_lossy());
        env::set_var("PATH", path);
    }

    let (wtx, wrx) = mpsc::channel(); // watchdog
    let (ttx, trx) = mpsc::channel(); // tray

    thread::spawn(move || {
        let result = service::watchdog();
        if let Err(e) = wtx.send(result) {
            log::error!("Failed to send watchdog result to main thread: {}", e);
        }
    });

    thread::spawn(move || {
        let _ = gtk::init();
        tray::spawn_tray(ttx.clone(),
            "GameMon - A Gaming Monitor".to_string(),
            GAMEMON_RESOURCE_DIR.as_path().join("gamemon.png"),
            vec![
                ("Show GUI".to_string(), "show_gui".to_string()),
                ("Check for Updates".to_string(), "updates".to_string()),
                ("Quit".to_string(), "quit".to_string()),
            ],
        );
        gtk::main();
    });

    while running.load(Ordering::SeqCst) {
        match wrx.try_recv() {
            Ok(Ok(_)) => log::info!("Watchdog started successfully."),
            Ok(Err(e)) => {
                log::error!("Watchdog encountered an error: {}", e);
                break;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Watchdog thread disconnected. Exiting...");
                break;
            }
            Err(_) => {}
        }

        match trx.try_recv() {
            Ok(message) => match message.as_str() {
                "quit" => {
                    log::info!("Received quit message from tray.");
                    break;
                }
                "show_gui" => {
                    log::info!("Received Show GUI message from tray.");
                    show_gui();
                }
                "updates" => {
                    log::info!("Received Check for Updates message from tray.");
                    match check_for_updates("tray".to_string()) {
                        Ok(_) => log::info!("Check for updates complete!"),
                        Err(e) => log::error!("Error checking for updates: {:?}", e),
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
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Tray thread disconnected. Exiting...");
                break;
            }
            Err(_) => {}
        }

        thread::sleep(Duration::from_secs(1));
    }

    log::info!("Main function exiting.");
}

pub fn show_gui() {
    let gui_path = GAMEMON_GUI_EXECUTABLE.as_path();

    if !gui_path.exists() {
        log::error!("Error: GameMon-gui not found at {:?}", gui_path);
        exit(1);
    }

    let gui_path_str = gui_path.to_str().expect("Failed to convert path to string");

    #[cfg(unix)] {
        match Command::new(gui_path_str).spawn() {
            Ok(_) => log::info!("Successfully spawned GameMon-gui."),
            Err(e) => {
                log::error!("Failed to spawn GameMon-gui: {}", e);
                exit(1);
            }
        }
    }

    #[cfg(windows)] {
        match config::run_windows_cmd(&gui_path_str) {
            Ok(_) => log::info!("{:?} executed successfully", &gui_path_str),
            Err(e) => log::error!("Failed to execute command '{}': {}", &gui_path_str, e),
        }
    }
}
