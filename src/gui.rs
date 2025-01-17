use iced::widget::{
    button, text_input, Button, Column, Container, Text, TextInput,
};
use iced_native::Event;
use iced_native::subscription::events;
use crate::config::{Config, Entry as ConfigEntry};
use std::{env, path::Path};

#[derive(Default)]
pub struct Gui {
    entries: Vec<ConfigEntry>,
    config_file_path: String,
    game_name: text_input::State,
    executable: text_input::State,
    start_commands: text_input::State,
    end_commands: text_input::State,
    game_name_value: String,
    executable_value: String,
    start_commands_value: String,
    end_commands_value: String,
    save_button: button::State,
    exit_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    GameNameChanged(String),
    ExecutableChanged(String),
    StartCommandsChanged(String),
    EndCommandsChanged(String),
    Save,
    Exit,
    EventOccurred(Event),
}

impl Application for Gui {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Gui, Command<Message>) {
        // Set GSETTINGS_SCHEMA_DIR to the current directory
        env::set_var("GSETTINGS_SCHEMA_DIR", "./schemas/");

        let settings = gio::Settings::new("com.example.process_monitor");
        let config_file_path = settings
            .value("config-file-path")
            .get::<String>()
            .unwrap_or("./config.toml".to_string());

        let mut gui = Gui {
            config_file_path,
            ..Gui::default()
        };

        // Load config when GUI is created
        gui.load_config();
        (gui, Command::none())
    }

    fn title(&self) -> String {
        String::from("GameMon Configuration")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::GameNameChanged(value) => {
                self.game_name_value = value;
            }
            Message::ExecutableChanged(value) => {
                self.executable_value = value;
            }
            Message::StartCommandsChanged(value) => {
                self.start_commands_value = value;
            }
            Message::EndCommandsChanged(value) => {
                self.end_commands_value = value;
            }
            Message::Save => {
                if let Err(err) = self.validate_and_save_entry() {
                    println!("Error: {}", err);
                }
            }
            Message::Exit => {
                self.save_config();
                std::process::exit(0);
            }
            Message::EventOccurred(event) => {
                if let Event::Window(iced_native::window::Event::CloseRequested) = event {
                    self.save_config();
                    std::process::exit(0);
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let game_name_input = TextInput::new(
            &mut self.game_name,
            "Game Name",
            &self.game_name_value,
            Message::GameNameChanged,
        )
        .padding(10)
        .size(20);

        let executable_input = TextInput::new(
            &mut self.executable,
            "Executable",
            &self.executable_value,
            Message::ExecutableChanged,
        )
        .padding(10)
        .size(20);

        let start_commands_input = TextInput::new(
            &mut self.start_commands,
            "Start Commands",
            &self.start_commands_value,
            Message::StartCommandsChanged,
        )
        .padding(10)
        .size(20);

        let end_commands_input = TextInput::new(
            &mut self.end_commands,
            "End Commands",
            &self.end_commands_value,
            Message::EndCommandsChanged,
        )
        .padding(10)
        .size(20);

        let save_button = Button::new(&mut self.save_button).label(Text::new("Save Entry"))
            .on_press(Message::Save);

        let exit_button = Button::new(&mut self.exit_button, Text::new("Exit"))
            .on_press(Message::Exit);

        let content = Column::new()
            .padding(20)
            .spacing(10)
            .push(game_name_input)
            .push(executable_input)
            .push(start_commands_input)
            .push(end_commands_input)
            .push(save_button)
            .push(exit_button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        events().map(Message::EventOccurred)
    }
}

impl Gui {
    fn load_config(&mut self) {
        if Path::new(&self.config_file_path).exists() {
            match Config::load_from_file(&self.config_file_path) {
                Ok(config) => {
                    self.entries = config.entries;
                    println!("Configuration loaded successfully.");
                }
                Err(e) => {
                    eprintln!("Failed to load config: {}", e);
                }
            }
        } else {
            let new_config = Config {
                entries: Vec::new(),
            };
            match new_config.save_to_file(&self.config_file_path) {
                Ok(_) => println!("New config file created at {}", self.config_file_path),
                Err(e) => eprintln!("Failed to create config file: {}", e),
            }
        }
    }

    fn save_config(&self) {
        let config = Config {
            entries: self.entries.clone(),
        };

        match config.save_to_file(&self.config_file_path) {
            Ok(_) => println!("Configuration saved successfully."),
            Err(e) => eprintln!("Failed to save config: {}", e),
        }
    }

    fn validate_and_save_entry(&mut self) -> Result<(), String> {
        if self.game_name_value.trim().is_empty() {
            return Err("Game name cannot be empty.".to_string());
        }

        if self.executable_value.trim().is_empty() {
            return Err("Executable cannot be empty.".to_string());
        }

        if self.start_commands_value.trim().is_empty() {
            return Err("Start commands cannot be empty.".to_string());
        }

        if self.end_commands_value.trim().is_empty() {
            return Err("End commands cannot be empty.".to_string());
        }

        self.entries.push(ConfigEntry {
            game_name: self.game_name_value.clone(),
            executable: self.executable_value.clone(),
            start_commands: self.start_commands_value
                .lines()
                .map(|line| line.trim().to_string())
                .collect(),
            end_commands: self.end_commands_value
                .lines()
                .map(|line| line.trim().to_string())
                .collect(),
        });

        Ok(())
    }
}