use std::env;
use gtk4;
use image::io::Reader as ImageReader;

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

// Initialize GTK
pub fn initialize_gtk() -> Result<(), String> {
    // Check the operating system
    if cfg!(target_os = "macos") {
        return Err("error, macos".to_string());
    } else if cfg!(target_os = "windows") {
        return Err("error, windows".to_string());
    } else if cfg!(target_os = "linux") {
        // Attempt to initialize GTK
        let mut attempts = 0;
        if let Err(err) = gtk_init() {
            println!("Failed to initialize GTK. Error: {}", err);
            attempts += 1;
            // If GTK fails, check the display server
            let display_server = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string());

            if display_server == "unknown" {
                return Err(format!("error, GTK could not be initialized: {}", err));
            }

            // Handle specific display server errors
            if display_server == "wayland" {
                // If the display server is Wayland, check the GDK_BACKEND
                
                while attempts < 4 {
                    match check_and_set_gdk_backend("wayland") {
                        Ok(()) => {
                            // Retry to initialize GTK after setting GDK_BACKEND
                            println!("GDK_BACKEND is set to wayland... Retrying. Attempt: {}", attempts + 1);
                            if let Err(err) = gtk_init() {
                                if attempts == 3 {
                                    return Err(format!("Failed to initialize GTK after {} attempts. Error: {}", attempts + 1, err));
                                }
                                attempts += 1;
                            } else {
                                return Ok(());
                            }
                        },
                        Err(err) => return Err(err),
                    }
                }
            } else if display_server == "x11" {
                println!("X11 detected. Taking specific action for X11.");
                // Add your X11-specific logic here
            } else {
                return Err(format!("error, {}", display_server));
            }
        }
        // GTK initialized successfully
        Ok(())
    } else {
        Err("error, unknown operating system".to_string())
    }
}

// Separate function to check and set GDK_BACKEND
fn check_and_set_gdk_backend(expected_backend: &str) -> Result<(), String> {
    match env::var("GDK_BACKEND") {
        Ok(value) => {
            if value != expected_backend {
                println!("GDK_BACKEND is not set to {}. Setting GDK_BACKEND to '{}'.", expected_backend, expected_backend);
                env::set_var("GDK_BACKEND", expected_backend);
                env::set_var("GTK_MODULES", "gdk-pixbuf");
                Ok(())
            } else {
                Ok(())
            }
        },
        Err(_) => {
            println!("GDK_BACKEND is not set. Setting GDK_BACKEND to '{}'.", expected_backend);
            env::set_var("GDK_BACKEND", expected_backend);
            Ok(())
        }
    }
}

fn gtk_init() -> Result<(), glib::BoolError> {
    gtk4::init()
}

