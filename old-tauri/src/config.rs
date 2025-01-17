use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub entries: Vec<Entry>,
    pub config_file_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub executable: String,
    pub start_commands: Vec<String>,
    pub end_commands: Vec<String>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            executable: String::new(),
            start_commands: Vec::new(),
            end_commands: Vec::new(),
        }
    }
}

impl Config {
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(file_path)?;
        let config: Config = serde_json::from_str(&data)?;
        Ok(config)
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(file_path, data)?;
        Ok(())
    }
}