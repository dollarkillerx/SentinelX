import React, { useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter as Router, Routes, Route, NavLink } from 'react-router-dom';
import {
  Shield, Server, Network, Filter, BarChart3,
  Settings, Home, LogOut, User, Menu, X
} from 'lucide-react';
import * as NavigationMenu from '@radix-ui/react-navigation-menu';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import * as AlertDialog from '@radix-ui/react-alert-dialog';
import * as Toggle from '@radix-ui/react-toggle';
import * as Separator from '@radix-ui/react-separator';
import * as Tooltip from '@radix-ui/react-tooltip';
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
    <Tooltip.Provider>
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

          <NavigationMenu.Root className="flex-1 p-4" orientation="vertical">
            <NavigationMenu.List className="space-y-2 list-none">
              {navItems.map(({ path, icon: Icon, label, requiredRole }) => {
                // Role-based visibility
                if (requiredRole) {
                  const roleHierarchy = { viewer: 1, operator: 2, admin: 3 };
                  const userLevel = roleHierarchy[user?.role || 'viewer'];
                  const requiredLevel = roleHierarchy[requiredRole];
                  if (userLevel < requiredLevel) return null;
                }

                return (
                  <NavigationMenu.Item key={path}>
                    <NavigationMenu.Link asChild>
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
                        {isSidebarOpen ? (
                          <span>{label}</span>
                        ) : (
                          <Tooltip.Root>
                            <Tooltip.Trigger asChild>
                              <span className="sr-only">{label}</span>
                            </Tooltip.Trigger>
                            <Tooltip.Portal>
                              <Tooltip.Content
                                className="bg-gray-700 text-white px-2 py-1 rounded text-sm"
                                side="right"
                                sideOffset={5}
                              >
                                {label}
                                <Tooltip.Arrow className="fill-gray-700" />
                              </Tooltip.Content>
                            </Tooltip.Portal>
                          </Tooltip.Root>
                        )}
                      </NavLink>
                    </NavigationMenu.Link>
                  </NavigationMenu.Item>
                );
              })}
            </NavigationMenu.List>
          </NavigationMenu.Root>

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
              <Tooltip.Root>
                <Tooltip.Trigger asChild>
                  <Toggle.Root
                    pressed={isSidebarOpen}
                    onPressedChange={setIsSidebarOpen}
                    className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition-colors data-[state=on]:bg-gray-600"
                  >
                    {isSidebarOpen ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
                    {isSidebarOpen && <span className="text-sm">Collapse</span>}
                  </Toggle.Root>
                </Tooltip.Trigger>
                <Tooltip.Portal>
                  <Tooltip.Content
                    className="bg-gray-700 text-white px-2 py-1 rounded text-sm"
                    side="top"
                    sideOffset={5}
                  >
                    {isSidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
                    <Tooltip.Arrow className="fill-gray-700" />
                  </Tooltip.Content>
                </Tooltip.Portal>
              </Tooltip.Root>

              <AlertDialog.Root>
                <AlertDialog.Trigger asChild>
                  <button className="flex items-center justify-center gap-2 px-3 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors">
                    <LogOut className="w-4 h-4" />
                    {isSidebarOpen && <span className="text-sm">Logout</span>}
                  </button>
                </AlertDialog.Trigger>
                <AlertDialog.Portal>
                  <AlertDialog.Overlay className="fixed inset-0 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
                  <AlertDialog.Content className="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border border-gray-700 bg-gray-800 p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg text-white">
                    <div className="flex flex-col space-y-2 text-center sm:text-left">
                      <AlertDialog.Title className="text-lg font-semibold">
                        Confirm Logout
                      </AlertDialog.Title>
                      <AlertDialog.Description className="text-sm text-gray-400">
                        Are you sure you want to logout? You will need to sign in again to access the control panel.
                      </AlertDialog.Description>
                    </div>
                    <div className="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
                      <AlertDialog.Cancel asChild>
                        <button className="inline-flex h-10 items-center justify-center rounded-md border border-gray-600 bg-transparent px-4 py-2 text-sm font-medium text-gray-300 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 focus:ring-offset-gray-800 disabled:cursor-not-allowed disabled:opacity-50">
                          Cancel
                        </button>
                      </AlertDialog.Cancel>
                      <AlertDialog.Action asChild>
                        <button
                          onClick={handleLogout}
                          className="inline-flex h-10 items-center justify-center rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 focus:ring-offset-gray-800 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          Logout
                        </button>
                      </AlertDialog.Action>
                    </div>
                  </AlertDialog.Content>
                </AlertDialog.Portal>
              </AlertDialog.Root>
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
    </Tooltip.Provider>
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
