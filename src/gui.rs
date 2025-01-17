use gtk4::{
    Application, ApplicationWindow, Button, ComboBoxText, Entry, Label, MessageDialog, Orientation,
    ScrolledWindow, TextView, Box as GtkBox,
    prelude::*,
};
use gio::Settings;
use std::{cell::RefCell, rc::Rc, path::Path};
use crate::config::{Config, Entry as ConfigEntry};

#[derive(Clone)]
pub struct Gui {
    entries: Vec<ConfigEntry>,
    settings: Settings,
}

impl Gui {
    pub fn new() -> Self {
        let settings = Settings::new("com.example.process_monitor");
        let config_file_path = settings
        .value("config-file-path")
        .get::<String>()
        .unwrap_or("./config.toml".to_string());

    
        let mut gui = Self {
            entries: Vec::new(),
            settings,
        };

        // Load config when GUI is created
        gui.load_config(&config_file_path);
        gui
    }

    pub fn load_config(&mut self, config_file_path: &str) {
        if Path::new(config_file_path).exists() {
            match Config::load_from_file(config_file_path) {
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
            match new_config.save_to_file(config_file_path) {
                Ok(_) => println!("New config file created at {}", config_file_path),
                Err(e) => eprintln!("Failed to create config file: {}", e),
            }
        }
    }

    pub fn save_config(&self, config_file_path: &str) {
        let config = Config {
            entries: self.entries.clone(),
        };

        match config.save_to_file(config_file_path) {
            Ok(_) => println!("Configuration saved successfully."),
            Err(e) => eprintln!("Failed to save config: {}", e),
        }
    }

    pub fn show_gui(&self) {
        let app = Application::new(Some("com.example.process_monitor"), Default::default());
        let gui = Rc::new(RefCell::new(self.clone()));

        app.connect_activate(move |app| {
            let window = ApplicationWindow::new(app);
            window.set_title(Some("GameMon Configuration"));
            window.set_default_size(800, 600);

            let outer_vbox = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(10)
                .build();

            let hbox = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(10)
                .build();

            let labels_vbox = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(5)
                .build();

            let entries_vbox = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(5)
                .build();

            let game_name_label = Label::new(Some("Game Name:"));
            let game_name_entry = Rc::new(RefCell::new(Entry::new()));
            labels_vbox.append(&game_name_label);
            entries_vbox.append(&*game_name_entry.borrow());

            let executable_label = Label::new(Some("Executable:"));
            let executable_entry = Entry::new();
            labels_vbox.append(&executable_label);
            entries_vbox.append(&executable_entry);

            let start_commands_label = Label::new(Some("Start Commands:"));
            let start_commands_view = TextView::new();
            start_commands_view.set_height_request(150);
            let start_commands_scrolled = ScrolledWindow::new();
            start_commands_scrolled.set_child(Some(&start_commands_view));
            labels_vbox.append(&start_commands_label);
            entries_vbox.append(&start_commands_scrolled);

            let end_commands_label = Label::new(Some("End Commands:"));
            let end_commands_view = TextView::new();
            end_commands_view.set_height_request(150);
            let end_commands_scrolled = ScrolledWindow::new();
            end_commands_scrolled.set_child(Some(&end_commands_view));
            labels_vbox.append(&end_commands_label);
            entries_vbox.append(&end_commands_scrolled);

            hbox.append(&labels_vbox);
            hbox.append(&entries_vbox);
            outer_vbox.append(&hbox);

            let combo_box_label = Label::new(Some("Choose Game:"));
            outer_vbox.append(&combo_box_label);

            let combo_box = ComboBoxText::new();
            gui.borrow().refresh_combo_box(&combo_box);

            outer_vbox.append(&combo_box);

            let button_box = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(10)
                .build();

            let save_button = Button::with_label("Save Entry");
            let window_clone = window.clone();
            save_button.connect_clicked({
                let gui_rc = Rc::clone(&gui);
                let game_name_entry_clone = Rc::clone(&game_name_entry);
                let executable_entry_clone = executable_entry.clone();
                let start_commands_view_clone = start_commands_view.clone();
                let end_commands_view_clone = end_commands_view.clone();
                move |_| {
                    if let Err(err) = Self::validate_and_save_entry(
                        &gui_rc,
                        &game_name_entry_clone,
                        &executable_entry_clone,
                        &start_commands_view_clone,
                        &end_commands_view_clone,
                    ) {
                        let dialog = MessageDialog::builder()
                            .modal(true)
                            .transient_for(&window_clone)
                            .buttons(gtk4::ButtonsType::Ok)
                            .message_type(gtk4::MessageType::Error)
                            .text("Invalid Input")
                            .secondary_text(&err)
                            .build();
                        dialog.connect_response(|dialog, _| dialog.close());
                        dialog.show();
                    }
                }
            });

            let exit_button = Button::with_label("Exit");
            exit_button.connect_clicked({
                let gui_rc = Rc::clone(&gui);
                let app_clone = app.clone();
                move |_| {
                    let gui = gui_rc.borrow();
                    let config_file_path = gui.settings
                        .value("config-file-path")
                        .get::<String>()
                        .unwrap_or_else(|| "".to_string()
                    );
                    gui.save_config(&config_file_path);
                    app_clone.quit();
                }
            });

            button_box.append(&save_button);
            button_box.append(&exit_button);
            outer_vbox.append(&button_box);

            window.set_child(Some(&outer_vbox));
            window.show();
        });

        app.run();
    }

    fn validate_and_save_entry(
        gui_rc: &Rc<RefCell<Self>>,
        game_name_entry: &Rc<RefCell<Entry>>,
        executable_entry: &Entry,
        start_commands_view: &TextView,
        end_commands_view: &TextView,
    ) -> Result<(), String> {
        let game_name = game_name_entry.borrow().text().to_string();
        if game_name.trim().is_empty() {
            return Err("Game name cannot be empty.".to_string());
        }

        let executable = executable_entry.text().to_string();
        if executable.trim().is_empty() {
            return Err("Executable cannot be empty.".to_string());
        }

        let start_commands_text = start_commands_view.buffer().text(
            &start_commands_view.buffer().start_iter(),
            &start_commands_view.buffer().end_iter(),
            false,
        );
        if start_commands_text.trim().is_empty() {
            return Err("Start commands cannot be empty.".to_string());
        }

        let end_commands_text = end_commands_view.buffer().text(
            &end_commands_view.buffer().start_iter(),
            &end_commands_view.buffer().end_iter(),
            false,
        );
        if end_commands_text.trim().is_empty() {
            return Err("End commands cannot be empty.".to_string());
        }

        let mut gui = gui_rc.borrow_mut();
        gui.entries.push(ConfigEntry {
            game_name,
            executable,
            start_commands: start_commands_text
                .lines()
                .map(|line| line.trim().to_string())
                .collect(),
            end_commands: end_commands_text
                .lines()
                .map(|line| line.trim().to_string())
                .collect(),
        });

        Ok(())
    }

    pub fn refresh_combo_box(&self, combo_box: &ComboBoxText) {
        combo_box.remove_all();
        for entry in &self.entries {
            combo_box.append_text(&entry.game_name);
        }
    }
}
