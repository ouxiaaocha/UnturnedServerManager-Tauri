use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;

#[derive(Serialize)]
pub struct SaveInfo {
    pub id: String,
    pub name: Option<String>,
    pub has_commands_dat: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommandsDatInfo {
    pub name: Option<String>,
    pub map: Option<String>,
    pub port: Option<u16>,
    pub max_players: Option<u32>,
    pub password: Option<String>,
    pub owner: Option<String>,
    pub cheats: bool,
    pub pve: bool,
    pub perspective: Option<String>,
    pub gslt: Option<String>,
    pub raw_lines: Vec<String>,
}

#[derive(Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub file_name: String,
    pub path: String,
}

fn validate_save_id(id: &str) -> Result<(), String> {
    if id.contains('/') || id.contains('\\') || id.contains("..")
        || id.contains(':') || id.contains('*') || id.contains('?')
        || id.contains('"') || id.contains('<') || id.contains('>')
        || id.contains('|') || id.is_empty()
    {
        return Err("存档 ID 包含非法字符".to_string());
    }
    Ok(())
}

fn resolve_save_dir(config: &ConfigService, save_id: &Option<String>) -> Result<(String, String), String> {
    let servers_config = config.load_servers_config();
    let profile = servers_config.servers.first().ok_or("没有配置服务器")?;
    let server_id = save_id.clone().unwrap_or_else(|| profile.id.clone());
    validate_save_id(&server_id)?;
    Ok((profile.server_root.clone(), server_id))
}

fn parse_commands_dat(content: &str) -> CommandsDatInfo {
    let mut info = CommandsDatInfo {
        name: None,
        map: None,
        port: None,
        max_players: None,
        password: None,
        owner: None,
        cheats: false,
        pve: false,
        perspective: None,
        gslt: None,
        raw_lines: Vec::new(),
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            info.raw_lines.push(line.to_string());
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let cmd = parts[0];
        let val = parts.get(1).map(|s| s.trim().to_string());

        match cmd {
            "Name" => info.name = val,
            "Map" => info.map = val,
            "Port" => info.port = val.and_then(|v| v.parse().ok()),
            "MaxPlayers" => info.max_players = val.and_then(|v| v.parse().ok()),
            "Password" => info.password = val,
            "Owner" => info.owner = val,
            "Cheats" => info.cheats = val.as_deref() != Some("false"),
            "PvE" => info.pve = val.as_deref() != Some("false"),
            "Perspective" => info.perspective = val,
            "GSLT" => info.gslt = val,
            _ => {}
        }

        info.raw_lines.push(line.to_string());
    }

    info
}

fn render_value_command(cmd: &str, value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("{} {}", cmd, v))
}

fn render_flag_command(cmd: &str, enabled: bool) -> Option<String> {
    if enabled {
        Some(cmd.to_string())
    } else {
        None
    }
}

fn build_commands_dat_lines(existing_lines: &[String], info: &CommandsDatInfo) -> Vec<String> {
    let mut new_values: HashMap<&str, Option<String>> = HashMap::new();
    new_values.insert("Name", render_value_command("Name", info.name.as_deref()));
    new_values.insert("Map", render_value_command("Map", info.map.as_deref()));
    new_values.insert("Port", info.port.map(|v| format!("Port {}", v)));
    new_values.insert("MaxPlayers", info.max_players.map(|v| format!("MaxPlayers {}", v)));
    new_values.insert("Password", render_value_command("Password", info.password.as_deref()));
    new_values.insert("Owner", render_value_command("Owner", info.owner.as_deref()));
    new_values.insert("Cheats", render_flag_command("Cheats", info.cheats));
    new_values.insert("PvE", render_flag_command("PvE", info.pve));
    new_values.insert("Perspective", render_value_command("Perspective", info.perspective.as_deref()));
    new_values.insert("GSLT", render_value_command("GSLT", info.gslt.as_deref()));

    let managed_set: std::collections::HashSet<&str> = MANAGED_COMMANDS.iter().copied().collect();
    let mut written_set: std::collections::HashSet<&str> = std::collections::HashSet::new();

    // Rewrite existing lines: update managed commands, keep everything else
    let mut output_lines: Vec<String> = Vec::new();
    for line in existing_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            output_lines.push(line.clone());
            continue;
        }

        let cmd = trimmed.splitn(2, ' ').next().unwrap_or("");
        if managed_set.contains(cmd) {
            written_set.insert(cmd);
            if let Some(Some(new_val)) = new_values.get(cmd) {
                output_lines.push(new_val.clone());
            }
            // Command not in new_values = user removed it (e.g. disabled cheats)
        } else {
            output_lines.push(line.clone());
        }
    }

    // Append managed commands that weren't in the existing file
    for &cmd in MANAGED_COMMANDS {
        if !written_set.contains(cmd) {
            if let Some(Some(new_val)) = new_values.get(cmd) {
                output_lines.push(new_val.clone());
            }
        }
    }

    output_lines
}

