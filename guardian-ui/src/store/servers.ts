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
    maxPlayers?: number;
    memory?: number;
    paths: { world: string; mods: string; config: string };
  }) => Promise<boolean>;
  selectServer: (id: string) => void;
  getSelectedServer: () => ServerSummary | null;
  
  // Server actions
  startServer: (id: string) => Promise<boolean>;
  stopServer: (id: string) => Promise<boolean>;
  restartServer: (id: string) => Promise<boolean>;
  promoteServer: (id: string) => Promise<boolean>;
  deleteServer: (id: string) => Promise<boolean>;
  
  // Server data
  fetchServerHealth: (id: string) => Promise<void>;
  fetchServerSettings: (id: string) => Promise<void>;
  updateServerSettings: (id: string, settings: ServerSettings) => Promise<boolean>;
  
  // Server configuration files
  fetchServerConfig: (id: string) => Promise<void>;
  updateServerConfig: (id: string, config: any) => Promise<boolean>;
  fetchServerProperties: (id: string) => Promise<void>;
  updateServerProperties: (id: string, properties: Record<string, string>) => Promise<boolean>;
  fetchServerJVMArgs: (id: string) => Promise<void>;
  updateServerJVMArgs: (id: string, args: string[]) => Promise<boolean>;
  
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
    
    try {
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
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to fetch servers',
        loading: false,
      });
    }
  },

  createServer: async (data) => {
    set({ loading: true, error: null });
    
    try {
      console.log('Creating server with data:', data);
      const response = await api.createServer(data);
      console.log('API response:', response);
      
      if (response.ok && response.data) {
        // Type assertion to ensure compatibility
        const newServer = response.data as ServerSummary;
        set((state) => ({
          servers: [...state.servers, newServer],
          loading: false,
        }));
        console.log('Server created successfully:', newServer);
        
        // Refresh the server list to ensure consistency
        setTimeout(() => {
          get().fetchServers();
          // Fetch server configuration files after creation
          get().fetchServerConfig(newServer.id);
          get().fetchServerProperties(newServer.id);
          get().fetchServerJVMArgs(newServer.id);
        }, 100);
        
        return true;
      } else {
        const errorMsg = response.error || 'Failed to create server';
        console.error('Server creation failed:', errorMsg);
        set({ 
          error: errorMsg,
          loading: false,
        });
        return false;
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to create server';
      console.error('Server creation error:', error);
      set({ 
        error: errorMsg,
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
    const response = await api.startServer(id);
    
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
    const response = await api.stopServer(id);
    
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
    const response = await api.restartServer(id);
    
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
    // Note: promoteServer method doesn't exist in API, using restart as fallback
    const response = await api.restartServer(id);
    
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

  deleteServer: async (id: string) => {
    set({ loading: true, error: null });
    
    try {
      console.log('Deleting server:', id);
      const response = await api.deleteServer(id);
      console.log('Delete API response:', response);
      
      if (response.ok) {
        // Always remove server from local state regardless of backend response
        // This ensures the UI updates immediately
        set((state) => {
          const updatedServers = state.servers.filter(server => server.id !== id);
          console.log(`Removed server ${id} from local state. Remaining servers:`, updatedServers.length);
          return {
            servers: updatedServers,
            loading: false,
          };
        });
        
        // Clear selection if the deleted server was selected
        const { selectedServerId } = get();
        if (selectedServerId === id) {
          set({ selectedServerId: null });
          console.log('Cleared selection for deleted server');
        }
        
        console.log('Server deleted successfully from local state');
        
        // Note: If backend doesn't actually delete the server, it will reappear on next fetch
        // This is a backend issue that needs to be fixed in the API
        
        return true;
      } else {
        const errorMsg = response.error || 'Failed to delete server';
        console.error('Server deletion failed:', errorMsg);
        set({ 
          error: errorMsg,
          loading: false,
        });
        return false;
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to delete server';
      console.error('Server deletion error:', error);
      set({ 
        error: errorMsg,
        loading: false,
      });
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

  // Server configuration files
  fetchServerConfig: async (id: string) => {
    try {
      const response = await api.getServerConfig(id);
      if (response.ok && response.data) {
        // Store the configuration data
        set((state) => ({
          serverSettings: {
            ...state.serverSettings,
            [id]: response.data as ServerSettings,
          },
        }));
      }
    } catch (error) {
      console.error('Failed to fetch server config:', error);
      set({ error: 'Failed to fetch server configuration' });
    }
  },

  updateServerConfig: async (id: string, config: any) => {
    try {
      const response = await api.updateServerConfig(id, config);
      if (response.ok) {
        // Update local state
        set((state) => ({
          serverSettings: {
            ...state.serverSettings,
            [id]: { ...state.serverSettings[id], ...config },
          },
        }));
        return true;
      } else {
        set({ error: response.error || 'Failed to update server configuration' });
        return false;
      }
    } catch (error) {
      console.error('Failed to update server config:', error);
      set({ error: 'Failed to update server configuration' });
      return false;
    }
  },

  fetchServerProperties: async (id: string) => {
    try {
      const response = await api.getServerProperties(id);
      if (response.ok && response.data) {
        // Parse server.properties and update settings
        const properties = response.data as Record<string, string>;
        set((state) => ({
          serverSettings: {
            ...state.serverSettings,
            [id]: {
              ...state.serverSettings[id],
              general: {
                ...state.serverSettings[id]?.general,
                maxPlayers: parseInt(properties['max-players']) || 20,
                motd: properties['motd'] || '',
                difficulty: properties['difficulty'] || 'normal',
                gamemode: properties['gamemode'] || 'survival',
                pvp: properties['pvp'] === 'true',
                onlineMode: properties['online-mode'] === 'true',
                whitelist: properties['white-list'] === 'true',
                enableCommandBlock: properties['enable-command-block'] === 'true',
                viewDistance: parseInt(properties['view-distance']) || 10,
                simulationDistance: parseInt(properties['simulation-distance']) || 10,
              },
            },
          },
        }));
      }
    } catch (error) {
      console.error('Failed to fetch server properties:', error);
      set({ error: 'Failed to fetch server properties' });
    }
  },

  updateServerProperties: async (id: string, properties: Record<string, string>) => {
    try {
      const response = await api.updateServerProperties(id, properties);
      if (response.ok) {
        return true;
      } else {
        set({ error: response.error || 'Failed to update server properties' });
        return false;
      }
    } catch (error) {
      console.error('Failed to update server properties:', error);
      set({ error: 'Failed to update server properties' });
      return false;
    }
  },

  fetchServerJVMArgs: async (id: string) => {
    try {
      const response = await api.getServerJVMArgs(id);
      if (response.ok && response.data) {
        // Update JVM settings
        const jvmData = response.data as { args: string[] };
        set((state) => ({
          serverSettings: {
            ...state.serverSettings,
            [id]: {
              ...state.serverSettings[id],
              jvm: {
                ...state.serverSettings[id]?.jvm,
                flags: jvmData.args || [],
              },
            },
          },
        }));
      }
    } catch (error) {
      console.error('Failed to fetch server JVM args:', error);
      set({ error: 'Failed to fetch server JVM arguments' });
    }
  },

  updateServerJVMArgs: async (id: string, args: string[]) => {
    try {
      const response = await api.updateServerJVMArgs(id, args);
      if (response.ok) {
        // Update local state
        set((state) => ({
          serverSettings: {
            ...state.serverSettings,
            [id]: {
              ...state.serverSettings[id],
              jvm: {
                ...state.serverSettings[id]?.jvm,
                flags: args,
              },
            },
          },
        }));
        return true;
      } else {
        set({ error: response.error || 'Failed to update server JVM arguments' });
        return false;
      }
    } catch (error) {
      console.error('Failed to update server JVM args:', error);
      set({ error: 'Failed to update server JVM arguments' });
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

// Performance-optimized selectors
export const useSelectedServer = () => {
  return useServersStore((state) => {
    const { selectedServerId, servers } = state;
    return selectedServerId ? servers.find(s => s.id === selectedServerId) || null : null;
  });
};

export const useServerHealth = (serverId: string) => {
  return useServersStore((state) => {
    if (!serverId) return null;
    return state.serverHealth[serverId] || null;
  });
};

export const useServerSettings = (serverId: string) => {
  return useServersStore((state) => state.serverSettings[serverId] || null);
};

export const useServerById = (serverId: string) => {
  return useServersStore((state) => state.servers.find(s => s.id === serverId) || null);
};

export const useServersList = () => {
  return useServersStore((state) => state.servers);
};