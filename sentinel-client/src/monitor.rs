use anyhow::Result;
use sentinel_common::{SystemInfo, SystemMetrics};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, Networks, Disks};
use tokio::sync::RwLock;

pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    networks: Arc<RwLock<Networks>>,
    disks: Arc<RwLock<Disks>>,
    network_history: Arc<RwLock<NetworkHistory>>,
}

struct NetworkHistory {
    samples: VecDeque<NetworkSample>,
    max_samples: usize,
}

#[derive(Clone)]
struct NetworkSample {
    rx_bytes: u64,
    tx_bytes: u64,
    timestamp: Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        Self {
            system: Arc::new(RwLock::new(system)),
            networks: Arc::new(RwLock::new(networks)),
            disks: Arc::new(RwLock::new(disks)),
            network_history: Arc::new(RwLock::new(NetworkHistory {
                samples: VecDeque::with_capacity(60),
                max_samples: 60,
            })),
        }
    }

    pub async fn collect_metrics() -> Result<SystemMetrics> {
        let monitor = Self::new();

        // Refresh system info
        {
            let mut system = monitor.system.write().await;
            system.refresh_all();
            tokio::time::sleep(Duration::from_millis(200)).await;
            system.refresh_cpu_all();
        }

        // Refresh network and disk info
        {
            let mut networks = monitor.networks.write().await;
            networks.refresh();
        }
        {
            let mut disks = monitor.disks.write().await;
            disks.refresh();
        }

        // Collect CPU and memory metrics
        let (cpu_usage, memory_used, memory_total, memory_usage) = {
            let system = monitor.system.read().await;
            let cpu_usage = system.global_cpu_usage();
            let memory_used = system.used_memory();
            let memory_total = system.total_memory();
            let memory_usage = (memory_used as f32 / memory_total as f32) * 100.0;
            (cpu_usage, memory_used, memory_total, memory_usage)
        };

        // Collect disk metrics
        let (disk_used, disk_total, disk_usage) = {
            let disks = monitor.disks.read().await;
            let mut total_space = 0u64;
            let mut used_space = 0u64;

            for disk in disks.iter() {
                total_space += disk.total_space();
                used_space += disk.total_space() - disk.available_space();
            }

            let usage = if total_space > 0 {
                (used_space as f32 / total_space as f32) * 100.0
            } else {
                0.0
            };

            (used_space, total_space, usage)
        };

        // Collect network metrics
        let (rx_bytes, tx_bytes) = {
            let networks = monitor.networks.read().await;
            let mut total_rx = 0u64;
            let mut total_tx = 0u64;

            for (interface_name, network) in networks.iter() {
                // Skip loopback interfaces
                if !interface_name.starts_with("lo") {
                    total_rx += network.total_received();
                    total_tx += network.total_transmitted();
                }
            }

            (total_rx, total_tx)
        };

        // Update network history and calculate rates
        let (rx_rate, tx_rate) = {
            let mut history = monitor.network_history.write().await;
            let now = Instant::now();

            let new_sample = NetworkSample {
                rx_bytes,
                tx_bytes,
                timestamp: now,
            };

            let rates = if let Some(last_sample) = history.samples.back() {
                let duration = now.duration_since(last_sample.timestamp);
                if duration.as_secs() > 0 {
                    let rx_rate = (rx_bytes.saturating_sub(last_sample.rx_bytes)) / duration.as_secs();
                    let tx_rate = (tx_bytes.saturating_sub(last_sample.tx_bytes)) / duration.as_secs();
                    (rx_rate, tx_rate)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            history.samples.push_back(new_sample);
            if history.samples.len() > history.max_samples {
                history.samples.pop_front();
            }

            rates
        };

        Ok(SystemMetrics {
            cpu_usage,
            memory_used,
            memory_total,
            memory_usage,
            disk_used,
            disk_total,
            disk_usage,
            network_rx_bytes: rx_bytes,
            network_tx_bytes: tx_bytes,
            network_rx_rate: rx_rate,
            network_tx_rate: tx_rate,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

}

pub struct MetricsReporter {
    #[allow(dead_code)]
    monitor: Arc<SystemMonitor>,
    #[allow(dead_code)]
    server_url: String,
    interval: Duration,
}

impl MetricsReporter {
    pub fn new(server_url: String, interval: Duration) -> Self {
        Self {
            monitor: Arc::new(SystemMonitor::new()),
            server_url,
            interval,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut ticker = tokio::time::interval(self.interval);

        loop {
            ticker.tick().await;

            match SystemMonitor::collect_metrics().await {
                Ok(metrics) => {
                    if let Err(e) = self.report_metrics(metrics).await {
                        tracing::error!("Failed to report metrics: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to collect metrics: {}", e);
                }
            }
        }
    }

    async fn report_metrics(&self, metrics: SystemMetrics) -> Result<()> {
        tracing::debug!("Reporting metrics: {:?}", metrics);
        Ok(())
    }
}

pub fn get_system_info() -> SystemInfo {
    let mut system = System::new_all();
    system.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let total_disk = disks.iter().map(|disk| disk.total_space()).sum();

    SystemInfo {
        os: System::name().unwrap_or_else(|| "Unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        cpu_cores: system.cpus().len(),
        total_memory: system.total_memory(),
        total_disk,
    }
}