use std::sync::{Arc, Mutex};
use tauri::State;

use crate::models::system_stats::SystemStats;
use crate::services::system_monitor::SystemMonitor;

#[tauri::command]
pub fn get_system_stats(
    monitor: State<'_, Arc<Mutex<SystemMonitor>>>,
) -> SystemStats {
    let mut m = monitor.lock().unwrap_or_else(|e| e.into_inner());
    m.stats()
}
