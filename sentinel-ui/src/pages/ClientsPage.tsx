import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { rpcClient, type ClientInfo } from '../api/rpc-client';
import {
  Server, Circle, RefreshCw, Trash2, MoreVertical,
  Cpu, HardDrive, Network, ChevronDown, ChevronUp
} from 'lucide-react';

function ClientsPage() {
  const [expandedClients, setExpandedClients] = useState<Set<string>>(new Set());

  const { data: clients, isLoading, refetch } = useQuery({
    queryKey: ['clients'],
    queryFn: () => rpcClient.listClients(),
    refetchInterval: 10000
  });

  const toggleExpand = (clientId: string) => {
    const newExpanded = new Set(expandedClients);
    if (newExpanded.has(clientId)) {
      newExpanded.delete(clientId);
    } else {
      newExpanded.add(clientId);
    }
    setExpandedClients(newExpanded);
  };

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold">Client Management</h1>
          <p className="text-gray-400 mt-2">Monitor and manage connected clients</p>
        </div>
        <button
          onClick={() => refetch()}
          className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          Refresh
        </button>
      </div>

      {isLoading && (
        <div className="flex justify-center py-12">
          <RefreshCw className="w-8 h-8 animate-spin text-purple-500" />
        </div>
      )}

      <div className="grid gap-4">
        {clients?.map((client) => (
          <div key={client.client_id} className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
            <div className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="p-3 bg-purple-900/40 rounded-lg">
                    <Server className="w-6 h-6 text-purple-400" />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold">{client.hostname}</h3>
                    <div className="flex items-center gap-4 mt-1">
                      <span className="text-sm text-gray-400">{client.ip_address}</span>
                      <span className="text-sm text-gray-400">{client.os} / {client.arch}</span>
                      <div className="flex items-center gap-1">
                        <Circle className="w-2 h-2 fill-green-500 text-green-500" />
                        <span className="text-sm text-green-500">Online</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  <button
                    onClick={() => toggleExpand(client.client_id)}
                    className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
                  >
                    {expandedClients.has(client.client_id) ? (
                      <ChevronUp className="w-5 h-5" />
                    ) : (
                      <ChevronDown className="w-5 h-5" />
                    )}
                  </button>
                  <button className="p-2 hover:bg-gray-700 rounded-lg transition-colors">
                    <MoreVertical className="w-5 h-5" />
                  </button>
                </div>
              </div>

              {expandedClients.has(client.client_id) && (
                <div className="mt-6 pt-6 border-t border-gray-700">
                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                    <div>
                      <p className="text-sm text-gray-400 mb-1">Client ID</p>
                      <p className="font-mono text-sm">{client.client_id}</p>
                    </div>
                    <div>
                      <p className="text-sm text-gray-400 mb-1">Version</p>
                      <p>{client.version}</p>
                    </div>
                    <div>
                      <p className="text-sm text-gray-400 mb-1">Operating System</p>
                      <p>{client.os}</p>
                    </div>
                    <div>
                      <p className="text-sm text-gray-400 mb-1">Architecture</p>
                      <p>{client.arch}</p>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 mt-6">
                    <div className="bg-gray-700/50 rounded-lg p-4">
                      <div className="flex items-center gap-2 mb-2">
                        <Cpu className="w-4 h-4 text-blue-400" />
                        <span className="text-sm text-gray-400">CPU Usage</span>
                      </div>
                      <p className="text-2xl font-bold">0.0%</p>
                    </div>
                    <div className="bg-gray-700/50 rounded-lg p-4">
                      <div className="flex items-center gap-2 mb-2">
                        <HardDrive className="w-4 h-4 text-green-400" />
                        <span className="text-sm text-gray-400">Memory Usage</span>
                      </div>
                      <p className="text-2xl font-bold">0.0%</p>
                    </div>
                    <div className="bg-gray-700/50 rounded-lg p-4">
                      <div className="flex items-center gap-2 mb-2">
                        <Network className="w-4 h-4 text-purple-400" />
                        <span className="text-sm text-gray-400">Bandwidth</span>
                      </div>
                      <p className="text-2xl font-bold">0 MB/s</p>
                    </div>
                  </div>

                  <div className="flex gap-2 mt-6">
                    <button className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors">
                      View Metrics
                    </button>
                    <button className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors">
                      Configure
                    </button>
                    <button className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors">
                      Disconnect
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        ))}

        {(!clients || clients.length === 0) && !isLoading && (
          <div className="bg-gray-800 rounded-lg border border-gray-700 p-12 text-center">
            <Server className="w-12 h-12 text-gray-600 mx-auto mb-4" />
            <p className="text-gray-400">No clients connected</p>
            <p className="text-sm text-gray-500 mt-2">
              Clients will appear here when they connect to the server
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

export default ClientsPage;