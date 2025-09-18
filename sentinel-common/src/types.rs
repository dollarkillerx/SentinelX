use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Client information structure containing identification and capability details
/// Used during client registration and heartbeat communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Unique client identifier (UUID)
    pub id: String,
    /// Client hostname for identification
    pub hostname: String,
    /// Client IP address
    pub ip: String,
    /// Client software version
    pub version: String,
    /// List of client capabilities (proxy, iptables, relay, monitoring)
    pub capabilities: Vec<String>,
    /// Static system information collected at startup
    pub system_info: SystemInfo,
}

/// Static system information collected once during client startup
/// Contains hardware and OS details that don't change frequently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system name and version
    pub os: String,
    /// Kernel version string
    pub kernel_version: String,
    /// Number of CPU cores available
    pub cpu_cores: usize,
    /// Total system memory in bytes
    pub total_memory: u64,
    /// Total disk space in bytes
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