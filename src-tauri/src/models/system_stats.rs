use serde::Serialize;

/// 系统资源快照
#[derive(Serialize, Clone)]
pub struct SystemStats {
    pub cpu_name: String,
    pub physical_core_count: Option<usize>,
    pub logical_core_count: usize,
    pub cpu_frequency_mhz: u64,
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub memory_percentage: f32,
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
}
