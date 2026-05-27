use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

use crate::services::process::ProcessManager;
use crate::services::rcon_client::RconClient;
use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;

/// Holds the currently active RCON settings for this session.
/// When starting with a non-default save, the RCON settings from that save
/// are stored here so rcon_connect uses them, without overwriting servers.json.
pub struct ActiveRcon {
    pub host: String,
    pub port: u16,
    pub password: String,
}

impl ActiveRcon {
    pub fn from_config(config: &ConfigService) -> Self {
        let servers = config.load_servers_config();
        if let Some(profile) = servers.servers.first() {
            Self {
                host: profile.rcon.host.clone(),
                port: profile.rcon.port,
                password: profile.rcon.password.clone(),
            }
        } else {
            Self {
                host: "127.0.0.1".to_string(),
                port: 27115,
                password: String::new(),
            }
        }
    }
}

#[derive(Serialize)]
pub struct ServerStatus {
    pub state: String,
    pub pid: Option<u32>,
    pub uptime_secs: u64,
    pub output_count: usize,
}

#[tauri::command]
pub fn get_server_status(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
) -> ServerStatus {
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.is_running();
    ServerStatus {
        state: pm.state().to_string(),
        pid: pm.pid(),
        uptime_secs: pm.uptime_secs(),
        output_count: pm.output_count(),
    }
}

#[tauri::command]
pub fn get_server_output(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    from_index: usize,
) -> Vec<String> {
    let pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.get_new_output(from_index)
}

#[tauri::command]
pub fn start_server(
    app: tauri::AppHandle,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    save_id: Option<String>,
    launch_mode: Option<String>,
) -> Result<String, String> {
    let (mut profile, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        let profile = servers.servers.first().ok_or("没有配置服务器")?.clone();
        let server_id = profile.id.clone();
        (profile, server_id)
    };

    // Override save_id and launch mode if provided
    let actual_id = save_id.unwrap_or_else(|| server_id.clone());

    // Validate save_id for path safety
    if actual_id.contains('/') || actual_id.contains('\\') || actual_id.contains("..")
        || actual_id.contains(':') || actual_id.contains('*') || actual_id.contains('?')
        || actual_id.contains('"') || actual_id.contains('<') || actual_id.contains('>')
        || actual_id.contains('|')
    {
        return Err("存档 ID 包含非法字符".to_string());
    }

    let mode = launch_mode.unwrap_or_else(|| "internet".to_string());
    let entry_prefix = if mode == "lan" { "+LanServer" } else { "+InternetServer" };
    profile.server_entry = format!("{}/{}", entry_prefix, actual_id);

    // If using a different save, read RCON settings from that save's Rocket.config.xml
    // Store them in ActiveRcon (session-only) so rcon_connect uses them,
    // WITHOUT overwriting servers.json
    if actual_id != server_id {
        let rocket_config_path = Path::new(&profile.server_root)
            .join("Servers")
            .join(&actual_id)
            .join("Rocket")
            .join("Rocket.config.xml");

        if rocket_config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&rocket_config_path) {
                if let Some(start) = content.find("Port=\"") {
                    let after = start + 6;
                    if let Some(end) = content[after..].find('"') {
                        if let Ok(port) = content[after..after + end].parse::<u16>() {
                            profile.rcon.port = port;
                        }
                    }
                }
                if let Some(start) = content.find("Password=\"") {
                    let after = start + 10;
                    if let Some(end) = content[after..].find('"') {
                        let pwd = content[after..after + end]
                            .replace("&amp;", "&")
                            .replace("&quot;", "\"")
                            .replace("&lt;", "<")
                            .replace("&gt;", ">")
                            .replace("&apos;", "'");
                        profile.rcon.password = pwd;
                    }
                }
            }
        }
    }

    // Update session RCON settings (does not modify servers.json)
    {
        let mut ar = active_rcon.lock().unwrap_or_else(|e| e.into_inner());
        ar.host = profile.rcon.host.clone();
        ar.port = profile.rcon.port;
        ar.password = profile.rcon.password.clone();
    }

    {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        pm.start(&profile).map_err(|e| {
            let ls = log.lock().unwrap_or_else(|e| e.into_inner());
            ls.log_operation(&format!("[ERROR] 启动服务器失败: {}", e));
            e
        })?;
    }

    let mode_str = if mode == "lan" { "局域网" } else { "互联网" };
    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("启动服务器: {} (存档: {}, 模式: {})", server_id, actual_id, mode_str));

    let _ = app.emit("server-started", &actual_id);

    Ok("服务器已启动".to_string())
}

#[tauri::command(async)]
pub async fn stop_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
) -> Result<String, String> {
    // Log the operation (lock acquired and released immediately)
    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("停止服务器");
    }

    // Try RCON graceful shutdown
    let should_send_shutdown = {
        let mut rcon_client = rcon.lock().unwrap_or_else(|e| e.into_inner());
        if !rcon_client.is_connected() {
            let profile = {
                let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
                let servers = cfg.load_servers_config();
                servers.servers.first().map(|p| (p.rcon.host.clone(), p.rcon.port, p.rcon.password.clone()))
            };
            if let Some((host, port, password)) = profile {
                let _ = rcon_client.connect(&host, port, &password);
            }
        }

        if rcon_client.is_connected() {
            let _ = rcon_client.send_command("save");
            true
        } else {
            false
        }
    }; // rcon lock released here

    if should_send_shutdown {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        let mut rcon_client = rcon.lock().unwrap_or_else(|e| e.into_inner());
        let _ = rcon_client.send_command("shutdown");
        rcon_client.disconnect();
    }

    // Wait for process to exit (lock NOT held across await)
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let is_running = {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            pm.is_running()
        };
        if !is_running {
            return Ok("服务器已停止".to_string());
        }
    }

    // Timeout: force stop
    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation("[Warning] 服务器未响应，执行强制停止");
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.force_stop()?;
    Ok("服务器已强制停止".to_string())
}

#[tauri::command]
pub fn force_stop_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<String, String> {
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.force_stop()?;
    Ok("服务器已强制停止".to_string())
}

#[tauri::command(async)]
pub async fn restart_server(
    app: tauri::AppHandle,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    save_id: Option<String>,
    launch_mode: Option<String>,
) -> Result<String, String> {
    // Log the operation
    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("重启服务器");
    }

    // Try RCON graceful shutdown using active session RCON
    let should_send_shutdown = {
        let mut rcon_client = rcon.lock().unwrap_or_else(|e| e.into_inner());
        if !rcon_client.is_connected() {
            let ar = active_rcon.lock().unwrap_or_else(|e| e.into_inner());
            let _ = rcon_client.connect(&ar.host, ar.port, &ar.password);
        }

        if rcon_client.is_connected() {
            let _ = rcon_client.send_command("save");
            true
        } else {
            false
        }
    };

    if should_send_shutdown {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        let mut rcon_client = rcon.lock().unwrap_or_else(|e| e.into_inner());
        let _ = rcon_client.send_command("shutdown");
        rcon_client.disconnect();
    }

    // Wait for process to exit
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
            pm.force_stop()?;
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Reuse start_server logic with the same save_id/launch_mode
    start_server(app, process, config, log, active_rcon, save_id, launch_mode)
}
