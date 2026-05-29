use std::fs;
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::services::config_service::ConfigService;

const VALID_CATEGORIES: &[&str] = &["app", "operation", "game"];

fn validate_category(category: &str) -> Result<(), String> {
    if !VALID_CATEGORIES.contains(&category) {
        return Err(format!("无效的日志类别: {}", category));
    }
    Ok(())
}

fn validate_date(date: &str) -> Result<(), String> {
    if date.len() != 10 {
        return Err("日期格式无效".to_string());
    }
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err("日期格式无效".to_string());
    }
    let y: u32 = parts[0].parse().map_err(|_| "年份无效")?;
    let m: u32 = parts[1].parse().map_err(|_| "月份无效")?;
    let d: u32 = parts[2].parse().map_err(|_| "日期无效")?;
    if !(2000..=2100).contains(&y) || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err("日期范围无效".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn read_log_file(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    category: String,
    date: String,
) -> Result<Vec<String>, String> {
    validate_category(&category)?;
    validate_date(&date)?;

    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let log_dir = cfg.logs_dir();
    let file_path = log_dir.join(&category).join(format!("{}.log", date));

    // Ensure the resolved path is still under the logs directory
    if let Ok(resolved) = file_path.canonicalize() {
        if !resolved.starts_with(log_dir.canonicalize().unwrap_or_default()) {
            return Err("路径越界".to_string());
        }
    }

    if !file_path.exists() {
        return Ok(vec![format!("暂无 {} 的日志记录", date)]);
    }

    let content = fs::read_to_string(&file_path).map_err(|e| format!("读取失败: {}", e))?;

    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    Ok(lines)
}

#[tauri::command]
pub fn list_log_dates(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    category: String,
) -> Vec<String> {
    if validate_category(&category).is_err() {
        return vec![];
    }

    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let log_dir = cfg.logs_dir().join(&category);

    if !log_dir.exists() {
        return vec![];
    }

    let mut dates: Vec<String> = fs::read_dir(&log_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".log") {
                        let date_part = name.trim_end_matches(".log").to_string();
                        if validate_date(&date_part).is_ok() {
                            return Some(date_part);
                        }
                    }
                    None
                })
                .collect()
        })
        .unwrap_or_default();

    dates.sort();
    dates.reverse();
    dates
}
