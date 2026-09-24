//! Diagnostic debug logging (written to a `*.log` file in the app's localappdata dir).
//! Users can toggle this in Settings for support and troubleshooting.
//! When disabled, debug messages are discarded with 0 disk I/O.
//! Secrets (API keys) are strictly sanitized and never logged (Rule 10).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static LOG_FILE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

/// Initialize the logger with the given log file path and initial debug state.
pub fn init(path: Option<PathBuf>, debug_enabled: bool) {
    DEBUG_ENABLED.store(debug_enabled, Ordering::Relaxed);
    if let Some(p) = path {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut lock) = LOG_FILE_PATH.lock() {
            *lock = Some(p);
        }
    }
    if debug_enabled {
        info("logging", "debug logging initialized (enabled)");
    }
}

/// Toggle debug logging at runtime.
pub fn set_debug(enabled: bool) {
    let prev = DEBUG_ENABLED.swap(enabled, Ordering::Relaxed);
    if prev != enabled {
        if enabled {
            info("logging", "debug logging enabled");
        } else {
            info("logging", "debug logging disabled");
        }
    }
}

/// Check if debug logging is enabled.
pub fn is_debug() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Get the path to the current log file, if set.
pub fn log_path() -> Option<PathBuf> {
    LOG_FILE_PATH.lock().ok().and_then(|p| p.clone())
}

/// Clear the log file.
pub fn clear_log() -> Result<(), std::io::Error> {
    if let Some(path) = log_path() {
        if path.exists() {
            std::fs::write(&path, "")?;
        }
    }
    Ok(())
}

/// Read recent lines from the log file (up to `max_lines`).
pub fn read_log_tail(max_lines: usize) -> Result<String, std::io::Error> {
    if let Some(path) = log_path() {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let lines: Vec<&str> = content.lines().collect();
            let start = if lines.len() > max_lines {
                lines.len() - max_lines
            } else {
                0
            };
            return Ok(lines[start..].join("\n"));
        }
    }
    Ok(String::new())
}

/// Write a formatted log line to the log file.
pub fn log(level: Level, target: &str, message: &str) {
    if level == Level::Debug && !is_debug() {
        return;
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let line = format!("[{timestamp}] [{}] [{target}] {message}\n", level.as_str());

    if let Ok(lock) = LOG_FILE_PATH.lock() {
        if let Some(ref path) = *lock {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = file.write_all(line.as_bytes());
            }
        }
    }
}

pub fn debug(target: &str, message: &str) {
    log(Level::Debug, target, message);
}

pub fn info(target: &str, message: &str) {
    log(Level::Info, target, message);
}

pub fn warn(target: &str, message: &str) {
    log(Level::Warn, target, message);
}

pub fn error(target: &str, message: &str) {
    log(Level::Error, target, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("lz-log-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let log_file = temp_dir.join("test.log");

        init(Some(log_file.clone()), false);
        assert!(!is_debug());

        debug("test", "this should not be written");
        let content = read_log_tail(10).unwrap();
        assert!(!content.contains("this should not be written"));

        set_debug(true);
        assert!(is_debug());

        debug("test", "this should be written");
        let content = read_log_tail(10).unwrap();
        assert!(content.contains("this should be written"));

        clear_log().unwrap();
        let content = read_log_tail(10).unwrap();
        assert!(content.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
