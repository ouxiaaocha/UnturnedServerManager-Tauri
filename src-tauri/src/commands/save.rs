use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::{TcpListener, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;
use crate::services::process::ProcessManager;

#[derive(Serialize)]
pub struct SaveInfo {
    pub id: String,
    pub name: Option<String>,
    pub has_commands_dat: bool,
}

#[derive(Serialize)]
pub struct SavePortInfo {
    pub save_id: String,
    pub name: Option<String>,
    pub game_port: u16,
    pub rcon_port: u16,
}

#[derive(Serialize)]
pub struct SavePortIssue {
    pub kind: String,
    pub port: u16,
    pub save_ids: Vec<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct SavePortCheckReport {
    pub ok: bool,
    pub saves: Vec<SavePortInfo>,
    pub issues: Vec<SavePortIssue>,
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

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GameConfigInfo {
    pub exists: bool,
    pub path: String,
    pub version: Option<String>,
    pub source_hash: String,
    pub line_ending: String,
    pub sections: Vec<GameConfigSection>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GameConfigSection {
    pub name: String,
    pub entries: Vec<GameConfigEntry>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GameConfigEntry {
    pub id: String,
    pub section: String,
    pub key: String,
    pub value: String,
    pub has_value: bool,
    pub value_kind: String,
    pub description: Vec<String>,
    pub default_hint: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct GameConfigChange {
    pub section: String,
    pub key: String,
    pub original_value: String,
    pub value: String,
}

#[derive(Debug, Clone)]
struct GameConfigEntryPosition {
    section: String,
    key: String,
    start_line: usize,
    end_line: usize,
    indent: String,
    is_block: bool,
}

#[derive(Debug, Clone)]
struct ParsedGameConfig {
    info: GameConfigInfo,
    lines: Vec<String>,
    final_newline: bool,
    positions: Vec<GameConfigEntryPosition>,
    section_bounds: HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GroupPermissionInfo {
    pub name: String,
    pub cooldown: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PermissionGroupInfo {
    pub id: String,
    pub display_name: String,
    pub prefix: String,
    pub suffix: String,
    pub color: String,
    pub members: Vec<String>,
    pub parent_group: Option<String>,
    pub priority: i32,
    pub permissions: Vec<GroupPermissionInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PermissionsConfigInfo {
    pub exists: bool,
    pub path: String,
    pub default_group: String,
    pub groups: Vec<PermissionGroupInfo>,
}

impl Default for PermissionGroupInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            display_name: String::new(),
            prefix: String::new(),
            suffix: String::new(),
            color: String::new(),
            members: Vec::new(),
            parent_group: None,
            priority: 100,
            permissions: Vec::new(),
        }
    }
}

fn validate_save_id(id: &str) -> Result<(), String> {
    crate::services::config_service::validate_id(id).map_err(|_| "存档 ID 包含非法字符".to_string())
}

fn resolve_delete_save_path(server_root: &str, save_id: &str) -> Result<PathBuf, String> {
    validate_save_id(save_id)?;

    let servers_dir = Path::new(server_root).join("Servers");
    let save_dir = servers_dir.join(save_id);

    if !save_dir.exists() {
        return Err("存档不存在".to_string());
    }

    if !save_dir.is_dir() {
        return Err("存档路径不是目录".to_string());
    }

    let servers_dir =
        fs::canonicalize(&servers_dir).map_err(|e| format!("解析 Servers 目录失败: {}", e))?;
    let save_dir = fs::canonicalize(&save_dir).map_err(|e| format!("解析存档目录失败: {}", e))?;

    if !save_dir.starts_with(&servers_dir) || save_dir == servers_dir {
        return Err("存档路径不安全".to_string());
    }

    Ok(save_dir)
}

fn resolve_save_dir(
    config: &ConfigService,
    save_id: &Option<String>,
) -> Result<(String, String), String> {
    let servers_config = config.load_servers_config();
    let profile = servers_config.servers.first().ok_or("没有配置服务器")?;
    let server_id = save_id.clone().unwrap_or_else(|| profile.id.clone());
    validate_save_id(&server_id)?;
    Ok((profile.server_root.clone(), server_id))
}

fn resolve_server_root_and_fallback_rcon(config: &ConfigService) -> Result<(String, u16), String> {
    let servers = config.load_servers_config();
    let profile = servers.servers.first().ok_or("没有配置服务器")?;
    Ok((profile.server_root.clone(), profile.rcon.port))
}

fn ensure_save_not_running(
    process: &Arc<Mutex<ProcessManager>>,
    save_id: &str,
) -> Result<(), String> {
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    if pm.is_running_for(save_id) {
        return Err(format!(
            "存档 {} 的服务器正在运行，请先停止后再修改",
            save_id
        ));
    }
    Ok(())
}

fn rocket_config_path(server_root: &str, save_id: &str) -> PathBuf {
    Path::new(server_root)
        .join("Servers")
        .join(save_id)
        .join("Rocket")
        .join("Rocket.config.xml")
}

fn read_rocket_rcon_settings_actual(path: &Path) -> Option<(u16, String)> {
    let content = fs::read_to_string(path).ok()?;
    let mut port = None;
    let mut password = String::new();

    if let Some(start) = content.find("Port=\"") {
        let after = start + 6;
        if let Some(end) = content[after..].find('"') {
            port = content[after..after + end].parse::<u16>().ok();
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

    Some((port?, password))
}

fn read_save_ports_for_report(
    server_root: &str,
    save: &SaveInfo,
    fallback_rcon_port: u16,
) -> SavePortInfo {
    let commands_path = detect_commands_dat_path(server_root, &save.id);
    let game_port = fs::read_to_string(&commands_path)
        .ok()
        .and_then(|content| parse_commands_dat(&content).port)
        .unwrap_or(27015);
    let rcon_port = read_rocket_rcon_settings_actual(&rocket_config_path(server_root, &save.id))
        .map(|(port, _)| port)
        .unwrap_or(fallback_rcon_port);

    SavePortInfo {
        save_id: save.id.clone(),
        name: save.name.clone(),
        game_port,
        rcon_port,
    }
}

fn is_udp_port_available(port: u16) -> bool {
    UdpSocket::bind(("0.0.0.0", port)).is_ok()
}

fn is_tcp_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn build_port_report(
    config: Arc<Mutex<ConfigService>>,
    process: Option<Arc<Mutex<ProcessManager>>>,
) -> Result<SavePortCheckReport, String> {
    let (server_root, fallback_rcon_port) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_server_root_and_fallback_rcon(&cfg)?
    };

    let saves = list_server_saves_blocking(config, Some(server_root.clone()));
    let port_infos: Vec<SavePortInfo> = saves
        .iter()
        .map(|save| read_save_ports_for_report(&server_root, save, fallback_rcon_port))
        .collect();
    let mut issues = Vec::new();
    let running_save_ids: HashSet<String> = process
        .as_ref()
        .map(|process| {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            pm.running_save_ids().into_iter().collect()
        })
        .unwrap_or_default();

    let mut game_ports: HashMap<u16, Vec<String>> = HashMap::new();
    let mut rcon_ports: HashMap<u16, Vec<String>> = HashMap::new();
    for info in &port_infos {
        game_ports
            .entry(info.game_port)
            .or_default()
            .push(info.save_id.clone());
        rcon_ports
            .entry(info.rcon_port)
            .or_default()
            .push(info.save_id.clone());
    }

    for (port, save_ids) in game_ports {
        if save_ids.len() > 1 {
            issues.push(SavePortIssue {
                kind: "duplicate_game".to_string(),
                port,
                message: format!("游戏端口 {} 被多个存档使用: {}", port, save_ids.join(", ")),
                save_ids,
            });
        }
    }

    for (port, save_ids) in rcon_ports {
        if save_ids.len() > 1 {
            issues.push(SavePortIssue {
                kind: "duplicate_rcon".to_string(),
                port,
                message: format!("RCON 端口 {} 被多个存档使用: {}", port, save_ids.join(", ")),
                save_ids,
            });
        }
    }

    for info in &port_infos {
        if running_save_ids.contains(&info.save_id) {
            continue;
        }
        if !is_udp_port_available(info.game_port) {
            issues.push(SavePortIssue {
                kind: "occupied_game".to_string(),
                port: info.game_port,
                save_ids: vec![info.save_id.clone()],
                message: format!("存档 {} 的游戏端口 {} 当前被系统占用", info.save_id, info.game_port),
            });
        }
        if !is_tcp_port_available(info.rcon_port) {
            issues.push(SavePortIssue {
                kind: "occupied_rcon".to_string(),
                port: info.rcon_port,
                save_ids: vec![info.save_id.clone()],
                message: format!("存档 {} 的 RCON 端口 {} 当前被系统占用", info.save_id, info.rcon_port),
            });
        }
    }

    Ok(SavePortCheckReport {
        ok: issues.is_empty(),
        saves: port_infos,
        issues,
    })
}

fn next_available_port(
    start: u16,
    used: &mut HashSet<u16>,
    is_available: fn(u16) -> bool,
) -> Result<u16, String> {
    let mut port = start.max(1024);
    while port < u16::MAX {
        if !used.contains(&port) && is_available(port) {
            used.insert(port);
            return Ok(port);
        }
        port = port.saturating_add(1);
    }
    Err("没有找到可用端口".to_string())
}

fn write_commands_dat_port(server_root: &str, save_id: &str, port: u16) -> Result<(), String> {
    let path = detect_commands_dat_path(server_root, save_id);
    let existing_content = fs::read_to_string(&path).unwrap_or_default();
    let existing_lines: Vec<String> = existing_content.lines().map(|line| line.to_string()).collect();
    let mut info = parse_commands_dat(&existing_content);
    info.port = Some(port);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 Commands.dat 目录失败: {}", e))?;
    }
    let content = build_commands_dat_lines(&existing_lines, &info).join("\n");
    crate::services::config_service::atomic_write(&path, &content)
        .map_err(|e| format!("写入 Commands.dat 失败: {}", e))
}

fn auto_assign_save_ports_blocking(
    config: Arc<Mutex<ConfigService>>,
    log: Arc<Mutex<LogService>>,
    process: Arc<Mutex<ProcessManager>>,
) -> Result<SavePortCheckReport, String> {
    let (server_root, fallback_rcon_port) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_server_root_and_fallback_rcon(&cfg)?
    };
    let saves = list_server_saves_blocking(config.clone(), Some(server_root.clone()));

    let mut used_game = HashSet::new();
    let mut used_rcon = HashSet::new();
    let mut next_game = 27015;
    let mut next_rcon = fallback_rcon_port.max(27115);

    for save in &saves {
        ensure_save_not_running(&process, &save.id)?;
        let current = read_save_ports_for_report(&server_root, save, fallback_rcon_port);

        let game_port = if used_game.contains(&current.game_port) || !is_udp_port_available(current.game_port) {
            let assigned = next_available_port(next_game, &mut used_game, is_udp_port_available)?;
            next_game = assigned.saturating_add(1);
            assigned
        } else {
            used_game.insert(current.game_port);
            current.game_port
        };

        let rcon_port = if used_rcon.contains(&current.rcon_port) || !is_tcp_port_available(current.rcon_port) {
            let assigned = next_available_port(next_rcon, &mut used_rcon, is_tcp_port_available)?;
            next_rcon = assigned.saturating_add(1);
            assigned
        } else {
            used_rcon.insert(current.rcon_port);
            current.rcon_port
        };

        if game_port != current.game_port {
            write_commands_dat_port(&server_root, &save.id, game_port)?;
        }

        if rcon_port != current.rcon_port {
            let rocket_path = rocket_config_path(&server_root, &save.id);
            if let Some((_, password)) = read_rocket_rcon_settings_actual(&rocket_path) {
                let _ = ConfigService::update_rocket_config(&server_root, &save.id, rcon_port, &password);
            }
        }
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("自动分配存档端口");
    }

    build_port_report(config, Some(process))
}

pub(crate) fn parse_commands_dat(content: &str) -> CommandsDatInfo {
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
    new_values.insert(
        "MaxPlayers",
        info.max_players.map(|v| format!("MaxPlayers {}", v)),
    );
    new_values.insert(
        "Password",
        render_value_command("Password", info.password.as_deref()),
    );
    new_values.insert(
        "Owner",
        render_value_command("Owner", info.owner.as_deref()),
    );
    new_values.insert("Cheats", render_flag_command("Cheats", info.cheats));
    new_values.insert("PvE", render_flag_command("PvE", info.pve));
    new_values.insert(
        "Perspective",
        render_value_command("Perspective", info.perspective.as_deref()),
    );
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

        let cmd = trimmed.split(' ').next().unwrap_or("");
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

pub(crate) fn detect_commands_dat_path(server_root: &str, server_id: &str) -> PathBuf {
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
    "Name",
    "Map",
    "Port",
    "MaxPlayers",
    "Password",
    "Owner",
    "Cheats",
    "PvE",
    "Perspective",
    "GSLT",
];

fn game_config_path(server_root: &str, server_id: &str) -> PathBuf {
    Path::new(server_root)
        .join("Servers")
        .join(server_id)
        .join("Config.txt")
}

fn detect_line_ending(content: &str) -> String {
    if content.contains("\r\n") {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    }
}

fn sha256_hex(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn split_config_lines(content: &str) -> (Vec<String>, bool) {
    let final_newline = content.ends_with('\n');
    let normalized = content.replace("\r\n", "\n");
    let mut lines: Vec<String> = normalized.lines().map(str::to_string).collect();
    if normalized.is_empty() {
        lines.clear();
    }
    (lines, final_newline)
}

fn is_identifier_line(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn line_indent(line: &str) -> String {
    line.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

fn strip_comment_prefix(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("//")?.trim_start();
    Some(
        rest.strip_prefix('>')
            .unwrap_or(rest)
            .trim_start()
            .to_string(),
    )
}

fn comment_text(comments: &[String]) -> String {
    comments
        .iter()
        .filter_map(|line| strip_comment_prefix(line))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_options(comments: &[String]) -> Vec<String> {
    for line in comments {
        let Some(text) = strip_comment_prefix(line) else {
            continue;
        };
        let Some((_, rest)) = text.split_once("Options:") else {
            continue;
        };
        return rest
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
    }
    Vec::new()
}

fn parse_default_hint(comments: &[String]) -> Option<String> {
    let hints: Vec<String> = comments
        .iter()
        .filter_map(|line| strip_comment_prefix(line))
        .filter(|line| {
            line.contains("Default:")
                || line.contains("Easy:")
                || line.contains("Normal:")
                || line.contains("Hard:")
                || line.contains("[0 to 1]")
        })
        .collect();
    if hints.is_empty() {
        None
    } else {
        Some(hints.join(" "))
    }
}

fn infer_game_config_value_kind(
    key: &str,
    value: &str,
    comments: &[String],
    is_block: bool,
) -> String {
    if is_block {
        return "raw_block".to_string();
    }

    let options = parse_options(comments);
    if !options.is_empty() {
        return "select".to_string();
    }

    let text = comment_text(comments);
    let lower_value = value.trim().to_ascii_lowercase();
    if key == "Links"
        || text.contains("Buttons shown")
        || text.contains("Format is a list")
        || text.contains("Refer to Scheduled_Shutdown_Warnings")
    {
        return "raw_block".to_string();
    }
    if text.contains("[0 to 1]") || text.contains("Percentage") {
        return "percent".to_string();
    }
    if key.contains("URL") || key.contains("Icon") || key.contains("Thumbnail") {
        return "url".to_string();
    }
    if lower_value == "true"
        || lower_value == "false"
        || text.contains("If true")
        || text.contains("If false")
        || text.contains("Whether")
        || text.contains("Should")
        || text.contains("Default: True")
        || text.contains("Default: False")
        || text.contains("Easy: True")
        || text.contains("Easy: False")
        || text.contains("Normal: True")
        || text.contains("Normal: False")
        || text.contains("Hard: True")
        || text.contains("Hard: False")
    {
        return "bool".to_string();
    }
    if value.trim().parse::<f64>().is_ok()
        || text.contains("seconds")
        || text.contains("milliseconds")
        || text.contains("minutes")
        || text.contains("hours")
        || text.contains("Default: 0")
        || text.contains("Default: 1")
        || text.contains("Default: 2")
        || text.contains("Default: 3")
        || text.contains("Default: 4")
        || text.contains("Default: 5")
        || text.contains("Default: 6")
        || text.contains("Default: 7")
        || text.contains("Default: 8")
        || text.contains("Default: 9")
    {
        return "number".to_string();
    }
    if value.len() > 80 || value.contains("\\n") || key.starts_with("Desc_") {
        return "long_text".to_string();
    }
    "text".to_string()
}

fn block_end_line(lines: &[String], start_line: usize) -> Option<usize> {
    let opener = lines.get(start_line)?.trim();
    let (open, close) = match opener.chars().next()? {
        '[' => ('[', ']'),
        '{' => ('{', '}'),
        _ => return None,
    };
    let mut depth = 0isize;
    for (index, line) in lines.iter().enumerate().skip(start_line) {
        for ch in line.chars() {
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
                if depth <= 0 {
                    return Some(index);
                }
            }
        }
    }
    Some(lines.len().saturating_sub(1))
}

fn make_game_config_entry(
    section: &str,
    key: &str,
    value: String,
    is_block: bool,
    comments: &[String],
) -> GameConfigEntry {
    let description = comments
        .iter()
        .filter_map(|line| strip_comment_prefix(line))
        .collect::<Vec<_>>();
    let value_kind = infer_game_config_value_kind(key, &value, comments, is_block);
    GameConfigEntry {
        id: format!("{section}.{key}"),
        section: section.to_string(),
        key: key.to_string(),
        has_value: !value.trim().is_empty(),
        value,
        value_kind,
        description,
        default_hint: parse_default_hint(comments),
        options: parse_options(comments),
    }
}

fn parse_game_config(content: &str, path: String) -> ParsedGameConfig {
    let (lines, final_newline) = split_config_lines(content);
    let line_ending = detect_line_ending(content);
    let mut version = None;
    let mut sections = Vec::<GameConfigSection>::new();
    let mut positions = Vec::<GameConfigEntryPosition>::new();
    let mut section_bounds = HashMap::<String, usize>::new();
    let mut section_index = HashMap::<String, usize>::new();
    let mut current_section: Option<String> = None;
    let mut pending_section: Option<String> = None;
    let mut comments = Vec::<String>::new();
    let mut index = 0usize;

    while index < lines.len() {
        let line = &lines[index];
        let trimmed = line.trim();

        if let Some(section) = pending_section.take() {
            if trimmed == "{" {
                current_section = Some(section.clone());
                section_index.insert(section.clone(), sections.len());
                sections.push(GameConfigSection {
                    name: section,
                    entries: Vec::new(),
                });
                comments.clear();
                index += 1;
                continue;
            }
        }

        if trimmed.starts_with("//") {
            comments.push(line.clone());
            index += 1;
            continue;
        }

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if current_section.is_none() {
            if let Some(rest) = trimmed.strip_prefix("Version ") {
                version = Some(rest.trim().to_string());
            } else if is_identifier_line(trimmed) {
                pending_section = Some(trimmed.to_string());
            }
            comments.clear();
            index += 1;
            continue;
        }

        if trimmed == "}" {
            if let Some(section) = current_section.take() {
                section_bounds.insert(section, index);
            }
            comments.clear();
            index += 1;
            continue;
        }

        let section = current_section.clone().unwrap_or_default();
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let key = parts.next().unwrap_or_default();
        let value = parts.next().map(str::trim_start).unwrap_or_default();
        let mut is_block = false;
        let mut end_line = index;
        let mut entry_value = value.to_string();

        if entry_value.is_empty() {
            let mut next = index + 1;
            while next < lines.len() && lines[next].trim().is_empty() {
                next += 1;
            }
            if next < lines.len() && matches!(lines[next].trim().chars().next(), Some('[' | '{')) {
                is_block = true;
                end_line = block_end_line(&lines, next).unwrap_or(next);
                entry_value = lines[next..=end_line].join("\n");
            }
        }

        let entry = make_game_config_entry(&section, key, entry_value, is_block, &comments);
        if let Some(section_pos) = section_index.get(&section).copied() {
            sections[section_pos].entries.push(entry);
        }
        positions.push(GameConfigEntryPosition {
            section,
            key: key.to_string(),
            start_line: index,
            end_line,
            indent: line_indent(line),
            is_block,
        });

        comments.clear();
        index = end_line + 1;
    }

    let info = GameConfigInfo {
        exists: true,
        path,
        version,
        source_hash: sha256_hex(content),
        line_ending,
        sections,
    };

    ParsedGameConfig {
        info,
        lines,
        final_newline,
        positions,
        section_bounds,
    }
}

fn find_game_config_entry<'a>(
    parsed: &'a ParsedGameConfig,
    section: &str,
    key: &str,
) -> Option<&'a GameConfigEntry> {
    parsed
        .info
        .sections
        .iter()
        .find(|item| item.name == section)?
        .entries
        .iter()
        .find(|entry| entry.key == key)
}

fn render_game_config_change(indent: &str, key: &str, value: &str, is_block: bool) -> Vec<String> {
    if value.trim().is_empty() {
        return vec![format!("{indent}{key}")];
    }
    if is_block || value.contains('\n') {
        let mut lines = vec![format!("{indent}{key}")];
        lines.extend(value.lines().map(str::to_string));
        lines
    } else {
        vec![format!("{indent}{key} {}", value.trim())]
    }
}

fn render_config_lines(lines: &[String], line_ending: &str, final_newline: bool) -> String {
    let mut output = lines.join(line_ending);
    if final_newline && !output.ends_with(line_ending) {
        output.push_str(line_ending);
    }
    output
}

fn apply_game_config_changes(
    current_content: &str,
    loaded_source_hash: &str,
    changes: &[GameConfigChange],
) -> Result<String, String> {
    let mut parsed = parse_game_config(current_content, String::new());
    let current_hash = parsed.info.source_hash.clone();
    let mut ordered_changes = changes.to_vec();
    ordered_changes.sort_by(|a, b| b.section.cmp(&a.section).then_with(|| b.key.cmp(&a.key)));

    for change in &ordered_changes {
        if change.section.trim().is_empty() || change.key.trim().is_empty() {
            return Err("配置项分组和名称不能为空".to_string());
        }

        if let Some(current_entry) =
            find_game_config_entry(&parsed, &change.section, &change.key).cloned()
        {
            if current_hash != loaded_source_hash && current_entry.value != change.original_value {
                return Err(format!(
                    "配置项 {}.{} 已被外部修改，请重新加载后再保存",
                    change.section, change.key
                ));
            }

            let position = parsed
                .positions
                .iter()
                .find(|item| item.section == change.section && item.key == change.key)
                .cloned()
                .ok_or_else(|| format!("找不到配置项位置: {}.{}", change.section, change.key))?;
            let rendered = render_game_config_change(
                &position.indent,
                &change.key,
                &change.value,
                position.is_block,
            );
            parsed
                .lines
                .splice(position.start_line..=position.end_line, rendered);
            parsed = parse_game_config(
                &render_config_lines(
                    &parsed.lines,
                    &parsed.info.line_ending,
                    parsed.final_newline,
                ),
                String::new(),
            );
        } else {
            let Some(section_end) = parsed.section_bounds.get(&change.section).copied() else {
                return Err(format!(
                    "配置分组 {} 不存在，无法安全追加新配置项",
                    change.section
                ));
            };
            let rendered = render_game_config_change("\t", &change.key, &change.value, false);
            parsed.lines.splice(section_end..section_end, rendered);
            parsed = parse_game_config(
                &render_config_lines(
                    &parsed.lines,
                    &parsed.info.line_ending,
                    parsed.final_newline,
                ),
                String::new(),
            );
        }
    }

    Ok(render_config_lines(
        &parsed.lines,
        &parsed.info.line_ending,
        parsed.final_newline,
    ))
}

fn permissions_config_path(server_root: &str, server_id: &str) -> PathBuf {
    Path::new(server_root)
        .join("Servers")
        .join(server_id)
        .join("Rocket")
        .join("Permissions.config.xml")
}

fn decode_xml_value(value: &[u8]) -> String {
    String::from_utf8_lossy(value)
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
}

fn event_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name).to_string()
}

fn permission_cooldown(event: &quick_xml::events::BytesStart<'_>) -> Result<u32, String> {
    for attr in event.attributes().with_checks(false) {
        let attr = attr.map_err(|e| format!("解析 Permission 属性失败: {}", e))?;
        if attr.key.as_ref() == b"Cooldown" {
            let value = decode_xml_value(attr.value.as_ref());
            return value
                .trim()
                .parse::<u32>()
                .map_err(|_| format!("权限冷却时间无效: {}", value));
        }
    }
    Ok(0)
}

pub(crate) fn parse_permissions_config(
    content: &str,
    path: String,
) -> Result<PermissionsConfigInfo, String> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut default_group = String::new();
    let mut groups = Vec::new();
    let mut current_group: Option<PermissionGroupInfo> = None;
    let mut current_permission: Option<GroupPermissionInfo> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let name = event_name(event.name().as_ref());
                if name == "Group" {
                    current_group = Some(PermissionGroupInfo::default());
                } else if name == "Permission" && current_group.is_some() {
                    current_permission = Some(GroupPermissionInfo {
                        name: String::new(),
                        cooldown: permission_cooldown(&event)?,
                    });
                }
                stack.push(name);
            }
            Ok(Event::Empty(event)) => {
                let name = event_name(event.name().as_ref());
                if name == "Group" {
                    groups.push(PermissionGroupInfo::default());
                } else if name == "Permission" {
                    if let Some(group) = current_group.as_mut() {
                        group.permissions.push(GroupPermissionInfo {
                            name: String::new(),
                            cooldown: permission_cooldown(&event)?,
                        });
                    }
                }
            }
            Ok(Event::Text(event)) => {
                let value = decode_xml_value(event.as_ref());
                let current = stack.last().map(String::as_str).unwrap_or_default();
                if let Some(group) = current_group.as_mut() {
                    match current {
                        "Id" => group.id = value,
                        "DisplayName" => group.display_name = value,
                        "Prefix" => group.prefix = value,
                        "Suffix" => group.suffix = value,
                        "Color" => group.color = value,
                        "Member" => group.members.push(value),
                        "ParentGroup" => group.parent_group = Some(value),
                        "Priority" => {
                            group.priority = value.trim().parse().unwrap_or(100);
                        }
                        "Permission" => {
                            if let Some(permission) = current_permission.as_mut() {
                                permission.name = value;
                            }
                        }
                        _ => {}
                    }
                } else if current == "DefaultGroup" {
                    default_group = value;
                }
            }
            Ok(Event::CData(event)) => {
                let value = String::from_utf8_lossy(event.as_ref()).to_string();
                if current_group.is_some() {
                    if stack.last().map(String::as_str) == Some("Permission") {
                        if let Some(permission) = current_permission.as_mut() {
                            permission.name = value;
                        }
                    }
                }
            }
            Ok(Event::End(event)) => {
                let name = event_name(event.name().as_ref());
                if name == "Permission" {
                    if let (Some(group), Some(permission)) =
                        (current_group.as_mut(), current_permission.take())
                    {
                        group.permissions.push(permission);
                    }
                } else if name == "Group" {
                    if let Some(group) = current_group.take() {
                        groups.push(group);
                    }
                }
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("解析 Permissions.config.xml 失败: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(PermissionsConfigInfo {
        exists: true,
        path,
        default_group,
        groups,
    })
}

fn normalize_permissions_config(
    mut config: PermissionsConfigInfo,
) -> Result<PermissionsConfigInfo, String> {
    config.exists = true;
    config.default_group = config.default_group.trim().to_string();

    for group in &mut config.groups {
        group.id = group.id.trim().to_string();
        group.display_name = group.display_name.trim().to_string();
        group.color = group.color.trim().to_string();
        group.members = normalize_unique_list(&group.members);
        group.parent_group = group
            .parent_group
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        for permission in &mut group.permissions {
            permission.name = permission.name.trim().to_string();
        }
    }

    validate_permissions_config(&config)?;
    Ok(config)
}

fn normalize_unique_list(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
            output.push(trimmed.to_string());
        }
    }
    output
}

pub(crate) fn validate_permissions_config(config: &PermissionsConfigInfo) -> Result<(), String> {
    if config.groups.is_empty() {
        return Err("至少需要一个权限组".to_string());
    }
    if config.default_group.trim().is_empty() {
        return Err("默认权限组不能为空".to_string());
    }

    let mut ids = HashSet::new();
    for group in &config.groups {
        if group.id.trim().is_empty() {
            return Err("权限组 ID 不能为空".to_string());
        }
        if !ids.insert(group.id.trim().to_string()) {
            return Err(format!("权限组 ID 重复: {}", group.id));
        }
        for permission in &group.permissions {
            if permission.name.trim().is_empty() {
                return Err(format!("权限组 {} 包含空权限名", group.id));
            }
        }
    }

    if !ids.contains(config.default_group.trim()) {
        return Err(format!("默认权限组不存在: {}", config.default_group));
    }

    for group in &config.groups {
        if let Some(parent_group) = group
            .parent_group
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            if parent_group == group.id {
                return Err(format!("权限组 {} 不能继承自身", group.id));
            }
            if !ids.contains(parent_group) {
                return Err(format!(
                    "权限组 {} 的父组不存在: {}",
                    group.id, parent_group
                ));
            }
        }
    }

    Ok(())
}

