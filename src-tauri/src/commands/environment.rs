use serde::Serialize;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

use crate::commands::setup::install_rocket_module_sync;
use crate::services::config_service::{validate_id, ConfigService};
use crate::services::local_command_bridge;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCheckItem {
    pub key: String,
    pub label: String,
    pub ok: bool,
    pub required: bool,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCheckReport {
    pub ok: bool,
    pub save_id: Option<String>,
    pub steam_cmd_path: Option<String>,
    pub server_root: Option<String>,
    pub items: Vec<EnvironmentCheckItem>,
}

fn item(
    key: &str,
    label: &str,
    ok: bool,
    required: bool,
    message: impl Into<String>,
    path: Option<String>,
) -> EnvironmentCheckItem {
    EnvironmentCheckItem {
        key: key.to_string(),
        label: label.to_string(),
        ok,
        required,
        message: message.into(),
        path,
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn contains_chinese(value: &str) -> bool {
    value.chars().any(|c| {
        matches!(
            c,
            '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}' | '\u{f900}'..='\u{faff}'
        )
    })
}

fn run_steamcmd_probe(steamcmd_path: &Path) -> Result<String, String> {
    let steamcmd_dir = steamcmd_path
        .parent()
        .ok_or_else(|| "SteamCMD 路径无效，无法获取所在目录".to_string())?;

    let mut cmd = Command::new(steamcmd_path);
    cmd.args(["+login", "anonymous", "+quit"])
        .current_dir(steamcmd_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 SteamCMD 连通测试失败: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_handle = stdout.map(|mut stream| {
        std::thread::spawn(move || {
            let mut output = String::new();
            let _ = stream.read_to_string(&mut output);
            output
        })
    });

    let stderr_handle = stderr.map(|mut stream| {
        std::thread::spawn(move || {
            let mut output = String::new();
            let _ = stream.read_to_string(&mut output);
            output
        })
    });

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("等待 SteamCMD 连通测试失败: {}", e))?
        {
            break status;
        }

        if started.elapsed() > Duration::from_secs(90) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("SteamCMD 连通测试超时，请检查网络或 SteamCMD 是否被占用".to_string());
        }

        std::thread::sleep(Duration::from_millis(250));
    };

    let mut combined = String::new();
    if let Some(handle) = stdout_handle {
        if let Ok(output) = handle.join() {
            combined.push_str(&output);
        }
    }
    if let Some(handle) = stderr_handle {
        if let Ok(output) = handle.join() {
            combined.push_str(&output);
        }
    }

    if !status.success() {
        return Err(format!(
            "SteamCMD 连通测试退出码异常: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "未知".to_string())
        ));
    }

    if combined.contains("FAILED") || combined.contains("Failed") || combined.contains("Error") {
        return Err("SteamCMD 输出中包含失败信息，请查看网络或 Steam 服务状态".to_string());
    }

    Ok("SteamCMD 可正常启动并匿名登录".to_string())
}

fn build_environment_report(
    config: &Arc<Mutex<ConfigService>>,
    include_steam_test: bool,
) -> EnvironmentCheckReport {
    let servers = {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.load_servers_config()
    };

    let Some(profile) = servers.servers.first() else {
        let items = vec![item(
            "config",
            "基础配置",
            false,
            true,
            "尚未配置 SteamCMD、服务端目录和默认存档",
            None,
        )];
        return EnvironmentCheckReport {
            ok: false,
            save_id: None,
            steam_cmd_path: None,
            server_root: None,
            items,
        };
    };

    let steamcmd_path = Path::new(&profile.steam_cmd_path);
    let server_root = Path::new(&profile.server_root);
    let steamcmd_has_chinese = contains_chinese(&profile.steam_cmd_path);
    let server_root_has_chinese = contains_chinese(&profile.server_root);
    let unturned_exe = server_root.join("Unturned.exe");
    let rocket_core = server_root
        .join("Modules")
        .join("Rocket.Unturned")
        .join("Rocket.Core.dll");
    let save_dir = server_root.join("Servers").join(&profile.id);
    let bridge_dll = local_command_bridge::bridge_dll_path(&profile.server_root, &profile.id).ok();

    let mut items = Vec::new();

    let steamcmd_path_ok = !steamcmd_has_chinese
        && steamcmd_path.is_file()
        && steamcmd_path
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("steamcmd.exe"))
            .unwrap_or(false);
    items.push(item(
        "steamcmd_path",
        "SteamCMD 路径",
        steamcmd_path_ok,
        true,
        if steamcmd_has_chinese {
            "SteamCMD 路径包含中文字符，请放到纯英文路径下"
        } else if steamcmd_path.is_file() {
            "已找到 steamcmd.exe"
        } else {
            "未找到 steamcmd.exe，请检查 SteamCMD 路径"
        },
        Some(profile.steam_cmd_path.clone()),
    ));

    if include_steam_test {
        let (ok, message) = if steamcmd_path.is_file() {
            match run_steamcmd_probe(steamcmd_path) {
                Ok(message) => (true, message),
                Err(error) => (false, error),
            }
        } else {
            (false, "SteamCMD 路径无效，无法执行连通测试".to_string())
        };
        items.push(item(
            "steamcmd_connectivity",
            "SteamCMD 网络连通",
            ok,
            false,
            message,
            Some(profile.steam_cmd_path.clone()),
        ));
    } else {
        items.push(item(
            "steamcmd_connectivity",
            "SteamCMD 网络连通",
            false,
            false,
            "未执行网络测试，点击设置页“完整检测”可测试 SteamCMD 服务器连通性",
            Some(profile.steam_cmd_path.clone()),
        ));
    }

    let server_root_ok = !server_root_has_chinese && server_root.is_dir() && unturned_exe.is_file();
    items.push(item(
        "server_root",
        "服务端目录",
        server_root_ok,
        true,
        if server_root_has_chinese {
            "服务端目录包含中文字符，请放到纯英文路径下"
        } else if server_root.is_dir() && unturned_exe.is_file() {
            "已找到 Unturned.exe"
        } else {
            "服务端目录不正确，未找到 Unturned.exe"
        },
        Some(path_string(&unturned_exe)),
    ));

    let save_id_ok = validate_id(&profile.id).is_ok();
    items.push(item(
        "save_dir",
        "当前存档目录",
        save_id_ok && save_dir.is_dir(),
        true,
        if !save_id_ok {
            "默认存档 ID 包含非法字符"
        } else if save_dir.is_dir() {
            "当前存档目录存在"
        } else {
            "当前存档目录不存在，请先初始化或选择正确存档"
        },
        Some(path_string(&save_dir)),
    ));

    items.push(item(
        "rocket_module",
        "Rocket.Unturned 模组",
        rocket_core.is_file(),
        true,
        if rocket_core.is_file() {
            "Rocket.Unturned 已安装到 Modules"
        } else {
            "未检测到 Rocket.Core.dll，可在设置页重新安装 Rocket"
        },
        Some(path_string(&rocket_core)),
    ));

    let bridge_ok = local_command_bridge::is_bridge_installed(&profile.server_root, &profile.id)
        .unwrap_or(false);
    items.push(item(
        "bridge_dll",
        "本地命令 Bridge 插件",
        bridge_ok,
        true,
        if bridge_ok {
            "UnturnedServerManagerBridge.dll 已安装且版本匹配"
        } else {
            "未检测到 Bridge DLL 或版本不匹配，可在设置页重新安装 Bridge"
        },
        bridge_dll.map(|path| path_string(&path)),
    ));

    let ok = items.iter().all(|entry| !entry.required || entry.ok);
    EnvironmentCheckReport {
        ok,
        save_id: Some(profile.id.clone()),
        steam_cmd_path: Some(profile.steam_cmd_path.clone()),
        server_root: Some(profile.server_root.clone()),
        items,
    }
}

