use std::{env, error::Error, io::Write};
use image::ImageReader;
use self_update::{backends::github::Update, cargo_crate_version, self_replace, ArchiveKind};
use notify_rust::Notification;

pub struct CustomIcon{
    file_path: String,
}

impl CustomIcon {
    pub fn new(file_path: &str) -> Self {
        CustomIcon {
            file_path: file_path.to_string(),
        }
    }

    pub fn get_icon(&self) -> tray_icon::Icon {
        let img = ImageReader::open(&self.file_path)
            .expect("Failed to open tray icon file")
            .decode()
            .expect("Failed to decode PNG image");

        // Convert the image to RGBA and create the icon
        let rgba = img.to_rgba8();
        let width = img.width() as u32;
        let height = img.height() as u32;
        tray_icon::Icon::from_rgba(rgba.into_raw(), width, height)
            .expect("Failed to create icon from decoded image")
    }
}

pub fn update() -> Result<(), Box<dyn Error>> {
    println!("Checking for updates");
    // Configure and initiate the update process
    let status = Update::configure()
        .repo_owner("Akinus21") // Replace with your GitHub username
        .repo_name("GameMon")  // Replace with your GitHub repo name
        .bin_name("GameMon")   // The name of your executable
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    // Log the successful update
    println!("Updated to version: {}", status.version());
    Ok(())
}



