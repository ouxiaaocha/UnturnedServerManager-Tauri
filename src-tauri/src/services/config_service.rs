use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use aes_gcm::{
    aead::rand_core::RngCore,
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use sha2::{Digest, Sha256};

use crate::models::config::{AppSettings, ServersConfig};
use std::collections::HashMap;

pub(crate) fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), String> {
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content).map_err(|e| format!("写入临时文件失败: {}", e))?;
    if let Err(e) = fs::rename(&tmp_path, path) {
        // rename 失败时清理临时文件，避免残留
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("重命名失败: {}", e));
    }
    Ok(())
}

/// Validate that a save/server ID does not contain path-traversal or illegal characters.
pub(crate) fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || id.contains(':')
        || id.contains('*')
        || id.contains('?')
        || id.contains('"')
        || id.contains('<')
        || id.contains('>')
        || id.contains('|')
    {
        return Err("ID 包含非法字符".to_string());
    }
    Ok(())
}

/// 获取或生成机器绑定的随机 salt（16 字节，存储在 config/machine_salt.bin）
fn get_or_create_salt(config_dir: &Path) -> Vec<u8> {
    let salt_path = config_dir.join("machine_salt.bin");

    // 尝试读取已有 salt
    if let Ok(salt) = fs::read(&salt_path) {
        if salt.len() >= 16 {
            return salt[..16].to_vec();
        }
    }

    // 生成新 salt 并持久化
    let mut salt = vec![0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let _ = fs::write(&salt_path, &salt);
    salt
}

/// 旧版密钥派生（仅 COMPUTERNAME + USERNAME），用于解密旧格式
fn legacy_machine_key() -> &'static [u8] {
    static KEY: OnceLock<Vec<u8>> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut hasher = Sha256::new();
        hasher.update(b"UnturnedSM-");
        if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            hasher.update(hostname.as_bytes());
        }
        if let Ok(username) = std::env::var("USERNAME") {
            hasher.update(username.as_bytes());
        }
        hasher.finalize().to_vec()
    })
}

/// 新版密钥派生（加入随机 salt），用于 enc2: 格式。
///
/// 密钥 = SHA256("UnturnedSM-v2-" + COMPUTERNAME + USERNAME + 随机 salt)。
///
/// 安全边界：此机制仅防止 servers.json 被拷贝到**其他机器**后明文泄露。
/// 因密钥材料（环境变量 COMPUTERNAME/USERNAME + 同目录 machine_salt.bin）对
/// 本机任何进程都可读，它**不能**抵御本机攻击者。此外 RCON 密码在
/// Commands.dat / Rocket.config.xml 中仍以明文存在（游戏本身要求），用户应知晓。
fn machine_key_with_salt(salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"UnturnedSM-v2-");
    if let Ok(hostname) = std::env::var("COMPUTERNAME") {
        hasher.update(hostname.as_bytes());
    }
    if let Ok(username) = std::env::var("USERNAME") {
        hasher.update(username.as_bytes());
    }
    hasher.update(salt);
    hasher.finalize().to_vec()
}

fn encode_password(plain: &str, salt: &[u8]) -> String {
    let key = machine_key_with_salt(salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    match cipher.encrypt(nonce, plain.as_bytes()) {
        Ok(ciphertext) => {
            let mut payload = nonce_bytes.to_vec();
            payload.extend_from_slice(&ciphertext);
            format!("enc2:{}", BASE64.encode(&payload))
        }
        Err(_) => plain.to_string(),
    }
}

/// 解密密码，支持 enc2:（新格式）、enc:（旧 AES-GCM 固定 nonce）、b64:（XOR）
/// 返回 (明文, 是否需要迁移)
fn decode_password(stored: &str, salt: &[u8]) -> (String, bool) {
    // enc2: 新格式（随机 nonce + salt 派生密钥）
    if let Some(enc_part) = stored.strip_prefix("enc2:") {
        if let Ok(decoded) = BASE64.decode(enc_part) {
            if decoded.len() >= 13 {
                let (nonce_bytes, ciphertext) = decoded.split_at(12);
                let key = machine_key_with_salt(salt);
                if let Ok(cipher) = Aes256Gcm::new_from_slice(&key) {
                    let nonce = Nonce::from_slice(nonce_bytes);
                    if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                        return (
                            String::from_utf8(plaintext).unwrap_or_else(|_| stored.to_string()),
                            false,
                        );
                    }
                }
            }
        }
    }
    // enc: 旧格式（固定零 nonce，legacy 密钥）— 需要迁移到 enc2:
    if let Some(enc_part) = stored.strip_prefix("enc:") {
        if let Ok(decoded) = BASE64.decode(enc_part) {
            let key = legacy_machine_key();
            if let Ok(cipher) = Aes256Gcm::new_from_slice(key) {
                let nonce_bytes = [0u8; 12];
                let nonce = Nonce::from_slice(&nonce_bytes);
                if let Ok(plaintext) = cipher.decrypt(nonce, decoded.as_slice()) {
                    let plain = String::from_utf8(plaintext).unwrap_or_else(|_| stored.to_string());
                    return (plain, true); // 需要迁移到 enc2:
                }
            }
        }
    }
    // b64: 旧 XOR 编码 — 需要迁移到 enc2:
    const XOR_KEY: &[u8] = b"UnturnedSM2024!@";
    if let Some(b64_part) = stored.strip_prefix("b64:") {
        if let Ok(decoded) = BASE64.decode(b64_part) {
            let decrypted: Vec<u8> = decoded
                .iter()
                .enumerate()
                .map(|(i, b)| b ^ XOR_KEY[i % XOR_KEY.len()])
                .collect();
            let plain = String::from_utf8(decrypted).unwrap_or_else(|_| stored.to_string());
            return (plain, true); // 需要迁移到 enc2:
        }
    }
    // 未编码，直接返回
    (stored.to_string(), true) // 也需要加密迁移
}

