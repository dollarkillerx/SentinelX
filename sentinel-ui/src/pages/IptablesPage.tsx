import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { rpcClient, IptablesRule } from '../api/rpc-client';
import {
  Shield, Plus, X, Filter, AlertTriangle,
  CheckCircle, XCircle, Globe, Wifi
} from 'lucide-react';

function IptablesPage() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedClient, setSelectedClient] = useState('');
  const [rules, setRules] = useState<IptablesRule[]>([]);
  const [newRule, setNewRule] = useState<IptablesRule>({
    action: 'ACCEPT',
    protocol: 'tcp',
    source: '',
    destination: '',
    port: undefined,
    interface: '',
    comment: ''
  });

  const { data: clients } = useQuery({
    queryKey: ['clients'],
    queryFn: () => rpcClient.listClients()
  });

  const handleAddRule = () => {
    const rule: IptablesRule = {
      ...newRule,
      port: newRule.port ? parseInt(newRule.port.toString()) : undefined
    };
    setRules([...rules, rule]);
    setNewRule({
      action: 'ACCEPT',
      protocol: 'tcp',
      source: '',
      destination: '',
      port: undefined,
      interface: '',
      comment: ''
    });
    setShowCreateModal(false);
  };

  const handleApplyRules = async () => {
    if (!selectedClient || rules.length === 0) return;

    try {
      await rpcClient.updateIptables(selectedClient, rules);
      alert('Rules applied successfully!');
    } catch (error) {
      console.error('Failed to apply rules:', error);
      alert('Failed to apply rules');
    }
  };

  const handleRemoveRule = (index: number) => {
    setRules(rules.filter((_, i) => i !== index));
  };

  const getActionColor = (action: string) => {
    switch (action) {
      case 'ACCEPT': return 'text-green-400 bg-green-900/40';
      case 'DROP': return 'text-red-400 bg-red-900/40';
      case 'REJECT': return 'text-yellow-400 bg-yellow-900/40';
      default: return 'text-gray-400 bg-gray-700';
    }
  };

  const getProtocolIcon = (protocol: string) => {
    switch (protocol) {
      case 'tcp': return <Globe className="w-4 h-4" />;
      case 'udp': return <Wifi className="w-4 h-4" />;
      default: return <Filter className="w-4 h-4" />;
    }
  };

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold">IPTables Management</h1>
          <p className="text-gray-400 mt-2">Configure firewall rules for clients</p>
        </div>
        <div className="flex gap-3">
          <select
            value={selectedClient}
            onChange={(e) => setSelectedClient(e.target.value)}
            className="px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:outline-none focus:ring-2 focus:ring-purple-500"
          >
            <option value="">Select client...</option>
            {clients?.map(client => (
              <option key={client.client_id} value={client.client_id}>
                {client.hostname} ({client.ip_address})
              </option>
            ))}
          </select>
          <button
            onClick={() => setShowCreateModal(true)}
            disabled={!selectedClient}
            className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
            Add Rule
          </button>
        </div>
      </div>

      {selectedClient && (
        <div className="bg-gray-800 rounded-lg border border-gray-700 p-6 mb-6">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-lg font-semibold">Current Rules</h2>
            {rules.length > 0 && (
              <button
                onClick={handleApplyRules}
                className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
              >
                Apply Rules
              </button>
            )}
          </div>

          <div className="space-y-3">
            {rules.map((rule, index) => (
              <div key={index} className="bg-gray-700/50 rounded-lg p-4 flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <span className={`px-3 py-1 rounded-lg text-sm font-semibold ${getActionColor(rule.action)}`}>
                    {rule.action}
                  </span>
                  <div className="flex items-center gap-2">
                    {getProtocolIcon(rule.protocol)}
                    <span className="text-sm">{rule.protocol.toUpperCase()}</span>
                  </div>
                  {rule.source && (
                    <div className="text-sm">
                      <span className="text-gray-400">From:</span> {rule.source}
                    </div>
                  )}
                  {rule.destination && (
                    <div className="text-sm">
                      <span className="text-gray-400">To:</span> {rule.destination}
                    </div>
                  )}
                  {rule.port && (
                    <div className="text-sm">
                      <span className="text-gray-400">Port:</span> {rule.port}
                    </div>
                  )}
                  {rule.comment && (
                    <div className="text-sm text-gray-500">
                      ({rule.comment})
                    </div>
                  )}
                </div>
                <button
                  onClick={() => handleRemoveRule(index)}
                  className="p-2 hover:bg-red-900/40 text-red-400 rounded-lg transition-colors"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            ))}

            {rules.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                No rules configured. Add a rule to get started.
              </div>
            )}
          </div>
        </div>
      )}

      {!selectedClient && (
        <div className="bg-gray-800 rounded-lg border border-gray-700 p-12 text-center">
          <Shield className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <p className="text-gray-400">Select a client to manage IPTables rules</p>
        </div>
      )}

      {/* Create Rule Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md">
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-semibold">Add IPTables Rule</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 hover:bg-gray-700 rounded-lg transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Action</label>
                <select
                  value={newRule.action}
                  onChange={(e) => setNewRule({ ...newRule, action: e.target.value as any })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  <option value="ACCEPT">ACCEPT</option>
                  <option value="DROP">DROP</option>
                  <option value="REJECT">REJECT</option>
                </select>
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Protocol</label>
                <select
                  value={newRule.protocol}
                  onChange={(e) => setNewRule({ ...newRule, protocol: e.target.value as any })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  <option value="tcp">TCP</option>
                  <option value="udp">UDP</option>
                  <option value="icmp">ICMP</option>
                  <option value="all">ALL</option>
                </select>
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Source IP (Optional)</label>
                <input
                  type="text"
                  placeholder="e.g., 192.168.1.0/24"
                  value={newRule.source || ''}
                  onChange={(e) => setNewRule({ ...newRule, source: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Destination IP (Optional)</label>
                <input
                  type="text"
                  placeholder="e.g., 10.0.0.0/8"
                  value={newRule.destination || ''}
                  onChange={(e) => setNewRule({ ...newRule, destination: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Port (Optional)</label>
                <input
                  type="number"
                  placeholder="e.g., 443"
                  value={newRule.port || ''}
                  onChange={(e) => setNewRule({ ...newRule, port: e.target.value ? parseInt(e.target.value) : undefined })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Interface (Optional)</label>
                <input
                  type="text"
                  placeholder="e.g., eth0"
                  value={newRule.interface || ''}
                  onChange={(e) => setNewRule({ ...newRule, interface: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Comment (Optional)</label>
                <input
                  type="text"
                  placeholder="Rule description"
                  value={newRule.comment || ''}
                  onChange={(e) => setNewRule({ ...newRule, comment: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                />
              </div>
            </div>

            <div className="flex gap-3 mt-6">
              <button
                onClick={handleAddRule}
                className="flex-1 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors"
              >
                Add Rule
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

export default IptablesPage;