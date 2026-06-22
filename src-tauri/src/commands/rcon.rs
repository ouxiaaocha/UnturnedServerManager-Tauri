use std::sync::{Arc, Mutex};
use tauri::State;

use crate::commands::server::ActiveRcon;
use crate::services::log_service::LogService;
use crate::services::rcon_client::RconClient;

#[tauri::command(async)]
pub async fn rcon_connect(
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    save_id: String,
) -> Result<String, String> {
    let target_save_id = save_id.trim().to_string();
    if target_save_id.is_empty() {
        return Err("请选择要连接的运行服务器".to_string());
    }

    let (host, port, password) = {
        let ar = active_rcon.lock().unwrap_or_else(|e| e.into_inner());
        let endpoint = ar.endpoint_for_save(&target_save_id).ok_or_else(|| {
            format!(
                "未找到存档 {} 的 RCON 会话配置，请重新启动该服务器后再连接",
                target_save_id
            )
        })?;
        (endpoint.host, endpoint.port, endpoint.password)
    };

    let mut client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    let welcome = client.connect(&host, port, &password).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(&format!("[ERROR] RCON 连接失败 ({}:{}): {}", host, port, e));
        e
    })?;
    client.set_connected_save_id(target_save_id.clone());

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("RCON 连接成功: {}", target_save_id));

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

#[tauri::command]
pub fn rcon_connected_save_id(rcon: State<'_, Arc<Mutex<RconClient>>>) -> Option<String> {
    let client = rcon.lock().unwrap_or_else(|e| e.into_inner());
    client.connected_save_id()
}
