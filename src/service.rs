// use sysinfo::{System, Pid};
use std::{process::Command, sync::{mpsc, Arc}, thread};
use std::time::Duration;
use crate::config::Config;
use dashmap::DashMap;
use crate::config::{GAMEMON_CONFIG_FILE, check_for_updates};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::collections::HashSet;

pub fn watchdog() -> Result<(), Box<dyn std::error::Error + Send>> {
    log::info!("Starting watchdog...");

    // Track active monitored executables and their stop signal senders
    let active_processes: Arc<DashMap<String, Option<mpsc::Sender<()>>>> = Arc::new(DashMap::new());

    let mut update_timer = 600;

    loop {
        if update_timer >= 600 {
            match check_for_updates() {
                Ok(_) => log::info!("Check for updates complete!"),
                Err(e) => log::error!("Error checking for updates: {:?}\n", e),
            }
            update_timer = 0;
        }

        // Reload config each loop
        let config_path = &GAMEMON_CONFIG_FILE.to_string_lossy();
        let config = Config::load_from_file(config_path)?;
        let entries = config.entries;

        // Get ps aux output once
        let ps_output = get_ps_aux_output().unwrap_or_default();

        // Collect currently running executables by checking if the exec name appears in ps output
        let mut running_executables = HashSet::new();

        for entry in &entries {
            if is_executable_running(&entry.executable, &ps_output) {
                // log::info!("✅ Match found for {}", &entry.executable);
                running_executables.insert(entry.executable.clone());
            } else {
                // log::info!("❌ No match found for {}", &entry.executable);
            }
        }

        // For each entry, manage starting or stopping monitoring threads
        for entry in &entries {
            let is_running = running_executables.contains(&entry.executable);
            let is_monitored = active_processes.contains_key(&entry.executable);

            if is_running && !is_monitored {
                log::info!("Detected process '{}' is running, starting monitor...", &entry.executable);

                let (tx, rx) = mpsc::channel();
                active_processes.insert(entry.executable.clone(), Some(tx));

                let executable_name = entry.executable.clone();
                let start_commands = entry.start_commands.clone();
                let end_commands = entry.end_commands.clone();
                let active_processes_clone = Arc::clone(&active_processes);

                thread::spawn(move || {
                    monitor_process(executable_name, start_commands, end_commands, active_processes_clone, rx);
                });
            } else if !is_running && is_monitored {
                if let Some(mut sender) = active_processes.get_mut(&entry.executable) {
                    if let Some(tx) = sender.take() {
                        log::info!("Process '{}' stopped, sending termination signal...", &entry.executable);
                        tx.send(()).unwrap_or_else(|_| {
                            log::info!("Failed to send termination signal to '{}', likely already closed.", &entry.executable);
                        });
                    }
                }
            }
        }

        thread::sleep(Duration::from_secs(5));
        update_timer += 5;
    }
}

/// Runs `ps aux` and returns the output as a String
fn get_ps_aux_output() -> Option<String> {
    Command::new("sh")
        .arg("-c")
        .arg("ps aux")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
}

/// Returns true if the executable name is found in the `ps aux` output (case-insensitive),
/// excluding lines containing 'grep' to avoid false positives.
fn is_executable_running(executable: &str, ps_output: &str) -> bool {
    let exec_lower = executable.to_lowercase();
    ps_output
        .to_lowercase()
        .lines()
        .any(|line| !line.contains("grep") && line.contains(&exec_lower))
}

// Get the PID for a given executable name
// #[cfg(unix)]
// fn get_pid_for_executable(executable: &str, sys: &System) -> Option<Pid> {
//     let target = executable.to_ascii_lowercase();

//     for (pid, proc) in sys.processes() {
//         // Convert OsStr process name to string lossily
//         let proc_name_cow: Cow<str> = proc.name().to_string_lossy();
//         let proc_name = proc_name_cow.to_ascii_lowercase();


//         if proc_name == target {
//             log::info!("✅ Found '{}' as PID {}", target, pid);
//             return Some(*pid);
//         }
//     }

//     None
// }

#[cfg(windows)]
fn get_pid_for_executable(_executable: &str, _sys: &System) -> Option<Pid> {
    // TODO: Implement Windows process lookup here
    log::info!("Windows version of get_pid_for_executable not implemented yet.");
    None
}


// Monitor a process and execute end commands when it exits
fn monitor_process(
    executable_name: String,
    start_commands: Vec<String>,
    end_commands: Vec<String>,
    active_processes: Arc<DashMap<String, Option<mpsc::Sender<()>>>>,
    rx: mpsc::Receiver<()>,
) {
    if let Err(e) = run_commands(&start_commands) {
        log::error!("Error running start commands: {}", e);
    }

    log::info!("Monitoring process '{}'. Waiting for termination signal...", executable_name);

    loop {
        match rx.recv() {
            Ok(_) => {
                log::info!("Received termination signal for process '{}'.", executable_name);
                break;
            }
            Err(_) => continue,
        }
    }

    if let Err(e) = run_commands(&end_commands) {
        log::error!("Error running end commands: {}", e);
    }

    active_processes.remove(&executable_name);
    log::info!("Removed '{}' from active monitoring.", executable_name);
}

// Run a list of commands
pub fn run_commands(commands: &[String]) -> Result<(), Box<dyn std::error::Error + Send>> {
    for cmd in commands {
        let cmd_string = cmd.to_string();

        #[cfg(windows)]
        {
            match config::run_windows_cmd(&cmd_string) {
                Ok(_) => log::info!("{:?} executed successfully", &cmd_string),
                Err(e) => log::error!("Failed to execute command '{}': {}", &cmd_string, e),
            }
        }

        #[cfg(unix)]
        {
            run_shell_command(&cmd_string);
        }
    }
    Ok(())
}

use std::process::Output;
use std::io;

/// Executes a shell command string and prints stdout/stderr.
/// Returns true if the command succeeded.
pub fn run_shell_command(command_str: &str) -> bool {
    if command_str.trim().is_empty() {
        log::error!("Empty command string; nothing to execute.");
        return false;
    }

    log::info!("🟢 Running command: {}", command_str);

    let output: io::Result<Output> = Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output();

    match output {
        Ok(output) => {
            if !output.stdout.is_empty() {
                log::info!("✅ STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::error!("⚠️ STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
            }

            if output.status.success() {
                log::info!("✅ Command executed successfully.");
                true
            } else {
                log::error!("❌ Command exited with status: {}", output.status);
                false
            }
        }
        Err(e) => {
            log::error!("❌ Failed to execute command '{}': {}", command_str, e);
            false
        }
    }
}
