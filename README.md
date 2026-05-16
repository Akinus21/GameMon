# GameMon - Your Automated Gaming Companion

GameMon is a cross-platform desktop application designed to enhance your gaming experience by automating tasks and managing processes related to your games. It allows you to define specific commands or scripts to be executed when a game starts and ends, offering a seamless way to optimize your system for gaming and restore it to its normal state afterward.

## Features

* **Process Monitoring**: GameMon intelligently monitors for specified game executables and detects when they start or stop.
* **Customizable Commands**: Execute tailored commands or scripts before and after your gaming sessions. This could include:
    * Toggling performance settings (e.g., CPU governor, fan curves)
    * Switching audio outputs
    * Starting or stopping background applications (e.g., Discord, streaming software)
    * Adjusting display settings
    * Mounting or unmounting game-specific virtual drives
* **Cross-Platform Compatibility**: Supports Linux, Windows, and macOS, ensuring flexibility across different operating systems.
* **Tray Icon Integration**: A discreet tray icon provides quick access to settings and information without interrupting your gameplay.
* **Graphical User Interface**: A user-friendly GUI simplifies the process of configuring game entries and associated commands.

## Use Cases

* **Boost Performance**: Automatically disable power-saving features and maximize performance when launching a demanding game.
* **Immersive Audio**: Switch to your preferred gaming headset when the game starts and revert to your default speakers afterward.
* **Streamlined Setup**: Launch necessary applications like Discord, OBS Studio, or your chat client along with your game.
* **VR Optimization**: Configure your system specifically for VR gaming, including launching SteamVR or Oculus software.
* **Clean Up After Gaming**: Close unnecessary applications, restore system settings, and unmount virtual drives when you're finished playing.

## Installation

### Homebrew (Recommended for Linux)

```bash
brew tap Akinus21/tap
brew install gamemon
```

This installs all three components and automatically creates:
- Desktop entry for the GUI (`~/.local/share/applications/gamemon.desktop`)
- Systemd user service that starts automatically (`gamemon.service`)
- All binaries: `gamemon-service`, `gamemon-gui`, `gamemon-update`

### Manual

Pre-built binaries are available on the project's release page. Simply download the appropriate version for your operating system.

```bash
# Download the latest release
curl -L -o gamemon.tar.gz https://github.com/Akinus21/GameMon/releases/latest/download/GameMon.tar.gz

# Extract and install
tar -xzf gamemon.tar.gz
sudo cp GameMon/GameMon-service /usr/local/bin/gamemon-service
sudo cp GameMon/GameMon-gui /usr/local/bin/gamemon-gui
sudo cp GameMon/GameMon-update /usr/local/bin/gamemon-update
sudo cp -r GameMon/resources /usr/share/gamemon/

# Run the service once to install resources locally
gamemon-service --install-resources
```

### Building from Source (for developers)

```bash
git clone https://github.com/Akinus21/GameMon.git
cd GameMon
cargo build --release
```

Binaries will be at `target/release/GameMon-*`.

## Commands

| Command | Description |
|---------|-------------|
| `gamemon-service` | The background daemon that monitors processes and triggers actions. Runs automatically via systemd. |
| `gamemon-gui` | Opens the graphical configuration interface. Can be launched from the tray or applications menu. |
| `gamemon-update` | Checks for and installs updates. Run manually or triggered from tray. |
| `gamemon-service --install-resources` | Copies all binaries and resources to `~/.local/share/gamemon/`. Useful for first-time setup. |

## Service Management (Systemd)

```bash
# Check service status
systemctl --user status gamemon

# Restart service
systemctl --user restart gamemon

# Stop service
systemctl --user stop gamemon

# Disable auto-start
systemctl --user disable gamemon
```

## Configuration

Configuration is stored at `~/.config/gamemon/config.toml` and managed entirely through the GUI. Add profiles for each game with:
- **Game Name**: Friendly name for the profile
- **Executable**: Process name to monitor (e.g., `eldenring.exe`, `wow.exe`)
- **Start Commands**: Commands to run when the game launches
- **End Commands**: Commands to run when the game exits

## ToDo
- Add other conditions to match on, including
    - Startup
    - Login
    - Shutdown
    - Sleep or suspend
    - Script or program returns as "true"
    - Hotkey pressed
    - Window Active
- Add Custom Delay for individual checks
- Clean up code
- Add flag system to indicate necessary action
   - Needs update
   - Installed
   - Just updated (true after update, spawns actions on first start after update, then set to false)
- Add priority system to prevent conflicting overlapping runs (ie: Don't run X profile actions, if Y profile is active)
