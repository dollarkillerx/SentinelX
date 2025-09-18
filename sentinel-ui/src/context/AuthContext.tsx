import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

interface User {
  id: string;
  username: string;
  role: 'admin' | 'operator' | 'viewer';
}

interface AuthContextType {
  user: User | null;
  login: (username: string, password: string) => Promise<boolean>;
  logout: () => void;
  isAuthenticated: boolean;
  isLoading: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Check for existing session
    const token = localStorage.getItem('sentinel_token');
    const userData = localStorage.getItem('sentinel_user');

    if (token && userData) {
      try {
        const parsedUser = JSON.parse(userData);
        setUser(parsedUser);
      } catch (error) {
        console.error('Failed to parse user data:', error);
        localStorage.removeItem('sentinel_token');
        localStorage.removeItem('sentinel_user');
      }
    }

    setIsLoading(false);
  }, []);

  const login = async (username: string, password: string): Promise<boolean> => {
    try {
      // Mock authentication - replace with actual API call
      if (username === 'admin' && password === 'admin') {
        const userData: User = {
          id: '1',
          username: 'admin',
          role: 'admin'
        };

        const mockToken = 'mock_jwt_token_' + Date.now();

        setUser(userData);
        localStorage.setItem('sentinel_token', mockToken);
        localStorage.setItem('sentinel_user', JSON.stringify(userData));

        return true;
      } else if (username === 'operator' && password === 'operator') {
        const userData: User = {
          id: '2',
          username: 'operator',
          role: 'operator'
        };

        const mockToken = 'mock_jwt_token_' + Date.now();

        setUser(userData);
        localStorage.setItem('sentinel_token', mockToken);
        localStorage.setItem('sentinel_user', JSON.stringify(userData));

        return true;
      }

      return false;
    } catch (error) {
      console.error('Login failed:', error);
      return false;
    }
  };

  const logout = () => {
    setUser(null);
    localStorage.removeItem('sentinel_token');
    localStorage.removeItem('sentinel_user');
  };

  const value: AuthContextType = {
    user,
    login,
    logout,
    isAuthenticated: !!user,
    isLoading
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}