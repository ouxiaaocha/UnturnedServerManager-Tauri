use serde::{Deserialize, Serialize};

/// 应用全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    #[serde(rename = "logRetentionDays")]
    pub log_retention_days: u32,
    #[serde(rename = "autoStartLastServer")]
    pub auto_start_last_server: bool,
    #[serde(rename = "autoUpdateHosting")]
    pub auto_update_hosting: bool,
    #[serde(rename = "closeToTray")]
    pub close_to_tray: bool,
    #[serde(rename = "closeActionRemembered")]
    pub close_action_remembered: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme: "Dark".to_string(),
            log_retention_days: 15,
            auto_start_last_server: false,
            auto_update_hosting: false,
            close_to_tray: false,
            close_action_remembered: false,
        }
    }
}

/// RCON 连接配置
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

/// 单个服务器实例的配置（路径、RCON、存档 ID）
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

/// 所有服务器实例的配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServersConfig {
    pub servers: Vec<ServerProfile>,
}
