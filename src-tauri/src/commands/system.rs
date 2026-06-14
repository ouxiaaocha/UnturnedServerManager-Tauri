use std::sync::{Arc, Mutex};
use tauri::State;

use crate::models::system_stats::SystemStats;
use crate::services::system_monitor::SystemMonitor;

#[tauri::command(async)]
pub async fn get_system_stats(
    monitor: State<'_, Arc<Mutex<SystemMonitor>>>,
) -> Result<SystemStats, String> {
    let monitor = monitor.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut m = monitor.lock().unwrap_or_else(|e| e.into_inner());
        m.stats()
    })
    .await
    .map_err(|e| format!("读取系统信息任务失败: {}", e))
}
