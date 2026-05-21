use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Restarting,
    Updating,
    Error,
}

impl std::fmt::Display for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "已停止"),
            Self::Starting => write!(f, "启动中"),
            Self::Running => write!(f, "运行中"),
            Self::Stopping => write!(f, "停止中"),
            Self::Restarting => write!(f, "重启中"),
            Self::Updating => write!(f, "更新中"),
            Self::Error => write!(f, "错误"),
        }
    }
}
