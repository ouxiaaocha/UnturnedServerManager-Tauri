use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    #[serde(rename = "logRetentionDays")]
    pub log_retention_days: u32,
    #[serde(rename = "autoStartLastServer")]
    pub auto_start_last_server: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme: "Dark".to_string(),
            log_retention_days: 30,
            auto_start_last_server: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub password: String,
}

impl Default for RconConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 27115,
            password: "changeme".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    pub id: String,
    pub name: String,
    #[serde(rename = "steamCmdPath")]
    pub steam_cmd_path: String,
    #[serde(rename = "serverRoot")]
    pub server_root: String,
    #[serde(rename = "serverEntry")]
    pub server_entry: String,
    pub rcon: RconConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServersConfig {
    pub servers: Vec<ServerProfile>,
}

impl Default for ServersConfig {
    fn default() -> Self {
        Self { servers: vec![] }
    }
}
