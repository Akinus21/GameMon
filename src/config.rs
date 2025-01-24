use serde::{Serialize, Deserialize};
use toml::ser;
use std::fs;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error + Send>> {
        let data = fs::read_to_string(file_path)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        let config: Config = toml::from_str(&data)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        Ok(config)
    }

    // Use TOML to save the configuration to a file
    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let data = ser::to_string_pretty(self)?;
        fs::write(file_path, data)?;
        Ok(())
    }

    // Checks if config.toml exists in the current directory. If not, creates it.
    pub fn get_config_path() -> Result<String, Box<dyn Error>> {
        
        //Define config directory
        let config_dir = dirs::config_dir().unwrap().join("gamemon"); 

        // Check if directory exists, if not, create it
        if !config_dir.exists() {
            match std::fs::create_dir(&config_dir){
                Ok(_) => println!("Configuration directory created successfully!"),
                Err(e) => println!("Error creating configuration directory: {:?}", e),
            }
        };

        // Define the config file path
        let config_file = config_dir.join("config.toml");

        // Check if the file exists
        if !config_file.exists() {
            // If the file doesn't exist, create it with an empty config
            let default_config = Config { entries: Vec::new() };
            default_config.save_to_file(config_file.to_str().unwrap())?;
        }

        // Return the file path as a string
        Ok(config_file.to_str().unwrap().to_string())
    }
}