use iced::alignment::Horizontal::{Left, Center as xCenter};
use iced::alignment::Vertical::Bottom;
use iced::widget::{
    button,
    column,
    container,
    pick_list,
    row,
    text,
    text_editor,
    text_input,
    Row,
    vertical_rule,
    vertical_space,
    horizontal_space,
};
use iced::Length::Fill;
use iced::Theme;
use serde::{Serialize, Deserialize};
use toml::ser;
use std::{env, fs};
use std::error::Error;
use dirs;

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
        // Get the current working directory
        let current_dir = std::env::current_dir()?;

        // Define the config file path
        let config_path = current_dir.join("config.toml");

        // Check if the file exists
        if !config_path.exists() {
            // If the file doesn't exist, create it with an empty config
            let default_config = Config { entries: Vec::new() };
            default_config.save_to_file(config_path.to_str().unwrap())?;
        }

        // Return the file path as a string
        Ok(config_path.to_str().unwrap().to_string())
    }
}

// #[derive(Default)]
struct Gui {
    game_name_field: String,      // List of game names
    selected_game_name: Option<String>, // Currently selected game name
    selected_game_entry: Option<Entry>, // Currently selected game entry
    game_executable_field: String, // List of game executables
    start_commands_field: text_editor::Content,  // List of start commands
    end_commands_field: text_editor::Content,    // List of end commands
    game_entries: Vec<Entry>,
    game_executables: Vec<String>,
    game_start_commands: Vec<String>,
    game_end_commands: Vec<String>,
    game_names: Vec<String>,
    entry_changed: bool,
}

