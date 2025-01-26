//! This crate provides the core functionality for GameMon,
//! including the GUI, tray, configuration management, and service logic.

// Declare the modules for the crate
pub mod app;
pub mod config;
pub mod tray;
pub mod service;

// Optionally, re-export commonly used items for convenience
// pub use mods::app;
// pub use mods::config;
// pub use mods::tray;
// pub use mods::watchdog;