fn push_text_element(output: &mut String, indent: &str, name: &str, value: &str) {
    if value.is_empty() {
        output.push_str(&format!("{indent}<{name} />\n"));
    } else {
        output.push_str(&format!("{indent}<{name}>{}</{name}>\n", xml_escape(value)));
    }
}

pub(crate) fn render_permissions_config(config: &PermissionsConfigInfo) -> String {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    output.push_str("<RocketPermissions xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n");
    push_text_element(&mut output, "  ", "DefaultGroup", &config.default_group);
    output.push_str("  <Groups>\n");

    for group in &config.groups {
        output.push_str("    <Group>\n");
        push_text_element(&mut output, "      ", "Id", &group.id);
        push_text_element(&mut output, "      ", "DisplayName", &group.display_name);
        push_text_element(&mut output, "      ", "Prefix", &group.prefix);
        push_text_element(&mut output, "      ", "Suffix", &group.suffix);
        push_text_element(&mut output, "      ", "Color", &group.color);

        if group.members.is_empty() {
            output.push_str("      <Members />\n");
        } else {
            output.push_str("      <Members>\n");
            for member in &group.members {
                push_text_element(&mut output, "        ", "Member", member);
            }
            output.push_str("      </Members>\n");
        }

        if let Some(parent_group) = group
            .parent_group
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            push_text_element(&mut output, "      ", "ParentGroup", parent_group);
        }

        push_text_element(
            &mut output,
            "      ",
            "Priority",
            &group.priority.to_string(),
        );

        if group.permissions.is_empty() {
            output.push_str("      <Permissions />\n");
        } else {
            output.push_str("      <Permissions>\n");
            for permission in &group.permissions {
                output.push_str(&format!(
                    "        <Permission Cooldown=\"{}\">{}</Permission>\n",
                    permission.cooldown,
                    xml_escape(&permission.name)
                ));
            }
            output.push_str("      </Permissions>\n");
        }

        output.push_str("    </Group>\n");
    }

    output.push_str("  </Groups>\n");
    output.push_str("</RocketPermissions>\n");
    output
}

