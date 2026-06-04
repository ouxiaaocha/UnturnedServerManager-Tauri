use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use sysinfo::System;

use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;

const UPDATE_OUTPUT_RETAIN_LINES: usize = 1000;
static UPDATE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

struct UpdateRunGuard;

impl UpdateRunGuard {
    fn acquire() -> Result<Self, String> {
        UPDATE_IN_PROGRESS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| Self)
            .map_err(|_| "已有更新任务正在运行，请等待当前更新完成".to_string())
    }
}

impl Drop for UpdateRunGuard {
    fn drop(&mut self) {
        UPDATE_IN_PROGRESS.store(false, Ordering::Release);
    }
}

fn build_update_args(server_root: &str) -> Vec<String> {
    vec![
        "+login".to_string(),
        "anonymous".to_string(),
        "+force_install_dir".to_string(),
        server_root.to_string(),
        "+app_update".to_string(),
        "1110390".to_string(),
        "validate".to_string(),
        "+quit".to_string(),
    ]
}

fn steamcmd_working_dir(steam_cmd_path: &str) -> &Path {
    Path::new(steam_cmd_path).parent().unwrap_or(Path::new(""))
}

fn has_steamcmd_success_marker(lines: &VecDeque<String>) -> bool {
    lines.iter().any(|line| {
        line.contains("Success!")
            && line.contains("App '1110390'")
            && line.contains("fully installed")
    })
}

fn push_update_line(lines: &Arc<Mutex<VecDeque<String>>>, line: String) {
    let mut output_lines = lines.lock().unwrap_or_else(|e| e.into_inner());
    if output_lines.len() >= UPDATE_OUTPUT_RETAIN_LINES {
        output_lines.pop_front();
    }
    output_lines.push_back(line);
}

fn is_unturned_process_alive(server_root: &str) -> bool {
    let target = Path::new(server_root).join("Unturned.exe");
    let target = target
        .canonicalize()
        .unwrap_or(target)
        .to_string_lossy()
        .to_lowercase();

    let system = System::new_all();
    system.processes().values().any(|process| {
        process
            .exe()
            .map(|exe| {
                let exe = exe
                    .canonicalize()
                    .unwrap_or_else(|_| exe.to_path_buf())
                    .to_string_lossy()
                    .to_lowercase();
                exe == target
            })
            .unwrap_or(false)
    })
}

#[tauri::command(async)]
pub async fn run_update(
    app: tauri::AppHandle,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
) -> Result<Vec<String>, String> {
    let config = config.inner().clone();
    let log = log.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_update_blocking(&app, &config, &log, "开始更新服务端")
    })
    .await
    .map_err(|e| format!("更新任务执行失败: {}", e))?
}

pub fn run_update_blocking(
    app: &tauri::AppHandle,
    config: &Arc<Mutex<ConfigService>>,
    log: &Arc<Mutex<LogService>>,
    operation: &str,
) -> Result<Vec<String>, String> {
    let _guard = UpdateRunGuard::acquire()?;

    let (steam_cmd_path, server_root) = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers = cfg.load_servers_config();
        let profile = servers.servers.first().ok_or("没有配置服务器")?;
        (profile.steam_cmd_path.clone(), profile.server_root.clone())
    };

    if is_unturned_process_alive(&server_root) {
        return Err(
            "检测到 Unturned.exe 仍在运行，请先停止服务器并等待进程完全退出后再更新".to_string(),
        );
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(operation);
    }

    let _ = app.emit("update-output", "[系统] 正在启动 SteamCMD...");

    let args = build_update_args(&server_root);

    let mut cmd = Command::new(&steam_cmd_path);
    cmd.args(&args)
        .current_dir(steamcmd_working_dir(&steam_cmd_path))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 SteamCMD 失败: {}", e))?;

    let output_lines: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

    // 在独立线程中读取 stderr，防止管道死锁，并保留 SteamCMD 的错误细节。
    let stderr_handle = child.stderr.take().map(|stderr| {
        let app_clone = app.clone();
        let output_lines = Arc::clone(&output_lines);
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let t = line.trim().to_string();
                if !t.is_empty() {
                    let _ = app_clone.emit("update-output", &t);
                    push_update_line(&output_lines, t);
                }
            }
        })
    });

    // 通过 Tauri 事件实时推送 stdout 输出
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = app_clone.emit("update-output", &line);
            push_update_line(&output_lines, line);
        }
    }

    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }
    let status = child.wait().map_err(|e| format!("等待失败: {}", e))?;

    let success_marker_seen = {
        let lines = output_lines.lock().unwrap_or_else(|e| e.into_inner());
        has_steamcmd_success_marker(&lines)
    };

    let result_msg = if status.success() && success_marker_seen {
        "更新完成".to_string()
    } else if status.success() {
        "更新失败：SteamCMD 正常退出，但没有检测到 Unturned 服务端安装成功标记".to_string()
    } else {
        format!("更新失败，退出码: {:?}", status.code())
    };

    let _ = app.emit("update-output", &result_msg);
    push_update_line(&output_lines, result_msg.clone());

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_operation(&result_msg);
    }

    let output = output_lines
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .cloned()
        .collect();

    if status.success() && success_marker_seen {
        Ok(output)
    } else {
        Err(result_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_update_args_targets_unturned_dedicated_server_with_validate() {
        let args = build_update_args("C:/Servers/U3DS");

        assert_eq!(
            args,
            vec![
                "+login",
                "anonymous",
                "+force_install_dir",
                "C:/Servers/U3DS",
                "+app_update",
                "1110390",
                "validate",
                "+quit",
            ]
        );
    }

    #[test]
    fn success_marker_requires_unturned_server_app_id() {
        let mut lines = VecDeque::new();
        lines.push_back("Success! App '1110390' fully installed.".to_string());

        assert!(has_steamcmd_success_marker(&lines));

        lines.clear();
        lines.push_back("Success! App '304930' fully installed.".to_string());

        assert!(!has_steamcmd_success_marker(&lines));
    }

    #[test]
    fn steamcmd_working_dir_uses_executable_parent() {
        assert_eq!(
            steamcmd_working_dir("E:/Steam CMD/steamcmd.exe"),
            Path::new("E:/Steam CMD")
        );
    }
}
