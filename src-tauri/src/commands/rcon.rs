use std::sync::{Arc, Mutex};
use tauri::State;

use crate::services::rcon_client::RconClient;
use crate::services::log_service::LogService;
use crate::commands::server::ActiveRcon;

#[tauri::command(async)]
pub async fn rcon_connect(
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
) -> Result<String, String> {
    let (host, port, password) = {
        let ar = active_rcon.lock().unwrap_or_else(|e| e.into_inner());
        (ar.host.clone(), ar.port, ar.password.clone())
    };

    let mut client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    let welcome = client.connect(&host, port, &password).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(&format!("[ERROR] RCON 连接失败 ({}:{}): {}", host, port, e));
        e
    })?;

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation("RCON 连接成功");

    Ok(welcome)
}

#[tauri::command]
pub fn rcon_disconnect(
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
) -> Result<(), String> {
    let mut client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    client.disconnect();
    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation("RCON 断开连接");
    Ok(())
}

#[tauri::command]
pub fn rcon_send(
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    command: String,
) -> Result<(), String> {
    let mut client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    if !client.is_connected() {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("[Warning] RCON 命令发送失败: 未连接");
        return Err("RCON 未连接".to_string());
    }
    client.send_command(&command).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(&format!("[ERROR] RCON 命令发送失败: {}", e));
        e
    })?;
    drop(client);

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    // Sanitize: mask potential password-like commands
    let cmd_lower = command.to_ascii_lowercase();
    let sanitized = if cmd_lower.starts_with("login ") || cmd_lower.starts_with("password ") {
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        format!("{} ******", parts[0])
    } else {
        command
    };
    ls.log_operation(&format!("RCON 命令: {}", sanitized));
    Ok(())
}

#[tauri::command]
pub fn rcon_poll(rcon: State<'_, Arc<Mutex<RconClient>>>) -> Vec<String> {
    let client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    client.get_responses()
}

#[tauri::command]
pub fn rcon_status(rcon: State<'_, Arc<Mutex<RconClient>>>) -> bool {
    let client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    client.is_connected()
}
