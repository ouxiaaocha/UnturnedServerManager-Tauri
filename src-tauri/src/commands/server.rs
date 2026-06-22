use serde::Serialize;
use std::collections::HashMap;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, State};

use crate::commands::update::run_update_blocking;
use crate::services::config_service::ConfigService;
use crate::services::local_command_bridge;
use crate::services::log_service::LogService;
use crate::services::process::ProcessManager;

const RESTART_SETTLE_TIMEOUT: Duration = Duration::from_secs(12);
const RESTART_COOLDOWN_AFTER_EXIT: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct RconEndpoint {
    pub host: String,
    pub port: u16,
    pub password: String,
}

/// 当前会话的 RCON 连接设置，按存档隔离。
/// 启动非默认存档时，会将该存档的 RCON 配置暂存于 endpoints，
/// 供 rcon_connect(save_id) 使用，而不会覆盖 servers.json。
pub struct ActiveRcon {
    endpoints: HashMap<String, RconEndpoint>,
}

impl ActiveRcon {
    pub fn from_config(config: &ConfigService) -> Self {
        let servers = config.load_servers_config();
        let mut endpoints = HashMap::new();
        if let Some(profile) = servers.servers.first() {
            endpoints.insert(
                profile.id.clone(),
                RconEndpoint {
                    host: profile.rcon.host.clone(),
                    port: profile.rcon.port,
                    password: profile.rcon.password.clone(),
                },
            );
        }
        Self {
            endpoints,
        }
    }

    pub fn set_for_save(&mut self, save_id: String, endpoint: RconEndpoint) {
        self.endpoints.insert(save_id, endpoint);
    }

    pub fn endpoint_for_save(&self, save_id: &str) -> Option<RconEndpoint> {
        self.endpoints.get(save_id).cloned()
    }
}

/// 自动更新托管的运行时状态
#[derive(Default)]
pub struct AutoUpdateState {
    enabled_running_seen: bool,
    expected_stop: bool,
    stopped_since: Option<Instant>,
    update_in_progress: bool,
    last_save_id: Option<String>,
    last_launch_mode: Option<String>,
}

impl AutoUpdateState {
    fn record_start(&mut self, save_id: String, launch_mode: String) {
        self.enabled_running_seen = true;
        self.expected_stop = false;
        self.stopped_since = None;
        self.update_in_progress = false;
        self.last_save_id = Some(save_id);
        self.last_launch_mode = Some(launch_mode);
    }

    fn mark_expected_stop(&mut self) {
        self.expected_stop = true;
        self.stopped_since = None;
    }

    pub fn current_save_id(&self, default_save_id: &str) -> String {
        self.last_save_id
            .clone()
            .unwrap_or_else(|| default_save_id.to_string())
    }

    pub fn current_launch_mode(&self) -> String {
        self.last_launch_mode
            .clone()
            .unwrap_or_else(|| "internet".to_string())
    }
}

fn decode_xml_attr(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
}

fn read_rocket_rcon_settings(path: &Path) -> Option<(u16, String)> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut port = None;
    let mut password = None;

    if let Some(start) = content.find("Port=\"") {
        let after = start + 6;
        if let Some(end) = content[after..].find('"') {
            port = content[after..after + end].parse::<u16>().ok();
        }
    }

    if let Some(start) = content.find("Password=\"") {
        let after = start + 10;
        if let Some(end) = content[after..].find('"') {
            password = Some(decode_xml_attr(&content[after..after + end]));
        }
    }

    Some((port?, password?))
}

#[derive(Serialize)]
pub struct ServerStatus {
    pub state: String,
    pub pid: Option<u32>,
    pub uptime_secs: u64,
    pub output_count: usize,
}

#[derive(Serialize)]
pub struct ServerSnapshot {
    pub save_id: String,
    pub state: String,
    pub pid: Option<u32>,
    pub uptime_secs: u64,
    pub output_count: usize,
    pub output: Vec<String>,
}

