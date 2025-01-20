use std::env;
use image::ImageReader;

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

