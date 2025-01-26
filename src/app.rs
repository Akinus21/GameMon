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
use crate::config;

pub struct Gui {
    game_name_field: String,      // List of game names
    selected_game_name: Option<String>, // Currently selected game name
    selected_game_entry: Option<config::Entry>, // Currently selected game entry
    game_executable_field: String, // List of game executables
    start_commands_field: text_editor::Content,  // List of start commands
    end_commands_field: text_editor::Content,    // List of end commands
    game_names: Vec<String>,
    entry_changed: bool,
}

impl Default for Gui{
    fn default() -> Self {
        let config = config::Config::load_from_file(&*config::Config::get_config_path().unwrap()).unwrap();
        let mut game_names = Vec::new();

        for entry in config.entries.clone() {
            game_names.push(entry.game_name.clone());
        }

        Self {
            game_name_field: "Enter game name...".to_string(),
            selected_game_name: None,
            selected_game_entry: None,
            game_executable_field: "Enter game executable...".to_string(),
            start_commands_field: text_editor::Content::with_text("Enter start commands..."),
            end_commands_field: text_editor::Content::with_text("Enter end commands..."),
            game_names,
            entry_changed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GameNameChanged(String),
    GameExectuableChanged(String),
    GameSelected(String),
    StartCommandsChanged(text_editor::Action),
    EndCommandsChanged(text_editor::Action),
    NewEntry,
    SaveEntry,
    RemoveEntry,
}

impl Gui {
    pub fn theme(&self) -> Theme {
        Theme::Dracula
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::GameNameChanged(content) => {
                self.game_name_field = content;
                self.entry_changed = true;
            }
            Message::GameExectuableChanged(content) => {
                self.game_executable_field = content;
                self.entry_changed = true;
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
                
                let config = config::Config::load_from_file(&*config::Config::get_config_path().unwrap()).unwrap();
                let selected_entry = config.entries.iter().find(|entry| entry.game_name == game_name).unwrap();
                self.selected_game_entry = Some(selected_entry.clone());

                self.game_name_field = selected_entry.game_name.clone();
                self.game_executable_field = selected_entry.executable.clone();
                self.start_commands_field = text_editor::Content::with_text(&selected_entry.start_commands.join("\n"));
                self.end_commands_field = text_editor::Content::with_text(&selected_entry.end_commands.join("\n"));
                self.entry_changed = false;
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
            }
            Message::SaveEntry => {
                self.save_current_entry();
            }
            Message::RemoveEntry => {
                let mut config = config::Config::load_from_file(&*config::Config::get_config_path().unwrap()).unwrap();
                if let Some(index) = config.entries.iter().position(|entry| entry.game_name == self.game_name_field){
                    config.entries.remove(index);
                    config::Config::save_to_file(&config, &*config::Config::get_config_path().unwrap()).unwrap();
                
                    self.game_name_field = "Enter game name...".to_string();
                    self.game_executable_field = "Enter game executable...".to_string();
                    self.start_commands_field = text_editor::Content::with_text("Enter start commands...");
                    self.end_commands_field = text_editor::Content::with_text("Enter end commands...");
                    self.game_names = config.entries.iter().map(|e| e.game_name.clone()).collect(); // Update game names
                }
            }
        }
    }

    pub fn view(&self) -> Row<Message> {
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
                    .placeholder("Start commands...")
                    .on_action(Message::StartCommandsChanged)
                    .size(14),

                // Vertical space
                vertical_space().height(10),
                
                // End Commands label and field
                text("End Commands:").align_x(Left),
                text_editor(&self.end_commands_field)
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


    pub fn save_current_entry(&mut self) {
        let mut config = config::Config::load_from_file(&*config::Config::get_config_path().unwrap()).unwrap();
        if let Some(index) = config.entries.iter().position(|entry| entry.game_name == self.game_name_field){
            
            println!("Entry already exists at index {}", index);

            config.entries[index].game_name = self.game_name_field.clone();
            config.entries[index].executable = self.game_executable_field.clone();
            config.entries[index].start_commands = self.start_commands_field.text().split("\n").map(|s| s.to_string()).collect();
            config.entries[index].end_commands = self.end_commands_field.text().split("\n").map(|s| s.to_string()).collect();

        } else {
            
            println!("Entry does not exist");

            let new_entry = config::Entry {
                game_name: self.game_name_field.clone(),
                executable: self.game_executable_field.clone(),
                start_commands: self.start_commands_field.text().split("\n").map(|s| s.to_string()).collect(),
                end_commands: self.end_commands_field.text().split("\n").map(|s| s.to_string()).collect(),
            };

            config.entries.push(new_entry);
        }
        config::Config::save_to_file(&config, &*config::Config::get_config_path().unwrap()).unwrap();
        self.game_names = config.entries.iter().map(|e| e.game_name.clone()).collect(); // Update game names
        self.entry_changed = false;
    }
}
