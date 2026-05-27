use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;

#[tauri::command(async)]
pub async fn run_update(
    app: tauri::AppHandle,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
) -> Result<Vec<String>, String> {
    let (steam_cmd_path, server_root) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        let profile = servers.servers.first().ok_or("没有配置服务器")?;
        (profile.steam_cmd_path.clone(), profile.server_root.clone())
    };

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation("开始更新服务端");
    }

    let _ = app.emit("update-output", "[系统] 正在启动 SteamCMD...");

    let args = vec![
        "+force_install_dir".to_string(),
        server_root,
        "+login".to_string(),
        "anonymous".to_string(),
        "+app_update".to_string(),
        "1110390".to_string(),
        "validate".to_string(),
        "+quit".to_string(),
    ];

    let mut cmd = Command::new(&steam_cmd_path);
    cmd.args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = cmd.spawn().map_err(|e| format!("启动 SteamCMD 失败: {}", e))?;

    let mut output_lines = Vec::new();

    // Drain stderr in a separate thread to prevent pipe deadlock
    let stderr_handle = child.stderr.take().map(|stderr| {
        std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let t = line.trim().to_string();
                if !t.is_empty() { lines.push(t); }
            }
            lines
        })
    });

    // Stream stdout lines in real-time via Tauri events
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = app_clone.emit("update-output", &line);
            output_lines.push(line);
        }
    }

    let _errs = stderr_handle.map(|h| h.join().unwrap_or_default()).unwrap_or_default();
    let status = child.wait().map_err(|e| format!("等待失败: {}", e))?;

    let result_msg = if status.success() {
        "更新完成".to_string()
    } else {
        format!("更新失败，退出码: {:?}", status.code())
    };

    let _ = app.emit("update-output", &result_msg);
    output_lines.push(result_msg.clone());

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(&result_msg);
    }

    Ok(output_lines)
}