fn save_permissions_config_to_path(
    path: &Path,
    config: PermissionsConfigInfo,
) -> Result<PermissionsConfigInfo, String> {
    if !path.exists() {
        return Err("Permissions.config.xml 不存在".to_string());
    }

    let normalized = normalize_permissions_config(config)?;
    let content = render_permissions_config(&normalized);
    crate::services::config_service::atomic_write(path, &content)?;
    Ok(normalized)
}

fn list_server_saves_blocking(
    config: Arc<Mutex<ConfigService>>,
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

#[tauri::command(async)]
pub async fn list_server_saves(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    server_root: Option<String>,
) -> Result<Vec<SaveInfo>, String> {
    let config = config.inner().clone();
    let saves = tauri::async_runtime::spawn_blocking(move || {
        list_server_saves_blocking(config, server_root)
    })
    .await
    .map_err(|e| format!("读取存档列表任务失败: {}", e))?;
    Ok(saves)
}

#[tauri::command(async)]
pub async fn check_save_port_conflicts(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<SavePortCheckReport, String> {
    let config = config.inner().clone();
    let process = process.inner().clone();
    tauri::async_runtime::spawn_blocking(move || build_port_report(config, Some(process)))
        .await
        .map_err(|e| format!("端口检测任务失败: {}", e))?
}

#[tauri::command(async)]
pub async fn auto_assign_save_ports(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<SavePortCheckReport, String> {
    let config = config.inner().clone();
    let log = log.inner().clone();
    let process = process.inner().clone();
    tauri::async_runtime::spawn_blocking(move || auto_assign_save_ports_blocking(config, log, process))
        .await
        .map_err(|e| format!("自动分配端口任务失败: {}", e))?
}

#[tauri::command(async)]
pub async fn delete_server_save(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    save_id: String,
) -> Result<(), String> {
    let save_id = save_id.trim().to_string();
    if save_id.is_empty() {
        return Err("请选择要删除的存档".to_string());
    }

    {
        ensure_save_not_running(process.inner(), &save_id)?;
    }

    let config = config.inner().clone();
    let log = log.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        let server_root = {
            let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
            let servers_config = cfg.load_servers_config();
            let profile = servers_config
                .servers
                .first()
                .ok_or_else(|| "没有配置服务器".to_string())?;
            profile.server_root.clone()
        };

        let save_dir = resolve_delete_save_path(&server_root, &save_id)?;
        fs::remove_dir_all(&save_dir).map_err(|e| format!("删除存档失败: {}", e))?;

        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(&format!("删除存档: {}", save_id));

        Ok(())
    })
    .await
    .map_err(|e| format!("删除存档任务失败: {}", e))?
}

fn read_commands_dat_blocking(
    config: Arc<Mutex<ConfigService>>,
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

    let content =
        fs::read_to_string(&path).map_err(|e| format!("读取 Commands.dat 失败: {}", e))?;
    Ok(parse_commands_dat(&content))
}

#[tauri::command(async)]
pub async fn read_commands_dat(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<CommandsDatInfo, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || read_commands_dat_blocking(config, save_id))
        .await
        .map_err(|e| format!("读取 Commands.dat 任务失败: {}", e))?
}

