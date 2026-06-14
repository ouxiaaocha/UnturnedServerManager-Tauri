use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{Datelike, Local, Timelike};

use crate::commands::schedule::ScheduleTask;
use crate::commands::server::AutoUpdateState;
use crate::services::config_service::ConfigService;
use crate::services::local_command_bridge;
use crate::services::log_service::LogService;
use crate::services::process::ProcessManager;

/// 启动后台调度线程，每 10 秒检查一次，匹配到重启时间时通过本地命令桥发送公告并执行重启
pub fn start_scheduler(
    config: Arc<Mutex<ConfigService>>,
    process: Arc<Mutex<ProcessManager>>,
    log: Arc<Mutex<LogService>>,
    auto_update: Arc<Mutex<AutoUpdateState>>,
) {
    std::thread::spawn(move || {
        let mut last_check_minute: i64 = -1;
        let mut last_cleanup_day: i32 = -1;
        let mut announced: std::collections::HashMap<String, Vec<u32>> =
            std::collections::HashMap::new();

        loop {
            std::thread::sleep(Duration::from_secs(10));

            let now = Local::now();
            let current_hour = now.hour();
            let current_minute = now.minute();
            let current_total_minutes = (current_hour * 60 + current_minute) as i64;
            let current_day = now.ordinal() as i32;

            // 每天执行一次日志清理
            if current_day != last_cleanup_day {
                last_cleanup_day = current_day;
                let retention_days = {
                    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
                    cfg.load_app_settings().log_retention_days
                };
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.cleanup_old_logs(retention_days);
            }

            // 每分钟只检查一次任务
            if current_total_minutes == last_check_minute {
                continue;
            }
            last_check_minute = current_total_minutes;

            let tasks = load_tasks(&config);
            if tasks.is_empty() {
                continue;
            }

            let is_running = {
                let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
                pm.is_running()
            };

            if !is_running {
                continue;
            }

            // 每小时清理一次不在当前任务列表中的孤儿条目，防止内存泄露
            if current_minute == 0 {
                let active_ids: std::collections::HashSet<String> = tasks.iter()
                    .map(|t| t.id.clone())
                    .collect();
                announced.retain(|id, _| active_ids.contains(id));
            }

            // weekday: 0=周日
            let weekday = now.weekday().num_days_from_sunday() as u8;

            for task in &tasks {
                if !task.enabled {
                    continue;
                }

                let minutes_until =
                    minutes_until_restart(task, current_hour, current_minute, weekday);

                if let Some(mins) = minutes_until {
                    // 检查是否需要发送公告
                    let task_announced = announced.entry(task.id.clone()).or_default();
                    for &announce_at in &task.announce_minutes {
                        if mins == announce_at && !task_announced.contains(&announce_at) {
                            send_announce(
                                &process,
                                &config,
                                &log,
                                &auto_update,
                                announce_at,
                                task.server_id.as_deref(),
                            );
                            task_announced.push(announce_at);
                        }
                    }

                    if mins == 0 {
                        execute_restart(
                            &process,
                            &config,
                            &log,
                            &auto_update,
                            task.server_id.as_deref(),
                        );
                        announced.remove(&task.id);
                    }
                } else {
                    // 不在重启窗口内，清除公告记录
                    announced.remove(&task.id);
                }
            }
        }
    });
}

fn load_tasks(config: &Arc<Mutex<ConfigService>>) -> Vec<ScheduleTask> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let path = cfg.config_dir().join("schedules.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let schedule: crate::commands::schedule::ScheduleConfig =
            serde_json::from_str(&content).unwrap_or_default();
        schedule.tasks
    } else {
        vec![]
    }
}

/// 根据 server_id 查找服务器配置并 clone profile
fn find_server_profile(
    config: &Arc<Mutex<ConfigService>>,
    server_id: Option<&str>,
) -> Option<crate::models::config::ServerProfile> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let servers = cfg.load_servers_config();
    if let Some(id) = server_id {
        servers.servers.iter().find(|s| s.id == id).cloned()
    } else {
        servers.servers.first().cloned()
    }
}

fn resolve_command_profile(
    config: &Arc<Mutex<ConfigService>>,
    auto_update: &Arc<Mutex<AutoUpdateState>>,
    server_id: Option<&str>,
) -> Option<crate::models::config::ServerProfile> {
    let mut profile = find_server_profile(config, server_id)?;
    if server_id.is_none() {
        let state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        profile.id = state.current_save_id(&profile.id);
        let entry_prefix = if state.current_launch_mode() == "lan" {
            "+LanServer"
        } else {
            "+InternetServer"
        };
        profile.server_entry = format!("{}/{}", entry_prefix, profile.id);
    }
    Some(profile)
}

