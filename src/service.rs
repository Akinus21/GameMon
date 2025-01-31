use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use std::{process::{Command, Stdio}, sync::{mpsc, Arc}, thread};
use std::time::Duration;
use crate::config::{self, Config};
use dashmap::DashMap;
use crate::config::{GAMEMON_CONFIG_FILE, GAMEMON_UPDATER, check_for_updates};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn watchdog() -> Result<(), Box<dyn std::error::Error + Send>> {
    println!("Starting watchdog...");

    // Keep track of active processes using DashMap with a sender to notify monitor threads
    let active_processes: Arc<DashMap<String, Option<mpsc::Sender<()>>>> = Arc::new(DashMap::new());

    // System info setup
    let refresh_kind = RefreshKind::everything();
    let mut sys = System::new_with_specifics(refresh_kind);

    let mut update_timer = 600;
    
    loop {
        if update_timer >= 600 {
            //run updater every 10 minutes

            match check_for_updates() {
                Ok(_) => println!("Check for updates complete!"),
                Err(e) => eprintln!("Error checking for updates: {:?}\n", e),
            }
            update_timer = 0;

        }
        
        // Reload configuration on each check
        let config_path = &GAMEMON_CONFIG_FILE.to_string_lossy();
        let config = Config::load_from_file(config_path)?;
        let entries = config.entries;
    
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_memory()
                .with_cpu()
                .with_disk_usage()
                .with_exe(UpdateKind::OnlyIfNotSet),
        );
    
        for entry in &entries {
            // Check if the executable is already being monitored
            let pid = get_pid_for_executable(&entry.executable, &sys);
    
            if let Some(pid) = pid {
                if !active_processes.contains_key(&entry.executable) {
                    // If not monitored, create a new channel and monitoring thread
                    println!("Detected process '{}' with PID {}", entry.executable, pid);
    
                    let (tx, rx) = mpsc::channel();
                    active_processes.insert(entry.executable.clone(), Some(tx)); // Persist the sender
    
                    let executable_name = entry.executable.clone();
                    let start_commands = entry.start_commands.clone();
                    let end_commands = entry.end_commands.clone();
                    let active_processes_clone = Arc::clone(&active_processes);
    
                    thread::spawn(move || {
                        monitor_process(executable_name, start_commands, end_commands, active_processes_clone, rx);
                    });
                }
            } else {
                // Process is not running, send stop signal to monitoring thread
                if let Some(mut sender) = active_processes.get_mut(&entry.executable) {
                    if let Some(tx) = sender.take() {
                        println!("Process '{}' has stopped. Sending termination signal.", entry.executable);
                        tx.send(()).unwrap_or_else(|_| {
                            println!("Failed to send termination signal to '{}', likely already closed.", entry.executable);
                        });
                    }
                }
            }
        }
    
        // Sleep for a short interval before the next check
        thread::sleep(Duration::from_secs(5));
        update_timer += 5;
    }
}    

// Get the PID for a given executable name
fn get_pid_for_executable(executable: &str, sys: &System) -> Option<sysinfo::Pid> {
    sys.processes().iter().find_map(|(pid, proc)| {
        if proc.name() == executable {
            Some(*pid)
        } else {
            None
        }
    })
}

// Monitor a process and execute end commands when it exits
fn monitor_process(
    executable_name: String,
    start_commands: Vec<String>,
    end_commands: Vec<String>,
    active_processes: Arc<DashMap<String, Option<mpsc::Sender<()>>>>,
    rx: mpsc::Receiver<()>,
) {
    // Execute start commands
    if let Err(e) = run_commands(&start_commands) {
        eprintln!("Error running start commands: {}", e);
    }

    println!("Monitoring process '{}'. Waiting for termination signal...", executable_name);

    // Loop until we successfully receive a message
    loop {
        match rx.recv() {
            Ok(_) => {
                // We received the signal, break out of the loop
                println!("Received termination signal for process '{}'.", executable_name);
                break;
            }
            Err(_e) => {
                continue;
            }
        }
    }

    // Execute end commands
    if let Err(e) = run_commands(&end_commands) {
        eprintln!("Error running end commands: {}", e);
    }

    // Remove the process from active monitoring
    active_processes.remove(&executable_name);
    println!("Removed '{}' from active monitoring.", executable_name);
}


// Run a list of commands
fn run_commands(commands: &[String]) -> Result<(), Box<dyn std::error::Error + Send>> {
    for cmd in commands {
        println!("Running command: {}", cmd);

        let cmd_string = cmd.to_string(); // Ensure `cmd` is owned, not borrowed

        
        #[cfg(windows)]
        {
            // On Windows, use "cmd"

            match config::run_windows_cmd(&cmd_string) {
                Ok(_) => println!("{:?} executed successfully", &cmd_string),
                Err(e) => eprintln!("Failed to execute command '{}': {}", &cmd_string, e),
            }
        }

        #[cfg(unix)]
        {
            // On Linux, use "sh -c"
            let mut command = Command::new("sh"); // Default to "sh" for Linux
            command.arg("-c").arg(cmd_string);
            if let Err(err) = command.spawn().and_then(|mut child| child.wait()) {
                eprintln!("Failed to execute command '{}': {}", cmd, err);
            };
        }
    }
    Ok(())
}