#[tauri::command]
pub fn save_commands_dat(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
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
    ensure_save_not_running(process.inner(), &server_id)?;

    let dir = Path::new(&server_root)
        .join("Servers")
        .join(&server_id)
        .join("Server");
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
    crate::services::config_service::atomic_write(&path, &content).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[ERROR] 保存 Commands.dat 失败 ({}): {}",
            server_id, e
        ));
        format!("保存 Commands.dat 失败: {}", e)
    })?;

    let _ =
        ConfigService::update_rocket_config(&server_root, &server_id, rcon_port, &rcon_password);

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存存档配置: {}", server_id));

    Ok("Commands.dat 已保存".to_string())
}

fn read_game_config_blocking(
    config: Arc<Mutex<ConfigService>>,
    save_id: Option<String>,
) -> Result<GameConfigInfo, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    let path = game_config_path(&server_root, &server_id);
    let path_text = path.to_string_lossy().to_string();
    if !path.exists() {
        return Ok(GameConfigInfo {
            exists: false,
            path: path_text,
            version: None,
            source_hash: String::new(),
            line_ending: "\n".to_string(),
            sections: Vec::new(),
        });
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取 Config.txt 失败: {}", e))?;
    Ok(parse_game_config(&content, path_text).info)
}

#[tauri::command(async)]
pub async fn read_game_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<GameConfigInfo, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || read_game_config_blocking(config, save_id))
        .await
        .map_err(|e| format!("读取 Config.txt 任务失败: {}", e))?
}

