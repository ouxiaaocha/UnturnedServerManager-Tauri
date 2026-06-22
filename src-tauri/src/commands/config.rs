use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::models::config::{AppSettings, RconConfig, ServerProfile, ServersConfig};
use crate::services::config_service::ConfigService;

fn contains_chinese(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}' | '\u{f900}'..='\u{faff}'))
}

#[derive(Serialize)]
pub struct DetectResult {
    pub steam_cmd_path: Option<String>,
    pub server_root: Option<String>,
    pub server_id: Option<String>,
}

#[tauri::command]
pub fn auto_detect_paths() -> DetectResult {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    let mut result = DetectResult {
        steam_cmd_path: None,
        server_root: None,
        server_id: None,
    };

    let mut steam_candidates = vec![
        exe_dir.join("SteamCMD").join("steamcmd.exe"),
        exe_dir.join("steamcmd.exe"),
        Path::new("C:/steamcmd/steamcmd.exe").to_path_buf(),
        Path::new("D:/steamcmd/steamcmd.exe").to_path_buf(),
        Path::new("E:/steamcmd/steamcmd.exe").to_path_buf(),
        Path::new("C:/Program Files/SteamCMD/steamcmd.exe").to_path_buf(),
        Path::new("D:/Program Files/SteamCMD/steamcmd.exe").to_path_buf(),
    ];
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        steam_candidates.push(
            Path::new(&userprofile)
                .join("steamcmd")
                .join("steamcmd.exe"),
        );
    }

    for path in &steam_candidates {
        if path.exists() {
            result.steam_cmd_path = Some(path.to_string_lossy().to_string());
            break;
        }
    }

    if let Some(ref steam_path) = result.steam_cmd_path {
        let steam_dir = Path::new(steam_path).parent().unwrap_or(Path::new(""));
        let u3ds = steam_dir.join("steamapps").join("common").join("U3DS");
        if u3ds.join("Unturned.exe").exists() {
            result.server_root = Some(u3ds.to_string_lossy().to_string());

            let servers_dir = u3ds.join("Servers");
            if servers_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&servers_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            result.server_id =
                                Some(entry.file_name().to_string_lossy().to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    result
}

#[tauri::command]
pub fn get_config(config: State<'_, Arc<Mutex<ConfigService>>>) -> ServersConfig {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.load_servers_config()
}

#[tauri::command]
pub fn get_app_settings(config: State<'_, Arc<Mutex<ConfigService>>>) -> AppSettings {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.load_app_settings()
}

#[tauri::command]
pub fn set_auto_update_hosting(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    enabled: bool,
    save_id: Option<String>,
) -> Result<AppSettings, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let mut settings = cfg.load_app_settings();

    if let Some(profile) = cfg.load_servers_config().servers.first() {
        let actual_id = save_id
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| profile.id.clone());
        ConfigService::update_auto_update_config(&profile.server_root, &actual_id, enabled)?;
    }

    settings.auto_update_hosting = enabled;
    cfg.save_app_settings(&settings)?;

    Ok(settings)
}

#[tauri::command]
pub fn set_log_retention_days(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    days: u32,
) -> Result<AppSettings, String> {
    if !(1..=3650).contains(&days) {
        return Err("日志保存时间必须在 1 到 3650 天之间".to_string());
    }

    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let mut settings = cfg.load_app_settings();
    settings.log_retention_days = days;
    cfg.save_app_settings(&settings)?;

    Ok(settings)
}

#[tauri::command]
pub fn save_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    servers: ServersConfig,
) -> Result<String, String> {
    for server in &servers.servers {
        crate::services::config_service::validate_id(&server.id)
            .map_err(|_| "服务器 ID 包含非法字符".to_string())?;
        if contains_chinese(&server.steam_cmd_path) {
            return Err("SteamCMD 路径不能包含中文字符".to_string());
        }
        if contains_chinese(&server.server_root) {
            return Err("服务端目录路径不能包含中文字符".to_string());
        }
    }

    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.save_servers_config(&servers)?;

    if let Some(profile) = servers.servers.first() {
        let _ = ConfigService::update_rocket_config(
            &profile.server_root,
            &profile.id,
            profile.rcon.port,
            &profile.rcon.password,
        );
    }

    Ok("配置已保存".to_string())
}

#[tauri::command]
pub fn is_first_run(config: State<'_, Arc<Mutex<ConfigService>>>) -> bool {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.is_first_run()
}

#[tauri::command]
pub fn save_wizard_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    steam_cmd_path: String,
    server_root: String,
    server_id: String,
    rcon_port: u16,
    rcon_password: String,
) -> Result<String, String> {
    crate::services::config_service::validate_id(&server_id)
        .map_err(|_| "服务器 ID 包含非法字符".to_string())?;
    if rcon_port == 0 {
        return Err("RCON 端口无效".to_string());
    }
    if contains_chinese(&steam_cmd_path) {
        return Err("SteamCMD 路径不能包含中文字符，请放到纯英文路径下".to_string());
    }
    if contains_chinese(&server_root) {
        return Err("服务端目录路径不能包含中文字符，请放到纯英文路径下".to_string());
    }

    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());

    let profile = ServerProfile {
        id: server_id.clone(),
        name: format!("{}服务器", server_id),
        steam_cmd_path,
        server_root: server_root.clone(),
        server_entry: format!("+InternetServer/{}", server_id),
        rcon: RconConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: rcon_port,
            password: rcon_password.clone(),
        },
    };

    let servers_config = ServersConfig {
        servers: vec![profile],
    };

    cfg.save_servers_config(&servers_config)?;

    let _ =
        ConfigService::update_rocket_config(&server_root, &server_id, rcon_port, &rcon_password);

    Ok("配置已保存".to_string())
}
