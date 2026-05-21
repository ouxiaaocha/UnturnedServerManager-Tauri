use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::{Local, Duration as ChronoDuration};

pub struct LogService {
    logs_dir: PathBuf,
}

impl LogService {
    pub fn new(logs_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(logs_dir.join("app"));
        let _ = fs::create_dir_all(logs_dir.join("operation"));
        let _ = fs::create_dir_all(logs_dir.join("game"));
        Self { logs_dir }
    }

    pub fn log_app(&self, message: &str) {
        self.write_log("app", message);
    }

    pub fn log_operation(&self, message: &str) {
        self.write_log("operation", message);
    }

    pub fn log_game(&self, message: &str) {
        self.write_log("game", message);
    }

    pub fn cleanup_old_logs(&self, retention_days: u32) {
        let cutoff = Local::now() - ChronoDuration::days(retention_days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

        for category in &["app", "operation", "game"] {
            let dir = self.logs_dir.join(category);
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if let Some(date_part) = name.strip_suffix(".log") {
                        if date_part < cutoff_str.as_str() {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }

    fn write_log(&self, category: &str, message: &str) {
        let now = Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let time = now.format("%H:%M:%S").to_string();
        let file_path = self.logs_dir.join(category).join(format!("{}.log", date));

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            let _ = writeln!(file, "[{}] {}", time, message);
        }
    }
}
