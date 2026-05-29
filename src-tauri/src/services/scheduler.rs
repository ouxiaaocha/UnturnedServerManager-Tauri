use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{Datelike, Local, Timelike};

use crate::commands::schedule::ScheduleTask;
use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;
use crate::services::process::ProcessManager;
use crate::services::rcon_client::RconClient;

pub fn start_scheduler(
    config: Arc<Mutex<ConfigService>>,
    process: Arc<Mutex<ProcessManager>>,
    rcon: Arc<Mutex<RconClient>>,
    log: Arc<Mutex<LogService>>,
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

            // Daily log cleanup (once per day)
            if current_day != last_cleanup_day {
                last_cleanup_day = current_day;
                let retention_days = {
                    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
                    cfg.load_app_settings().log_retention_days
                };
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.cleanup_old_logs(retention_days);
            }

            // Only check once per minute
            if current_total_minutes == last_check_minute {
                continue;
            }
            last_check_minute = current_total_minutes;

            // Load tasks
            let tasks = load_tasks(&config);
            if tasks.is_empty() {
                continue;
            }

            // Check if server is running
            let is_running = {
                let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
                pm.is_running()
            };

            if !is_running {
                continue;
            }

            // Calculate weekday (0=Sunday)
            let weekday = now.weekday().num_days_from_sunday() as u8;

            for task in &tasks {
                if !task.enabled {
                    continue;
                }

                let minutes_until =
                    minutes_until_restart(task, current_hour, current_minute, weekday);

                if let Some(mins) = minutes_until {
                    // Check if announcement needed
                    let task_announced = announced.entry(task.id.clone()).or_default();
                    for &announce_at in &task.announce_minutes {
                        if mins == announce_at && !task_announced.contains(&announce_at) {
                            send_announce(&rcon, &config, &log, announce_at);
                            task_announced.push(announce_at);
                        }
                    }

                    // Time to restart
                    if mins == 0 {
                        execute_restart(&process, &rcon, &config, &log);
                        announced.remove(&task.id);
                    }
                } else {
                    // Not in restart window, clear announcement records
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
        }
    }

    #[test]
    fn interval_restart_returns_zero_at_cycle_boundary() {
        let task = interval_task(6);

        assert_eq!(minutes_until_restart(&task, 6, 0, 0), Some(0));
    }
}

fn send_announce(
    rcon: &Arc<Mutex<RconClient>>,
    config: &Arc<Mutex<ConfigService>>,
    log: &Arc<Mutex<LogService>>,
    minutes: u32,
) {
    let msg = format!("say 服务器将在 {} 分钟后重启", minutes);

    // Extract RCON config before acquiring rcon lock
    let rcon_config = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        servers
            .servers
            .first()
            .map(|p| (p.rcon.host.clone(), p.rcon.port, p.rcon.password.clone()))
    };

    let mut client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    if !client.is_connected() {
        if let Some((host, port, password)) = rcon_config {
            let _ = client.connect(&host, port, &password);
        }
    }

    if client.is_connected() {
        let _ = client.send_command(&msg);
    }

    drop(client);

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("定时任务公告: {}分钟后重启", minutes));
}

fn execute_restart(
    process: &Arc<Mutex<ProcessManager>>,
    rcon: &Arc<Mutex<RconClient>>,
    config: &Arc<Mutex<ConfigService>>,
    log: &Arc<Mutex<LogService>>,
) {
    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("定时任务: 执行重启");
    }

    // Extract RCON config before acquiring rcon lock
    let rcon_config = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        servers
            .servers
            .first()
            .map(|p| (p.rcon.host.clone(), p.rcon.port, p.rcon.password.clone()))
    };

    // Shutdown via RCON
    {
        let mut client = rcon.lock().unwrap_or_else(|e| e.into_inner());
        if !client.is_connected() {
            if let Some((host, port, password)) = rcon_config {
                let _ = client.connect(&host, port, &password);
            }
        }

        if client.is_connected() {
            let _ = client.send_command("save");
            std::thread::sleep(Duration::from_secs(1));
            let _ = client.send_command("shutdown");
            client.disconnect();
        }
    }

    // Wait for process to exit
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

    // Force stop if still running
    {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        if pm.is_running() {
            let _ = pm.force_stop();
        }
    }

    std::thread::sleep(Duration::from_secs(2));

    // Restart
    let profile = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        servers.servers.first().cloned()
    };
    if let Some(profile) = profile {
        {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            let _ = pm.start(&profile);
        }
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("定时任务: 服务器已重启");
    }
}
