# GameMon - Your Automated Gaming Companion

GameMon is a cross-platform desktop application designed to enhance your gaming experience by automating tasks and managing processes related to your games.  It allows you to define specific commands or scripts to be executed when a game starts and ends, offering a seamless way to optimize your system for gaming and restore it to its normal state afterward.

## Features

* Process Monitoring: GameMon intelligently monitors for specified game executables and detects when they start or stop.
* Customizable Commands: Execute tailored commands or scripts before and after your gaming sessions. This could include:
    * Toggling performance settings (e.g., CPU governor, fan curves)
    * Switching audio outputs
    * Starting or stopping background applications (e.g., Discord, streaming software)
    * Adjusting display settings
    * Mounting or unmounting game-specific virtual drives
* Cross-Platform Compatibility: Supports Linux, Windows, and macOS, ensuring flexibility across different operating systems.
* Tray Icon Integration: A discreet tray icon provides quick access to settings and information without interrupting your gameplay.
* Graphical User Interface: A user-friendly GUI simplifies the process of configuring game entries and associated commands.

## Use Cases

* Boost Performance: Automatically disable power-saving features and maximize performance when launching a demanding game.
* Immersive Audio: Switch to your preferred gaming headset when the game starts and revert to your default speakers afterward.
* Streamlined Setup: Launch necessary applications like Discord, OBS Studio, or your chat client along with your game.
* VR Optimization: Configure your system specifically for VR gaming, including launching SteamVR or Oculus software.
* Clean Up After Gaming: Close unnecessary applications, restore system settings, and unmount virtual drives when you're finished playing.

## Installation

Pre-built binaries for GameMon will be available for download on the project's release page. Simply download the appropriate version for your operating system and follow the installation instructions.

### Building from Source (for developers)

GameMon is written in Rust. To build from source, you'll need to have a Rust development environment set up.

1. Clone the repository:
   ```bash
   git clone https://github.com/Akinus21/GameMon.git
2. Navigate to the project directory:
   ```bash
   cd gamemon
3. Build the project:
   ```bash
   cargo build --release
The resulting executable will be found in the target/release directory.

## ToDo
|__ Add other conditions to match on, including
    |__ Startup
    |__ Login
    |__ Shutdown
    |__ Sleep or suspend
    |__ Script or program returns as "true"
    |__ Hotkey pressed
    |__ Window Active
|__ Add Custom Delay for individual checks
|__ Clean up code
