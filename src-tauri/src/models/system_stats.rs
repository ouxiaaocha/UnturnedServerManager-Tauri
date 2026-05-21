use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub memory_percentage: f32,
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
}
