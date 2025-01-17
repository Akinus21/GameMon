// This file defines the main application structure, including methods for starting and stopping the monitoring of processes, as well as handling the execution of configured commands.

use std::{process::Command, sync::Arc};
use tokio::sync::Mutex;
use crate::config::{Config, Entry};

pub struct App {
    config: Arc<Mutex<Config>>,
    monitoring: bool,
}

impl App {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            monitoring: false,
        }
    }

    pub async fn start_monitoring(&mut self) {
        self.monitoring = true;
        // Logic to start monitoring processes
    }

    pub async fn stop_monitoring(&mut self) {
        self.monitoring = false;
        // Logic to stop monitoring processes
    }

    pub fn execute_start_commands(&self, entry: &Entry) {
        for command in &entry.start_commands {
            Command::new(command).spawn().ok();
        }
    }

    pub fn execute_end_commands(&self, entry: &Entry) {
        for command in &entry.end_commands {
            Command::new(command).spawn().ok();
        }
    }
}