use serde::{Serialize, Deserialize};
use toml::ser;
use std::{fs, io};
use std::error::Error;
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub static GAMEMON_DIR: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming")).join("gamemon")
    } else {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share")).join("gamemon")
    }
});

pub static GAMEMON_RESOURCE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_DIR.join("resources")
    } else {
        GAMEMON_DIR.join("resources")
    }
});

pub static GAMEMON_ICON: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_RESOURCE_DIR.join("gamemon.ico")
    } else {
        GAMEMON_RESOURCE_DIR.join("gamemon.png")
    }
});

pub static GAMEMON_LOGO: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_RESOURCE_DIR.join("gamemon.png")
    } else {
        GAMEMON_RESOURCE_DIR.join("gamemon.png")
    }
});

pub static GAMEMON_BIN_DIR: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_RESOURCE_DIR.join("\\bin")
    } else {
        GAMEMON_RESOURCE_DIR.join("/bin")
    }
});


pub static GAMEMON_EXECUTABLE: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_DIR.join("GameMon.exe")
    } else {
        GAMEMON_DIR.join("GameMon")
    }
});

pub static GAMEMON_UPDATER: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_DIR.join("GameMon-update.exe")
    } else {
        GAMEMON_DIR.join("GameMon-update")
    }
});

pub static GAMEMON_GUI_EXECUTABLE: Lazy<PathBuf> = Lazy::new(|| {
    if cfg!(target_os = "windows") {
        GAMEMON_DIR.join("GameMon-gui.exe")
    } else {
        GAMEMON_DIR.join("GameMon-gui")
    }
});

pub static GAMEMON_CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_dir().unwrap().join("gamemon")
});

pub static GAMEMON_CONFIG_FILE: Lazy<PathBuf> = Lazy::new(|| {
    GAMEMON_CONFIG_DIR.join("config.toml")
});

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub game_name: String,
    pub executable: String,
    pub start_commands: Vec<String>,
    pub end_commands: Vec<String>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            game_name: String::new(),
            executable: String::new(),
            start_commands: Vec::new(),
            end_commands: Vec::new(),
        }
    }
}

impl Config {
    // Use TOML to load the configuration from a file
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn Error + Send>> {
        let data = fs::read_to_string(file_path)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        if data.trim().is_empty() {
            println!("Config file is empty. Initializing a new empty config.");
            return Ok(Config::default());
        }

        let config: Config = toml::from_str(&data)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
        
        Ok(config)
    }

    // Use TOML to save the configuration to a file
    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let data = ser::to_string_pretty(self)?;
        fs::write(file_path, data)?;
        Ok(())
    }
}

pub fn ensure_paths_exist() -> io::Result<()> {
    let paths_to_create = [
        &*GAMEMON_DIR,
        &*GAMEMON_RESOURCE_DIR,
        &*GAMEMON_CONFIG_DIR,
    ];

    for path in paths_to_create {
        if !path.exists() {
            println!("Creating directory: {}", path.display());
            fs::create_dir_all(path)?;
        }
    }

    let files_to_create = [
        &*GAMEMON_CONFIG_FILE
    ];

    for file in files_to_create {
        if !file.exists() {
            println!("Creating empty file: {}", file.display());
            fs::File::create(file)?;
        }
    }

    Ok(())
}