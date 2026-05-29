use crate::models::system_stats::SystemStats;
use std::time::{Duration, Instant};
use sysinfo::{Networks, System};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

pub struct SystemMonitor {
    system: System,
    networks: Networks,
    initial_bytes_received: u64,
    initial_bytes_transmitted: u64,
    last_refresh: Option<Instant>,
    cached_stats: Option<SystemStats>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();

        let networks = Networks::new_with_refreshed_list();
        let (initial_received, initial_transmitted) = Self::sum_network_bytes(&networks);

        Self {
            system,
            networks,
            initial_bytes_received: initial_received,
            initial_bytes_transmitted: initial_transmitted,
            last_refresh: None,
            cached_stats: None,
        }
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.networks.refresh(false);
        self.last_refresh = Some(Instant::now());
        self.cached_stats = Some(self.compute_stats());
    }

    pub fn stats(&mut self) -> SystemStats {
        let should_refresh = self
            .last_refresh
            .map(|last| last.elapsed() >= SAMPLE_INTERVAL)
            .unwrap_or(true);

        if should_refresh {
            self.refresh();
        }

        self.cached_stats
            .clone()
            .unwrap_or_else(|| self.compute_stats())
    }

    fn compute_stats(&self) -> SystemStats {
        let cpu_usage = self.system.global_cpu_usage();
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_percentage = if total_memory > 0 {
            used_memory as f32 / total_memory as f32 * 100.0
        } else {
            0.0
        };

        let (total_received, total_transmitted) = Self::sum_network_bytes(&self.networks);

        SystemStats {
            cpu_usage,
            total_memory,
            used_memory,
            memory_percentage,
            bytes_received: total_received.saturating_sub(self.initial_bytes_received),
            bytes_transmitted: total_transmitted.saturating_sub(self.initial_bytes_transmitted),
        }
    }

    fn sum_network_bytes(networks: &Networks) -> (u64, u64) {
        let mut received = 0u64;
        let mut transmitted = 0u64;
        for (_name, data) in networks.iter() {
            received += data.total_received();
            transmitted += data.total_transmitted();
        }
        (received, transmitted)
    }
}