/// 配置文件管理服务，支持内存缓存、AES-256-GCM 密码加密和原子写入
pub struct ConfigService {
    base_dir: PathBuf,
    servers_cache: Mutex<Option<ServersConfig>>,
    app_settings_cache: Mutex<Option<AppSettings>>,
}

impl ConfigService {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            servers_cache: Mutex::new(None),
            app_settings_cache: Mutex::new(None),
        }
    }

    pub fn ensure_directories(&self) {
        let dirs = [
            "config",
            "logs/app",
            "logs/operation",
            "logs/game",
            "data",
            "backups",
        ];
        for dir in dirs {
            let path = self.base_dir.join(dir);
            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        self.base_dir.join("config")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    fn get_salt(&self) -> Vec<u8> {
        get_or_create_salt(&self.config_dir())
    }

    pub fn load_servers_config(&self) -> ServersConfig {
        if let Some(config) = self
            .servers_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        {
            return config;
        }

        let path = self.config_dir().join("servers.json");
        let salt = self.get_salt();
        let config = if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            let mut config: ServersConfig = serde_json::from_str(&content).unwrap_or_default();
            // 解密密码，检测是否需要迁移
            let mut needs_migration = false;
            for server in &mut config.servers {
                let (plain, migrate) = decode_password(&server.rcon.password, &salt);
                server.rcon.password = plain;
                if migrate {
                    needs_migration = true;
                }
            }
            // 自动迁移旧格式密码到 enc2:
            if needs_migration && !config.servers.is_empty() {
                let _ = self.save_servers_config(&config);
            }
            config
        } else {
            ServersConfig::default()
        };

        *self.servers_cache.lock().unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
        config
    }

    pub fn save_servers_config(&self, config: &ServersConfig) -> Result<(), String> {
        let path = self.config_dir().join("servers.json");
        let salt = self.get_salt();
        // Encode passwords before saving
        let mut config_to_save = config.clone();
        for server in &mut config_to_save.servers {
            server.rcon.password = encode_password(&server.rcon.password, &salt);
        }
        let content = serde_json::to_string_pretty(&config_to_save)
            .map_err(|e| format!("序列化失败: {}", e))?;
        atomic_write(&path, &content)?;
        *self.servers_cache.lock().unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
        Ok(())
    }

    pub fn load_app_settings(&self) -> AppSettings {
        if let Some(settings) = self
            .app_settings_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        {
            return settings;
        }

        let path = self.config_dir().join("appsettings.json");
        let settings = if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppSettings::default()
        };

        *self
            .app_settings_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(settings.clone());
        settings
    }

    pub fn save_app_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let path = self.config_dir().join("appsettings.json");
        let content =
            serde_json::to_string_pretty(settings).map_err(|e| format!("序列化失败: {}", e))?;
        atomic_write(&path, &content)?;
        *self
            .app_settings_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(settings.clone());
        Ok(())
    }

    pub fn is_first_run(&self) -> bool {
        let path = self.config_dir().join("servers.json");
        if !path.exists() {
            return true;
        }
        let config = self.load_servers_config();
        config.servers.is_empty()
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\'', "&apos;")
    }

    pub fn update_rocket_config(
        server_root: &str,
        server_id: &str,
        port: u16,
        password: &str,
    ) -> Result<(), String> {
        validate_id(server_id)?;

        let path = Path::new(server_root)
            .join("Servers")
            .join(server_id)
            .join("Rocket")
            .join("Rocket.config.xml");

        if !path.exists() {
            return Err("Rocket.config.xml 不存在".to_string());
        }

        let content = fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;

        // 定位 RCON 元素边界，在元素内部替换属性，避免误匹配注释或其他元素中的同名属性
        let rcon_start = content
            .find("<RCON ")
            .or_else(|| content.find("<RCON\t"))
            .or_else(|| content.find("<RCON\n"));
        let new_content = if let Some(start) = rcon_start {
            // 找到 RCON 元素的结束位置（以 /> 或 > 结尾）
            let rcon_region = &content[start..];
            if let Some(end_offset) = rcon_region.find('>') {
                let rcon_end = start + end_offset + 1;
                let mut element = content[start..rcon_end].to_string();

                // 替换 Port 属性
                if let Some(attr_start) = element.find("Port=\"") {
                    let val_start = attr_start + 6;
                    if let Some(val_end) = element[val_start..].find('"') {
                        element = format!(
                            "{}{}{}",
                            &element[..val_start],
                            port,
                            &element[val_start + val_end..]
                        );
                    }
                }

                // 替换 Password 属性
                let escaped_password = Self::xml_escape(password);
                if let Some(attr_start) = element.find("Password=\"") {
                    let val_start = attr_start + 10;
                    if let Some(val_end) = element[val_start..].find('"') {
                        element = format!(
                            "{}{}{}",
                            &element[..val_start],
                            escaped_password,
                            &element[val_start + val_end..]
                        );
                    }
                }

                // 确保 Enabled="true"
                if let Some(attr_start) = element.find("Enabled=\"") {
                    let val_start = attr_start + 9;
                    if let Some(val_end) = element[val_start..].find('"') {
                        element = format!(
                            "{}{}{}",
                            &element[..val_start],
                            "true",
                            &element[val_start + val_end..]
                        );
                    }
                }

                format!("{}{}{}", &content[..start], element, &content[rcon_end..])
            } else {
                content
            }
        } else {
            content
        };

        atomic_write(&path, &new_content)
    }

    pub fn update_auto_update_config(
        server_root: &str,
        server_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        validate_id(server_id)?;

        let path = Path::new(server_root)
            .join("Servers")
            .join(server_id)
            .join("Config.json");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let mut root = if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;
            serde_json::from_str::<serde_json::Value>(&content)
                .unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if !root.is_object() {
            root = serde_json::json!({});
        }

        let root_object = root.as_object_mut().ok_or("Config.json 格式无效")?;
        let server_value = root_object
            .entry("Server".to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !server_value.is_object() {
            *server_value = serde_json::json!({});
        }
        let server_object = server_value.as_object_mut().ok_or("Server 配置格式无效")?;

        server_object.insert(
            "Enable_Update_Shutdown".to_string(),
            serde_json::Value::Bool(enabled),
        );

        if enabled {
            server_object
                .entry("Update_Steam_Beta_Name".to_string())
                .or_insert_with(|| serde_json::Value::String("public".to_string()));
            server_object
                .entry("Update_Shutdown_Warnings".to_string())
                .or_insert_with(|| serde_json::json!(["30:00", "10:00", "5:00", "1:00"]));
        }

        let content =
            serde_json::to_string_pretty(&root).map_err(|e| format!("序列化失败: {}", e))?;
        atomic_write(&path, &content)
    }

    pub fn load_plugin_notes(&self) -> HashMap<String, String> {
        let path = self.config_dir().join("plugin_notes.json");
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    pub fn save_plugin_notes(&self, notes: &HashMap<String, String>) -> Result<(), String> {
        let path = self.config_dir().join("plugin_notes.json");
        let content =
            serde_json::to_string_pretty(notes).map_err(|e| format!("序列化失败: {}", e))?;
        atomic_write(&path, &content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_passwords_use_unique_ciphertext_for_same_plaintext() {
        let salt = vec![0u8; 16];
        let first = encode_password("same-secret", &salt);
        let second = encode_password("same-secret", &salt);

        assert_ne!(first, second);
        let (plain1, _) = decode_password(&first, &salt);
        let (plain2, _) = decode_password(&second, &salt);
        assert_eq!(plain1, "same-secret");
        assert_eq!(plain2, "same-secret");
    }

    #[test]
    fn update_auto_update_config_preserves_existing_server_settings() {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "usm-auto-update-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config_dir = root.join("Servers").join("Server");
        fs::create_dir_all(&config_dir).unwrap();
        let path = config_dir.join("Config.json");
        fs::write(
            &path,
            r#"{"Server":{"Name":"KeepMe","Update_Steam_Beta_Name":"preview"}}"#,
        )
        .unwrap();

        ConfigService::update_auto_update_config(root.to_str().unwrap(), "Server", true).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&content).unwrap();
        let server = value.get("Server").unwrap();

        assert_eq!(server.get("Name").unwrap(), "KeepMe");
        assert_eq!(server.get("Update_Steam_Beta_Name").unwrap(), "preview");
        assert_eq!(server.get("Enable_Update_Shutdown").unwrap(), true);
        assert!(server.get("Update_Shutdown_Warnings").unwrap().is_array());

        let _ = fs::remove_dir_all(root);
    }
}
