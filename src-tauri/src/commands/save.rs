use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use quick_xml::events::Event;
use quick_xml::Reader;
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
    save_id: Option<String>,
    permissions_config: PermissionsConfigInfo,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

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
    save_id: Option<String>,
    port: u16,
    password: String,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

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
    save_id: Option<String>,
    workshop_config: WorkshopDownloadConfig,
) -> Result<String, String> {
    let (server_root, server_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        resolve_save_dir(&cfg, &save_id)?
    };

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