#[derive(Serialize)]
pub struct RunningServerInfo {
    pub save_id: String,
    pub state: String,
    pub pid: Option<u32>,
    pub uptime_secs: u64,
    pub output_count: usize,
    pub launch_mode: String,
}

#[tauri::command]
pub fn get_server_status(process: State<'_, Arc<Mutex<ProcessManager>>>) -> ServerStatus {
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
    save_id: Option<String>,
) -> Vec<String> {
    let pm = process.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(save_id) = save_id.filter(|id| !id.trim().is_empty()) {
        pm.get_new_output_for(save_id.trim(), from_index)
    } else {
        pm.get_new_output(from_index)
    }
}

#[tauri::command]
pub fn get_server_snapshot(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    from_index: usize,
    save_id: Option<String>,
) -> ServerSnapshot {
    let resolved_save_id = resolve_actual_save_id(config.inner(), save_id).unwrap_or_default();
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    if resolved_save_id.is_empty() {
        pm.is_running();
        let output_count = pm.output_count();
        return ServerSnapshot {
            save_id: String::new(),
            state: pm.state().to_string(),
            pid: pm.pid(),
            uptime_secs: pm.uptime_secs(),
            output_count,
            output: pm.get_new_output(from_index),
        };
    }

    pm.is_running_for(&resolved_save_id);
    let output_count = pm.output_count_for(&resolved_save_id);
    ServerSnapshot {
        save_id: resolved_save_id.clone(),
        state: pm.state_for(&resolved_save_id).to_string(),
        pid: pm.pid_for(&resolved_save_id),
        uptime_secs: pm.uptime_secs_for(&resolved_save_id),
        output_count,
        output: pm.get_new_output_for(&resolved_save_id, from_index),
    }
}

