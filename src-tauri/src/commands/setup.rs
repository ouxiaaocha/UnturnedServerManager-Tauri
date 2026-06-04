use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn emit(app: &AppHandle, msg: &str) {
    let _ = app.emit("installer-progress", msg.to_string());
}

/// 过滤 -batchmode -nographics 模式下正常的 shader/GPU 警告
fn is_shader_noise(s: &str) -> bool {
    s.contains("shader is not supported on this GPU")
        || s.contains("Shader Unsupported:")
        || s.contains("fallback shader")
        || s.contains("subshaders removal was intentional")
        || s.contains("#pragma only_renderers")
        || s.contains("Fallback off?")
}

/// 检测 Rocket.Unturned 模块是否已安装
#[tauri::command]
pub fn detect_rocket_module(server_root: String) -> Result<bool, String> {
    if server_root.is_empty() {
        return Err("服务端目录为空".to_string());
    }
    let rocket_dir = Path::new(&server_root)
        .join("Modules")
        .join("Rocket.Unturned");
    Ok(rocket_dir.exists()
        && rocket_dir
            .read_dir()
            .map(|mut d| d.next().is_some())
            .unwrap_or(false))
}

pub fn install_rocket_module_sync(server_root: &str) -> Result<usize, String> {
    let src = Path::new(server_root)
        .join("Extras")
        .join("Rocket.Unturned");
    if !src.exists() {
        return Err(format!(
            "未找到 Extras/Rocket.Unturned 目录 ({})",
            src.display()
        ));
    }

    let dst = Path::new(server_root)
        .join("Modules")
        .join("Rocket.Unturned");
    copy_dir_recursive(&src, &dst)
}

/// 将 Rocket.Unturned 从 Extras 目录复制到 Modules（后台线程执行）
#[tauri::command]
pub fn install_rocket_module(
    app: AppHandle,
    log: State<'_, Arc<Mutex<LogService>>>,
    server_root: String,
) -> Result<(), String> {
    let src = Path::new(&server_root)
        .join("Extras")
        .join("Rocket.Unturned");
    if !src.exists() {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[ERROR] 安装 Rocket 失败: 未找到 Extras/Rocket.Unturned 目录 ({})",
            src.display()
        ));
        return Err(format!(
            "未找到 Extras/Rocket.Unturned 目录 ({})",
            src.display()
        ));
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app("[系统] 开始安装 Rocket.Unturned 模块");
    }

    emit(&app, "[系统] 正在安装 Rocket.Unturned 模块...");

    let log_clone = log.inner().clone();
    let dst_existed_before = Path::new(&server_root)
        .join("Modules")
        .join("Rocket.Unturned")
        .exists();
    std::thread::spawn(move || {
        let dst = Path::new(&server_root)
            .join("Modules")
            .join("Rocket.Unturned");
        match install_rocket_module_sync(&server_root) {
            Ok(count) => {
                let ls = log_clone.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_app(&format!(
                    "[系统] Rocket.Unturned 安装成功，已复制 {} 个文件",
                    count
                ));
                emit(&app, &format!("[系统] 已复制 {} 个文件", count));
                emit(&app, "DONE:已安装");
            }
            Err(e) => {
                let ls = log_clone.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_app(&format!("[ERROR] Rocket.Unturned 安装失败: {}", e));
                // 仅删除本次尝试创建的目录
                if !dst_existed_before {
                    let _ = fs::remove_dir_all(&dst);
                }
                emit(&app, &format!("ERROR:安装失败: {}", e));
            }
        }
    });

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    fs::create_dir_all(dst).map_err(|e| format!("创建目录 {}: {}", dst.display(), e))?;
    let mut count = 0;
    for entry in fs::read_dir(src).map_err(|e| format!("读取目录 {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("读取条目: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            count += copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "复制 {} -> {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
            count += 1;
        }
    }
    Ok(count)
}

/// 检测存档是否已初始化 Rocket（Rocket 目录 + Rocket.config.xml）
#[tauri::command]
pub fn check_save_rocket_status(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    server_root: Option<String>,
    save_id: String,
) -> Result<bool, String> {
    let server_root = if let Some(sr) = server_root.filter(|s| !s.is_empty()) {
        sr
    } else {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers_config = cfg.load_servers_config();
        match servers_config.servers.first() {
            Some(p) => p.server_root.clone(),
            None => return Ok(false),
        }
    };
    if save_id.is_empty() {
        return Ok(false);
    }
    let rocket_config = Path::new(&server_root)
        .join("Servers")
        .join(&save_id)
        .join("Rocket")
        .join("Rocket.config.xml");
    Ok(rocket_config.exists())
}

