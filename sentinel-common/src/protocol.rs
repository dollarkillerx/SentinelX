use serde::{Deserialize, Serialize};
use crate::types::{ClientInfo, SystemMetrics, Task};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub client_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub client_id: String,
    pub token: String,
    pub metrics: Option<SystemMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_clients: u32,
    pub online_clients: u32,
    pub total_cpu_usage: f32,
    pub total_memory_usage: f32,
    pub total_bandwidth_rx: u64,
    pub total_bandwidth_tx: u64,
}