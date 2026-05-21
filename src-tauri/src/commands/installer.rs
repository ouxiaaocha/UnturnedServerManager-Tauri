use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::services::config_service::ConfigService;
use crate::services::log_service::LogService;

fn emit(app: &AppHandle, msg: &str) {
    let _ = app.emit("installer-progress", msg.to_string());
}

fn exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

/// Spawn background download + init. Emits installer-progress events.
/// Final event: "DONE:<path>" on success, "ERROR:<message>" on failure.
#[tauri::command]
pub fn download_steamcmd(app: AppHandle) -> Result<(), String> {
    emit(&app, "[系统] 开始下载 SteamCMD...");

    std::thread::spawn(move || {
        match do_download_steamcmd(&app) {
            Ok(path) => emit(&app, &format!("DONE:{}", path)),
            Err(e) => emit(&app, &format!("ERROR:{}", e)),
        }
    });

    Ok(())
}

fn do_download_steamcmd(app: &AppHandle) -> Result<String, String> {
    let base_dir = exe_dir();
    let steamcmd_dir = base_dir.join("SteamCMD");
    let steamcmd_exe = steamcmd_dir.join("steamcmd.exe");

    if steamcmd_exe.exists() {
        emit(app, "[系统] SteamCMD 已存在，跳过下载");
        return Ok(steamcmd_exe.to_string_lossy().to_string());
    }

    // Clean up any previous failed attempt
    if steamcmd_dir.exists() {
        let _ = fs::remove_dir_all(&steamcmd_dir);
    }
    fs::create_dir_all(&steamcmd_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    // Run download logic, clean up on failure
    let result = do_download_steamcmd_inner(app, &steamcmd_dir, &steamcmd_exe);
    if result.is_err() {
        emit(app, "[系统] 下载失败，正在清理...");
        let _ = fs::remove_dir_all(&steamcmd_dir);
    }
    result
}

fn do_download_steamcmd_inner(app: &AppHandle, steamcmd_dir: &Path, steamcmd_exe: &Path) -> Result<String, String> {
    let zip_path = steamcmd_dir.join("steamcmd.zip");

    emit(app, "[系统] 正在下载 SteamCMD (约 5MB)...");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut response = client
        .get("https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip")
        .send()
        .map_err(|e| format!("下载失败 (请检查网络): {}", e))?;

    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut bytes = Vec::new();

    let mut buffer = [0u8; 65536];
    loop {
        let n = response.read(&mut buffer).map_err(|e| format!("下载中断: {}", e))?;
        if n == 0 { break; }
        bytes.extend_from_slice(&buffer[..n]);
        downloaded += n as u64;
        if total > 0 {
            let pct = downloaded * 100 / total;
            emit(app, &format!("[下载] {}% ({:.1}MB/{:.1}MB)", pct,
                downloaded as f64 / 1_048_576.0, total as f64 / 1_048_576.0));
        }
    }

    emit(app, "[系统] 下载完成，正在解压...");
    fs::write(&zip_path, &bytes).map_err(|e| format!("写入失败: {}", e))?;

    // PowerShell Expand-Archive for reliability
    let ps_result = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                zip_path.display(), steamcmd_dir.display())])
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status();

    let _ = fs::remove_file(&zip_path);

    if ps_result.map(|s| !s.success()).unwrap_or(true) {
        // Fallback: zip crate
        emit(app, "[系统] 备用解压方案...");
        let zip_file = fs::File::open(&zip_path).map_err(|_| "zip 文件丢失".to_string())?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("读取 zip: {}", e))?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("zip 条目: {}", e))?;
            let out_path = match file.enclosed_name() {
                Some(p) => steamcmd_dir.join(p), None => continue,
            };
            if file.is_dir() { fs::create_dir_all(&out_path).ok(); }
            else {
                if let Some(p) = out_path.parent() { fs::create_dir_all(p).ok(); }
                let mut out = fs::File::create(&out_path).map_err(|e| format!("创建文件失败: {}", e))?;
                std::io::copy(&mut file, &mut out).map_err(|e| format!("解压失败: {}", e))?;
            }
        }
    }

    if !steamcmd_exe.exists() {
        return Err("解压后未找到 steamcmd.exe".to_string());
    }

    emit(app, "[系统] 正在初始化 SteamCMD (首次自更新)...");

    let exe_path = steamcmd_exe.clone();
    let work_dir = steamcmd_dir.clone();
    let mut init_cmd = Command::new(&exe_path);
    init_cmd.args(["+login", "anonymous", "+quit"])
        .current_dir(&work_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    #[cfg(windows)]
    init_cmd.creation_flags(0x08000000);

    let mut child = init_cmd.spawn().map_err(|e| format!("启动 SteamCMD 失败: {}", e))?;

    // Read stderr in a separate thread to prevent pipe deadlock
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

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let t = line.trim().to_string();
            if !t.is_empty() { emit(app, &t); }
        }
    }

    let errs = stderr_handle.map(|h| h.join().unwrap_or_default()).unwrap_or_default();
    let status = child.wait().map_err(|e| format!("等待退出: {}", e))?;

    if status.success() || steamcmd_exe.exists() {
        let path = steamcmd_exe.to_string_lossy().to_string();
        emit(app, "[系统] SteamCMD 准备就绪");
        Ok(path)
    } else {
        Err(format!("初始化失败 (退出码 {:?})\n{}", status.code(), errs.join("\n")))
    }
}