#[tauri::command(async)]
pub async fn check_runtime_environment(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    include_steam_test: Option<bool>,
) -> Result<EnvironmentCheckReport, String> {
    let config = config.inner().clone();
    let include_steam_test = include_steam_test.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        Ok(build_environment_report(&config, include_steam_test))
    })
    .await
    .map_err(|e| format!("运行环境检测失败: {}", e))?
}

#[tauri::command(async)]
pub async fn install_runtime_requirement(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    target: String,
) -> Result<String, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let profile = {
            let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
            cfg.load_servers_config()
                .servers
                .first()
                .cloned()
                .ok_or_else(|| "没有配置服务器".to_string())?
        };

        match target.as_str() {
            "rocket" => {
                let count = install_rocket_module_sync(&profile.server_root)?;
                Ok(format!("Rocket.Unturned 已重新安装，复制 {} 个文件", count))
            }
            "bridge" => {
                local_command_bridge::ensure_bridge_installed(&profile.server_root, &profile.id)?;
                Ok("本地命令 Bridge 已重新安装".to_string())
            }
            "all" => {
                let rocket_core = Path::new(&profile.server_root)
                    .join("Modules")
                    .join("Rocket.Unturned")
                    .join("Rocket.Core.dll");
                let mut messages = Vec::new();
                if !rocket_core.is_file() {
                    let count = install_rocket_module_sync(&profile.server_root)?;
                    messages.push(format!("Rocket.Unturned 已安装，复制 {} 个文件", count));
                } else {
                    messages.push("Rocket.Unturned 已存在".to_string());
                }
                local_command_bridge::ensure_bridge_installed(&profile.server_root, &profile.id)?;
                messages.push("本地命令 Bridge 已安装".to_string());
                Ok(messages.join("；"))
            }
            _ => Err("未知修复目标".to_string()),
        }
    })
    .await
    .map_err(|e| format!("安装运行条件失败: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_items_decide_report_ok() {
        let entries = [
            item("required_ok", "Required OK", true, true, "", None),
            item("optional_bad", "Optional Bad", false, false, "", None),
        ];
        assert!(entries.iter().all(|entry| !entry.required || entry.ok));
    }
}