fn minutes_until_restart(
    task: &ScheduleTask,
    current_hour: u32,
    current_minute: u32,
    weekday: u8,
) -> Option<u32> {
    let current_total = current_hour * 60 + current_minute;

    match task.task_type.as_str() {
        "daily" => {
            if let Some(ref time_str) = task.time {
                let target = parse_time(time_str)?;
                if current_total <= target && target - current_total <= 30 {
                    return Some(target - current_total);
                }
            }
            None
        }
        "weekly" => {
            if task.weekday != Some(weekday) {
                return None;
            }
            if let Some(ref time_str) = task.time {
                let target = parse_time(time_str)?;
                if current_total <= target && target - current_total <= 30 {
                    return Some(target - current_total);
                }
            }
            None
        }
        "interval" => {
            if let Some(interval_hours) = task.interval_hours {
                let interval_mins = interval_hours * 60;
                if interval_mins == 0 {
                    return None;
                }
                let mins_into_cycle = current_total % interval_mins;
                let mins_remaining = (interval_mins - mins_into_cycle) % interval_mins;
                if mins_remaining <= 30 {
                    return Some(mins_remaining);
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_time(time_str: &str) -> Option<u32> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let h: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        if h > 23 || m > 59 {
            return None;
        }
        Some(h * 60 + m)
    } else {
        None
    }
}

fn send_announce(
    process: &Arc<Mutex<ProcessManager>>,
    config: &Arc<Mutex<ConfigService>>,
    log: &Arc<Mutex<LogService>>,
    auto_update: &Arc<Mutex<AutoUpdateState>>,
    minutes: u32,
    server_id: Option<&str>,
) {
    let msg = format!("say 服务器将在 {} 分钟后重启", minutes);
    if let Some(profile) = resolve_command_profile(config, auto_update, server_id) {
        match local_command_bridge::enqueue_command(&profile.server_root, &profile.id, &msg) {
            Ok(command) => {
                let pm = process.lock().unwrap_or_else(|e| e.into_inner());
                pm.record_sent_command(&command);
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_operation(&format!("定时任务公告: {}分钟后重启", minutes));
            }
            Err(e) => {
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_operation(&format!("[Warning] 定时任务公告发送失败: {}", e));
            }
        }
    }
}

fn execute_restart(
    process: &Arc<Mutex<ProcessManager>>,
    config: &Arc<Mutex<ConfigService>>,
    log: &Arc<Mutex<LogService>>,
    auto_update: &Arc<Mutex<AutoUpdateState>>,
    server_id: Option<&str>,
) {
    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("定时任务: 执行重启");
    }

    let profile = match resolve_command_profile(config, auto_update, server_id) {
        Some(profile) => profile,
        None => {
            let ls = log.lock().unwrap_or_else(|e| e.into_inner());
            ls.log_operation("[ERROR] 定时任务重启失败: 没有配置服务器");
            return;
        }
    };

    for command in ["save", "shutdown"] {
        match local_command_bridge::enqueue_command(&profile.server_root, &profile.id, command) {
            Ok(sent) => {
                let pm = process.lock().unwrap_or_else(|e| e.into_inner());
                pm.record_sent_command(&sent);
            }
            Err(e) => {
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_operation(&format!(
                    "[Warning] 定时任务命令发送失败 ({}): {}",
                    command, e
                ));
            }
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    // 等待进程退出，最多 30 秒
    for _ in 0..30 {
        std::thread::sleep(Duration::from_secs(1));
        let is_running = {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            pm.is_running()
        };
        if !is_running {
            break;
        }
    }

    // 仍在运行则强制停止
    {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        if pm.is_running() {
            let _ = pm.force_stop();
        }
    }

    std::thread::sleep(Duration::from_secs(2));

    let _ = local_command_bridge::ensure_bridge_installed(&profile.server_root, &profile.id);
    {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        let _ = pm.start(&profile);
    }
    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation("定时任务: 服务器已重启");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interval_task(hours: u32) -> ScheduleTask {
        ScheduleTask {
            id: "interval".to_string(),
            enabled: true,
            task_type: "interval".to_string(),
            time: None,
            interval_hours: Some(hours),
            weekday: None,
            announce_minutes: vec![30, 10, 5, 1],
            server_id: None,
        }
    }

    #[test]
    fn interval_restart_returns_zero_at_cycle_boundary() {
        let task = interval_task(6);

        assert_eq!(minutes_until_restart(&task, 6, 0, 0), Some(0));
    }
}