fn commands_dat_primary_path(server_root: &str, server_id: &str) -> PathBuf {
    Path::new(server_root)
        .join("Servers")
        .join(server_id)
        .join("Server")
        .join("Commands.dat")
}

fn commands_dat_legacy_path(server_root: &str, server_id: &str) -> PathBuf {
    Path::new(server_root)
        .join("Servers")
        .join(server_id)
        .join("Commands.dat")
}

fn detect_commands_dat_path(server_root: &str, server_id: &str) -> PathBuf {
    let primary = commands_dat_primary_path(server_root, server_id);
    if primary.exists() {
        return primary;
    }

    let legacy = commands_dat_legacy_path(server_root, server_id);
    if legacy.exists() {
        return legacy;
    }

    primary
}

/// Known Commands.dat commands that we manage structurally.
const MANAGED_COMMANDS: &[&str] = &[
    "Name", "Map", "Port", "MaxPlayers", "Password", "Owner",
    "Cheats", "PvE", "Perspective", "GSLT",
];

#[tauri::command]
pub fn list_server_saves(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    server_root: Option<String>,
) -> Vec<SaveInfo> {
    let server_root = if let Some(sr) = server_root.filter(|s| !s.is_empty()) {
        sr
    } else {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers_config = cfg.load_servers_config();
        match servers_config.servers.first() {
            Some(p) => p.server_root.clone(),
            None => return vec![],
        }
    };

    let servers_dir = Path::new(&server_root).join("Servers");
    if !servers_dir.exists() {
        return vec![];
    }

    let mut saves = Vec::new();
    if let Ok(entries) = fs::read_dir(&servers_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.is_empty() {
                    continue;
                }
                let primary_path = entry.path().join("Server").join("Commands.dat");
                let legacy_path = entry.path().join("Commands.dat");
                let commands_dat_path = if primary_path.exists() {
                    primary_path
                } else {
                    legacy_path
                };
                let has_commands_dat = commands_dat_path.exists();
                let server_name = if has_commands_dat {
                    fs::read_to_string(&commands_dat_path)
                        .ok()
                        .and_then(|content| {
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if let Some(rest) = trimmed.strip_prefix("Name ") {
                                    let n = rest.trim().to_string();
                                    if !n.is_empty() {
                                        return Some(n);
                                    }
                                }
                            }
                            None
                        })
                } else {
                    None
                };
                saves.push(SaveInfo {
                    id: name,
                    name: server_name,
                    has_commands_dat,
                });
            }
        }
    }

    saves
}

