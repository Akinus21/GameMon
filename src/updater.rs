use std::fs::{self};
use std::path::Path;
use std::process::Command;
use std::env;
use rfd::{MessageDialog, MessageLevel, MessageButtons};

mod logger;
use game_mon::config::GAMEMON_DIR;

use serde::Deserialize;
use reqwest::header::{ACCEPT, USER_AGENT};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

// GitHub release asset and release structs
#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GithubReleaseFull {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[tokio::main]
async fn main() {
    logger::Logger::init().expect("Failed to initialize logger");

    check_update_marker();

    let current_version = env!("CARGO_PKG_VERSION");
    let current_tag = format!("{}", current_version);

    // Fetch latest release info AND download the asset
    let (latest_tag, archive_path) = match fetch_latest_release_and_download().await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("❌ Failed to fetch and download latest release: {}", err);
            return;
        }
    };

    println!("🛠️  Current: {}, Latest: {}", current_tag, latest_tag);

    if current_tag != latest_tag {
        // Prompt user to update
        let want_update = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("🎉 A new update is available! 🎉")
            .set_description(&format!(
                "You are running version {} but version {} is now available.\nDo you want to update?",
                current_tag, latest_tag
            ))
            .set_buttons(MessageButtons::YesNo)
            .show(); // returns Yes if yes clicked

        log::info!("User update prompt response: {}", want_update);

        if want_update.to_string() == "Yes" {
            // Extract downloaded archive
            let extraction_result = {
                #[cfg(unix)]
                {
                    let tmp_extract_dir = Path::new(&env::temp_dir()).join("GameMon_update");
                    if tmp_extract_dir.exists() {
                        fs::remove_dir_all(&tmp_extract_dir).ok();
                    }
                    extract_tar_gz(Path::new(&archive_path), &tmp_extract_dir)
                }
                #[cfg(windows)]
                {
                    let tmp_extract_dir = Path::new(&env::temp_dir()).join("GameMon_update");
                    if tmp_extract_dir.exists() {
                        fs::remove_dir_all(&tmp_extract_dir).ok();
                    }
                    extract_zip(Path::new(&archive_path), &tmp_extract_dir)
                }
            };

            if let Err(e) = extraction_result {
                let error_file = GAMEMON_DIR.join("update_error.log");
                let err_msg = format!("Extraction failed: {}\n", e);
                fs::write(&error_file, &err_msg).ok();

                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title("❌ Update Failed ❌")
                    .set_description(&format!(
                        "Update extraction failed.\nPlease check the error log at:\n{}",
                        error_file.display()
                    ))
                    .set_buttons(MessageButtons::Ok)
                    .show();

                log::error!("Update extraction failed: {}", e);
                return;
            }

            match run_deferred_installer(
                &env::temp_dir().to_string_lossy(),
                &latest_tag,
            ) {
                Ok(()) => {
                    MessageDialog::new()
                        .set_level(MessageLevel::Info)
                        .set_title("🎉🎉 Update Complete! 🎉🎉")
                        .set_description("Your update was successful! Enjoy the new features! 🎈🎊🎉")
                        .set_buttons(MessageButtons::Ok)
                        .show();
                    log::info!("Update installed successfully.");
                }
                Err(e) => {
                    let error_file = GAMEMON_DIR.join("update_error.log");
                    let err_msg = format!("Installer error: {}\n", e);
                    fs::write(&error_file, &err_msg).ok();

                    MessageDialog::new()
                        .set_level(MessageLevel::Error)
                        .set_title("❌ Update Failed ❌")
                        .set_description(&format!(
                            "Update installation failed.\nPlease check the error log at:\n{}",
                            error_file.display()
                        ))
                        .set_buttons(MessageButtons::Ok)
                        .show();

                    log::error!("Update installation failed: {}", e);
                }
            }
        } else {
            log::info!("User declined the update.");
        }
    } else {
        println!("✅ Already up to date.");
    }
}

