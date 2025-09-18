import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { rpcClient } from '../api/rpc-client';
import {
  BarChart3, TrendingUp, TrendingDown, Activity,
  Cpu, HardDrive, Network, Clock, RefreshCw
} from 'lucide-react';
import {
  LineChart, Line, BarChart, Bar, PieChart, Pie, Cell,
  XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer
} from 'recharts';

function MetricsPage() {
  const [timeRange, setTimeRange] = useState('1h');

  const { data: metrics, refetch } = useQuery({
    queryKey: ['metrics-summary'],
    queryFn: () => rpcClient.getMetricsSummary(),
    refetchInterval: 5000
  });

  const { data: clients } = useQuery({
    queryKey: ['clients'],
    queryFn: () => rpcClient.listClients()
  });

  // Mock data for charts
  const cpuData = [
    { time: '00:00', value: 45 },
    { time: '00:10', value: 52 },
    { time: '00:20', value: 48 },
    { time: '00:30', value: 65 },
    { time: '00:40', value: 58 },
    { time: '00:50', value: 62 },
  ];

  const memoryData = [
    { time: '00:00', value: 60 },
    { time: '00:10', value: 62 },
    { time: '00:20', value: 65 },
    { time: '00:30', value: 68 },
    { time: '00:40', value: 64 },
    { time: '00:50', value: 66 },
  ];

  const bandwidthData = [
    { time: '00:00', rx: 1024, tx: 512 },
    { time: '00:10', rx: 1536, tx: 768 },
    { time: '00:20', rx: 2048, tx: 1024 },
    { time: '00:30', rx: 1792, tx: 896 },
    { time: '00:40', rx: 2304, tx: 1152 },
    { time: '00:50', rx: 2560, tx: 1280 },
  ];

  const clientDistribution = [
    { name: 'Linux', value: 45, color: '#8b5cf6' },
    { name: 'Windows', value: 30, color: '#3b82f6' },
    { name: 'macOS', value: 20, color: '#10b981' },
    { name: 'Other', value: 5, color: '#64748b' },
  ];

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold">System Metrics</h1>
          <p className="text-gray-400 mt-2">Real-time performance monitoring</p>
        </div>
        <div className="flex gap-3">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:outline-none focus:ring-2 focus:ring-purple-500"
          >
            <option value="1h">Last Hour</option>
            <option value="6h">Last 6 Hours</option>
            <option value="24h">Last 24 Hours</option>
            <option value="7d">Last 7 Days</option>
          </select>
          <button
            onClick={() => refetch()}
            className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh
          </button>
        </div>
      </div>

      {/* Key Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">Total Clients</span>
            <Users className="w-4 h-4 text-purple-400" />
          </div>
          <p className="text-2xl font-bold">{metrics?.total_clients || 0}</p>
          <div className="flex items-center gap-1 mt-2">
            <TrendingUp className="w-4 h-4 text-green-400" />
            <span className="text-sm text-green-400">+12%</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">Avg CPU</span>
            <Cpu className="w-4 h-4 text-blue-400" />
          </div>
          <p className="text-2xl font-bold">{(metrics?.total_cpu_usage || 0).toFixed(1)}%</p>
          <div className="flex items-center gap-1 mt-2">
            <TrendingDown className="w-4 h-4 text-red-400" />
            <span className="text-sm text-red-400">-5%</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">Avg Memory</span>
            <HardDrive className="w-4 h-4 text-green-400" />
          </div>
          <p className="text-2xl font-bold">{(metrics?.total_memory_usage || 0).toFixed(1)}%</p>
          <div className="flex items-center gap-1 mt-2">
            <TrendingUp className="w-4 h-4 text-green-400" />
            <span className="text-sm text-green-400">+3%</span>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-400 text-sm">Total Bandwidth</span>
            <Network className="w-4 h-4 text-yellow-400" />
          </div>
          <p className="text-2xl font-bold">
            {formatBytes((metrics?.total_bandwidth_rx || 0) + (metrics?.total_bandwidth_tx || 0))}
          </p>
          <div className="flex items-center gap-1 mt-2">
            <Activity className="w-4 h-4 text-yellow-400" />
            <span className="text-sm text-yellow-400">Active</span>
          </div>
        </div>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
        {/* CPU Usage Chart */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">CPU Usage</h3>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={cpuData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9ca3af" />
              <YAxis stroke="#9ca3af" />
              <Tooltip
                contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                labelStyle={{ color: '#9ca3af' }}
              />
              <Line type="monotone" dataKey="value" stroke="#8b5cf6" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* Memory Usage Chart */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">Memory Usage</h3>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={memoryData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9ca3af" />
              <YAxis stroke="#9ca3af" />
              <Tooltip
                contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                labelStyle={{ color: '#9ca3af' }}
              />
              <Line type="monotone" dataKey="value" stroke="#10b981" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* Bandwidth Chart */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">Network Bandwidth</h3>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={bandwidthData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9ca3af" />
              <YAxis stroke="#9ca3af" />
              <Tooltip
                contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                labelStyle={{ color: '#9ca3af' }}
              />
              <Legend />
              <Bar dataKey="rx" fill="#3b82f6" name="Received" />
              <Bar dataKey="tx" fill="#f59e0b" name="Transmitted" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Client Distribution */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">Client Distribution</h3>
          <ResponsiveContainer width="100%" height={250}>
            <PieChart>
              <Pie
                data={clientDistribution}
                cx="50%"
                cy="50%"
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
              >
                {clientDistribution.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Client Metrics Table */}
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h3 className="text-lg font-semibold mb-4">Client Performance</h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="text-left border-b border-gray-700">
                <th className="pb-3 text-gray-400">Client</th>
                <th className="pb-3 text-gray-400">CPU</th>
                <th className="pb-3 text-gray-400">Memory</th>
                <th className="pb-3 text-gray-400">Disk</th>
                <th className="pb-3 text-gray-400">Network</th>
                <th className="pb-3 text-gray-400">Status</th>
              </tr>
            </thead>
            <tbody>
              {clients?.map((client) => (
                <tr key={client.client_id} className="border-b border-gray-700/50">
                  <td className="py-3">
                    <div>
                      <p className="font-semibold">{client.hostname}</p>
                      <p className="text-sm text-gray-400">{client.ip_address}</p>
                    </div>
                  </td>
                  <td className="py-3">
                    <div className="flex items-center gap-2">
                      <div className="w-24 bg-gray-700 rounded-full h-2">
                        <div className="bg-blue-500 h-2 rounded-full" style={{ width: '45%' }}></div>
                      </div>
                      <span className="text-sm">45%</span>
                    </div>
                  </td>
                  <td className="py-3">
                    <div className="flex items-center gap-2">
                      <div className="w-24 bg-gray-700 rounded-full h-2">
                        <div className="bg-green-500 h-2 rounded-full" style={{ width: '62%' }}></div>
                      </div>
                      <span className="text-sm">62%</span>
                    </div>
                  </td>
                  <td className="py-3">
                    <div className="flex items-center gap-2">
                      <div className="w-24 bg-gray-700 rounded-full h-2">
                        <div className="bg-yellow-500 h-2 rounded-full" style={{ width: '78%' }}></div>
                      </div>
                      <span className="text-sm">78%</span>
                    </div>
                  </td>
                  <td className="py-3">
                    <span className="text-sm">2.4 MB/s</span>
                  </td>
                  <td className="py-3">
                    <span className="px-2 py-1 bg-green-900/40 text-green-400 rounded text-sm">
                      Healthy
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

export default MetricsPage;