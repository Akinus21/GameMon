use iced::alignment::Horizontal::{Left, Center as xCenter};
use iced::alignment::Vertical::Bottom;
use iced::widget::{
    button,
    column,
    container,
    row,
    text,
    text_editor,
    text_input,
    vertical_rule,
    vertical_space,
    horizontal_space,
    scrollable,
};
use iced::Length::Fill;
use iced::Theme;
use iced::theme::Palette;
use iced::Color;
use crate::config;
use crate::config::{GAMEMON_CONFIG_FILE, ensure_paths_exist};

fn get_system_palette() -> Palette {
    let is_dark = detect_gtk_dark_mode();
    
    if is_dark {
        Palette {
            background: Color::from_rgb8(0x1E, 0x1E, 0x1E),
            text: Color::from_rgb8(0xE0, 0xE0, 0xE0),
            primary: Color::from_rgb8(0x6C, 0xAF, 0xE8),
            success: Color::from_rgb8(0x4A, 0xD4, 0x8C),
            danger: Color::from_rgb8(0xE5, 0x4A, 0x4A),
        }
    } else {
        Palette {
            background: Color::from_rgb8(0xFC, 0xFC, 0xFC),
            text: Color::from_rgb8(0x2E, 0x2E, 0x2E),
            primary: Color::from_rgb8(0x1A, 0x73, 0xE8),
            success: Color::from_rgb8(0x13, 0xB8, 0x65),
            danger: Color::from_rgb8(0xD9, 0x34, 0x25),
        }
    }
}

