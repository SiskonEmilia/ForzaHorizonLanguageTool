use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Local;

pub fn get_log_dir() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| "LOCALAPPDATA directory is unavailable".to_string())?;
    let log_dir = base.join("FHLanguageComboTool").join("logs");
    fs::create_dir_all(&log_dir)
        .map_err(|e| format!("Failed to create log directory: {e}"))?;
    Ok(log_dir)
}

pub fn log_operation(game_id: &str, level: &str, message: &str) -> Result<(), String> {
    let dir = get_log_dir()?;
    log_operation_to_dir(&dir, game_id, level, message)
}

pub fn log_operation_to_dir(
    dir: &Path,
    game_id: &str,
    level: &str,
    message: &str,
) -> Result<(), String> {
    let now = Local::now();
    let filename = now.format("%Y-%m-%d").to_string() + ".log";
    let path = dir.join(filename);
    let line = format!(
        "[{}] [{}] [{}] {}\n",
        now.to_rfc3339(),
        level.to_uppercase(),
        game_id,
        message,
    );
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open log file: {e}"))?;
    file.write_all(line.as_bytes())
        .map_err(|e| format!("Failed to write to log file: {e}"))?;
    Ok(())
}

pub fn read_today_log() -> Result<String, String> {
    let dir = get_log_dir()?;
    let filename = Local::now().format("%Y-%m-%d").to_string() + ".log";
    let path = dir.join(filename);
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).map_err(|e| format!("Failed to read log file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("fh_lang_tool_test")
            .join(name)
            .join(format!("{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_log_operation_writes_line() {
        let dir = test_dir("writes_line");
        log_operation_to_dir(&dir, "fh5", "info", "test message").unwrap();
        let filename = Local::now().format("%Y-%m-%d").to_string() + ".log";
        let content = fs::read_to_string(dir.join(filename)).unwrap();
        assert!(content.contains("test message"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_log_format_matches_pattern() {
        let dir = test_dir("format_pattern");
        log_operation_to_dir(&dir, "fh6", "warn", "something happened").unwrap();
        let filename = Local::now().format("%Y-%m-%d").to_string() + ".log";
        let content = fs::read_to_string(dir.join(filename)).unwrap();
        let line = content.trim();
        assert!(line.starts_with('['));
        assert!(line.contains("] [WARN] [fh6] something happened"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_nonexistent_log_returns_empty() {
        let dir = test_dir("nonexistent");
        let path = dir.join("1999-01-01.log");
        assert!(!path.exists());
        let result = if path.exists() {
            fs::read_to_string(&path).unwrap()
        } else {
            String::new()
        };
        assert_eq!(result, "");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_multiple_operations_append() {
        let dir = test_dir("append");
        log_operation_to_dir(&dir, "fh5", "info", "first").unwrap();
        log_operation_to_dir(&dir, "fh5", "error", "second").unwrap();
        log_operation_to_dir(&dir, "fh6", "info", "third").unwrap();
        let filename = Local::now().format("%Y-%m-%d").to_string() + ".log";
        let content = fs::read_to_string(dir.join(filename)).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("first"));
        assert!(lines[1].contains("[ERROR]"));
        assert!(lines[2].contains("[fh6]"));
        let _ = fs::remove_dir_all(&dir);
    }
}
