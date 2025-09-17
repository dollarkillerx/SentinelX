import { ClientInfo, MetricsSummary, StartRelayRequest, StopRelayRequest } from '@/types'

class JSONRPCClient {
  private baseUrl: string
  private id: number = 1

  constructor(baseUrl: string = '/api') {
    this.baseUrl = baseUrl
  }

  private async request<T>(method: string, params?: any): Promise<T> {
    const response = await fetch(this.baseUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method,
        params: params || [],
        id: this.id++,
      }),
    })

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }

    const data = await response.json()

    if (data.error) {
      throw new Error(data.error.message || 'RPC Error')
    }

    return data.result
  }

  async listClients(): Promise<ClientInfo[]> {
    return this.request<ClientInfo[]>('client.list')
  }

  async getMetricsSummary(): Promise<MetricsSummary> {
    return this.request<MetricsSummary>('metrics.get_summary')
  }

  async startRelay(request: StartRelayRequest): Promise<any> {
    return this.request('relay.start', request)
  }

  async stopRelay(request: StopRelayRequest): Promise<any> {
    return this.request('relay.stop', request)
  }
}

export const api = new JSONRPCClient()