import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { rpcClient, RelayConfig } from '../api/rpc-client';
import {
  Network, Plus, X, ArrowRight, Globe, Shield,
  Activity, Play, Pause, Trash2
} from 'lucide-react';

function RelaysPage() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newRelay, setNewRelay] = useState<{
    entryClientId: string;
    exitClientId: string;
    entryPoint: string;
    exitPoint: string;
    transportType: 'Direct' | 'Snowflake' | 'WebRTC';
  }>({
    entryClientId: '',
    exitClientId: '',
    entryPoint: '',
    exitPoint: '',
    transportType: 'Direct'
  });

  const { data: clients } = useQuery({
    queryKey: ['clients'],
    queryFn: () => rpcClient.listClients()
  });

  const [activeRelays, setActiveRelays] = useState<any[]>([]);

  const handleCreateRelay = async () => {
    try {
      await rpcClient.startRelay(
        newRelay.entryClientId,
        newRelay.exitClientId,
        {
          entry_point: newRelay.entryPoint,
          exit_point: newRelay.exitPoint,
          transport_type: newRelay.transportType
        }
      );

      setActiveRelays([...activeRelays, {
        id: Date.now(),
        ...newRelay,
        status: 'active',
        createdAt: new Date().toISOString()
      }]);

      setShowCreateModal(false);
      setNewRelay({
        entryClientId: '',
        exitClientId: '',
        entryPoint: '',
        exitPoint: '',
        transportType: 'Direct'
      });
    } catch (error) {
      console.error('Failed to create relay:', error);
    }
  };

  const handleStopRelay = async (relay: any) => {
    try {
      await rpcClient.stopRelay(
        relay.entryClientId,
        relay.entryPoint,
        relay.exitPoint
      );
      setActiveRelays(activeRelays.filter(r => r.id !== relay.id));
    } catch (error) {
      console.error('Failed to stop relay:', error);
    }
  };

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold">Relay Routes</h1>
          <p className="text-gray-400 mt-2">Configure relay routing between clients</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          Create Relay
        </button>
      </div>

      <div className="grid gap-4">
        {activeRelays.map((relay) => (
          <div key={relay.id} className="bg-gray-800 rounded-lg border border-gray-700 p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="p-3 bg-purple-900/40 rounded-lg">
                  <Network className="w-6 h-6 text-purple-400" />
                </div>
                <div className="flex items-center gap-4">
                  <div>
                    <p className="text-sm text-gray-400">Entry</p>
                    <p className="font-semibold">{relay.entryPoint}</p>
                    <p className="text-xs text-gray-500">{relay.entryClientId}</p>
                  </div>
                  <ArrowRight className="w-5 h-5 text-gray-500" />
                  <div>
                    <p className="text-sm text-gray-400">Exit</p>
                    <p className="font-semibold">{relay.exitPoint}</p>
                    <p className="text-xs text-gray-500">{relay.exitClientId}</p>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-4">
                <div className="text-right">
                  <p className="text-sm text-gray-400">Transport</p>
                  <p className="font-semibold">{relay.transportType}</p>
                </div>
                <div className="flex items-center gap-2">
                  <div className="flex items-center gap-1">
                    <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                    <span className="text-sm text-green-500">Active</span>
                  </div>
                  <button
                    onClick={() => handleStopRelay(relay)}
                    className="p-2 hover:bg-red-900/40 text-red-400 rounded-lg transition-colors"
                  >
                    <Pause className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        ))}

        {activeRelays.length === 0 && (
          <div className="bg-gray-800 rounded-lg border border-gray-700 p-12 text-center">
            <Network className="w-12 h-12 text-gray-600 mx-auto mb-4" />
            <p className="text-gray-400">No active relay routes</p>
            <p className="text-sm text-gray-500 mt-2">
              Create a relay route to forward traffic between clients
            </p>
          </div>
        )}
      </div>

      {/* Create Relay Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md">
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-semibold">Create Relay Route</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 hover:bg-gray-700 rounded-lg transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Entry Client</label>
                <select
                  value={newRelay.entryClientId}
                  onChange={(e) => setNewRelay({ ...newRelay, entryClientId: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  <option value="">Select client...</option>
                  {clients?.map(client => (
                    <option key={client.client_id} value={client.client_id}>
                      {client.hostname} ({client.ip_address})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Entry Point</label>
                <input
                  type="text"
                  placeholder="e.g., 0.0.0.0:8080"
                  value={newRelay.entryPoint}
                  onChange={(e) => setNewRelay({ ...newRelay, entryPoint: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Exit Client</label>
                <select
                  value={newRelay.exitClientId}
                  onChange={(e) => setNewRelay({ ...newRelay, exitClientId: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  <option value="">Select client...</option>
                  {clients?.map(client => (
                    <option key={client.client_id} value={client.client_id}>
                      {client.hostname} ({client.ip_address})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Exit Point</label>
                <input
                  type="text"
                  placeholder="e.g., target.com:443"
                  value={newRelay.exitPoint}
                  onChange={(e) => setNewRelay({ ...newRelay, exitPoint: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Transport Type</label>
                <select
                  value={newRelay.transportType}
                  onChange={(e) => setNewRelay({ ...newRelay, transportType: e.target.value as any })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  <option value="Direct">Direct</option>
                  <option value="Snowflake">Snowflake</option>
                  <option value="WebRTC">WebRTC</option>
                </select>
              </div>
            </div>

            <div className="flex gap-3 mt-6">
              <button
                onClick={handleCreateRelay}
                className="flex-1 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors"
              >
                Create Relay
              </button>
              <button
                onClick={() => setShowCreateModal(false)}
                className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default RelaysPage;