#[tauri::command]
pub fn list_running_servers(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Vec<RunningServerInfo> {
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    let save_ids = pm.running_save_ids();
    save_ids
        .into_iter()
        .map(|save_id| RunningServerInfo {
            state: pm.state_for(&save_id).to_string(),
            pid: pm.pid_for(&save_id),
            uptime_secs: pm.uptime_secs_for(&save_id),
            output_count: pm.output_count_for(&save_id),
            launch_mode: pm.launch_mode_for(&save_id),
            save_id,
        })
        .collect()
}

#[tauri::command]
pub fn send_server_command(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    save_id: Option<String>,
    command: String,
) -> Result<String, String> {
    let target_save_id = resolve_actual_save_id(config.inner(), save_id)?;
    {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        if !pm.is_running_for(&target_save_id) {
            return Err("服务器未运行，无法发送本地命令".to_string());
        }
    }

    let (server_root, save_id) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        let profile = servers.servers.first().ok_or("没有配置服务器")?;
        let save_id = if target_save_id.is_empty() {
            let state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
            state
                .last_save_id
                .clone()
                .unwrap_or_else(|| profile.id.clone())
        } else {
            target_save_id.clone()
        };
        (profile.server_root.clone(), save_id)
    };

    let command = local_command_bridge::enqueue_command(&server_root, &save_id, &command)?;
    let pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.record_sent_command_for(&save_id, &command);
    Ok("命令已写入本地命令桥".to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn start_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    save_id: Option<String>,
    launch_mode: Option<String>,
) -> Result<String, String> {
    start_server_inner(
        process.inner(),
        config.inner(),
        log.inner(),
        active_rcon.inner(),
        auto_update.inner(),
        save_id,
        launch_mode,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn start_server_inner(
    process: &Arc<Mutex<ProcessManager>>,
    config: &Arc<Mutex<ConfigService>>,
    log: &Arc<Mutex<LogService>>,
    active_rcon: &Arc<Mutex<ActiveRcon>>,
    auto_update: &Arc<Mutex<AutoUpdateState>>,
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

    let actual_id = save_id.unwrap_or_else(|| server_id.clone());

    // 路径安全验证
    crate::services::config_service::validate_id(&actual_id)
        .map_err(|_| "存档 ID 包含非法字符".to_string())?;

    let mode = launch_mode.unwrap_or_else(|| "internet".to_string());
    ensure_no_port_conflict(process, &config, &profile.server_root, &actual_id)?;
    let entry_prefix = if mode == "lan" {
        "+LanServer"
    } else {
        "+InternetServer"
    };
    profile.server_entry = format!("{}/{}", entry_prefix, actual_id);

    // 优先使用所选存档的 Rocket.config.xml 作为当前会话的 RCON 配置，
    // 因为 servers.json 中的默认配置可能滞后于存档级别的 RCON 设置。
    let rocket_config_path = Path::new(&profile.server_root)
        .join("Servers")
        .join(&actual_id)
        .join("Rocket")
        .join("Rocket.config.xml");
    if let Some((port, password)) = read_rocket_rcon_settings(&rocket_config_path) {
        profile.rcon.port = port;
        profile.rcon.password = password;
    }

    local_command_bridge::ensure_bridge_installed(&profile.server_root, &actual_id)?;

    {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        if cfg.load_app_settings().auto_update_hosting {
            let _ =
                ConfigService::update_auto_update_config(&profile.server_root, &actual_id, true);
        }
    }

    // 更新会话级 RCON 设置（不修改 servers.json）
    {
        let mut ar = active_rcon.lock().unwrap_or_else(|e| e.into_inner());
        ar.set_for_save(
            actual_id.clone(),
            RconEndpoint {
                host: profile.rcon.host.clone(),
                port: profile.rcon.port,
                password: profile.rcon.password.clone(),
            },
        );
    }

    {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        pm.start(&actual_id, &mode, &profile).map_err(|e| {
            let ls = log.lock().unwrap_or_else(|e| e.into_inner());
            ls.log_operation(&format!("[ERROR] 启动服务器失败: {}", e));
            e
        })?;
    }

    let mode_str = if mode == "lan" {
        "局域网"
    } else {
        "互联网"
    };
    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
    ls.log_operation(&format!(
        "启动服务器: {} (存档: {}, 模式: {})",
        server_id, actual_id, mode_str
    ));

    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.record_start(actual_id, mode);
    }

    Ok("服务器已启动".to_string())
}

fn resolve_actual_save_id(
    config: &Arc<Mutex<ConfigService>>,
    save_id: Option<String>,
) -> Result<String, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let servers = cfg.load_servers_config();
    let profile = servers.servers.first().ok_or("没有配置服务器")?;
    let actual_id = save_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| profile.id.clone());
    crate::services::config_service::validate_id(&actual_id)
        .map_err(|_| "存档 ID 包含非法字符".to_string())?;
    Ok(actual_id)
}

fn read_save_ports(server_root: &str, save_id: &str, fallback_rcon_port: u16) -> (u16, u16) {
    let game_port = {
        let path = crate::commands::save::detect_commands_dat_path(server_root, save_id);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| crate::commands::save::parse_commands_dat(&content).port)
            .unwrap_or(27015)
    };

    let rocket_config_path = Path::new(server_root)
        .join("Servers")
        .join(save_id)
        .join("Rocket")
        .join("Rocket.config.xml");
    let rcon_port = read_rocket_rcon_settings(&rocket_config_path)
        .map(|(port, _)| port)
        .unwrap_or(fallback_rcon_port);

    (game_port, rcon_port)
}

fn ensure_no_port_conflict(
    process: &Arc<Mutex<ProcessManager>>,
    config: &Arc<Mutex<ConfigService>>,
    server_root: &str,
    save_id: &str,
) -> Result<(), String> {
    let fallback_rcon_port = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        servers
            .servers
            .first()
            .map(|profile| profile.rcon.port)
            .unwrap_or(27115)
    };
    let (target_game_port, target_rcon_port) =
        read_save_ports(server_root, save_id, fallback_rcon_port);
    let running_save_ids = {
        let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
        pm.running_save_ids()
    };

    for running_save_id in running_save_ids {
        if running_save_id == save_id {
            continue;
        }
        let (game_port, rcon_port) =
            read_save_ports(server_root, &running_save_id, fallback_rcon_port);
        if game_port == target_game_port {
            return Err(format!(
                "端口冲突：存档 {} 已占用游戏端口 {}",
                running_save_id, target_game_port
            ));
        }
        if rcon_port == target_rcon_port {
            return Err(format!(
                "端口冲突：存档 {} 已占用 RCON 端口 {}",
                running_save_id, target_rcon_port
            ));
        }
    }

    if !is_udp_port_available(target_game_port) {
        return Err(format!("游戏端口 {} 当前被系统占用", target_game_port));
    }
    if !is_tcp_port_available(target_rcon_port) {
        return Err(format!("RCON 端口 {} 当前被系统占用", target_rcon_port));
    }

    Ok(())
}