async fn fetch_latest_release_and_download() -> Result<(String, String), Box<dyn std::error::Error>> {
    let url = "https://api.github.com/repos/Akinus21/GameMon/releases/latest";
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "GameMon-Updater")
        .header(ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let release: GithubReleaseFull = response.json().await?;
    log::info!("Latest release tag: {}", release.tag_name);

    // Determine platform and extension to match asset filename
    #[cfg(unix)]
    let (platform_substring, archive_ext) = ("linux", "tar.gz");
    #[cfg(windows)]
    let (platform_substring, archive_ext) = ("windows", "zip");

    // Find the asset matching platform and extension
    let asset_opt = release.assets.iter().find(|asset| {
        let name = asset.name.to_lowercase();
        name.contains(platform_substring) && name.ends_with(archive_ext)
    });

    let asset = match asset_opt {
        Some(a) => a,
        None => return Err("No matching asset found for this platform".into()),
    };

    // Download the asset to temp dir
    let tmp_dir = std::env::temp_dir();
    let archive_filename = format!("GameMon-update.{}", archive_ext);
    let archive_path = tmp_dir.join(&archive_filename);

    log::info!("Downloading asset: {} to {:?}", asset.name, archive_path);

    let mut resp = client.get(&asset.browser_download_url).send().await?;

    if !resp.status().is_success() {
        return Err(format!("Failed to download asset: HTTP {}", resp.status()).into());
    }

    let mut file = File::create(&archive_path).await?;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    log::info!("Downloaded asset successfully.");

    Ok((release.tag_name, archive_path.to_string_lossy().to_string()))
}

fn run_deferred_installer(tmp_dir: &str, latest_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let script_path = Path::new("/tmp/GameMon-updater-install.sh");
        let update_marker = GAMEMON_DIR.join(".update-pending");

        fs::write(&update_marker, latest_version)?;

        // Use $HOME for user path in script
        let installer_script = format!(r#"#!/bin/bash
sleep 2

pkill -x GameMon-service || true
sleep 1
cp -r "{tmp_dir}/GameMon_update/GameMon-service" "$HOME/.local/share/gamemon/GameMon-service"

pkill -x GameMon-gui || true
sleep 1
cp -r "{tmp_dir}/GameMon_update/GameMon-gui" "$HOME/.local/share/gamemon/GameMon-gui"

pkill -x GameMon-update || true
sleep 1
cp -r "{tmp_dir}/GameMon_update/GameMon-update" "$HOME/.local/share/gamemon/GameMon-update"

cp -r "{tmp_dir}/GameMon_update/resources" "$HOME/.local/share/gamemon/resources"

chmod +x "$HOME/.local/share/gamemon/GameMon-service"
chmod +x "$HOME/.local/share/gamemon/GameMon-gui"
chmod +x "$HOME/.local/share/gamemon/GameMon-update"

rm -rf "{tmp_dir}/GameMon_update"
rm -f "{tmp_dir}/GameMon-update.tar.gz"
rm -f "{update_marker}"
touch .update-complete

"$HOME/.local/share/gamemon/GameMon-service" &

rm -- "$0"
"#,
            tmp_dir = tmp_dir,
            update_marker = update_marker.to_str().unwrap()
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

        // Use %APPDATA%\gamemon for user path in script
        let installer_script = format!(r#"
@echo off
timeout /T 2 /NOBREAK

set "GAMEMON_DIR=%APPDATA%\gamemon"
set "TMPDIR=%TEMP%\GameMon_update"

REM Kill running processes
taskkill /F /IM GameMon-service.exe >nul 2>&1
taskkill /F /IM GameMon-gui.exe >nul 2>&1
taskkill /F /IM GameMon.exe >nul 2>&1
timeout /T 1 >nul

REM Copy updated files
xcopy /Y "%TMPDIR%\GameMon-service.exe" "%GAMEMON_DIR%\"
xcopy /Y "%TMPDIR%\GameMon-gui.exe" "%GAMEMON_DIR%\"
xcopy /E /I /Y "%TMPDIR%\GameMon-update" "%GAMEMON_DIR%\GameMon-update"
xcopy /E /I /Y "%TMPDIR%\resources" "%GAMEMON_DIR%\resources"

REM Cleanup
rmdir /S /Q "%TMPDIR%" 2>nul
del "%TEMP%\GameMon-update.zip" 2>nul
del "%GAMEMON_DIR%\.update-pending" 2>nul
type nul > "%GAMEMON_DIR%\.update_complete"

REM Restart service
start "" "%GAMEMON_DIR%\GameMon-service.exe"

REM Delete this script
del "%~f0"

"#,
            tmpdir = tmp_dir.replace("\\", "\\\\"),
            update_marker = update_marker.to_str().unwrap().replace("\\", "\\\\")
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
        log::warn!("⚠️ Previous update may have failed or is in progress.");
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
