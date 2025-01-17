use std::env;
use std::process::exit;

// use util::gtk_init;
use crate::gui::Counter;
use tray_icon::{TrayIconEvent, menu::{MenuEvent}};
use crate::tray::Tray;

mod gui;
mod config;
mod tray;
mod util;
 
pub fn main() -> iced::Result {
    iced::run("A cool counter", Counter::update, Counter::view)
}