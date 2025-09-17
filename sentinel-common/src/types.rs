use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub kernel_version: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub total_disk: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_usage: f32,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_usage: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub network_rx_rate: u64,
    pub network_tx_rate: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    UpdateIptables,
    ConfigureProxy,
    StartRelay,
    StopRelay,
    UpdateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptablesRule {
    pub action: Action,
    pub chain: String,
    pub protocol: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub dport: Option<u16>,
    pub sport: Option<u16>,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Insert,
    Append,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub target_addr: String,
    pub rate_limit: Option<u32>,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfig {
    pub entry_point: String,
    pub exit_point: String,
    pub transport_type: TransportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportType {
    Direct,
    Encrypted,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientStatus {
    Online,
    Offline,
    Error(String),
}