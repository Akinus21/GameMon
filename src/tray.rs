use libappindicator::AppIndicator;
use libappindicator::AppIndicatorStatus;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, *};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
#[cfg(windows)]
use tray_item::TrayItem;

use crate::config::CURRENT_VERSION;

pub fn spawn_tray(
    sender: mpsc::Sender<String>,
    title: String,
    icon_path: PathBuf,
    menu_var: Vec<(String, String)>, // Change &str to String for owned data
) {
    #[cfg(unix)]
    {
        // Linux/macOS solution using GTK and AppIndicator
        let application = gtk::Application::new(
            Some("com.example.trayapp"),
            gtk::gio::ApplicationFlags::FLAGS_NONE,
        );

        // Wrap the sender in an Arc<Mutex<>> to make it thread-safe and shareable
        let sender = Arc::new(Mutex::new(sender));

        // Clone `menu` so it moves into the closure
        let menu_clone = menu_var.clone(); 

        application.connect_activate(move |_| {
            // Build tray application
            let mut indicator = AppIndicator::new("Example", "applications-internet");
            indicator.set_status(AppIndicatorStatus::Active);
            indicator.set_title(&title);
            let icon = icon_path.to_str().unwrap();
            println!("DEBUG: Icon at {:?}", icon);
            indicator.set_icon(icon);

            // Build menu
            let mut new_menu = gtk::Menu::new();

            // Add app name label
                // Get current version from the package's version defined in Cargo.toml
            let app_name_item = gtk::MenuItem::with_label(&format!("GameMon v{}", CURRENT_VERSION.to_string()));
            new_menu.append(&app_name_item);

            // Add Separator
            let separator = gtk::SeparatorMenuItem::new();
            new_menu.append(&separator);


            // Add items from function call 
            for item in menu_clone.clone() { // Cloning the owned `menu` vector
                let mi = gtk::MenuItem::with_label(&item.0);
                // Clone the Arc<Mutex<Sender>> for the closure
                let sender_clone = Arc::clone(&sender);
                mi.connect_activate(move |_| {
                    if let Ok(sender) = sender_clone.lock() {
                        sender
                            .send(item.1.clone().to_string()) // Clone the String to send it
                            .expect("Failed to send message");
                    }
                });
                new_menu.append(&mi);
            }

            new_menu.show_all();
            indicator.set_menu(&mut new_menu);
        });

        application.run();
    }

    #[cfg(target_os = "windows")]
    {
        // Windows solution using tray-item crate

        // Load the icon image from file and get data
        let img = ImageReader::open(GAMEMON_LOGO.as_path()).unwrap().decode().unwrap();
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8(); // Convert to RGBA8
        let data = rgba.into_raw(); // Get raw pixel data
        let hicon = load_icon_from_png(&*GAMEMON_LOGO.to_string_lossy());

        // Create a TrayItem
        let mut tray = TrayItem::new(
            "GameMon",
            IconSource::RawIcon(hicon as isize),
        )
        .unwrap();

        // Create a menu for the tray
        tray.add_label("GameMon").unwrap();

        tray.inner_mut().add_separator().unwrap();

        // let mut menu = TrayItemMenu::new();

        for item in menu_var {
            let sender = sender.clone(); // Clone the sender before using it in the closure
            let _ = tray.add_menu_item(&item.0.clone(), move || {
                // Send the selected action through the sender
                sender.send(item.1.clone()).expect("Failed to send message");
            });
        }

        loop{
            thread::sleep(time::Duration::from_secs(5));
        }

    }
}

#[cfg(windows)]
use winresource::WindowsResource;
#[cfg(windows)]
use image::GenericImageView;
#[cfg(windows)]
use image::ImageReader;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::HICON;
#[cfg(windows)]
use windows_sys::Win32::Graphics::Gdi::{
    BITMAPV5HEADER, CreateDIBSection, DeleteObject, GetDC, ReleaseDC, RGBQUAD,
};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, ICONINFO};

#[cfg(windows)]
pub fn load_icon_from_png(file_path: &str) -> HICON {
    // Load the image
    let img = ImageReader::open(file_path).unwrap().decode().unwrap();
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();
    let raw_data = rgba.into_raw();

    unsafe {
        // Create a BITMAPV5HEADER
        let mut bi = BITMAPV5HEADER {
            bV5Size: std::mem::size_of::<BITMAPV5HEADER>() as u32,
            bV5Width: width as i32,
            bV5Height: -(height as i32), // Negative for top-down DIB
            bV5Planes: 1,
            bV5BitCount: 32,
            bV5Compression: 3, // BI_BITFIELDS
            bV5SizeImage: (width * height * 4) as u32,
            bV5RedMask: 0x00FF0000,
            bV5GreenMask: 0x0000FF00,
            bV5BlueMask: 0x000000FF,
            bV5AlphaMask: 0xFF000000,
            ..std::mem::zeroed()
        };

        // Get device context
        let hdc = GetDC(std::ptr::null_mut());


        // Create DIB section
        let mut bits: *mut c_void = ptr::null_mut();  // `bits` will hold the pointer to pixel data
        let hbitmap = CreateDIBSection(
            hdc,
            &bi as *const _ as *const _,
            0,
            &mut bits as *mut _ as *mut *mut c_void, // Correctly pass a mutable pointer to bits
            ptr::null_mut(),
            0
        );
        ReleaseDC(std::ptr::null_mut(), hdc);

        if hbitmap.is_null() {
            return std::ptr::null_mut();
        }

        // Copy image data to the HBITMAP memory
        ptr::copy_nonoverlapping(raw_data.as_ptr(), bits as *mut u8, raw_data.len());

        // Create an empty mask bitmap
        let mut mask_bits: *mut c_void = ptr::null_mut();  // Create a mutable pointer for mask bits
        let hmask = CreateDIBSection(
            hdc,
            &bi as *const _ as *const _,
            0,
            &mut mask_bits as *mut _ as *mut *mut c_void, // Correctly pass a mutable pointer to mask_bits
            ptr::null_mut(),
            0
        );
        if hmask.is_null() {
            DeleteObject(hbitmap as _);
            return std::ptr::null_mut();
        }

        // Create an ICONINFO structure
        let icon_info = ICONINFO {
            fIcon: 1, // 1 = Icon, 0 = Cursor
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hmask,
            hbmColor: hbitmap,
        };

        // Create icon
        let hicon = CreateIconIndirect(&icon_info);

        // Clean up bitmaps
        DeleteObject(hbitmap as _);
        DeleteObject(hmask as _);

        hicon
    }
}

