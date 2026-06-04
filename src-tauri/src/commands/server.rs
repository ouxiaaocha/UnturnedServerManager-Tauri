use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, State};

use crate::commands::update::run_update_blocking;
use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;
use crate::services::process::ProcessManager;
use crate::services::rcon_client::RconClient;

/// 当前会话的 RCON 连接设置。
/// 启动非默认存档时，会将该存档的 RCON 配置暂存于此，
/// 供 rcon_connect 使用，而不会覆盖 servers.json。
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
    pub state: String,
    pub pid: Option<u32>,
    pub uptime_secs: u64,
    pub output_count: usize,
    pub output: Vec<String>,
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
) -> Vec<String> {
    let pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.get_new_output(from_index)
}

#[tauri::command]
pub fn get_server_snapshot(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    from_index: usize,
) -> ServerSnapshot {
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.is_running();
    let output_count = pm.output_count();
    ServerSnapshot {
        state: pm.state().to_string(),
        pid: pm.pid(),
        uptime_secs: pm.uptime_secs(),
        output_count,
        output: pm.get_new_output(from_index),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn start_server(
    app: tauri::AppHandle,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    save_id: Option<String>,
    launch_mode: Option<String>,
) -> Result<String, String> {
    start_server_inner(
        app,
        process.inner(),
        rcon.inner(),
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
    app: tauri::AppHandle,
    process: &Arc<Mutex<ProcessManager>>,
    rcon: &Arc<Mutex<RconClient>>,
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

    {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        if cfg.load_app_settings().auto_update_hosting {
            let _ = ConfigService::update_auto_update_config(&profile.server_root, &actual_id, true);
        }
    }

    // 更新会话级 RCON 设置（不修改 servers.json）
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

    let _ = app.emit("server-started", &actual_id);

    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.record_start(actual_id, mode);
    }

    // 后台预连接 RCON，避免 stop/restart 时才发起连接导致延迟
    {
        let rcon_clone = Arc::clone(rcon);
        let ar_clone = Arc::clone(active_rcon);
        let log_clone = Arc::clone(log);
        std::thread::spawn(move || {
            // 等待服务器 RCON 端口就绪
            std::thread::sleep(Duration::from_secs(3));

            for attempt in 1..=3u32 {
                let (host, port, password) = {
                    let ar = ar_clone.lock().unwrap_or_else(|e| e.into_inner());
                    (ar.host.clone(), ar.port, ar.password.clone())
                };

                let mut client = rcon_clone.lock().unwrap_or_else(|e| e.into_inner());
                if client.is_connected() {
                    return; // 已连接（例如前端已手动连接）
                }

                match client.connect(&host, port, &password) {
                    Ok(_) => {
                        let ls = log_clone.lock().unwrap_or_else(|e| e.into_inner());
                        ls.log_operation("RCON 自动预连接成功");
                        return;
                    }
                    Err(_) => {
                        drop(client);
                        if attempt < 3 {
                            std::thread::sleep(Duration::from_secs(2));
                        }
                    }
                }
            }
        });
    }

    Ok("服务器已启动".to_string())
}

/// 通过 RCON 优雅关闭服务器：先 save，再 shutdown，然后渐进式轮询等待进程退出。
/// 超时后自动强制停止。返回 Ok("stopped") 或 Ok("forced") 或 Err。
async fn graceful_shutdown(
    process: &Arc<Mutex<ProcessManager>>,
    rcon: &Arc<Mutex<RconClient>>,
    active_rcon: &Arc<Mutex<ActiveRcon>>,
    log: &Arc<Mutex<LogService>>,
) -> Result<&'static str, String> {
    // 尝试通过 RCON 优雅关闭
    let shutdown_sent = {
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
    }; // rcon 锁在此释放

    if shutdown_sent {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let mut rcon_client = rcon.lock().unwrap_or_else(|e| e.into_inner());
        let _ = rcon_client.send_command("shutdown");
        rcon_client.disconnect();
    }

    // 渐进式轮询等待进程退出：前期高频、后期低频，总超时约 10s
    for i in 0..44 {
        let ms = if i < 20 { 100 } else if i < 30 { 200 } else { 500 };
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        let is_running = {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            pm.is_running()
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
    pm.force_stop()?;
    Ok("forced")
}

#[tauri::command(async)]
pub async fn stop_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
) -> Result<String, String> {
    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.mark_expected_stop();
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("停止服务器");
    }

    let result = graceful_shutdown(
        process.inner(),
        rcon.inner(),
        active_rcon.inner(),
        log.inner(),
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
    rcon: Arc<Mutex<RconClient>>,
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

            let running = {
                let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
                pm.is_running()
            };

            let session = {
                let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());

                if running {
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
                        app_for_update.clone(),
                        &process,
                        &rcon,
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
                    let _ = app_for_update.emit(
                        "update-output",
                        &format!("[ERROR] 自动更新托管失败: {}", e),
                    );
                    let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
                    state.update_in_progress = false;
                }
            }
        }
    });
}

/// 获取当前电脑的公网 IP 地址（5 秒超时）
#[tauri::command]
pub fn get_public_ip() -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    let resp = client
        .get("https://api.ipify.org?format=json")
        .send()
        .map_err(|e| format!("获取公网IP失败: {}", e))?;
    let body = resp
        .text()
        .map_err(|e| format!("读取IP响应失败: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("解析IP响应失败: {}", e))?;
    json["ip"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("响应中未找到IP字段".to_string())
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
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 Commands.dat 失败: {}", e))?;
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
}

#[tauri::command]
pub fn force_stop_server(
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
) -> Result<String, String> {
    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.mark_expected_stop();
    }
    let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
    pm.force_stop()?;
    Ok("服务器已强制停止".to_string())
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub async fn restart_server(
    app: tauri::AppHandle,
    process: State<'_, Arc<Mutex<ProcessManager>>>,
    rcon: State<'_, Arc<Mutex<RconClient>>>,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    active_rcon: State<'_, Arc<Mutex<ActiveRcon>>>,
    auto_update: State<'_, Arc<Mutex<AutoUpdateState>>>,
    save_id: Option<String>,
    launch_mode: Option<String>,
) -> Result<String, String> {
    {
        let mut state = auto_update.lock().unwrap_or_else(|e| e.into_inner());
        state.mark_expected_stop();
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("重启服务器");
    }

    graceful_shutdown(
        process.inner(),
        rcon.inner(),
        active_rcon.inner(),
        log.inner(),
    )
    .await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 复用 start_server_inner 重新启动
    start_server_inner(
        app,
        process.inner(),
        rcon.inner(),
        config.inner(),
        log.inner(),
        active_rcon.inner(),
        auto_update.inner(),
        save_id,
        launch_mode,
    )
}
