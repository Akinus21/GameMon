use iced::alignment::Horizontal::{Left, Center as xCenter, Right};
use iced::alignment::Vertical::{Top, Center as yCenter, Bottom};
use iced::widget::text_editor::Edit;
use iced::widget::{
    button,
    column,
    container,
    pick_list,
    row,
    text,
    text_editor,
    text_input,
    Column,
    Row,
    Scrollable,
    TextInput,
    vertical_rule,
    vertical_space,
    horizontal_space,
};
use iced::Length::Fill;
use iced::{application, Element, theme};
use crate::config::{Config, Entry};

pub fn show_gui() -> iced::Result {
    iced::run("A cool counter", Gui::update, Gui::view)
}

#[derive(Default)]
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
    fn update(&mut self, message: Message) {
        match message {
            Message::GameNameChanged(content) => {
                self.game_name_field = content.clone();
                // println!("Game Name changed: {}", content);
            }
            Message::GameExectuableChanged(content) => {
                self.game_executable_field = content.clone();
                // println!("Game Executable changed: {}", content);
            }
            Message::StartCommandsChanged(action) => {
                self.start_commands_field.perform(action);
            }
            Message::EndCommandsChanged(action) => {
                self.end_commands_field.perform(action);
            }
            Message::GameSelected(game_name) => {
                self.selected_game_name = Some(game_name.clone());
                
                // Save the current entry before switching if the game name field is not empty
                if !self.game_name_field.is_empty() {
                    self.save_current_entry();
                }
                
                let entries = Config::load_from_file(&*Config::get_config_path().unwrap()).unwrap().entries;
                let selected_entry = entries.iter().find(|entry| entry.game_name == game_name).unwrap();
                self.selected_game_entry = Some(selected_entry.clone());

                self.game_name_field = selected_entry.game_name.clone();
                self.game_executable_field = selected_entry.executable.clone();
                self.start_commands_field = text_editor::Content::with_text(&selected_entry.start_commands.join("\n"));
                self.end_commands_field = text_editor::Content::with_text(&selected_entry.end_commands.join("\n"));
                // println!("Game selected: {}", game_name);
            }
            Message::NewEntry => {
                self.game_name_field = "Enter game name...".to_string();
                self.game_executable_field = "Enter game executable...".to_string();
                self.start_commands_field = text_editor::Content::with_text("Enter start commands...");
                self.end_commands_field = text_editor::Content::with_text("Enter end commands...");
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
                println!("View loading");

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
                .placeholder("Select a game..."),

                // Vertical space
                container("").height(Fill),

            ]
            .padding(20)
            .align_x(xCenter)
        )
        .padding(10)
        .align_x(Left);

        //definte right container
        let right_container = container(
            column![

                row![
                    // Game Name label and field
                    text("Game Name:").align_x(Left).size(16).align_y(yCenter),

                    horizontal_space().width(10),

                    text_input(
                        // &self.game_names[0].clone(),
                        "Enter game name...",
                        &self.game_name_field,
                    )
                    .padding(10)
                    .size(16)
                    .on_input(Message::GameNameChanged),
                    
                    // Horizontal space
                    horizontal_space().width(10),

                    // Game Executable label and field
                    text("Game Executable:").align_x(Left).size(16).align_y(yCenter),
                    
                    horizontal_space().width(10),
                    
                    text_input(
                        // &self.game_executables[0].clone(),
                        "Enter game executable...",
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
                entry.start_commands.join("\n").trim_end().to_string()  // Join with newlines
            );
            
            self.game_end_commands.push(
                entry.end_commands.join("\n").trim_end().to_string()         // Join with newlines
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
    }
}