#[tauri::command]
pub fn read_commands_dat(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<CommandsDatInfo, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    let path = detect_commands_dat_path(&server_root, &server_id);

    if !path.exists() {
        return Ok(CommandsDatInfo {
            name: None,
            map: None,
            port: None,
            max_players: None,
            password: None,
            owner: None,
            cheats: false,
            pve: false,
            perspective: None,
            gslt: None,
            raw_lines: Vec::new(),
        });
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取 Commands.dat 失败: {}", e))?;
    Ok(parse_commands_dat(&content))
}

#[tauri::command]
pub fn save_commands_dat(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    save_id: Option<String>,
    info: CommandsDatInfo,
) -> Result<String, String> {
    let (server_root, server_id, rcon_port, rcon_password) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let (sr, sid) = resolve_save_dir(&cfg, &save_id)?;
        let servers_config = cfg.load_servers_config();
        let (rp, rpw) = if let Some(profile) = servers_config.servers.first() {
            (profile.rcon.port, profile.rcon.password.clone())
        } else {
            (27115, String::new())
        };
        (sr, sid, rp, rpw)
    };

    let dir = Path::new(&server_root).join("Servers").join(&server_id).join("Server");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let path = dir.join("Commands.dat");

    // Read existing file to preserve comments, blank lines, and unrecognized commands
    let existing_lines: Vec<String> = if path.exists() {
        fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .map(|l| l.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let content = build_commands_dat_lines(&existing_lines, &info).join("\n");
    crate::services::config_service::atomic_write(&path, &content)
        .map_err(|e| {
            let ls = log.lock().unwrap_or_else(|e| e.into_inner());
            ls.log_app(&format!("[ERROR] 保存 Commands.dat 失败 ({}): {}", server_id, e));
            format!("保存 Commands.dat 失败: {}", e)
        })?;

    // Sync Rocket.config.xml with RCON settings
    let _ = ConfigService::update_rocket_config(&server_root, &server_id, rcon_port, &rcon_password);

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存存档配置: {}", server_id));

    Ok("Commands.dat 已保存".to_string())
}

#[tauri::command]
pub fn list_plugins(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Vec<PluginInfo> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        match resolve_save_dir(&cfg, &save_id) {
            Ok(v) => v,
            Err(_) => return vec![],
        }
    };

    let plugins_dir = Path::new(&server_root)
        .join("Servers")
        .join(&server_id)
        .join("Rocket")
        .join("Plugins");

    if !plugins_dir.exists() {
        return vec![];
    }

    let mut plugins = Vec::new();
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("dll".to_string()) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                plugins.push(PluginInfo {
                    name,
                    file_name,
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    plugins
}

#[tauri::command]
pub fn open_plugin_config_dir(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    let plugins_dir = Path::new(&server_root)
        .join("Servers")
        .join(&server_id)
        .join("Rocket")
        .join("Plugins");

    if !plugins_dir.exists() {
        return Err("插件目录不存在".to_string());
    }

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(plugins_dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        return Err("当前平台暂不支持打开目录".to_string());
    }

    Ok("已打开插件目录".to_string())
}

#[tauri::command]
pub fn load_plugin_notes(
    config: State<'_, Arc<Mutex<ConfigService>>>,
) -> HashMap<String, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.load_plugin_notes()
}

#[tauri::command]
pub fn save_plugin_notes(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    notes: HashMap<String, String>,
) -> Result<String, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.save_plugin_notes(&notes)?;
    Ok("插件备注已保存".to_string())
}

#[derive(Serialize)]
pub struct RocketRconInfo {
    pub port: u16,
    pub password: String,
}

#[tauri::command]
pub fn read_rocket_rcon_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<RocketRconInfo, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    let path = Path::new(&server_root)
        .join("Servers")
        .join(&server_id)
        .join("Rocket")
        .join("Rocket.config.xml");

    if !path.exists() {
        return Ok(RocketRconInfo { port: 27115, password: String::new() });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 Rocket.config.xml 失败: {}", e))?;

    let mut port: u16 = 27115;
    let mut password = String::new();

    if let Some(start) = content.find("Port=\"") {
        let after = start + 6;
        if let Some(end) = content[after..].find('"') {
            port = content[after..after + end].parse().unwrap_or(27115);
        }
    }

    if let Some(start) = content.find("Password=\"") {
        let after = start + 10;
        if let Some(end) = content[after..].find('"') {
            password = content[after..after + end]
                .replace("&amp;", "&")
                .replace("&quot;", "\"")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&apos;", "'");
        }
    }

    Ok(RocketRconInfo { port, password })
}

#[tauri::command]
pub fn save_rocket_rcon_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    save_id: Option<String>,
    port: u16,
    password: String,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    ConfigService::update_rocket_config(&server_root, &server_id, port, &password)
        .map_err(|e| {
            let ls = log.lock().unwrap_or_else(|e| e.into_inner());
            ls.log_app(&format!("[ERROR] 保存 RCON 配置失败 ({}): {}", server_id, e));
            e
        })?;

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存 RCON 配置: 存档 {}", server_id));

    Ok("RCON 配置已保存".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_commands_dat_lines_uses_bare_flags_and_omits_disabled_flags() {
        let existing_lines = vec![
            "// server config".to_string(),
            "Name Old Server".to_string(),
            "Cheats".to_string(),
            "PvE false".to_string(),
            "CustomLine keep-me".to_string(),
        ];

        let info = CommandsDatInfo {
            name: Some("New Server".to_string()),
            map: Some("PEI".to_string()),
            port: Some(27015),
            max_players: Some(24),
            password: Some("secret".to_string()),
            owner: Some("76561198000000000".to_string()),
            cheats: false,
            pve: true,
            perspective: Some("Both".to_string()),
            gslt: Some("token".to_string()),
            raw_lines: vec![],
        };

        let output = build_commands_dat_lines(&existing_lines, &info);

        assert!(output.iter().any(|line| line == "// server config"));
        assert!(output.iter().any(|line| line == "Name New Server"));
        assert!(output.iter().any(|line| line == "Map PEI"));
        assert!(output.iter().any(|line| line == "Port 27015"));
        assert!(output.iter().any(|line| line == "MaxPlayers 24"));
        assert!(output.iter().any(|line| line == "Password secret"));
        assert!(output.iter().any(|line| line == "Owner 76561198000000000"));
        assert!(output.iter().any(|line| line == "PvE"));
        assert!(output.iter().any(|line| line == "Perspective Both"));
        assert!(output.iter().any(|line| line == "GSLT token"));
        assert!(output.iter().any(|line| line == "CustomLine keep-me"));
        assert!(!output.iter().any(|line| line == "Cheats"));
        assert!(!output.iter().any(|line| line == "Cheats false"));
        assert!(!output.iter().any(|line| line == "PvE true"));
    }
}