#[tauri::command]
pub fn save_game_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    save_id: Option<String>,
    source_hash: String,
    changes: Vec<GameConfigChange>,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };
    ensure_save_not_running(process.inner(), &server_id)?;

    if changes.is_empty() {
        return Ok("Config.txt 无变更".to_string());
    }

    let path = game_config_path(&server_root, &server_id);
    if !path.exists() {
        return Err("Config.txt 不存在，请先运行一次服务端生成配置文件".to_string());
    }

    let current_content =
        fs::read_to_string(&path).map_err(|e| format!("读取 Config.txt 失败: {}", e))?;
    let next_content = apply_game_config_changes(&current_content, &source_hash, &changes)?;
    crate::services::config_service::atomic_write(&path, &next_content).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[ERROR] 保存 Config.txt 失败 ({}): {}",
            server_id, e
        ));
        format!("保存 Config.txt 失败: {}", e)
    })?;

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存高级配置: {}", server_id));

    Ok("Config.txt 已保存".to_string())
}

fn read_permissions_config_blocking(
    config: Arc<Mutex<ConfigService>>,
    save_id: Option<String>,
) -> Result<PermissionsConfigInfo, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    let path = permissions_config_path(&server_root, &server_id);
    let path_text = path.to_string_lossy().to_string();
    if !path.exists() {
        return Ok(PermissionsConfigInfo {
            exists: false,
            path: path_text,
            default_group: String::new(),
            groups: Vec::new(),
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 Permissions.config.xml 失败: {}", e))?;
    parse_permissions_config(&content, path_text)
}

#[tauri::command(async)]
pub async fn read_permissions_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<PermissionsConfigInfo, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || read_permissions_config_blocking(config, save_id))
        .await
        .map_err(|e| format!("读取 Permissions.config.xml 任务失败: {}", e))?
}

#[tauri::command]
pub fn save_permissions_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    save_id: Option<String>,
    permissions_config: PermissionsConfigInfo,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };
    ensure_save_not_running(process.inner(), &server_id)?;

    let path = permissions_config_path(&server_root, &server_id);
    save_permissions_config_to_path(&path, permissions_config).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[ERROR] 保存 Permissions.config.xml 失败 ({}): {}",
            server_id, e
        ));
        format!("保存 Permissions.config.xml 失败: {}", e)
    })?;

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存权限组配置: {}", server_id));

    Ok("权限组配置已保存".to_string())
}