fn is_udp_port_available(port: u16) -> bool {
    UdpSocket::bind(("0.0.0.0", port)).is_ok()
}

fn is_tcp_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn local_command_target(
    config: &Arc<Mutex<ConfigService>>,
    auto_update: &Arc<Mutex<AutoUpdateState>>,
    save_id: Option<String>,
) -> Result<(String, String), String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let servers = cfg.load_servers_config();
    let profile = servers.servers.first().ok_or("没有配置服务器")?;
    let save_id = if let Some(id) = save_id.filter(|id| !id.trim().is_empty()) {
        id
    } else {
        let state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state
            .last_save_id
            .clone()
            .unwrap_or_else(|| profile.id.clone())
    };
    crate::services::config_service::validate_id(&save_id)
        .map_err(|_| "存档 ID 包含非法字符".to_string())?;
    Ok((profile.server_root.clone(), save_id))
}

/// 通过本地命令桥优雅关闭服务器：先 save，再 shutdown，然后渐进式轮询等待进程退出。
/// 超时后自动强制停止。返回 Ok("stopped") 或 Ok("forced") 或 Err。
async fn local_bridge_shutdown(
    process: &Arc<Mutex<ProcessManager>>,
    config: &Arc<Mutex<ConfigService>>,
    auto_update: &Arc<Mutex<AutoUpdateState>>,
    log: &Arc<Mutex<LogService>>,
    target_save_id: Option<String>,
) -> Result<&'static str, String> {
    let (server_root, save_id) = local_command_target(config, auto_update, target_save_id)?;
    let save_command = local_command_bridge::enqueue_command(&server_root, &save_id, "save")?;
    {
        let pm = process.lock().unwrap_or_else(|e| e.into_inner());
        pm.record_sent_command_for(&save_id, &save_command);
    }

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let shutdown_command =
        local_command_bridge::enqueue_command(&server_root, &save_id, "shutdown")?;
    {
        let pm = process.lock().unwrap_or_else(|e| e.into_inner());
        pm.record_sent_command_for(&save_id, &shutdown_command);
    }

    // 渐进式轮询等待进程退出：前期高频、后期低频，总超时约 10s
    for i in 0..44 {
        let ms = if i < 20 {
            100
        } else if i < 30 {
            200
        } else {
            500
        };
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        let is_running = {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            pm.is_running_for(&save_id)
        };
        if !is_running {
            return Ok("stopped");
        }
    }

    // 超时：强制停止
    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("[Warning] 服务器未响应，执行强制停止");
    }
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.force_stop_for(&save_id)?;
    Ok("forced")
}

async fn wait_before_restart(process: &Arc<Mutex<ProcessManager>>, save_id: &str) {
    {
        let pm = process.lock().unwrap_or_else(|e| e.into_inner());
        pm.record_system_message_for(save_id, "服务器进程已退出，等待后台资源完全释放...");
    }

    let started = Instant::now();
    while started.elapsed() < RESTART_SETTLE_TIMEOUT {
        let running = {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            pm.is_running_for(save_id)
        };
        if !running {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    tokio::time::sleep(RESTART_COOLDOWN_AFTER_EXIT).await;
}

#[tauri::command(async)]
pub async fn stop_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    save_id: Option<String>,
) -> Result<String, String> {
    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.mark_expected_stop();
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("停止服务器");
    }

    let result = local_bridge_shutdown(
        process.inner(),
        config.inner(),
        auto_update.inner(),
        log.inner(),
        save_id,
    )
    .await?;

    Ok(if result == "forced" {
        "服务器已强制停止".to_string()
    } else {
        "服务器已停止".to_string()
    })
}

