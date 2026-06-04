use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::services::config_service::validate_id;
use crate::services::process::normalize_server_command;

pub const BRIDGE_DLL_NAME: &str = "UnturnedServerManagerBridge.dll";
const BRIDGE_DLL_BYTES: &[u8] = include_bytes!("../../bridge/UnturnedServerManagerBridge.dll");

pub fn bridge_dir(server_root: &str, save_id: &str) -> Result<PathBuf, String> {
    validate_id(save_id).map_err(|_| "存档 ID 包含非法字符".to_string())?;
    Ok(Path::new(server_root)
        .join("Servers")
        .join(save_id)
        .join("Rocket")
        .join("Plugins")
        .join("UnturnedServerManagerBridge"))
}

fn plugins_dir(server_root: &str, save_id: &str) -> Result<PathBuf, String> {
    validate_id(save_id).map_err(|_| "存档 ID 包含非法字符".to_string())?;
    Ok(Path::new(server_root)
        .join("Servers")
        .join(save_id)
        .join("Rocket")
        .join("Plugins"))
}

pub fn bridge_dll_path(server_root: &str, save_id: &str) -> Result<PathBuf, String> {
    Ok(plugins_dir(server_root, save_id)?.join(BRIDGE_DLL_NAME))
}

pub fn is_bridge_installed(server_root: &str, save_id: &str) -> Result<bool, String> {
    let dll_path = bridge_dll_path(server_root, save_id)?;
    Ok(fs::read(&dll_path)
        .map(|existing| existing == BRIDGE_DLL_BYTES)
        .unwrap_or(false))
}

fn queue_path(server_root: &str, save_id: &str) -> Result<PathBuf, String> {
    Ok(bridge_dir(server_root, save_id)?.join("commands.queue"))
}

fn ready_path(server_root: &str, save_id: &str) -> Result<PathBuf, String> {
    Ok(bridge_dir(server_root, save_id)?.join("bridge.ready"))
}

pub fn ensure_bridge_installed(server_root: &str, save_id: &str) -> Result<(), String> {
    let rocket_core = Path::new(server_root)
        .join("Modules")
        .join("Rocket.Unturned")
        .join("Rocket.Core.dll");
    if !rocket_core.exists() {
        return Err("本地命令桥需要先安装 Rocket.Unturned 模块".to_string());
    }

    let plugins = plugins_dir(server_root, save_id)?;
    let dir = bridge_dir(server_root, save_id)?;
    fs::create_dir_all(&plugins).map_err(|e| format!("创建 Rocket 插件目录失败: {}", e))?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建本地命令桥目录失败: {}", e))?;

    let dll_path = plugins.join(BRIDGE_DLL_NAME);
    let should_write = fs::read(&dll_path)
        .map(|existing| existing != BRIDGE_DLL_BYTES)
        .unwrap_or(true);
    if should_write {
        fs::write(&dll_path, BRIDGE_DLL_BYTES)
            .map_err(|e| format!("写入本地命令桥插件失败: {}", e))?;
    }

    let queue = queue_path(server_root, save_id)?;
    if !queue.exists() {
        fs::write(&queue, "").map_err(|e| format!("创建本地命令队列失败: {}", e))?;
    }

    let ready = ready_path(server_root, save_id)?;
    let _ = fs::remove_file(ready);

    Ok(())
}

pub fn enqueue_command(server_root: &str, save_id: &str, command: &str) -> Result<String, String> {
    let command = normalize_server_command(command)?.to_string();

    let ready = ready_path(server_root, save_id)?;
    if !ready.exists() {
        return Err("本地命令桥未就绪，请重启服务器后再发送命令".to_string());
    }

    let queue = queue_path(server_root, save_id)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&queue)
        .map_err(|e| format!("打开本地命令队列失败: {}", e))?;
    writeln!(file, "{}", command).map_err(|e| format!("写入本地命令队列失败: {}", e))?;
    file.flush()
        .map_err(|e| format!("刷新本地命令队列失败: {}", e))?;

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("usm-bridge-test-{}", unique))
    }

    #[test]
    fn enqueue_command_requires_ready_file() {
        let root = temp_root();
        let dir = bridge_dir(root.to_str().unwrap(), "PEI").unwrap();
        fs::create_dir_all(&dir).unwrap();

        let err = enqueue_command(root.to_str().unwrap(), "PEI", "save").unwrap_err();

        let _ = fs::remove_dir_all(root);
        assert_eq!(err, "本地命令桥未就绪，请重启服务器后再发送命令");
    }

    #[test]
    fn enqueue_command_appends_to_queue_when_ready() {
        let root = temp_root();
        let dir = bridge_dir(root.to_str().unwrap(), "PEI").unwrap();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("bridge.ready"), "ready").unwrap();

        let command = enqueue_command(root.to_str().unwrap(), "PEI", " save ").unwrap();
        let queue = fs::read_to_string(dir.join("commands.queue")).unwrap();

        let _ = fs::remove_dir_all(root);
        assert_eq!(command, "save");
        assert_eq!(queue, "save\n");
    }
}