fn list_plugins_blocking(
    config: Arc<Mutex<ConfigService>>,
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
            if path.extension().map(|e| e.to_string_lossy().to_lowercase())
                == Some("dll".to_string())
            {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
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

#[tauri::command(async)]
pub async fn list_plugins(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<Vec<PluginInfo>, String> {
    let config = config.inner().clone();
    let plugins =
        tauri::async_runtime::spawn_blocking(move || list_plugins_blocking(config, save_id))
            .await
            .map_err(|e| format!("读取插件列表任务失败: {}", e))?;
    Ok(plugins)
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

fn load_plugin_notes_blocking(config: Arc<Mutex<ConfigService>>) -> HashMap<String, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.load_plugin_notes()
}

#[tauri::command(async)]
pub async fn load_plugin_notes(
    config: State<'_, Arc<Mutex<ConfigService>>>,
) -> Result<HashMap<String, String>, String> {
    let config = config.inner().clone();
    let notes = tauri::async_runtime::spawn_blocking(move || load_plugin_notes_blocking(config))
        .await
        .map_err(|e| format!("读取插件备注任务失败: {}", e))?;
    Ok(notes)
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

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkshopDownloadConfig {
    pub file_ids: Vec<u64>,
    pub ignore_children_file_ids: Vec<u64>,
    pub query_cache_max_age_seconds: u32,
    pub max_query_retries: u32,
    pub use_cached_downloads: bool,
    pub should_monitor_updates: bool,
    pub shutdown_update_detected_timer: u32,
    pub shutdown_update_detected_message: String,
    pub shutdown_kick_message: String,
}

impl Default for WorkshopDownloadConfig {
    fn default() -> Self {
        Self {
            file_ids: Vec::new(),
            ignore_children_file_ids: Vec::new(),
            query_cache_max_age_seconds: 600,
            max_query_retries: 2,
            use_cached_downloads: true,
            should_monitor_updates: true,
            shutdown_update_detected_timer: 600,
            shutdown_update_detected_message: "Workshop file update detected, shutdown in: {0}"
                .to_string(),
            shutdown_kick_message: "Shutdown for Workshop file update.".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct RocketRconInfo {
    pub port: u16,
    pub password: String,
    /// 密码是否已设置（非空）
    pub has_password: bool,
}

fn read_rocket_rcon_config_blocking(
    config: Arc<Mutex<ConfigService>>,
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
        return Ok(RocketRconInfo {
            port: 27115,
            password: String::new(),
            has_password: false,
        });
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("读取 Rocket.config.xml 失败: {}", e))?;

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

    let has_password = !password.is_empty();
    // 掩码处理：不将原始密码发送到前端
    let masked = if has_password {
        "*".repeat(password.len().min(8))
    } else {
        String::new()
    };
    Ok(RocketRconInfo {
        port,
        password: masked,
        has_password,
    })
}

#[tauri::command(async)]
pub async fn read_rocket_rcon_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<RocketRconInfo, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || read_rocket_rcon_config_blocking(config, save_id))
        .await
        .map_err(|e| format!("读取 Rocket.config.xml 任务失败: {}", e))?
}

#[tauri::command]
pub fn save_rocket_rcon_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    save_id: Option<String>,
    port: u16,
    password: String,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };
    ensure_save_not_running(process.inner(), &server_id)?;

    // 密码为空时表示保留原密码，从现有配置中读取
    let actual_password = if password.is_empty() {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers_config = cfg.load_servers_config();
        servers_config
            .servers
            .first()
            .map(|p| p.rcon.password.clone())
            .unwrap_or_default()
    } else {
        password.clone()
    };

    ConfigService::update_rocket_config(&server_root, &server_id, port, &actual_password).map_err(
        |e| {
            let ls = log.lock().unwrap_or_else(|e| e.into_inner());
            ls.log_app(&format!(
                "[ERROR] 保存 RCON 配置失败 ({}): {}",
                server_id, e
            ));
            e
        },
    )?;

    {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let mut servers_config = cfg.load_servers_config();
        if let Some(profile) = servers_config.servers.first_mut() {
            if profile.id == server_id {
                profile.rcon.port = port;
                profile.rcon.password = actual_password;
                cfg.save_servers_config(&servers_config)?;
            }
        }
    }

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存 RCON 配置: 存档 {}", server_id));

    Ok("RCON 配置已保存".to_string())
}

fn workshop_config_path(server_root: &str, server_id: &str) -> PathBuf {
    Path::new(server_root)
        .join("Servers")
        .join(server_id)
        .join("WorkshopDownloadConfig.json")
}

fn read_workshop_config_blocking(
    config: Arc<Mutex<ConfigService>>,
    save_id: Option<String>,
) -> Result<WorkshopDownloadConfig, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

    let path = workshop_config_path(&server_root, &server_id);

    if !path.exists() {
        return Ok(WorkshopDownloadConfig::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 WorkshopDownloadConfig.json 失败: {}", e))?;

    let raw: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 WorkshopDownloadConfig.json 失败: {}", e))?;

    let config = WorkshopDownloadConfig {
        file_ids: raw
            .get("File_IDs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.as_u64()
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        ignore_children_file_ids: raw
            .get("Ignore_Children_File_IDs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.as_u64()
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        query_cache_max_age_seconds: raw
            .get("Query_Cache_Max_Age_Seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(600) as u32,
        max_query_retries: raw
            .get("Max_Query_Retries")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as u32,
        use_cached_downloads: raw
            .get("Use_Cached_Downloads")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        should_monitor_updates: raw
            .get("Should_Monitor_Updates")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        shutdown_update_detected_timer: raw
            .get("Shutdown_Update_Detected_Timer")
            .and_then(|v| v.as_u64())
            .unwrap_or(600) as u32,
        shutdown_update_detected_message: raw
            .get("Shutdown_Update_Detected_Message")
            .and_then(|v| v.as_str())
            .unwrap_or("Workshop file update detected, shutdown in: {0}")
            .to_string(),
        shutdown_kick_message: raw
            .get("Shutdown_Kick_Message")
            .and_then(|v| v.as_str())
            .unwrap_or("Shutdown for Workshop file update.")
            .to_string(),
    };

    Ok(config)
}

#[tauri::command(async)]
pub async fn read_workshop_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<WorkshopDownloadConfig, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || read_workshop_config_blocking(config, save_id))
        .await
        .map_err(|e| format!("读取 WorkshopDownloadConfig.json 任务失败: {}", e))?
}

#[tauri::command]
pub fn save_workshop_config(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    save_id: Option<String>,
    workshop_config: WorkshopDownloadConfig,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };
    ensure_save_not_running(process.inner(), &server_id)?;

    let path = workshop_config_path(&server_root, &server_id);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let json_value = serde_json::json!({
        "File_IDs": workshop_config.file_ids,
        "Ignore_Children_File_IDs": workshop_config.ignore_children_file_ids,
        "Query_Cache_Max_Age_Seconds": workshop_config.query_cache_max_age_seconds,
        "Max_Query_Retries": workshop_config.max_query_retries,
        "Use_Cached_Downloads": workshop_config.use_cached_downloads,
        "Should_Monitor_Updates": workshop_config.should_monitor_updates,
        "Shutdown_Update_Detected_Timer": workshop_config.shutdown_update_detected_timer,
        "Shutdown_Update_Detected_Message": workshop_config.shutdown_update_detected_message,
        "Shutdown_Kick_Message": workshop_config.shutdown_kick_message,
    });

    let content =
        serde_json::to_string_pretty(&json_value).map_err(|e| format!("序列化失败: {}", e))?;

    crate::services::config_service::atomic_write(&path, &content).map_err(|e| {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[ERROR] 保存 WorkshopDownloadConfig.json 失败 ({}): {}",
            server_id, e
        ));
        format!("保存 WorkshopDownloadConfig.json 失败: {}", e)
    })?;

    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!("保存创意工坊配置: {}", server_id));

    Ok("创意工坊配置已保存".to_string())
}

