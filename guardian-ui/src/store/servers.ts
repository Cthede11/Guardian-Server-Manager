import React from 'react';
import { create } from 'zustand';
import { api } from '@/lib/api';
import type { ServerSummary, ServerHealth, ServerSettings } from '@/lib/types';

interface ServersState {
  // Server list
  servers: ServerSummary[];
  selectedServerId: string | null;
  
  // Server details
  serverHealth: Record<string, ServerHealth>;
  serverSettings: Record<string, ServerSettings>;
  
  // Loading states
  loading: boolean;
  error: string | null;
  
  // Actions
  fetchServers: () => Promise<void>;
  createServer: (data: {
    name: string;
    loader: string;
    version: string;
    paths: { world: string; mods: string; config: string };
  }) => Promise<boolean>;
  selectServer: (id: string) => void;
  getSelectedServer: () => ServerSummary | null;
  
  // Server actions
  startServer: (id: string) => Promise<boolean>;
  stopServer: (id: string) => Promise<boolean>;
  restartServer: (id: string) => Promise<boolean>;
  promoteServer: (id: string) => Promise<boolean>;
  
  // Server data
  fetchServerHealth: (id: string) => Promise<void>;
  fetchServerSettings: (id: string) => Promise<void>;
  updateServerSettings: (id: string, settings: ServerSettings) => Promise<boolean>;
  
  // Utility
  getServerById: (id: string) => ServerSummary | null;
  clearError: () => void;
}

export const useServersStore = create<ServersState>((set, get) => ({
  // Initial state
  servers: [],
  selectedServerId: null,
  serverHealth: {},
  serverSettings: {},
  loading: false,
  error: null,

  // Actions
  fetchServers: async () => {
    set({ loading: true, error: null });
    
    const response = await api.getServers();
    
    if (response.ok && response.data) {
      // Type assertion to ensure compatibility
      const servers = response.data as ServerSummary[];
      set({ 
        servers,
        loading: false,
      });
    } else {
      set({ 
        error: response.error || 'Failed to fetch servers',
        loading: false,
      });
    }
  },

  createServer: async (data) => {
    set({ loading: true, error: null });
    
    const response = await api.createServer(data);
    
    if (response.ok && response.data) {
      // Type assertion to ensure compatibility
      const newServer = response.data as ServerSummary;
      set((state) => ({
        servers: [...state.servers, newServer],
        loading: false,
      }));
      return true;
    } else {
      set({ 
        error: response.error || 'Failed to create server',
        loading: false,
      });
      return false;
    }
  },

  selectServer: (id: string) => {
    set({ selectedServerId: id });
  },

  getSelectedServer: () => {
    const { servers, selectedServerId } = get();
    return selectedServerId ? servers.find(s => s.id === selectedServerId) || null : null;
  },

  startServer: async (id: string) => {
    const response = await api.serverAction(id, 'start');
    
    if (response.ok) {
      // Optimistically update the server status
      set((state) => ({
        servers: state.servers.map(server =>
          server.id === id 
            ? { ...server, status: 'starting' as const } 
            : server
        ),
      }));
      return true;
    } else {
      set({ error: response.error || 'Failed to start server' });
      return false;
    }
  },

  stopServer: async (id: string) => {
    const response = await api.serverAction(id, 'stop');
    
    if (response.ok) {
      // Optimistically update the server status
      set((state) => ({
        servers: state.servers.map(server =>
          server.id === id 
            ? { ...server, status: 'stopping' as const } 
            : server
        ),
      }));
      return true;
    } else {
      set({ error: response.error || 'Failed to stop server' });
      return false;
    }
  },

  restartServer: async (id: string) => {
    const response = await api.serverAction(id, 'restart');
    
    if (response.ok) {
      // Optimistically update the server status
      set((state) => ({
        servers: state.servers.map(server =>
          server.id === id 
            ? { ...server, status: 'starting' as const } 
            : server
        ),
      }));
      return true;
    } else {
      set({ error: response.error || 'Failed to restart server' });
      return false;
    }
  },

  promoteServer: async (id: string) => {
    const response = await api.serverAction(id, 'promote');
    
    if (response.ok) {
      // Optimistically update the blue/green status
      set((state) => ({
        servers: state.servers.map(server =>
          server.id === id 
            ? { 
                ...server, 
                blueGreen: {
                  ...server.blueGreen,
                  active: server.blueGreen.active === 'blue' 
                    ? 'green' as const 
                    : 'blue' as const,
                }
              } 
            : server
        ),
      }));
      return true;
    } else {
      set({ error: response.error || 'Failed to promote server' });
      return false;
    }
  },

  fetchServerHealth: async (id: string) => {
    const response = await api.getServerHealth(id);
    
    if (response.ok && response.data) {
      // Type assertion to ensure compatibility
      const health = response.data as ServerHealth;
      set((state) => ({
        serverHealth: {
          ...state.serverHealth,
          [id]: health,
        },
      }));
    }
  },

  fetchServerSettings: async (id: string) => {
    const response = await api.getServerSettings(id);
    
    if (response.ok && response.data) {
      // Type assertion to ensure compatibility
      const settings = response.data as ServerSettings;
      set((state) => ({
        serverSettings: {
          ...state.serverSettings,
          [id]: settings,
        },
      }));
    }
  },

  updateServerSettings: async (id: string, settings: ServerSettings) => {
    const response = await api.updateServerSettings(id, settings);
    
    if (response.ok && response.data) {
      // Type assertion to ensure compatibility
      const updatedSettings = response.data as ServerSettings;
      set((state) => ({
        serverSettings: {
          ...state.serverSettings,
          [id]: updatedSettings,
        },
      }));
      return true;
    } else {
      set({ error: response.error || 'Failed to update server settings' });
      return false;
    }
  },

  getServerById: (id: string) => {
    const { servers } = get();
    return servers.find(s => s.id === id) || null;
  },

  clearError: () => {
    set({ error: null });
  },
}));

// Helper hooks
export const useSelectedServer = () => {
  const { selectedServerId, servers } = useServersStore();
  return selectedServerId ? servers.find(s => s.id === selectedServerId) || null : null;
};

export const useServerHealth = (serverId: string) => {
  const { serverHealth, fetchServerHealth } = useServersStore();
  
  React.useEffect(() => {
    fetchServerHealth(serverId);
  }, [serverId, fetchServerHealth]);

  return serverHealth[serverId] || null;
};

export const useServerSettings = (serverId: string) => {
  const { serverSettings, fetchServerSettings } = useServersStore();
  
  React.useEffect(() => {
    fetchServerSettings(serverId);
  }, [serverId, fetchServerSettings]);

  return serverSettings[serverId] || null;
};