impl Default for Gui{
    fn default() -> Self {
        let game_entries = Config::load_from_file(&*Config::get_config_path().unwrap()).unwrap().entries;
        let mut game_executables = Vec::new();
        let mut game_start_commands = Vec::new();
        let mut game_end_commands = Vec::new();
        let mut game_names = Vec::new();

        for entry in game_entries.clone() {
            game_names.push(entry.game_name.clone());
            game_executables.push(entry.executable.clone());
            game_start_commands.push(
                entry.start_commands.join("\n").trim_ascii_end().to_string()  // Join with newlines
            );
            
            game_end_commands.push(
                entry.end_commands.join("\n").trim_ascii_end().to_string()      // Join with newlines
            );
        }

        Self {
            game_name_field: "Enter game name...".to_string(),
            selected_game_name: None,
            selected_game_entry: None,
            game_executable_field: "Enter game executable...".to_string(),
            start_commands_field: text_editor::Content::with_text("Enter start commands..."),
            end_commands_field: text_editor::Content::with_text("Enter end commands..."),
            game_entries: game_entries,
            game_executables: game_executables,
            game_start_commands: game_start_commands,
            game_end_commands: game_end_commands,
            game_names: game_names,
            entry_changed: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    GameNameChanged(String),
    GameExectuableChanged(String),
    GameSelected(String),
    StartCommandsChanged(text_editor::Action),
    EndCommandsChanged(text_editor::Action),
    ViewLoading,
    NewEntry,
    SaveEntry,
    RemoveEntry,
}

impl Gui {
    fn theme(&self) -> Theme {
        Theme::Dracula
    }
    fn update(&mut self, message: Message) {
        
        match message {
            Message::GameNameChanged(content) => {
                self.game_name_field = content.clone();
                self.entry_changed = true;
                // println!("Game Name changed: {}", content);
            }
            Message::GameExectuableChanged(content) => {
                self.game_executable_field = content.clone();
                self.entry_changed = true;
                // println!("Game Executable changed: {}", content);
            }
            Message::StartCommandsChanged(action) => {
                self.start_commands_field.perform(action);
                self.entry_changed = true;
            }
            Message::EndCommandsChanged(action) => {
                self.end_commands_field.perform(action);
                self.entry_changed = true;
            }
            Message::GameSelected(game_name) => {
                self.selected_game_name = Some(game_name.clone());
                
                // Save the current entry before switching if the game name field is not empty
                if self.entry_changed {
                    self.save_current_entry();
                }
                
                let entries = Config::load_from_file(&*Config::get_config_path().unwrap()).unwrap().entries;
                let selected_entry = entries.iter().find(|entry| entry.game_name == game_name).unwrap();
                self.selected_game_entry = Some(selected_entry.clone());

                self.game_name_field = selected_entry.game_name.clone();
                self.game_executable_field = selected_entry.executable.clone();
                self.start_commands_field = text_editor::Content::with_text(&selected_entry.start_commands.join("\n"));
                self.end_commands_field = text_editor::Content::with_text(&selected_entry.end_commands.join("\n"));
                self.entry_changed = false;
                // println!("Game selected: {}", game_name);
            }
            Message::NewEntry => {
                if self.entry_changed {
                    self.save_current_entry();
                }
                self.game_name_field = "Enter game name...".to_string();
                self.game_executable_field = "Enter game executable...".to_string();
                self.start_commands_field = text_editor::Content::with_text("Enter start commands...");
                self.end_commands_field = text_editor::Content::with_text("Enter end commands...");
                self.entry_changed = false;
                // println!("New entry button clicked");
            }
            Message::SaveEntry => {
                // println!("Save entry button clicked");

                self.save_current_entry();

            }
            Message::RemoveEntry => {
                println!("Remove entry button clicked");

                let mut entries = Config::load_from_file(&*Config::get_config_path().unwrap()).unwrap().entries;
                if let Some(index) = entries.iter().position(|entry| entry.game_name == self.game_name_field){
                    
                    println!("Entry exists at index {}", index);

                    entries.remove(index);

                    self.game_entries = entries.clone();
                    let new_config = Config { entries: entries.clone() };
                    self.refresh_stored_config(new_config.clone());
                    Config::save_to_file(&new_config.clone(), &*Config::get_config_path().unwrap()).unwrap();
                
                    self.game_name_field = "Enter game name...".to_string();
                    self.game_executable_field = "Enter game executable...".to_string();
                    self.start_commands_field = text_editor::Content::with_text("Enter start commands...");
                    self.end_commands_field = text_editor::Content::with_text("Enter end commands...");

                    
                } else {
                    println!("Entry does not exist");
                }
            }
            Message::ViewLoading => {
                
                let config_path = &*Config::get_config_path().unwrap();
                let config = match Config::load_from_file(config_path) {
                    Ok(config) => config,
                    Err(_) => return (), // Handle the error appropriately
                };
                
                self.refresh_stored_config(config);
            }
        }
    }

    fn view(&self) -> Row<Message> {
    
        // Define placeholders for the text editors
        // let start_commands_placeholder = self.game_start_commands[0].to_string();
        // let end_commands_placeholder = self.game_end_commands[0].to_string();

        // Definte left container
        let left_container = container(
            column![
                // list picker
                pick_list(
                    self.game_names.clone(),
                    self.selected_game_name.clone(),
                    Message::GameSelected,
                )
                .placeholder("Select a game...")
                .on_open(Message::ViewLoading),

                // Vertical space
                container("").height(Fill),

            ]
            .padding(20)
            .align_x(xCenter)
        )
        .padding(10)
        .align_x(Left);

        //define right container
        let right_container = container(
            column![

                row![
                    // Game Name label and field
                    column![
                        vertical_space().height(10),
                        text("Game Name:").align_x(Left).size(16).align_y(Bottom),
                    ],
                    
                    horizontal_space().width(10),

                    text_input(
                        // &self.game_names[0].clone(),
                        "Game name...",
                        &self.game_name_field,
                    )
                    .padding(10)
                    .size(16)
                    .on_input(Message::GameNameChanged),
                    
                    // Horizontal space
                    horizontal_space().width(10),

                    // Game Executable label and field
                    column![
                        vertical_space().height(10),
                        text("Game Executable:").align_x(Left).size(16).align_y(Bottom),
                    ],

                    horizontal_space().width(10),
                    
                    text_input(
                        // &self.game_executables[0].clone(),
                        "Game executable...",
                        &self.game_executable_field,
                    ).padding(10)
                    .size(16)
                    .on_input(Message::GameExectuableChanged),

                    horizontal_space().width(10),
                ],

                // Vertical space
                vertical_space().height(10),

                // Start Commands label and field
                text("Start Commands:").align_x(Left),
                text_editor(&self.start_commands_field)
                    // .placeholder(start_commands_placeholder)
                    .placeholder("Start commands...")
                    .on_action(Message::StartCommandsChanged)
                    .size(14),

                // Vertical space
                vertical_space().height(10),
                
                // End Commands label and field
                text("End Commands:").align_x(Left),
                text_editor(&self.end_commands_field)
                    // .placeholder(end_commands_placeholder)
                    .placeholder("End commands...")
                    .on_action(Message::EndCommandsChanged)
                    .size(14),

                // Vertical space
                vertical_space().height(Fill),

                row![
                    // new entry button
                    button("New Entry")
                        .on_press(Message::NewEntry)
                        .padding(10),

                    // Horizontal space
                    horizontal_space().width(10),

                    // remove entry button
                    button("Remove Entry")
                        .on_press(Message::RemoveEntry)
                        .padding(10),

                    // Horizontal space
                    horizontal_space().width(10),

                    // Save Entry button
                    button("Save Entry")
                        .on_press(Message::SaveEntry)
                        .padding(10),
                    
                ]
                

            ]
            .padding(20)
            .align_x(Left)
        );

        row![
            // Add the container for the left column
            left_container,

            // Add a vertical rule to separate the pick list from the text input
            vertical_rule(10),

            // Add the container for the right column
            right_container,
        ]
    }

    fn refresh_stored_config(&mut self, config: Config) {
        self.game_entries = config.entries.clone();
        self.game_executables = Vec::new();
        self.game_start_commands = Vec::new();
        self.game_end_commands = Vec::new();
        self.game_names = Vec::new();

        for entry in self.game_entries.clone() {
            self.game_names.push(entry.game_name.clone());
            self.game_executables.push(entry.executable.clone());
            self.game_start_commands.push(
                entry.start_commands.join("\n").trim_ascii_end().to_string()  // Join with newlines
            );
            
            self.game_end_commands.push(
                entry.end_commands.join("\n").trim_ascii_end().to_string()      // Join with newlines
            );
        }
    }

    fn save_current_entry(&mut self) {
        let mut entries = Config::load_from_file(&*Config::get_config_path().unwrap()).unwrap().entries;
        if let Some(index) = entries.iter().position(|entry| entry.game_name == self.game_name_field){
            
            println!("Entry already exists at index {}", index);

            entries[index].game_name = self.game_name_field.clone();
            entries[index].executable = self.game_executable_field.clone();
            entries[index].start_commands = self.start_commands_field.text().split("\n").map(|s| s.to_string()).collect();
            entries[index].end_commands = self.end_commands_field.text().split("\n").map(|s| s.to_string()).collect();

            self.game_entries = entries.clone();
            let new_config = Config { entries: entries.clone() };
            self.refresh_stored_config(new_config.clone());
            Config::save_to_file(&new_config.clone(), &*Config::get_config_path().unwrap()).unwrap();
        

        } else {
            
            println!("Entry does not exist");

            let new_entry = Entry {
                game_name: self.game_name_field.clone(),
                executable: self.game_executable_field.clone(),
                start_commands: self.start_commands_field.text().split("\n").map(|s| s.to_string()).collect(),
                end_commands: self.end_commands_field.text().split("\n").map(|s| s.to_string()).collect(),
            };

            entries.push(new_entry);
            self.game_entries = entries.clone();
            let new_config = Config { entries: entries.clone() };
            self.refresh_stored_config(new_config.clone());
            Config::save_to_file(&new_config.clone(), &*Config::get_config_path().unwrap()).unwrap();
        
        }
        self.entry_changed = false;
    }
}

pub fn main() -> iced::Result {

    //run updater
    let _child = std::process::Command::new("./GameMon-update")
        .spawn();

    // Check the OS and set the directory accordingly
    if cfg!(target_os = "linux") {
        let dir_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("gamemon");

        // Check if the directory exists
        if !dir_path.exists() {
            println!("Home directory does not exist. Creating it...");
            if let Err(e) = fs::create_dir_all(&dir_path) {
                eprintln!("Failed to create directory: {}", e);
            } else {
                println!("Directory created at {:?}", dir_path);
            }
        } else {
            println!("Directory already exists at {:?}", dir_path);
        }

        env::set_current_dir(dir_path ).expect("Failed to change directory");
        env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");


    } else if cfg!(target_os = "windows") {
        println!("Running on Windows");
        // Windows-specific actions
    } else if cfg!(target_os = "macos") {
        println!("Running on macOS");
        // macOS-specific actions
    } else {
        println!("Running on an unknown OS");
        // Fallback actions
    }

    // Start the GUI application
    iced::application("GameMon", Gui::update, Gui::view).theme(Gui::theme).run()
    // iced::run("GameMon", Gui::update, Gui::view)
}