fn load_workshop_mod_notes_blocking(config: Arc<Mutex<ConfigService>>) -> HashMap<String, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let path = cfg.config_dir().join("workshop_mod_notes.json");
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

#[tauri::command(async)]
pub async fn load_workshop_mod_notes(
    config: State<'_, Arc<Mutex<ConfigService>>>,
) -> Result<HashMap<String, String>, String> {
    let config = config.inner().clone();
    let notes =
        tauri::async_runtime::spawn_blocking(move || load_workshop_mod_notes_blocking(config))
            .await
            .map_err(|e| format!("读取模组备注任务失败: {}", e))?;
    Ok(notes)
}

#[tauri::command]
pub fn save_workshop_mod_notes(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    notes: HashMap<String, String>,
) -> Result<String, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let path = cfg.config_dir().join("workshop_mod_notes.json");
    let content = serde_json::to_string_pretty(&notes).map_err(|e| format!("序列化失败: {}", e))?;
    crate::services::config_service::atomic_write(&path, &content)?;
    Ok("模组备注已保存".to_string())
}

fn validate_workshop_url(url: &str) -> Result<(), String> {
    const WORKSHOP_HOME: &str = "https://steamcommunity.com/app/304930/workshop/";
    const MOD_PAGE_PREFIX: &str = "https://steamcommunity.com/sharedfiles/filedetails/?id=";

    if url == WORKSHOP_HOME {
        return Ok(());
    }

    if let Some(file_id) = url.strip_prefix(MOD_PAGE_PREFIX) {
        if !file_id.is_empty() && file_id.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(());
        }
    }

    Err("仅支持打开 Steam 创意工坊链接".to_string())
}

