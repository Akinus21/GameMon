use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use std::fs;
use std::env;

#[cfg(target_os = "linux")]
mod platform_logger {
    use std::process::Command;

    pub struct PlatformLogger;

    impl PlatformLogger {
        pub fn new() -> Self {
            PlatformLogger
        }

        pub fn log(&self, level: &str, msg: &str) {
            let full_command = format!(
                "logger -p user.{} -t {} '{}'",
                level,
                crate::logger::get_project_name(),
                msg.replace('\'', "'\\''") // escape single quotes
            );

            let _ = Command::new("sh")
                .arg("-c")
                .arg(&full_command)
                .output();
        }
    }
}

#[cfg(target_os = "windows")]
mod platform_logger {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::HANDLE,
            System::EventLog::{
                RegisterEventSourceW, DeregisterEventSource, ReportEventW,
                EVENTLOG_ERROR_TYPE, EVENTLOG_WARNING_TYPE, EVENTLOG_INFORMATION_TYPE,
            },
        }
    };
    use std::ffi::{OsStr, OsStrExt};
    
    pub struct PlatformLogger {
        handle: HANDLE,
    }

    impl PlatformLogger {
        pub fn new() -> Self {
            unsafe {
                let source = to_wide(&get_project_name());
                let handle = RegisterEventSourceW(std::ptr::null_mut(), PCWSTR(source.as_ptr()));
                Self { handle }
            }
        }

        pub fn log(&self, level: u16, msg: &str) {
            unsafe {
                let wide_msg = to_wide(msg);
                let strings = [PCWSTR(wide_msg.as_ptr())];
                ReportEventW(
                    self.handle,
                    level,
                    0,
                    0x1000, // custom event ID
                    None,
                    1,
                    0,
                    strings.as_ptr(),
                    std::ptr::null(),
                );
            }
        }
    }

    impl Drop for PlatformLogger {
        fn drop(&mut self) {
            unsafe {
                DeregisterEventSource(self.handle);
            }
        }
    }

    fn to_wide(s: &str) -> Vec<u16> {
        OsStrExt::to_bytes(OsStr::new(s)).chain(Some(0)).collect()
    }
}

fn get_project_name() -> String {
    // Try CARGO_MANIFEST_DIR first
    if let Ok(cargo_dir) = env::var("CARGO_MANIFEST_DIR") {
        let file_path = format!("{}/Cargo.toml", cargo_dir);
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Some(name_line) = content.lines().find(|line| line.trim().starts_with("name")) {
                let parts: Vec<&str> = name_line.split('=').collect();
                if parts.len() == 2 {
                    return parts[1].trim().trim_matches('"').to_string();
                }
            }
        }
    }

    // Fallback: use binary name
    env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "GameMon".to_string())
}

pub struct Logger {
    inner: Box<platform_logger::PlatformLogger>,
}

impl Logger {
    pub fn init() -> Result<(), SetLoggerError> {
        let logger = Box::new(Logger {
            inner: Box::new(platform_logger::PlatformLogger::new()),
        });

        // Leak the box to get a 'static reference
        let leaked = Box::leak(logger);
        
        log::set_logger(leaked).unwrap();
        log::set_max_level(LevelFilter::Info);

        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info // Adjust as needed (include Warn, Error, etc)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            #[cfg(target_os = "linux")]
            {
                let level = match record.level() {
                    Level::Error => "err",
                    Level::Warn => "warning",
                    Level::Info => "info",
                    Level::Debug => "debug",
                    Level::Trace => "debug",
                };
                self.inner.log(level, &format!("{}", record.args()));
            }

            #[cfg(target_os = "windows")]
            {
                let level = match record.level() {
                    Level::Error => EVENTLOG_ERROR_TYPE,
                    Level::Warn => EVENTLOG_WARNING_TYPE,
                    Level::Info => EVENTLOG_INFORMATION_TYPE,
                    Level::Debug => EVENTLOG_INFORMATION_TYPE,
                    Level::Trace => EVENTLOG_INFORMATION_TYPE,
                };
                self.inner.log(level, &format!("{}", record.args()));
            }
        }
    }

    fn flush(&self) {}
}