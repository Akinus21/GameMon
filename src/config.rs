use serde::{Serialize, Deserialize};
use toml::ser;
use std::fs;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(file_path)?;
        let config: Config = toml::from_str(&data)?;
        Ok(config)
    }

    // Use TOML to save the configuration to a file
    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let data = ser::to_string_pretty(self)?;
        fs::write(file_path, data)?;
        Ok(())
    }
}