/// 初始化服务器存档：启动服务端直到 "Loading level: 100%" 后关闭。
/// 在后台线程执行，通过事件推送进度。
#[tauri::command]
pub fn init_server_save(
    app: AppHandle,
    config: State<'_, Arc<Mutex<ConfigService>>>,
    log: State<'_, Arc<Mutex<LogService>>>,
    server_root: Option<String>,
    save_name: String,
) -> Result<(), String> {
    let server_root = if let Some(sr) = server_root.filter(|s| !s.is_empty()) {
        sr
    } else {
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
        let servers_config = cfg.load_servers_config();
        match servers_config.servers.first() {
            Some(p) => p.server_root.clone(),
            None => {
                let ls = log.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_app("[ERROR] 初始化存档失败: 没有配置服务器");
                return Err("没有配置服务器".to_string());
            }
        }
    };
    if save_name.is_empty() {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app("[ERROR] 初始化存档失败: 存档名称不能为空");
        return Err("存档名称不能为空".to_string());
    }
    if save_name
        .chars()
        .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app("[ERROR] 初始化存档失败: 存档名称不能包含中文字符");
        return Err("存档名称不能包含中文字符".to_string());
    }
    if crate::services::config_service::validate_id(&save_name).is_err() {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app("[ERROR] 初始化存档失败: 存档名称包含非法字符");
        return Err("存档名称包含非法字符".to_string());
    }

    let exe_path = Path::new(&server_root).join("Unturned.exe");
    if !exe_path.exists() {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[ERROR] 初始化存档失败: 找不到 Unturned.exe ({})",
            exe_path.display()
        ));
        return Err(format!("找不到 Unturned.exe ({})", exe_path.display()));
    }

    {
        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!("[系统] 开始初始化存档 \"{}\"", save_name));
    }

    emit(&app, &format!("[系统] 正在初始化存档 \"{}\"...", save_name));
    emit(&app, "[系统] 首次启动需要一些时间，请耐心等待...");

    // 记录存档目录是否已存在，避免失败时误删已有数据
    let save_dir = Path::new(&server_root).join("Servers").join(&save_name);
    let save_dir_lower = Path::new(&server_root)
        .join("Servers")
        .join(save_name.to_lowercase());
    let existed_before = save_dir.exists() || save_dir_lower.exists();

    let log_clone = log.inner().clone();
    std::thread::spawn(move || {
        match do_init_save(&app, &exe_path, &server_root, &save_name) {
            Ok(()) => {
                let ls = log_clone.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_app(&format!("[系统] 存档 \"{}\" 初始化成功", save_name));
                emit(&app, &format!("DONE:{}", save_name));
            }
            Err(e) => {
                let ls = log_clone.lock().unwrap_or_else(|e| e.into_inner());
                ls.log_app(&format!("[ERROR] 存档 \"{}\" 初始化失败: {}", save_name, e));
                // 仅清理新创建的部分数据（不删除已有存档）
                if !existed_before {
                    let _ = fs::remove_dir_all(&save_dir);
                    let _ = fs::remove_dir_all(&save_dir_lower);
                }
                emit(&app, &format!("ERROR:{}", e));
            }
        }
    });

    Ok(())
}

fn do_init_save(
    app: &AppHandle,
    exe_path: &Path,
    server_root: &str,
    save_name: &str,
) -> Result<(), String> {
    let mut cmd = Command::new(exe_path);
    cmd.args([
        "-batchmode",
        "-nographics",
        &format!("+LanServer/{}", save_name),
    ])
    .current_dir(server_root)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = cmd.spawn().map_err(|e| format!("启动服务端失败: {}", e))?;

    let mut loaded = false;
    let mut output_count = 0;

    let stderr_handle = child.stderr.take().map(|stderr| {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let t = line.trim().to_string();
                if !t.is_empty()
                    && !is_shader_noise(&t)
                    && (t.contains("Error") || t.contains("Exception"))
                {
                    emit(&app_clone, &format!("[错误] {}", t));
                }
            }
        })
    });

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() || is_shader_noise(&trimmed) {
                continue;
            }

            output_count += 1;
            // 每隔几行发送一次事件，避免事件洪泛
            if output_count % 5 == 0
                || trimmed.contains("Loading level")
                || trimmed.contains("Error")
                || trimmed.contains("Server Code")
            {
                emit(app, &trimmed);
            }

            if trimmed.contains("Loading level: 100%") {
                loaded = true;
                emit(app, "[系统] 服务端加载完成，正在关闭...");

                // 等待文件写入完成
                std::thread::sleep(Duration::from_secs(5));

                // 强制终止进程（首次运行时 RCON 不可用，无法优雅关闭）
                let _ = child.kill();
                let _ = child.wait();
                emit(app, "[系统] 服务端已关闭");
                break;
            }
        }
    }

    if !loaded {
        let _ = child.kill();
        let _ = child.wait();
    }

    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    if !loaded {
        return Err("服务端未能加载到 100%".to_string());
    }

    let save_dir = Path::new(server_root).join("Servers").join(save_name);
    if save_dir.exists() {
        emit(app, &format!("[系统] 存档 \"{}\" 初始化成功", save_name));
        Ok(())
    } else {
        // 尝试小写路径（SteamCMD 有时会创建小写目录）
        let lower_dir = Path::new(server_root)
            .join("Servers")
            .join(save_name.to_lowercase());
        if lower_dir.exists() {
            emit(app, &format!("[系统] 存档 \"{}\" 初始化成功", save_name));
            Ok(())
        } else {
            Err(format!("存档目录未创建: {}", save_dir.display()))
        }
    }
}