/// 自动更新监控的停止信号，应用退出时设为 true
pub type AutoUpdateStopSignal = Arc<AtomicBool>;

pub fn start_auto_update_monitor(
    app: tauri::AppHandle,
    process: Arc<Mutex<ProcessManager>>,
    config: Arc<Mutex<ConfigService>>,
    log: Arc<Mutex<LogService>>,
    active_rcon: Arc<Mutex<ActiveRcon>>,
    auto_update: Arc<Mutex<AutoUpdateState>>,
    stop_signal: AutoUpdateStopSignal,
) {
    std::thread::spawn(move || {
        while !stop_signal.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_secs(5));

            if stop_signal.load(Ordering::Relaxed) {
                break;
            }

            let enabled = {
                let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
                cfg.load_app_settings().auto_update_hosting
            };

            if !enabled {
                let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
                state.update_in_progress = false;
                state.stopped_since = None;
                continue;
            }

            let running_count = {
                let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
                pm.running_count()
            };

            let session = {
                let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());

                if running_count > 0 {
                    state.enabled_running_seen = true;
                    state.stopped_since = None;
                    continue;
                }

                if state.update_in_progress || !state.enabled_running_seen {
                    continue;
                }

                if state.expected_stop {
                    state.expected_stop = false;
                    state.enabled_running_seen = false;
                    state.stopped_since = None;
                    continue;
                }

                let stopped_since = state.stopped_since.get_or_insert_with(Instant::now);
                if stopped_since.elapsed() < Duration::from_secs(10) {
                    continue;
                }

                let save_id = state.last_save_id.clone();
                let launch_mode = state.last_launch_mode.clone();
                state.update_in_progress = true;
                state.enabled_running_seen = false;
                state.stopped_since = None;
                (save_id, launch_mode)
            };

            let (save_id, launch_mode) = session;
            let app_for_update = app.clone();
            let _ = app_for_update.emit(
                "update-output",
                "[系统] 检测到服务器进程已退出，自动更新托管开始执行 SteamCMD 更新...",
            );
            {
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_operation("[自动更新托管] 检测到服务器退出，开始更新");
            }

            let update_result = run_update_blocking(
                &app_for_update,
                &config,
                &log,
                "[自动更新托管] 开始更新服务端",
            );

            match update_result {
                Ok(_) => {
                    let _ = app_for_update.emit(
                        "update-output",
                        "[系统] 自动更新完成，正在重新启动服务器...",
                    );
                    if let Err(e) = start_server_inner(
                        &process,
                        &config,
                        &log,
                        &active_rcon,
                        &auto_update,
                        save_id,
                        launch_mode,
                    ) {
                        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                        ls.log_operation(&format!("[ERROR] 自动更新后启动服务器失败: {}", e));
                        let _ = app_for_update.emit(
                            "update-output",
                            &format!("[ERROR] 自动更新后启动服务器失败: {}", e),
                        );
                        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
                        state.update_in_progress = false;
                    }
                }
                Err(e) => {
                    let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                    ls.log_operation(&format!("[ERROR] 自动更新托管失败: {}", e));
                    let _ = app_for_update
                        .emit("update-output", &format!("[ERROR] 自动更新托管失败: {}", e));
                    let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
                    state.update_in_progress = false;
                }
            }
        }
    });
}

fn extract_ip_address(body: &str) -> Option<String> {
    let trimmed = body.trim().trim_matches('"');
    if trimmed.parse::<IpAddr>().is_ok() {
        return Some(trimmed.to_string());
    }

    body.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':'))
        .find(|part| !part.is_empty() && part.parse::<IpAddr>().is_ok())
        .map(|part| part.to_string())
}

