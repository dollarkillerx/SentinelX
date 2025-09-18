import axios from 'axios';

const API_URL = 'http://localhost:8080/rpc';

let requestId = 1;

export interface ClientInfo {
  client_id: string;
  hostname: string;
  ip_address: string;
  os: string;
  arch: string;
  version: string;
}

export interface ClientMetrics {
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  active_connections: number;
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
  transport_type: 'Direct' | 'Snowflake' | 'WebRTC';
}

export interface IptablesRule {
  action: 'ACCEPT' | 'DROP' | 'REJECT';
  protocol: 'tcp' | 'udp' | 'icmp' | 'all';
  source?: string;
  destination?: string;
  port?: number;
  interface?: string;
  comment?: string;
}

class RpcClient {
  private getAuthHeaders() {
    const token = localStorage.getItem('sentinel_token');
    return token ? { Authorization: `Bearer ${token}` } : {};
  }

  private async call<T = any>(method: string, params?: any): Promise<T> {
    try {
      const response = await axios.post(API_URL, {
        jsonrpc: '2.0',
        id: requestId++,
        method,
        params: params || {}
      }, {
        headers: {
          'Content-Type': 'application/json',
          ...this.getAuthHeaders()
        }
      });

      if (response.data.error) {
        // Handle authentication errors
        if (response.data.error.code === -32001 || response.data.error.message?.includes('unauthorized')) {
          localStorage.removeItem('sentinel_token');
          localStorage.removeItem('sentinel_user');
          window.location.reload();
        }
        throw new Error(response.data.error.message);
      }

      return response.data.result;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        if (error.response?.status === 401) {
          localStorage.removeItem('sentinel_token');
          localStorage.removeItem('sentinel_user');
          window.location.reload();
        }
        throw new Error(`API Error: ${error.response?.data?.message || error.message}`);
      }
      throw error;
    }
  }

  async registerClient(clientInfo: ClientInfo) {
    return this.call('client.register', { client_info: clientInfo });
  }

  async listClients(): Promise<ClientInfo[]> {
    return this.call('client.list');
  }

  async getMetricsSummary(): Promise<MetricsSummary> {
    return this.call('metrics.get_summary');
  }

  async startRelay(entryClientId: string, exitClientId: string, config: RelayConfig) {
    return this.call('relay.start', {
      entry_client_id: entryClientId,
      exit_client_id: exitClientId,
      entry_point: config.entry_point,
      exit_point: config.exit_point,
      transport_type: config.transport_type
    });
  }

  async stopRelay(clientId: string, entryPoint: string, exitPoint: string) {
    return this.call('relay.stop', {
      client_id: clientId,
      entry_point: entryPoint,
      exit_point: exitPoint
    });
  }

  async updateIptables(clientId: string, rules: IptablesRule[]) {
    return this.call('iptables.update', {
      client_id: clientId,
      rules
    });
  }

  async applyIptablesRule(clientId: string, rule: IptablesRule) {
    return this.call('iptables.apply_rule', {
      client_id: clientId,
      rule
    });
  }
}

export const rpcClient = new RpcClient();