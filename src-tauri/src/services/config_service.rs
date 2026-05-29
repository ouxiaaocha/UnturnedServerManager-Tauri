use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
    fs::rename(&tmp_path, path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(())
}

fn machine_key() -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"UnturnedSM-");
    if let Ok(hostname) = std::env::var("COMPUTERNAME") {
        hasher.update(hostname.as_bytes());
    }
    if let Ok(username) = std::env::var("USERNAME") {
        hasher.update(username.as_bytes());
    }
    hasher.finalize().to_vec()
}

fn encode_password(plain: &str) -> String {
    let key = machine_key();
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

fn decode_password(stored: &str) -> String {
    if let Some(enc_part) = stored.strip_prefix("enc2:") {
        if let Ok(decoded) = BASE64.decode(enc_part) {
            if decoded.len() >= 13 {
                let (nonce_bytes, ciphertext) = decoded.split_at(12);
                let key = machine_key();
                if let Ok(cipher) = Aes256Gcm::new_from_slice(&key) {
                    let nonce = Nonce::from_slice(nonce_bytes);
                    if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                        return String::from_utf8(plaintext).unwrap_or_else(|_| stored.to_string());
                    }
                }
            }
        }
    }
    // Backward compatibility: previous AES-GCM format used a fixed zero nonce.
    if let Some(enc_part) = stored.strip_prefix("enc:") {
        if let Ok(decoded) = BASE64.decode(enc_part) {
            let key = machine_key();
            if let Ok(cipher) = Aes256Gcm::new_from_slice(&key) {
                let nonce_bytes = [0u8; 12];
                let nonce = Nonce::from_slice(&nonce_bytes);
                if let Ok(plaintext) = cipher.decrypt(nonce, decoded.as_slice()) {
                    return String::from_utf8(plaintext).unwrap_or_else(|_| stored.to_string());
                }
            }
        }
    }
    // Backward compatibility: old XOR-encoded passwords
    const XOR_KEY: &[u8] = b"UnturnedSM2024!@";
    if let Some(b64_part) = stored.strip_prefix("b64:") {
        if let Ok(decoded) = BASE64.decode(b64_part) {
            let decrypted: Vec<u8> = decoded
                .iter()
                .enumerate()
                .map(|(i, b)| b ^ XOR_KEY[i % XOR_KEY.len()])
                .collect();
            return String::from_utf8(decrypted).unwrap_or_else(|_| stored.to_string());
        }
    }
    // Return as-is if not encoded
    stored.to_string()
}

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
        let config = if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            let mut config: ServersConfig = serde_json::from_str(&content).unwrap_or_default();
            // Decode passwords
            for server in &mut config.servers {
                server.rcon.password = decode_password(&server.rcon.password);
            }
            config
        } else {
            ServersConfig::default()
        };

        *self
            .servers_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
        config
    }

    pub fn save_servers_config(&self, config: &ServersConfig) -> Result<(), String> {
        let path = self.config_dir().join("servers.json");
        // Encode passwords before saving
        let mut config_to_save = config.clone();
        for server in &mut config_to_save.servers {
            server.rcon.password = encode_password(&server.rcon.password);
        }
        let content = serde_json::to_string_pretty(&config_to_save)
            .map_err(|e| format!("序列化失败: {}", e))?;
        atomic_write(&path, &content)?;
        *self
            .servers_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
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
        // Validate server_id does not contain path separators
        if server_id.contains('/') || server_id.contains('\\') || server_id.contains("..") {
            return Err("服务器 ID 包含非法字符".to_string());
        }

        let path = Path::new(server_root)
            .join("Servers")
            .join(server_id)
            .join("Rocket")
            .join("Rocket.config.xml");

        if !path.exists() {
            return Err("Rocket.config.xml 不存在".to_string());
        }

        let content = fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;

        let mut new_content = content;

        // Replace Port
        if let Some(start) = new_content.find("Port=\"") {
            let after = start + 6;
            if let Some(end) = new_content[after..].find('"') {
                new_content = format!(
                    "{}{}{}",
                    &new_content[..after],
                    port,
                    &new_content[after + end..]
                );
            }
        }

        // Replace Password with XML-escaped value
        let escaped_password = Self::xml_escape(password);
        if let Some(start) = new_content.find("Password=\"") {
            let after = start + 10;
            if let Some(end) = new_content[after..].find('"') {
                new_content = format!(
                    "{}{}{}",
                    &new_content[..after],
                    escaped_password,
                    &new_content[after + end..]
                );
            }
        }

        // Ensure Enabled="true"
        if let Some(start) = new_content.find("Enabled=\"") {
            let after = start + 9;
            if let Some(end) = new_content[after..].find('"') {
                new_content = format!(
                    "{}{}{}",
                    &new_content[..after],
                    "true",
                    &new_content[after + end..]
                );
            }
        }

        atomic_write(&path, &new_content)
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
        let first = encode_password("same-secret");
        let second = encode_password("same-secret");

        assert_ne!(first, second);
        assert_eq!(decode_password(&first), "same-secret");
        assert_eq!(decode_password(&second), "same-secret");
    }
}
