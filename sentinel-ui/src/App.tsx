import React, { useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter as Router, Routes, Route, NavLink } from 'react-router-dom';
import {
  Shield, Server, Network, Filter, BarChart3,
  Settings, Home, LogOut, User
} from 'lucide-react';
import { AuthProvider, useAuth } from './context/AuthContext';
import ProtectedRoute from './components/ProtectedRoute';
import Dashboard from './pages/Dashboard';
import ClientsPage from './pages/ClientsPage';
import RelaysPage from './pages/RelaysPage';
import IptablesPage from './pages/IptablesPage';
import MetricsPage from './pages/MetricsPage';
import "./index.css";

const queryClient = new QueryClient();

function MainApp() {
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const { user, logout } = useAuth();

  const navItems = [
    { path: '/', icon: Home, label: 'Dashboard' },
    { path: '/clients', icon: Server, label: 'Clients' },
    { path: '/relays', icon: Network, label: 'Relay Routes', requiredRole: 'operator' as const },
    { path: '/iptables', icon: Filter, label: 'IPTables', requiredRole: 'operator' as const },
    { path: '/metrics', icon: BarChart3, label: 'Metrics' },
  ];

  const handleLogout = () => {
    logout();
  };

  return (
    <div className="flex h-screen bg-gray-900 text-white">
      {/* Sidebar */}
      <div className={`${isSidebarOpen ? 'w-64' : 'w-16'} bg-gray-800 transition-all duration-300 flex flex-col`}>
        <div className="p-4 border-b border-gray-700">
          <div className="flex items-center gap-3">
            <Shield className="w-8 h-8 text-purple-500" />
            {isSidebarOpen && (
              <div>
                <h1 className="text-xl font-bold">Sentinel</h1>
                <p className="text-xs text-gray-400">Control Panel</p>
              </div>
            )}
          </div>
        </div>

        <nav className="flex-1 p-4">
          <ul className="space-y-2">
            {navItems.map(({ path, icon: Icon, label, requiredRole }) => {
              // Role-based visibility
              if (requiredRole) {
                const roleHierarchy = { viewer: 1, operator: 2, admin: 3 };
                const userLevel = roleHierarchy[user?.role || 'viewer'];
                const requiredLevel = roleHierarchy[requiredRole];
                if (userLevel < requiredLevel) return null;
              }

              return (
                <li key={path}>
                  <NavLink
                    to={path}
                    className={({ isActive }) =>
                      `flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                        isActive
                          ? 'bg-purple-600 text-white'
                          : 'hover:bg-gray-700 text-gray-300'
                      }`
                    }
                  >
                    <Icon className="w-5 h-5" />
                    {isSidebarOpen && <span>{label}</span>}
                  </NavLink>
                </li>
              );
            })}
          </ul>
        </nav>

        {/* User Info and Controls */}
        <div className="p-4 border-t border-gray-700 space-y-2">
          {isSidebarOpen && user && (
            <div className="px-3 py-2 bg-gray-700/50 rounded-lg">
              <div className="flex items-center gap-2">
                <User className="w-4 h-4 text-gray-400" />
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium truncate">{user.username}</p>
                  <p className="text-xs text-gray-400 capitalize">{user.role}</p>
                </div>
              </div>
            </div>
          )}

          <div className="flex gap-2">
            <button
              onClick={() => setIsSidebarOpen(!isSidebarOpen)}
              className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition-colors"
            >
              <Settings className="w-4 h-4" />
              {isSidebarOpen && <span className="text-sm">Toggle</span>}
            </button>

            <button
              onClick={handleLogout}
              className="flex items-center justify-center gap-2 px-3 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
              title="Logout"
            >
              <LogOut className="w-4 h-4" />
              {isSidebarOpen && <span className="text-sm">Logout</span>}
            </button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-auto">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/clients" element={<ClientsPage />} />
          <Route path="/relays" element={
            <ProtectedRoute requiredRole="operator">
              <RelaysPage />
            </ProtectedRoute>
          } />
          <Route path="/iptables" element={
            <ProtectedRoute requiredRole="operator">
              <IptablesPage />
            </ProtectedRoute>
          } />
          <Route path="/metrics" element={<MetricsPage />} />
        </Routes>
      </div>
    </div>
  );
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Router>
          <ProtectedRoute>
            <MainApp />
          </ProtectedRoute>
        </Router>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
