use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::services::config_service::{atomic_write, ConfigService};

/// 定时重启任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTask {
    pub id: String,
    pub enabled: bool,
    /// 任务类型: "daily"（每日）、"interval"（固定间隔）、"weekly"（每周）
    #[serde(rename = "type")]
    pub task_type: String,
    /// 执行时间，格式 "HH:MM"，用于 daily 和 weekly 类型
    pub time: Option<String>,
    /// 间隔小时数，用于 interval 类型
    pub interval_hours: Option<u32>,
    /// 星期几（0=周日），用于 weekly 类型
    pub weekday: Option<u8>,
    /// 重启前发送公告的分钟数列表
    pub announce_minutes: Vec<u32>,
    /// 指定操作的服务器 ID，为空时默认使用第一个服务器
    #[serde(default)]
    pub server_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleConfig {
    pub tasks: Vec<ScheduleTask>,
}

#[tauri::command]
pub fn get_schedules(config: State<'_, Arc<Mutex<ConfigService>>>) -> ScheduleConfig {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let path = cfg.config_dir().join("schedules.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ScheduleConfig::default()
    }
}

#[tauri::command]
pub fn save_schedules(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    tasks: ScheduleConfig,
) -> Result<String, String> {
    for task in &tasks.tasks {
        match task.task_type.as_str() {
            "daily" => {
                if task.time.is_none() {
                    return Err("每日任务必须指定时间".to_string());
                }
            }
            "weekly" => {
                if task.time.is_none() {
                    return Err("每周任务必须指定时间".to_string());
                }
                if task.weekday.is_none() || task.weekday.unwrap() > 6 {
                    return Err("每周任务必须指定星期几 (0-6)".to_string());
                }
            }
            "interval" => {
                if let Some(hours) = task.interval_hours {
                    if hours == 0 || hours > 168 {
                        return Err("间隔时间必须在 1-168 小时之间".to_string());
                    }
                } else {
                    return Err("间隔任务必须指定间隔时间".to_string());
                }
            }
            _ => return Err(format!("无效的任务类型: {}", task.task_type)),
        }
    }

    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let path = cfg.config_dir().join("schedules.json");
    let content = serde_json::to_string_pretty(&tasks).map_err(|e| format!("序列化失败: {}", e))?;
    atomic_write(&path, &content)?;
    Ok("定时任务已保存".to_string())
}