#[tauri::command]
pub fn download_server(app: AppHandle, steamcmd_path: String) -> Result<(), String> {
    emit(&app, "[系统] 开始下载 Unturned 服务端...");

    let path = if !steamcmd_path.is_empty() {
        steamcmd_path
    } else {
        return Err("SteamCMD 路径为空".to_string());
    };

    std::thread::spawn(move || {
        match do_download_server(&app, &path) {
            Ok(p) => emit(&app, &format!("DONE:{}", p)),
            Err(e) => emit(&app, &format!("ERROR:{}", e)),
        }
    });

    Ok(())
}

fn do_download_server(app: &AppHandle, steamcmd_path: &str) -> Result<String, String> {
    if !Path::new(steamcmd_path).exists() {
        return Err(format!("SteamCMD 不存在: {}", steamcmd_path));
    }

    // Server installs to: {steamcmd_dir}/steamapps/common/U3DS
    let steamcmd_dir = Path::new(steamcmd_path).parent().unwrap_or(Path::new(""));
    let server_root = steamcmd_dir.join("steamapps").join("common").join("U3DS");
    let server_root_str = server_root.to_string_lossy().to_string();

    let result = do_download_server_inner(app, steamcmd_path, &server_root, &server_root_str);
    if result.is_err() {
        emit(app, "[系统] 下载失败，正在清理...");
        let _ = fs::remove_dir_all(&server_root);
    }
    result
}

fn do_download_server_inner(app: &AppHandle, steamcmd_path: &str, server_root: &Path, server_root_str: &str) -> Result<String, String> {
    emit(app, "[系统] 正在下载 Unturned 服务端 (首次约 10-15 分钟)...");

    if let Some(p) = server_root.parent() { fs::create_dir_all(p).ok(); }

    let mut cmd = Command::new(steamcmd_path);
    cmd.args([
        "+force_install_dir", &server_root_str,
        "+login", "anonymous",
        "+app_update", "1110390", "validate",
        "+quit",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    let mut child = cmd.spawn().map_err(|e| format!("启动 SteamCMD: {}", e))?;

    // Read stderr in a separate thread to prevent pipe deadlock
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

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let t = line.trim().to_string();
            if !t.is_empty() { emit(app, &t); }
        }
    }

    let errs = stderr_handle.map(|h| h.join().unwrap_or_default()).unwrap_or_default();
    let status = child.wait().map_err(|e| format!("等待退出: {}", e))?;

    if status.success() {
        // SteamCMD may create lowercase directory. Detect actual dir.
        let actual = if server_root.exists() {
            server_root_str.to_string()
        } else {
            // Try lowercase
            let lower = server_root.parent().unwrap_or(Path::new("")).join("u3ds");
            if lower.exists() {
                lower.to_string_lossy().to_string()
            } else {
                server_root_str.to_string()
            }
        };
        emit(app, "[系统] 服务端下载完成!");
        Ok(actual)
    } else {
        Err(format!("下载失败 (退出码 {:?})\n{}", status.code(), errs.join("\n")))
    }
}
