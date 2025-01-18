use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use std::{collections::HashSet, process::Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::config::Config;
use notify_rust::Notification;

pub fn watchdog() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting watchdog...");
    let notification = notify_rust::Notification::new()
        .summary("GameMon")
        .body("GameMon is running.")
        .icon("dialog-information")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show();


    // Keep track of active processes
    let active_processes: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    
    // System info setup
    let refresh_kind = RefreshKind::everything();
    let mut sys = System::new_with_specifics(refresh_kind);

    loop {
        // Reload configuration on each check
        let config_path = &*Config::get_config_path().unwrap(); // Adjust the path as needed
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
            let active_processes_clone = Arc::clone(&active_processes);
            let active_processes = active_processes.lock().unwrap();
            if active_processes.contains(&entry.executable) {
                continue;
            }

            // Look for the executable in the running processes
            if let Some((pid, _proc)) = find_process(&entry.executable, &sys) {
                println!("Detected process '{}' with PID {}", entry.executable, pid);
                let _notification = notify_rust::Notification::new()
                    .summary("GameMon")
                    .body(format!("Detected process '{}' with PID {}", entry.executable, pid).as_str())
                    .icon("dialog-information")
                    .timeout(notify_rust::Timeout::Milliseconds(5000))
                    .show();

                // Mark the process as active
                drop(active_processes); // Release the lock before calling spawn
                active_processes_clone.lock().unwrap().insert(entry.executable.clone());

                // Execute start commands
                run_commands(&entry.start_commands);

                // Monitor the process
                let executable_name = entry.executable.clone();
                let end_commands = entry.end_commands.clone();
                monitor_process(pid, executable_name, end_commands, active_processes_clone);
                println!("Continuing to monitor system processes...");
            }
        }

        // Sleep for a short interval before the next check
        thread::sleep(Duration::from_secs(5));
    }
}

// Find a process by its executable name
fn find_process<'a>(executable: &str, sys: &'a System) -> Option<(sysinfo::Pid, &'a sysinfo::Process)> {
    sys.processes()
        .iter()
        .find_map(|(pid, proc)| {
            if proc.name() == executable {
                Some((*pid, proc))
            } else {
                None
            }
        })
}

// Run a list of commands
fn run_commands(commands: &[String]) {
    for cmd in commands {
        println!("Running command: {}", cmd);
        let _notification = notify_rust::Notification::new()
            .summary("GameMon")
            .body(format!("Running command: {}", cmd).as_str())
            .icon("dialog-information")
            .timeout(notify_rust::Timeout::Milliseconds(5000))
            .show();

        if let Err(err) = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .and_then(|mut child| child.wait())
        {
            eprintln!("Failed to execute command '{}': {}", cmd, err);
            let _notification = notify_rust::Notification::new()
                .summary("GameMon")
                .body(format!("Failed to execute command '{}': {}", cmd, err).as_str())
                .icon("dialog-information")
                .timeout(notify_rust::Timeout::Milliseconds(5000))
                .show();
        }
    }
}

// Monitor a process and execute end commands when it exits
fn monitor_process(
    pid: sysinfo::Pid,
    executable_name: String,
    end_commands: Vec<String>,
    active_processes: Arc<Mutex<HashSet<String>>>,
) {
    println!("Monitoring process '{}' with PID {}", executable_name, pid);

    thread::spawn(move || {
        let mut sys = System::new_with_specifics(
            RefreshKind::default().with_processes(ProcessRefreshKind::everything()),
        );

        loop {
            // Refresh system to check if the process is still running
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing()
                    .with_memory()
                    .with_cpu()
                    .with_disk_usage()
                    .with_exe(UpdateKind::OnlyIfNotSet),
            );

            if sys.process(pid).is_none() {
                println!("Process '{}' exited. Running end commands.", executable_name);

                // Execute end commands
                run_commands(&end_commands);

                // Remove the process from active monitoring
                let mut active_processes = active_processes.lock().unwrap();
                active_processes.remove(&executable_name);
                println!("Removed '{}' from active monitoring.", executable_name);

                // Allow for re-detection if the process starts again
                break; // Exit the thread to recheck the process in the main loop
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}