#[tauri::command]
pub fn open_url(url: String) -> Result<String, String> {
    validate_workshop_url(&url)?;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("explorer")
            .arg(&url)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        return Err("当前平台暂不支持打开链接".to_string());
    }

    Ok("已打开链接".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}", prefix, suffix))
    }

    #[test]
    fn resolve_delete_save_path_accepts_existing_save_directory() {
        let root = unique_temp_dir("usm_delete_save_ok");
        let save_dir = root.join("Servers").join("Save_1");
        fs::create_dir_all(&save_dir).unwrap();

        let resolved = resolve_delete_save_path(root.to_str().unwrap(), "Save_1").unwrap();

        assert_eq!(resolved, fs::canonicalize(&save_dir).unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_delete_save_path_rejects_path_traversal() {
        let root = unique_temp_dir("usm_delete_save_traversal");
        fs::create_dir_all(root.join("Servers")).unwrap();

        let err = resolve_delete_save_path(root.to_str().unwrap(), "../outside").unwrap_err();

        assert_eq!(err, "存档 ID 包含非法字符");
        let _ = fs::remove_dir_all(root);
    }

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

    fn sample_game_config() -> &'static str {
        "// > Unturned Server Configuration File\r\nVersion 1\r\n\r\nBrowser\r\n{\r\n\t// > Long description in the lower-right of the server lobby menu.\r\n\t// > Default: empty\r\n\tDesc_Full <color=#FF0000>中文服务器</color>\\n欢迎\r\n\r\n\t// > How the server is monetized (if at all).\r\n\t// > Options: Unspecified, Any, None, NonGameplay, Monetized\r\n\t// > Default: Unspecified\r\n\tMonetization\r\n\r\n\t// > Buttons shown in the server lobby menu.\r\n\tLinks\r\n\t[\r\n\t    {\r\n\t        Message QQ群\r\n\t        URL https://example.com/\r\n\t    }\r\n\t]\r\n}\r\n\r\nGameplay\r\n{\r\n\t// > Should a hit confirmation be shown when players deal damage?\r\n\t// > Easy: True    Normal: True    Hard: False\r\n\tHitmarkers True\r\n\r\n\t// > Percentage [0 to 1] of XP to retain.\r\n\tLose_Experience_PvP 1\r\n}\r\n"
    }

    #[test]
    fn parse_game_config_reads_sections_comments_utf8_and_blocks() {
        let parsed = parse_game_config(sample_game_config(), "Config.txt".to_string());

        assert_eq!(parsed.info.version.as_deref(), Some("1"));
        assert_eq!(parsed.info.line_ending, "\r\n");
        assert_eq!(parsed.info.sections.len(), 2);

        let browser = parsed
            .info
            .sections
            .iter()
            .find(|section| section.name == "Browser")
            .unwrap();
        assert_eq!(browser.entries.len(), 3);

        let desc = browser
            .entries
            .iter()
            .find(|entry| entry.key == "Desc_Full")
            .unwrap();
        assert_eq!(desc.value, "<color=#FF0000>中文服务器</color>\\n欢迎");
        assert_eq!(desc.value_kind, "long_text");
        assert!(desc.description[0].contains("Long description"));

        let monetization = browser
            .entries
            .iter()
            .find(|entry| entry.key == "Monetization")
            .unwrap();
        assert!(!monetization.has_value);
        assert_eq!(
            monetization.options,
            vec!["Unspecified", "Any", "None", "NonGameplay", "Monetized"]
        );

        let links = browser
            .entries
            .iter()
            .find(|entry| entry.key == "Links")
            .unwrap();
        assert_eq!(links.value_kind, "raw_block");
        assert!(links.value.contains("Message QQ群"));
    }

    #[test]
    fn infer_game_config_kind_prefers_config_comments_over_warning_suffix() {
        let max_ip_warnings = vec![
            "// > Whether rejections for Max_Clients_With_Same_IP_Address should log to command output."
                .to_string(),
            "// > Default: True".to_string(),
        ];
        assert_eq!(
            infer_game_config_value_kind(
                "Max_Clients_With_Same_IP_Address_Log_Warnings",
                "",
                &max_ip_warnings,
                false
            ),
            "bool"
        );

        let fake_lag_warnings = vec![
            "// > Whether fake lag detection should log to command output.".to_string(),
            "// > Default: False".to_string(),
        ];
        assert_eq!(
            infer_game_config_value_kind("Fake_Lag_Log_Warnings", "", &fake_lag_warnings, false),
            "bool"
        );

        let scheduled_warnings = vec![
            "// > Broadcast \"shutting down for scheduled maintenance\" warnings at these intervals."
                .to_string(),
            "// > Format is a list of hours:minutes:seconds.".to_string(),
        ];
        assert_eq!(
            infer_game_config_value_kind(
                "Scheduled_Shutdown_Warnings",
                "",
                &scheduled_warnings,
                false
            ),
            "raw_block"
        );

        let update_warnings = vec![
            "// > Broadcast \"shutting down for update\" warnings at these intervals.".to_string(),
            "// > Refer to Scheduled_Shutdown_Warnings for an explanation of the format."
                .to_string(),
        ];
        assert_eq!(
            infer_game_config_value_kind("Update_Shutdown_Warnings", "", &update_warnings, false),
            "raw_block"
        );

        let spawn_chance = vec![
            "// > Percentage [0 to 1] of item spawns to use.".to_string(),
            "// > Easy: 0.35    Normal: 0.35    Hard: 0.15".to_string(),
        ];
        assert_eq!(
            infer_game_config_value_kind("Spawn_Chance", "5", &spawn_chance, false),
            "percent"
        );

        let has_durability = vec!["// > Easy: False    Normal: True    Hard: True".to_string()];
        assert_eq!(
            infer_game_config_value_kind("Has_Durability", "false", &has_durability, false),
            "bool"
        );
    }

    #[test]
    fn apply_game_config_changes_preserves_unrelated_new_server_keys() {
        let loaded = parse_game_config(sample_game_config(), "Config.txt".to_string());
        let current = sample_game_config().replace(
            "\tHitmarkers True\r\n",
            "\tHitmarkers True\r\n\tNew_Server_Setting 42\r\n",
        );

        let output = apply_game_config_changes(
            &current,
            &loaded.info.source_hash,
            &[GameConfigChange {
                section: "Browser".to_string(),
                key: "Desc_Full".to_string(),
                original_value: "<color=#FF0000>中文服务器</color>\\n欢迎".to_string(),
                value: "<color=#00FF00>中文新描述</color>\\n欢迎".to_string(),
            }],
        )
        .unwrap();

        assert!(output.contains("New_Server_Setting 42"));
        assert!(output.contains("<color=#00FF00>中文新描述</color>\\n欢迎"));
        assert!(output.contains("\r\n"));
    }

    #[test]
    fn apply_game_config_changes_detects_external_same_key_conflict() {
        let loaded = parse_game_config(sample_game_config(), "Config.txt".to_string());
        let current = sample_game_config().replace("Hitmarkers True", "Hitmarkers False");

        let err = apply_game_config_changes(
            &current,
            &loaded.info.source_hash,
            &[GameConfigChange {
                section: "Gameplay".to_string(),
                key: "Hitmarkers".to_string(),
                original_value: "True".to_string(),
                value: "False".to_string(),
            }],
        )
        .expect_err("same key changed externally should conflict");

        assert!(err.contains("已被外部修改"));
    }

    #[test]
    fn apply_game_config_changes_can_restore_empty_default_value() {
        let loaded = parse_game_config(sample_game_config(), "Config.txt".to_string());
        let output = apply_game_config_changes(
            sample_game_config(),
            &loaded.info.source_hash,
            &[GameConfigChange {
                section: "Gameplay".to_string(),
                key: "Hitmarkers".to_string(),
                original_value: "True".to_string(),
                value: String::new(),
            }],
        )
        .unwrap();

        assert!(output.contains("\tHitmarkers\r\n"));
        assert!(!output.contains("Hitmarkers True"));
    }

    #[test]
    fn validate_workshop_url_allows_only_expected_steam_workshop_urls() {
        assert!(validate_workshop_url("https://steamcommunity.com/app/304930/workshop/").is_ok());
        assert!(validate_workshop_url(
            "https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890"
        )
        .is_ok());

        assert!(validate_workshop_url(
            "https://steamcommunity.com/sharedfiles/filedetails/?id=123&cmd=calc"
        )
        .is_err());
        assert!(validate_workshop_url("https://example.com/").is_err());
        assert!(
            validate_workshop_url("https://steamcommunity.com/sharedfiles/filedetails/?id=")
                .is_err()
        );
    }

    fn sample_permissions_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="utf-8"?>
<RocketPermissions xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <DefaultGroup>default</DefaultGroup>
  <Groups>
    <Group>
      <Id>default</Id>
      <DisplayName>Guest</DisplayName>
      <Prefix />
      <Suffix />
      <Color>white</Color>
      <Members />
      <Priority>100</Priority>
      <Permissions>
        <Permission Cooldown="0">p</Permission>
        <Permission Cooldown="0">compass</Permission>
        <Permission Cooldown="0">rocket</Permission>
      </Permissions>
    </Group>
    <Group>
      <Id>vip</Id>
      <DisplayName>VIP</DisplayName>
      <Prefix />
      <Suffix />
      <Color>FF9900</Color>
      <Members>
        <Member>76561198016438091</Member>
      </Members>
      <ParentGroup>default</ParentGroup>
      <Priority>100</Priority>
      <Permissions>
        <Permission Cooldown="0">effect</Permission>
        <Permission Cooldown="120">heal</Permission>
        <Permission Cooldown="30">v</Permission>
      </Permissions>
    </Group>
  </Groups>
</RocketPermissions>"#
    }

    fn sample_permissions_config() -> PermissionsConfigInfo {
        parse_permissions_config(
            sample_permissions_xml(),
            "Permissions.config.xml".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn parse_permissions_config_reads_groups_members_and_cooldowns() {
        let config = sample_permissions_config();

        assert!(config.exists);
        assert_eq!(config.default_group, "default");
        assert_eq!(config.groups.len(), 2);

        let default = &config.groups[0];
        assert_eq!(default.id, "default");
        assert_eq!(default.display_name, "Guest");
        assert_eq!(default.prefix, "");
        assert_eq!(default.suffix, "");
        assert!(default.members.is_empty());
        assert_eq!(default.permissions[1].name, "compass");
        assert_eq!(default.permissions[1].cooldown, 0);

        let vip = &config.groups[1];
        assert_eq!(vip.parent_group.as_deref(), Some("default"));
        assert_eq!(vip.members, vec!["76561198016438091"]);
        assert_eq!(vip.permissions[1].name, "heal");
        assert_eq!(vip.permissions[1].cooldown, 120);
    }

    #[test]
    fn render_permissions_config_round_trips_structured_data() {
        let config = sample_permissions_config();
        let rendered = render_permissions_config(&config);
        let parsed = parse_permissions_config(&rendered, "roundtrip.xml".to_string()).unwrap();

        assert_eq!(parsed.default_group, config.default_group);
        assert_eq!(parsed.groups, config.groups);
    }

    #[test]
    fn save_permissions_config_to_path_requires_existing_file() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "missing-permissions-{}.xml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let err = save_permissions_config_to_path(&path, sample_permissions_config())
            .expect_err("missing file should fail");

        assert!(err.contains("Permissions.config.xml 不存在"));
    }

    #[test]
    fn save_permissions_config_to_path_writes_valid_xml() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "permissions-write-{}.xml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, sample_permissions_xml()).unwrap();

        let mut config = sample_permissions_config();
        config.groups[1]
            .members
            .push("76561198016438091".to_string());
        config.groups[1]
            .members
            .push("76561198000000000".to_string());
        config.groups[1].permissions.push(GroupPermissionInfo {
            name: "vehicle.repair".to_string(),
            cooldown: 45,
        });

        save_permissions_config_to_path(&path, config).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        let parsed = parse_permissions_config(&saved, path.to_string_lossy().to_string()).unwrap();

        let vip = parsed
            .groups
            .iter()
            .find(|group| group.id == "vip")
            .unwrap();
        assert_eq!(vip.members, vec!["76561198016438091", "76561198000000000"]);
        assert!(vip
            .permissions
            .iter()
            .any(|permission| permission.name == "vehicle.repair" && permission.cooldown == 45));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_permissions_config_rejects_invalid_relationships() {
        let mut duplicate = sample_permissions_config();
        duplicate.groups[1].id = "default".to_string();
        assert!(validate_permissions_config(&duplicate)
            .unwrap_err()
            .contains("重复"));

        let mut missing_default = sample_permissions_config();
        missing_default.default_group = "missing".to_string();
        assert!(validate_permissions_config(&missing_default)
            .unwrap_err()
            .contains("默认权限组不存在"));

        let mut self_parent = sample_permissions_config();
        self_parent.groups[1].parent_group = Some("vip".to_string());
        assert!(validate_permissions_config(&self_parent)
            .unwrap_err()
            .contains("不能继承自身"));
    }

    #[test]
    fn normalize_permissions_config_rejects_empty_permission_name() {
        let mut config = sample_permissions_config();
        config.groups[0].permissions.push(GroupPermissionInfo {
            name: "  ".to_string(),
            cooldown: 0,
        });

        assert!(normalize_permissions_config(config)
            .unwrap_err()
            .contains("空权限名"));
    }
}