fn detect_gtk_dark_mode() -> bool {
    std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("dark"))
        .unwrap_or_else(|| {
            std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
                .output()
                .ok()
                .map(|o| {
                    let theme = String::from_utf8_lossy(&o.stdout).to_lowercase();
                    theme.contains("dark") || theme.contains("darker")
                })
                .unwrap_or(false)
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    System,
    Dark,
    Light,
}

impl Default for ThemeType {
    fn default() -> Self {
        ThemeType::System
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewState {
    Profiles,
    Settings,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState::Profiles
    }
}

pub struct Gui {
    game_names: Vec<String>,
    selected_game_name: Option<String>,
    selected_game_entry: Option<config::Entry>,
    game_name_field: String,
    game_executable_field: String,
    start_commands_field: text_editor::Content,
    end_commands_field: text_editor::Content,
    entry_changed: bool,
    view_state: ViewState,
    selected_theme: ThemeType,
}

impl Default for Gui {
    fn default() -> Self {
        if let Err(e) = ensure_paths_exist() {
            log::error!("Error ensuring paths exist: {}", e);
        }
        let config = config::Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()).unwrap();
        let game_names: Vec<String> = config.entries.iter().map(|e| e.game_name.clone()).collect();

        Self {
            game_names,
            selected_game_name: None,
            selected_game_entry: None,
            game_name_field: "Enter game name...".to_string(),
            game_executable_field: "Enter game executable...".to_string(),
            start_commands_field: text_editor::Content::with_text("Enter start commands..."),
            end_commands_field: text_editor::Content::with_text("Enter end commands..."),
            entry_changed: false,
            view_state: ViewState::Profiles,
            selected_theme: ThemeType::System,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GameSelected(String),
    StartCommandsChanged(text_editor::Action),
    TestStartCommands,
    EndCommandsChanged(text_editor::Action),
    TestEndCommands,
    NewEntry,
    SaveEntry,
    RemoveEntry,
    GameNameChanged(String),
    GameExectuableChanged(String),
    OpenSettings,
    CloseSettings,
    ThemeSelected(ThemeType),
}

impl Gui {
    pub fn theme(&self) -> Theme {
        match self.selected_theme {
            ThemeType::Dark => iced::Theme::Dark,
            ThemeType::Light => iced::Theme::Light,
            ThemeType::System => {
                iced::Theme::custom("System".to_string(), get_system_palette())
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::GameSelected(game_name) => {
                self.view_state = ViewState::Profiles;
                if self.entry_changed {
                    self.save_current_entry();
                }
                let config = config::Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()).unwrap();
                if let Some(selected_entry) = config.entries.iter().find(|entry| entry.game_name == game_name) {
                    self.selected_game_name = Some(game_name.clone());
                    self.selected_game_entry = Some(selected_entry.clone());
                    self.game_name_field = selected_entry.game_name.clone();
                    self.game_executable_field = selected_entry.executable.clone();
                    self.start_commands_field = text_editor::Content::with_text(&selected_entry.start_commands.join("\n"));
                    self.end_commands_field = text_editor::Content::with_text(&selected_entry.end_commands.join("\n"));
                    self.entry_changed = false;
                }
            }
            Message::StartCommandsChanged(action) => {
                self.start_commands_field.perform(action);
                self.entry_changed = true;
            }
            Message::TestStartCommands => {
                for cmd in self.start_commands_field.text().lines() {
                    if !cmd.trim().is_empty() {
                        if let Err(e) = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .spawn()
                        {
                            log::error!("Failed to run start command '{}': {}", cmd, e);
                        }
                    }
                }
            }
            Message::EndCommandsChanged(action) => {
                self.end_commands_field.perform(action);
                self.entry_changed = true;
            }
            Message::TestEndCommands => {
                for cmd in self.end_commands_field.text().lines() {
                    if !cmd.trim().is_empty() {
                        if let Err(e) = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .spawn()
                        {
                            log::error!("Failed to run end command '{}': {}", cmd, e);
                        }
                    }
                }
            }
            Message::NewEntry => {
                if self.entry_changed {
                    self.save_current_entry();
                }
                self.selected_game_name = None;
                self.selected_game_entry = None;
                self.game_name_field = "Enter game name...".to_string();
                self.game_executable_field = "Enter game executable...".to_string();
                self.start_commands_field = text_editor::Content::with_text("Enter start commands...");
                self.end_commands_field = text_editor::Content::with_text("Enter end commands...");
                self.entry_changed = false;
                self.view_state = ViewState::Profiles;
            }
            Message::SaveEntry => {
                self.save_current_entry();
            }
            Message::RemoveEntry => {
                let mut config = config::Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()).unwrap();
                if let Some(index) = config.entries.iter().position(|entry| entry.game_name == self.game_name_field) {
                    config.entries.remove(index);
                    config::Config::save_to_file(&config, &GAMEMON_CONFIG_FILE.to_string_lossy()).unwrap();
                    self.game_names = config.entries.iter().map(|e| e.game_name.clone()).collect();
                    self.selected_game_name = None;
                    self.selected_game_entry = None;
                    self.game_name_field = "Enter game name...".to_string();
                    self.game_executable_field = "Enter game executable...".to_string();
                    self.start_commands_field = text_editor::Content::with_text("Enter start commands...");
                    self.end_commands_field = text_editor::Content::with_text("Enter end commands...");
                }
            }
            Message::GameNameChanged(content) => {
                self.game_name_field = content;
                self.entry_changed = true;
            }
            Message::GameExectuableChanged(content) => {
                self.game_executable_field = content;
                self.entry_changed = true;
            }
            Message::OpenSettings => {
                if self.entry_changed {
                    self.save_current_entry();
                }
                self.view_state = ViewState::Settings;
            }
            Message::CloseSettings => {
                self.view_state = ViewState::Profiles;
            }
            Message::ThemeSelected(theme) => {
                self.selected_theme = theme;
            }
        }
    }

    pub fn view(&'_ self) -> iced::widget::Row<'_, Message> {
        let left_panel = self.left_panel();
        let right_panel = match self.view_state {
            ViewState::Profiles => self.profiles_view(),
            ViewState::Settings => self.settings_view(),
        };

        row![
            left_panel,
            vertical_rule(10),
            right_panel,
        ]
    }

    fn left_panel(&self) -> iced::widget::container::Container<'_, Message> {
        let profile_list: Vec<_> = self.game_names.iter()
            .map(|name| {
                let is_selected = self.selected_game_name.as_deref() == Some(name);
                let label = if is_selected { format!("▶ {}", name) } else { name.clone() };
                button(text(label).size(14))
                    .width(Fill)
                    .padding(8)
                    .on_press(Message::GameSelected(name.clone()))
            })
            .collect();

        let list_container = if profile_list.is_empty() {
            column![text("No profiles yet").size(12).color([0.5, 0.5, 0.5])]
                .align_x(xCenter)
                .into()
        } else {
            column![]
                .padding(5)
                .into()
        };

        let settings_btn = button(text("⚙ Settings").size(12))
            .padding(8)
            .on_press(Message::OpenSettings);

        container(
            column![
                text("Profiles").size(16).align_x(xCenter),
                vertical_space().height(10),
                scrollable(list_container).height(Fill),
                vertical_space().height(10),
                settings_btn
            ]
            .padding(15)
            .align_x(xCenter)
        )
        .padding(10)
        .width(200)
        .align_x(Left)
    }

    fn profiles_view(&self) -> iced::widget::container::Container<'_, Message> {
        container(
            column![
                row![
                    column![
                        vertical_space().height(10),
                        text("Game Name:").align_x(Left).size(16).align_y(Bottom),
                    ],
                    horizontal_space().width(10),
                    text_input("Game name...", &self.game_name_field)
                        .padding(10)
                        .size(16)
                        .on_input(Message::GameNameChanged),
                    horizontal_space().width(10),
                    column![
                        vertical_space().height(10),
                        text("Game Executable:").align_x(Left).size(16).align_y(Bottom),
                    ],
                    horizontal_space().width(10),
                    text_input("Game executable...", &self.game_executable_field)
                        .padding(10)
                        .size(16)
                        .on_input(Message::GameExectuableChanged),
                    horizontal_space().width(10),
                ],
                vertical_space().height(10),
                text("Start Commands:").align_x(Left),
                row![
                    text_editor(&self.start_commands_field)
                        .placeholder("Start commands...")
                        .on_action(Message::StartCommandsChanged)
                        .size(14),
                    horizontal_space().width(10),
                    button("Run")
                        .on_press(Message::TestStartCommands)
                        .padding(5)
                        .height(30)
                        .width(50),
                ],
                vertical_space().height(10),
                text("End Commands:").align_x(Left),
                row![
                    text_editor(&self.end_commands_field)
                        .placeholder("End commands...")
                        .on_action(Message::EndCommandsChanged)
                        .size(14),
                    horizontal_space().width(10),
                    button("Run")
                        .on_press(Message::TestEndCommands)
                        .padding(5)
                        .height(30)
                        .width(50),
                ],
                vertical_space().height(Fill),
                row![
                    button("New Entry")
                        .on_press(Message::NewEntry)
                        .padding(10),
                    horizontal_space().width(10),
                    button("Remove Entry")
                        .on_press(Message::RemoveEntry)
                        .padding(10),
                    horizontal_space().width(10),
                    button("Save Entry")
                        .on_press(Message::SaveEntry)
                        .padding(10),
                ]
            ]
            .padding(20)
            .align_x(Left)
        )
        .padding(10)
        .width(Fill)
    }

    fn settings_view(&self) -> iced::widget::container::Container<'_, Message> {
        let theme_button = |name: &str, theme: ThemeType, is_selected: bool| {
            let label = if is_selected { format!("● {}", name) } else { format!("  {}", name) };
            button(text(label).size(14))
                .padding(10)
                .on_press(Message::ThemeSelected(theme))
        };

        container(
            column![
                row![
                    button("← Back").padding(8).on_press(Message::CloseSettings),
                    horizontal_space().width(10),
                    text("Settings").size(20).align_y(Bottom),
                ],
                vertical_space().height(20),
                text("Theme").size(16),
                vertical_space().height(5),
                theme_button("System Default", ThemeType::System, self.selected_theme == ThemeType::System),
                vertical_space().height(5),
                theme_button("Dark", ThemeType::Dark, self.selected_theme == ThemeType::Dark),
                vertical_space().height(5),
                theme_button("Light", ThemeType::Light, self.selected_theme == ThemeType::Light),
            ]
            .padding(20)
            .align_x(Left)
        )
        .padding(10)
        .width(Fill)
    }

    fn save_current_entry(&mut self) {
        let mut config = config::Config::load_from_file(&GAMEMON_CONFIG_FILE.to_string_lossy()).unwrap();
        if let Some(index) = config.entries.iter().position(|entry| entry.game_name == self.game_name_field) {
            config.entries[index].game_name = self.game_name_field.clone();
            config.entries[index].executable = self.game_executable_field.clone();
            config.entries[index].start_commands = self.start_commands_field.text().split("\n").map(|s| s.to_string()).collect();
            config.entries[index].end_commands = self.end_commands_field.text().split("\n").map(|s| s.to_string()).collect();
        } else {
            let new_entry = config::Entry {
                game_name: self.game_name_field.clone(),
                executable: self.game_executable_field.clone(),
                start_commands: self.start_commands_field.text().split("\n").map(|s| s.to_string()).collect(),
                end_commands: self.end_commands_field.text().split("\n").map(|s| s.to_string()).collect(),
            };
            config.entries.push(new_entry);
        }
        config::Config::save_to_file(&config, &GAMEMON_CONFIG_FILE.to_string_lossy()).unwrap();
        self.game_names = config.entries.iter().map(|e| e.game_name.clone()).collect();
        self.entry_changed = false;
    }
}