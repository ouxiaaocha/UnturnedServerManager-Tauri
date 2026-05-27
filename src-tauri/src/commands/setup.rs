use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn emit(app: &AppHandle, msg: &str) {
    let _ = app.emit("installer-progress", msg.to_string());
}

/// Filter out shader/GPU warnings that are normal in -batchmode -nographics
fn is_shader_noise(s: &str) -> bool {
    s.contains("shader is not supported on this GPU")
        || s.contains("Shader Unsupported:")
        || s.contains("fallback shader")
        || s.contains("subshaders removal was intentional")
        || s.contains("#pragma only_renderers")
        || s.contains("Fallback off?")
}

/// Check if Rocket.Unturned module is installed.
#[tauri::command]
pub fn detect_rocket_module(server_root: String) -> Result<bool, String> {
    if server_root.is_empty() {
        return Err("服务端目录为空".to_string());
    }
    let rocket_dir = Path::new(&server_root).join("Modules").join("Rocket.Unturned");
    Ok(rocket_dir.exists() && rocket_dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false))
}

/// Copy Rocket.Unturned from Extras to Modules (background thread).
#[tauri::command]
pub fn install_rocket_module(app: AppHandle, server_root: String) -> Result<(), String> {
    let src = Path::new(&server_root).join("Extras").join("Rocket.Unturned");
    if !src.exists() {
        return Err(format!("未找到 Extras/Rocket.Unturned 目录 ({})", src.display()));
    }

    emit(&app, "[系统] 正在安装 Rocket.Unturned 模块...");

    std::thread::spawn(move || {
        let dst = Path::new(&server_root).join("Modules").join("Rocket.Unturned");
        match copy_dir_recursive(&src, &dst) {
            Ok(count) => {
                emit(&app, &format!("[系统] 已复制 {} 个文件", count));
                emit(&app, "DONE:已安装");
            }
            Err(e) => {
                // Clean up partial copy
                let _ = fs::remove_dir_all(&dst);
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
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制 {} -> {}: {}", src_path.display(), dst_path.display(), e))?;
            count += 1;
        }
    }
    Ok(count)
}

/// Check if a save has Rocket initialized (Rocket folder + Rocket.config.xml).
#[tauri::command]
pub fn check_save_rocket_status(server_root: String, save_id: String) -> Result<bool, String> {
    if server_root.is_empty() || save_id.is_empty() {
        return Ok(false);
    }
    let rocket_config = Path::new(&server_root)
        .join("Servers")
        .join(&save_id)
        .join("Rocket")
        .join("Rocket.config.xml");
    Ok(rocket_config.exists())
}

/// Initialize a server save by running the server once until "Loading level: 100%".
/// Uses background thread + events.
#[tauri::command]
pub fn init_server_save(app: AppHandle, server_root: String, save_name: String) -> Result<(), String> {
    if save_name.is_empty() {
        return Err("存档名称不能为空".to_string());
    }
    if save_name.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}') {
        return Err("存档名称不能包含中文字符".to_string());
    }
    if save_name.contains('/') || save_name.contains('\\') || save_name.contains("..") || save_name.contains(':') {
        return Err("存档名称包含非法字符".to_string());
    }

    let exe_path = Path::new(&server_root).join("Unturned.exe");
    if !exe_path.exists() {
        return Err(format!("找不到 Unturned.exe ({})", exe_path.display()));
    }

    emit(&app, &format!("[系统] 正在初始化存档 \"{}\"...", save_name));
    emit(&app, "[系统] 首次启动需要一些时间，请耐心等待...");

    std::thread::spawn(move || {
        match do_init_save(&app, &exe_path, &server_root, &save_name) {
            Ok(()) => {
                emit(&app, &format!("DONE:{}", save_name));
            }
            Err(e) => {
                // Clean up partial save directory
                let save_dir = Path::new(&server_root).join("Servers").join(&save_name);
                let _ = fs::remove_dir_all(&save_dir);
                // Also try lowercase
                let save_dir_lower = Path::new(&server_root).join("Servers").join(save_name.to_lowercase());
                let _ = fs::remove_dir_all(&save_dir_lower);
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
        "-batchmode", "-nographics",
        &format!("+LanServer/{}", save_name),
    ])
    .current_dir(server_root)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动服务端失败: {}", e))?;

    let mut loaded = false;
    let mut output_count = 0;

    // Read stdout line by line
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() || is_shader_noise(&trimmed) { continue; }

            output_count += 1;
            // Only emit every few lines to avoid flooding
            if output_count % 5 == 0 || trimmed.contains("Loading level") || trimmed.contains("Error") || trimmed.contains("Server Code") {
                emit(app, &trimmed);
            }

            if trimmed.contains("Loading level: 100%") {
                loaded = true;
                emit(app, "[系统] 服务端加载完成，正在关闭...");

                // Wait a moment for files to be written
                std::thread::sleep(Duration::from_secs(5));

                // Force kill the server process (RCON not available on first run)
                let _ = child.kill();
                let _ = child.wait();
                emit(app, "[系统] 服务端已关闭");
                break;
            }
        }
    }

    // Also drain stderr
    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let t = line.trim().to_string();
            if !t.is_empty() && !is_shader_noise(&t) && (t.contains("Error") || t.contains("Exception")) {
                emit(app, &format!("[错误] {}", t));
            }
        }
    }

    if !loaded {
        let _ = child.kill();
        let _ = child.wait();
        return Err("服务端未能加载到 100%".to_string());
    }

    // Verify save directory was created
    let save_dir = Path::new(server_root).join("Servers").join(save_name);
    if save_dir.exists() {
        emit(app, &format!("[系统] 存档 \"{}\" 初始化成功", save_name));
        Ok(())
    } else {
        // Try lowercase (SteamCMD sometimes creates lowercase)
        let lower_dir = Path::new(server_root).join("Servers").join(save_name.to_lowercase());
        if lower_dir.exists() {
            emit(app, &format!("[系统] 存档 \"{}\" 初始化成功", save_name));
            Ok(())
        } else {
            Err(format!("存档目录未创建: {}", save_dir.display()))
        }
    }
}
