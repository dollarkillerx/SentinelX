import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { rpcClient } from '../api/rpc-client';
import {
  Users, Cpu, HardDrive, Activity, Network, Clock
} from 'lucide-react';

function Dashboard() {
  const { data: metrics, isLoading, error } = useQuery({
    queryKey: ['metrics-summary'],
    queryFn: () => rpcClient.getMetricsSummary(),
    refetchInterval: 5000
  });

  const { data: clients } = useQuery({
    queryKey: ['clients'],
    queryFn: () => rpcClient.listClients()
  });

  const statCards = [
    {
      title: 'Total Clients',
      value: metrics?.total_clients || 0,
      icon: Users,
      color: 'bg-purple-600'
    },
    {
      title: 'Online Clients',
      value: metrics?.online_clients || 0,
      icon: Activity,
      color: 'bg-green-600'
    },
    {
      title: 'CPU Usage',
      value: `${(metrics?.total_cpu_usage || 0).toFixed(1)}%`,
      icon: Cpu,
      color: 'bg-blue-600'
    },
    {
      title: 'Memory Usage',
      value: `${(metrics?.total_memory_usage || 0).toFixed(1)}%`,
      icon: HardDrive,
      color: 'bg-yellow-600'
    },
    {
      title: 'Bandwidth In',
      value: formatBytes(metrics?.total_bandwidth_rx || 0),
      icon: Network,
      color: 'bg-indigo-600'
    },
    {
      title: 'Bandwidth Out',
      value: formatBytes(metrics?.total_bandwidth_tx || 0),
      icon: Network,
      color: 'bg-pink-600'
    }
  ];

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-gray-400 mt-2">System Overview</p>
      </div>

      {error && (
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-4 mb-6 text-red-400">
          Error loading metrics: {error.message}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
        {statCards.map((card, idx) => (
          <div key={idx} className="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <div className="flex items-center justify-between mb-4">
              <div className={`p-3 rounded-lg ${card.color}`}>
                <card.icon className="w-6 h-6" />
              </div>
              <Clock className="w-4 h-4 text-gray-500" />
            </div>
            <div>
              <p className="text-gray-400 text-sm mb-1">{card.title}</p>
              <p className="text-2xl font-bold">{card.value}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h2 className="text-xl font-semibold mb-4">Recent Clients</h2>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="text-left border-b border-gray-700">
                <th className="pb-3 text-gray-400">Client ID</th>
                <th className="pb-3 text-gray-400">Hostname</th>
                <th className="pb-3 text-gray-400">IP Address</th>
                <th className="pb-3 text-gray-400">OS</th>
                <th className="pb-3 text-gray-400">Status</th>
              </tr>
            </thead>
            <tbody>
              {clients?.slice(0, 5).map((client) => (
                <tr key={client.client_id} className="border-b border-gray-700/50">
                  <td className="py-3 font-mono text-sm">{client.client_id}</td>
                  <td className="py-3">{client.hostname}</td>
                  <td className="py-3">{client.ip_address}</td>
                  <td className="py-3">{client.os}</td>
                  <td className="py-3">
                    <span className="px-2 py-1 bg-green-900/40 text-green-400 rounded text-sm">
                      Online
                    </span>
                  </td>
                </tr>
              ))}
              {(!clients || clients.length === 0) && (
                <tr>
                  <td colSpan={5} className="py-8 text-center text-gray-500">
                    No clients connected
                  </td>
                </tr>
              )}
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

export default Dashboard;