use std::fs::{self};
use std::path::Path;
use std::process::Command;
mod logger;
use game_mon::config::GAMEMON_DIR;

fn main() {
    // Initialize logging (if needed)
    logger::Logger::init().expect("Failed to initialize logger");

    // Check if a previous update was interrupted
    check_update_marker();

    // Example usage of run_deferred_installer (replace with actual logic as needed)
    let tmp_dir = std::env::temp_dir().to_str().unwrap().to_string();
    let latest_version = "v1.2.3"; // Replace with actual version fetched from GitHub

    // Extract archive before running deferred installer
    #[cfg(unix)]
    {
        let tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.tar.gz");
        let tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
        extract_tar_gz(&tmp_archive_path, &tmp_extract_dir).expect("Failed to extract tar.gz");
    }

    #[cfg(windows)]
    {
        let tmp_archive_path = Path::new(&tmp_dir).join("GameMon-update.zip");
        let tmp_extract_dir = Path::new(&tmp_dir).join("GameMon_update");
        extract_zip(&tmp_archive_path, &tmp_extract_dir).expect("Failed to extract zip");
    }

    if let Err(e) = run_deferred_installer(&tmp_dir, latest_version) {
        eprintln!("Error running installer: {}", e);
    }
}

fn run_deferred_installer(tmp_dir: &str, latest_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let script_path = Path::new("/tmp/GameMon-updater-install.sh");
        let update_marker = GAMEMON_DIR.join(".update-pending");

        fs::write(&update_marker, latest_version)?;

        let installer_script = format!(r#"#!/bin/bash
sleep 2

mkdir -p "{gamedir}/backup"
date=$(date +%Y%m%d_%H%M%S)
cp -r "{gamedir}" "{gamedir}/backup/$date"

pkill GameMon

cp -r "{tmp_dir}/GameMon_update"/* "{gamedir}/"

rm -rf "{tmp_dir}/GameMon_update"
rm -f "{tmp_dir}/GameMon-update.tar.gz"
rm -f "{gamedir}/.update-pending"

"{gamedir}/GameMon" &

rm -- "$0"
"#,
            gamedir = GAMEMON_DIR.to_str().unwrap(),
            tmp_dir = tmp_dir
        );

        fs::write(&script_path, installer_script)?;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
        Command::new("sh").arg(script_path).spawn()?;
        log::info!("Delegated update installer script to detached process.");
        std::process::exit(0);
    }

    #[cfg(windows)]
    {
        let script_path = Path::new(tmp_dir).join("GameMon-updater-install.bat");
        let update_marker = GAMEMON_DIR.join(".update-pending");

        fs::write(&update_marker, latest_version)?;

        let installer_script = format!(r#"
@echo off
timeout /T 2 /NOBREAK

mkdir "{gamedir}\backup"
set datetime=%date:~10,4%%date:~4,2%%date:~7,2%_%time:~0,2%%time:~3,2%%time:~6,2%
xcopy /E /I /Y "{gamedir}" "{gamedir}\backup\%datetime%"

Taskkill /F /IM GameMon.exe

xcopy /Y /E "{tmpdir}\GameMon_update\*" "{gamedir}\"

rmdir /S /Q "{tmpdir}\GameMon_update"
del "{tmpdir}\GameMon-update.zip"
del "{gamedir}\.update-pending"

start "" "{gamedir}\GameMon.exe"

(del "%~f0")
"#,
            gamedir = GAMEMON_DIR.to_str().unwrap().replace("\\", "\\\\"),
            tmpdir = tmp_dir.replace("\\", "\\\\")
        );

        fs::write(&script_path, installer_script)?;
        Command::new("cmd")
            .args(&["/C", script_path.to_str().unwrap()])
            .spawn()?;
        log::info!("Delegated update installer batch to detached process.");
        std::process::exit(0);
    }
}

fn check_update_marker() {
    let update_marker = GAMEMON_DIR.join(".update-pending");
    if update_marker.exists() {
        log::warn!("Previous update may have failed or is in progress.");
        // optionally alert user or recover
    }
}

#[cfg(unix)]
fn extract_tar_gz(tar_gz_path: &Path, extract_to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let tar_gz = fs::File::open(tar_gz_path)?;
    let decompressor = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressor);
    if !extract_to.exists() {
        fs::create_dir_all(extract_to)?;
    }
    archive.unpack(extract_to)?;
    Ok(())
}

#[cfg(windows)]
fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use zip::read::ZipArchive;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    if !extract_to.exists() {
        fs::create_dir_all(extract_to)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = extract_to.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
