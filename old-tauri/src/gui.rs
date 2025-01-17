use eframe::egui;
use std::path::Path;
use crate::config::{Config, Entry};

#[derive(Clone)]
pub struct Gui {
    entries: Vec<Entry>,
    config_file_path: String,  // Store the path of the configuration file
    show_gui: bool,  // Flag to trigger GUI showing
}

impl Gui {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            config_file_path: String::new(),
            show_gui: false,  // Start with GUI not visible
        }
    }

    // Method to trigger showing the GUI
    pub fn show_gui(&mut self) {
        self.show_gui = true;
    }

    // Function to load the configuration from a TOML file
    pub fn load_config(&mut self) {
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
        }
    }

    // Function to save the current entries to a TOML file
    pub fn save_config(&self) {
        let config = Config {
            entries: self.entries.clone(),
            config_file_path: self.config_file_path.clone(),
        };

        match config.save_to_file(&self.config_file_path) {
            Ok(_) => println!("Configuration saved successfully."),
            Err(e) => eprintln!("Failed to save config: {}", e),
        }
    }

    // Function to show the GUI
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.show_gui {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Process Monitor Configuration");

                // Display and edit the config file path
                ui.horizontal(|ui| {
                    ui.label("Config File Path:");
                    ui.text_edit_singleline(&mut self.config_file_path);
                });

                // Load the configuration from the specified file path
                if ui.button("Load Config").clicked() {
                    self.load_config();
                }

                // Save the configuration to the specified file path
                if ui.button("Save Config").clicked() {
                    self.save_config();
                }

                if ui.button("Add Entry").clicked() {
                    self.entries.push(Entry::default());
                }

                // Collect the entries to remove in a separate vector
                let mut entries_to_remove = Vec::new();

                for (index, entry) in self.entries.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label("Executable:");
                        ui.text_edit_singleline(&mut entry.executable);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Start Commands:");
                        for command in &mut entry.start_commands {
                            ui.text_edit_singleline(command);
                        }
                        if ui.button("Add Start Command").clicked() {
                            entry.start_commands.push(String::new());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("End Commands:");
                        for command in &mut entry.end_commands {
                            ui.text_edit_singleline(command);
                        }
                        if ui.button("Add End Command").clicked() {
                            entry.end_commands.push(String::new());
                        }
                    });

                    if ui.button("Remove Entry").clicked() {
                        // Add the index of the entry to be removed
                        entries_to_remove.push(index);
                    }
                }

                // Remove entries after the UI has been rendered
                for index in entries_to_remove.iter().rev() {
                    self.entries.remove(*index);
                }
            });

            // Reset the flag after showing the GUI
            self.show_gui = false;
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.save_config();
        self.show(ctx);
    }
}
