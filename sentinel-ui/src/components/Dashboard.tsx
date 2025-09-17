import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { ClientInfo, MetricsSummary } from '@/types'
import { api } from '@/lib/api'
import { formatBytes, formatBitrate } from '@/lib/utils'
import { Activity, Server, Network, HardDrive, Cpu, MemoryStick } from 'lucide-react'

export function Dashboard() {
  const [clients, setClients] = useState<ClientInfo[]>([])
  const [summary, setSummary] = useState<MetricsSummary | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchData = async () => {
    try {
      setLoading(true)
      const [clientsData, summaryData] = await Promise.all([
        api.listClients(),
        api.getMetricsSummary()
      ])
      setClients(clientsData)
      setSummary(summaryData)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch data')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 10000) // Refresh every 10 seconds
    return () => clearInterval(interval)
  }, [])

  const getStatusColor = (hostname: string) => {
    // Simple logic - in reality this would come from actual client status
    return 'bg-green-500'
  }

  if (loading && !clients.length) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <Activity className="mx-auto h-8 w-8 animate-spin" />
          <p className="mt-2 text-muted-foreground">Loading dashboard...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <p className="text-destructive">Error: {error}</p>
          <Button onClick={fetchData} className="mt-4">
            Retry
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">SentinelX Dashboard</h1>
            <p className="text-muted-foreground">Monitor and manage your distributed traffic system</p>
          </div>
          <Button onClick={fetchData} variant="outline">
            <Activity className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>

        {/* Summary Cards */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-6">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Clients</CardTitle>
              <Server className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{summary?.total_clients || 0}</div>
              <p className="text-xs text-muted-foreground">
                {summary?.online_clients || 0} online
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
              <Cpu className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {summary?.total_cpu_usage?.toFixed(1) || '0.0'}%
              </div>
              <p className="text-xs text-muted-foreground">
                Average across all clients
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
              <MemoryStick className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {summary?.total_memory_usage?.toFixed(1) || '0.0'}%
              </div>
              <p className="text-xs text-muted-foreground">
                Average across all clients
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Network Traffic</CardTitle>
              <Network className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {formatBitrate((summary?.total_bandwidth_rx || 0) + (summary?.total_bandwidth_tx || 0))}
              </div>
              <p className="text-xs text-muted-foreground">
                Combined RX/TX rate
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Clients Table */}
        <Card>
          <CardHeader>
            <CardTitle>Connected Clients</CardTitle>
            <CardDescription>
              Real-time status and metrics for all registered clients
            </CardDescription>
          </CardHeader>
          <CardContent>
            {clients.length === 0 ? (
              <div className="text-center py-8">
                <Server className="mx-auto h-12 w-12 text-muted-foreground" />
                <p className="mt-2 text-muted-foreground">No clients connected</p>
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Hostname</TableHead>
                    <TableHead>IP Address</TableHead>
                    <TableHead>Version</TableHead>
                    <TableHead>OS</TableHead>
                    <TableHead>CPU Cores</TableHead>
                    <TableHead>Memory</TableHead>
                    <TableHead>Capabilities</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {clients.map((client) => (
                    <TableRow key={client.id}>
                      <TableCell>
                        <div className={`w-3 h-3 rounded-full ${getStatusColor(client.hostname)}`} />
                      </TableCell>
                      <TableCell className="font-medium">{client.hostname}</TableCell>
                      <TableCell>{client.ip}</TableCell>
                      <TableCell>{client.version}</TableCell>
                      <TableCell>{client.system_info.os}</TableCell>
                      <TableCell>{client.system_info.cpu_cores}</TableCell>
                      <TableCell>{formatBytes(client.system_info.total_memory)}</TableCell>
                      <TableCell>
                        <div className="flex flex-wrap gap-1">
                          {client.capabilities.map((cap) => (
                            <span
                              key={cap}
                              className="inline-flex items-center rounded-md bg-primary/10 px-2 py-1 text-xs font-medium text-primary"
                            >
                              {cap}
                            </span>
                          ))}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}