/// 获取当前电脑的公网 IP 地址（多源回退，每个接口 4 秒超时）。
/// 用 spawn_blocking 执行阻塞网络请求，避免占用 Tauri 命令线程。
#[tauri::command(async)]
pub async fn get_public_ip() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        const PUBLIC_IP_ENDPOINTS: &[&str] = &[
            "https://api.ipify.org",
            "https://checkip.amazonaws.com",
            "https://ifconfig.me/ip",
            "https://ip.3322.net",
            "https://4.ipw.cn",
            "https://myip.ipip.net",
        ];

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        let mut errors = Vec::new();
        for endpoint in PUBLIC_IP_ENDPOINTS {
            match client.get(*endpoint).send() {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        errors.push(format!("{}: HTTP {}", endpoint, resp.status()));
                        continue;
                    }

                    match resp.text() {
                        Ok(body) => {
                            if let Some(ip) = extract_ip_address(&body) {
                                return Ok(ip);
                            }
                            errors.push(format!("{}: 响应中未找到IP地址", endpoint));
                        }
                        Err(e) => errors.push(format!("{}: 读取响应失败 ({})", endpoint, e)),
                    }
                }
                Err(e) => errors.push(format!("{}: {}", endpoint, e)),
            }
        }

        Err(format!("获取公网IP失败: {}", errors.join("; ")))
    })
    .await
    .map_err(|e| format!("获取公网IP任务失败: {}", e))?
}

/// 从存档的 Commands.dat 中读取游戏端口
#[tauri::command]
pub fn get_server_port(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<u16, String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let servers = cfg.load_servers_config();
    let profile = servers.servers.first().ok_or("没有配置服务器")?;
    let actual_id = save_id.unwrap_or_else(|| profile.id.clone());

    crate::services::config_service::validate_id(&actual_id)
        .map_err(|_| "存档 ID 包含非法字符".to_string())?;

    let path = crate::commands::save::detect_commands_dat_path(&profile.server_root, &actual_id);
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 Commands.dat 失败: {}", e))?;
    let info = crate::commands::save::parse_commands_dat(&content);
    Ok(info.port.unwrap_or(27015))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn read_rocket_rcon_settings_decodes_xml_password() {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("rocket-rcon-{}.xml", unique));

        std::fs::write(
            &path,
            r#"<RocketSettings><RCON Enabled="true" Port="27200" Password="a&amp;b&quot;c&apos;d" /></RocketSettings>"#,
        )
        .unwrap();

        let settings = read_rocket_rcon_settings(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(settings.0, 27200);
        assert_eq!(settings.1, "a&b\"c'd");
    }

    #[test]
    fn extract_ip_address_supports_plain_json_and_text_responses() {
        assert_eq!(
            extract_ip_address("203.0.113.7\n").as_deref(),
            Some("203.0.113.7")
        );
        assert_eq!(
            extract_ip_address(r#"{"ip":"203.0.113.8"}"#).as_deref(),
            Some("203.0.113.8")
        );
        assert_eq!(
            extract_ip_address("当前 IP：203.0.113.9 来自测试网络").as_deref(),
            Some("203.0.113.9")
        );
    }
}

#[tauri::command]
pub fn force_stop_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    save_id: Option<String>,
) -> Result<String, String> {
    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.mark_expected_stop();
    }
    let target_save_id = resolve_actual_save_id(config.inner(), save_id)?;
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.force_stop_for(&target_save_id)?;
    Ok("服务器已强制停止".to_string())
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub async fn restart_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    save_id: Option<String>,
    launch_mode: Option<String>,
) -> Result<String, String> {
    let target_save_id = resolve_actual_save_id(config.inner(), save_id.clone())?;
    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.mark_expected_stop();
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("重启服务器");
    }

    local_bridge_shutdown(
        process.inner(),
        config.inner(),
        auto_update.inner(),
        log.inner(),
        Some(target_save_id.clone()),
    )
    .await?;

    wait_before_restart(process.inner(), &target_save_id).await;

    // 复用 start_server_inner 重新启动
    start_server_inner(
        process.inner(),
        config.inner(),
        log.inner(),
        active_rcon.inner(),
        auto_update.inner(),
        Some(target_save_id),
        launch_mode,
    )
}
