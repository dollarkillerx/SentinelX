export interface ClientInfo {
  id: string;
  hostname: string;
  ip: string;
  version: string;
  capabilities: string[];
  system_info: SystemInfo;
}

export interface SystemInfo {
  os: string;
  kernel_version: string;
  cpu_cores: number;
  total_memory: number;
  total_disk: number;
}

export interface SystemMetrics {
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  memory_usage: number;
  disk_used: number;
  disk_total: number;
  disk_usage: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  network_rx_rate: number;
  network_tx_rate: number;
  timestamp: number;
}

export interface MetricsSummary {
  total_clients: number;
  online_clients: number;
  total_cpu_usage: number;
  total_memory_usage: number;
  total_bandwidth_rx: number;
  total_bandwidth_tx: number;
}

export interface RelayConfig {
  entry_point: string;
  exit_point: string;
  transport_type: 'Direct' | 'Encrypted' | 'WebSocket';
}

export interface StartRelayRequest {
  entry_client_id: string;
  exit_client_id: string;
  entry_point: string;
  exit_point: string;
  transport_type: 'Direct' | 'Encrypted' | 'WebSocket';
}

export interface StopRelayRequest {
  client_id: string;
  entry_point: string;
  exit_point: string;
}