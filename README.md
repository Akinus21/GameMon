# GameMon
Gaming Monitor is a system tray application that monitors specific executables and executes configured commands on process start and end. It includes a GUI for configuring entries.

## Project Structure

- **src/main.rs**: Entry point of the application. Initializes the system tray, sets up the GUI, and starts monitoring specified processes.
- **src/app.rs**: Defines the main application structure, including methods for starting and stopping process monitoring and handling command execution.
- **src/config.rs**: Manages configuration of entries, including loading and saving configuration data, and defining the structure for each entry.
- **src/gui.rs**: Implements the GUI using a library like eframe or egui, providing an interface for users to manage entries.
- **src/tray.rs**: Handles system tray functionality, including creating the tray icon and responding to user interactions.

## Setup Instructions

1. Ensure you have Rust and Cargo installed on your machine.
2. Clone the repository:
   ```
   git clone <repository-url>
   ```
3. Navigate to the project directory:
   ```
   cd rust-system-tray-app
   ```
4. Build the project:
   ```
   cargo build
   ```

## Usage Guidelines

- Run the application using:
  ```
  cargo run
  ```
- Use the GUI to add, edit, or remove executable entries and their associated commands.

## Dependencies

This project may use libraries such as `eframe`, `sysinfo`, and `serde` for GUI, system information, and serialization respectively. Ensure to check `Cargo.toml` for the complete list of dependencies.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.
