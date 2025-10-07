import { create } from "zustand";
import type { ServerSummary, ServerSettings, ServerHealth } from "@/lib/types.gen";
import { api } from "@/lib/client";

interface ServersState {
  // Server list and selection
  selectedId?: string;
  selectedServerId?: string; // Alias for compatibility
  summaries: Record<string, ServerSummary>;
  
  // Server details
  settings: Record<string, ServerSettings>;
  health: Record<string, ServerHealth>;
  
  // Loading states
  loading: boolean;
  error: string | null;
  
  // Actions
  select: (id: string) => void;
  fetchSummary: (id: string, signal?: AbortSignal) => Promise<void>;
  fetchServers: (signal?: AbortSignal) => Promise<void>;
  createServer: (data: any) => Promise<boolean>;
  deleteServer: (id: string) => Promise<boolean>;
  
  // Server control
  startServer: (id: string) => Promise<boolean>;
  stopServer: (id: string) => Promise<boolean>;
  restartServer: (id: string) => Promise<boolean>;
  promoteServer: (id: string) => Promise<boolean>;
  
  // Server data
  fetchSettings: (id: string, signal?: AbortSignal) => Promise<void>;
  updateSettings: (id: string, settings: ServerSettings) => Promise<boolean>;
  fetchHealth: (id: string, signal?: AbortSignal) => Promise<void>;
  
  // Utility
  getServerById: (id: string) => ServerSummary | null;
  clearError: () => void;
}

export const useServers = create<ServersState>((set, get) => ({
  // Initial state
  selectedId: undefined,
  selectedServerId: undefined,
  summaries: {},
  settings: {},
  health: {},
  loading: false,
  error: null,

  // Actions
  select: (id: string) => {
    set({ selectedId: id, selectedServerId: id });
  },

  fetchSummary: async (id: string, signal?: AbortSignal) => {
    try {
      const summary = await api.getServerSummary(id);
      if (signal?.aborted) return;
      
      set((state) => ({
        summaries: { ...state.summaries, [id]: summary }
      }));
    } catch (error) {
      if (!signal?.aborted) {
        set({ error: error instanceof Error ? error.message : 'Failed to fetch server summary' });
      }
    }
  },

  fetchServers: async (signal?: AbortSignal) => {
    set({ loading: true, error: null });
    
    try {
      console.log('Fetching servers from API...');
      const servers = await api.getServers();
      console.log('API returned servers:', servers);
      if (signal?.aborted) return;
      
      // Defensive programming: ensure servers is an array
      if (!Array.isArray(servers)) {
        console.error('Servers is not an array:', servers);
        throw new Error('Invalid server data format received from API');
      }
      
      const summaries = servers.reduce((acc: Record<string, ServerSummary>, server: ServerSummary) => {
        // Defensive programming: ensure server has required properties
        if (!server || typeof server !== 'object' || !server.id) {
          console.warn('Invalid server object:', server);
          return acc;
        }
        acc[server.id] = server;
        return acc;
      }, {} as Record<string, ServerSummary>);
      
      console.log('Setting summaries:', summaries);
      console.log('Number of servers:', Object.keys(summaries).length);
      set({ summaries, loading: false });
      console.log('State updated successfully');
      
      // Force a re-render by logging the current state
      setTimeout(() => {
        const currentState = get();
        console.log('Current state after fetchServers:', {
          summariesCount: Object.keys(currentState.summaries).length,
          loading: currentState.loading,
          error: currentState.error
        });
      }, 100);
    } catch (error) {
      console.error('Error fetching servers:', error);
      console.error('Error type:', typeof error);
      console.error('Error constructor:', error?.constructor?.name);
      if (error instanceof Error) {
        console.error('Error message:', error.message);
        console.error('Error stack:', error.stack);
      } else if (typeof error === 'object' && error !== null) {
        console.error('Error object details:', JSON.stringify(error, null, 2));
      }
      if (!signal?.aborted) {
        set({ 
          error: error instanceof Error ? error.message : 'Failed to fetch servers',
          loading: false 
        });
      }
    }
  },

  createServer: async (data: any) => {
    set({ loading: true, error: null });
    
    try {
      console.log('Creating server with data:', data);
      const server = await api.createServer(data);
      console.log('Server created:', server);
      
      set((state) => {
        const newSummaries = { ...state.summaries, [server.id]: server };
        console.log('Updating summaries with new server:', newSummaries);
        return {
          summaries: newSummaries,
          loading: false
        };
      });
      
      return true;
    } catch (error) {
      console.error('Error creating server:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Failed to create server',
        loading: false 
      });
      return false;
    }
  },

  deleteServer: async (id: string) => {
    set({ loading: true, error: null });
    
    try {
      await api.deleteServer(id);
      
      set((state) => {
        const { [id]: deleted, ...remaining } = state.summaries;
        return {
          summaries: remaining,
          selectedId: state.selectedId === id ? undefined : state.selectedId,
          selectedServerId: state.selectedServerId === id ? undefined : state.selectedServerId,
          loading: false
        };
      });
      
      return true;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to delete server',
        loading: false 
      });
      return false;
    }
  },

  // Server control
  startServer: async (id: string) => {
    try {
      await api.startServer(id);
      
      // Optimistically update status
      set((state) => ({
        summaries: {
          ...state.summaries,
          [id]: { ...state.summaries[id], status: 'starting' }
        }
      }));
      
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to start server' });
      return false;
    }
  },

  stopServer: async (id: string) => {
    try {
      await api.stopServer(id);
      
      // Optimistically update status
      set((state) => ({
        summaries: {
          ...state.summaries,
          [id]: { ...state.summaries[id], status: 'stopping' }
        }
      }));
      
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to stop server' });
      return false;
    }
  },

  restartServer: async (id: string) => {
    try {
      await api.restartServer(id);
      
      // Optimistically update status
      set((state) => ({
        summaries: {
          ...state.summaries,
          [id]: { ...state.summaries[id], status: 'starting' }
        }
      }));
      
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to restart server' });
      return false;
    }
  },

  promoteServer: async (id: string) => {
    try {
      await api.promoteServer(id);
      
      // Optimistically update blue/green status
      set((state) => {
        const server = state.summaries[id];
        if (!server?.blueGreen) return state;
        
        return {
          summaries: {
            ...state.summaries,
            [id]: {
              ...server,
              blueGreen: {
                ...server.blueGreen,
                active: server.blueGreen.active === 'blue' ? 'green' : 'blue'
              }
            }
          }
        };
      });
      
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to promote server' });
      return false;
    }
  },

  // Server data
  fetchSettings: async (id: string, signal?: AbortSignal) => {
    try {
      const settings = await api.getServerSettings(id);
      if (signal?.aborted) return;
      
      set((state) => ({
        settings: { ...state.settings, [id]: settings }
      }));
    } catch (error) {
      if (!signal?.aborted) {
        set({ error: error instanceof Error ? error.message : 'Failed to fetch server settings' });
      }
    }
  },

  updateSettings: async (id: string, settings: ServerSettings) => {
    try {
      const updatedSettings = await api.updateServerSettings(id, settings);
      
      set((state) => ({
        settings: { ...state.settings, [id]: updatedSettings }
      }));
      
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to update server settings' });
      return false;
    }
  },

  fetchHealth: async (id: string, signal?: AbortSignal) => {
    try {
      const health = await api.getServerHealth(id);
      if (signal?.aborted) return;
      
      set((state) => ({
        health: { ...state.health, [id]: health }
      }));
    } catch (error) {
      if (!signal?.aborted) {
        set({ error: error instanceof Error ? error.message : 'Failed to fetch server health' });
      }
    }
  },

  // Utility
  getServerById: (id: string) => {
    const { summaries } = get();
    return summaries[id] || null;
  },

  clearError: () => {
    set({ error: null });
  },
}));

// Performance-optimized selectors
export const useSelectedServer = () => {
  return useServers((state) => {
    const { selectedId, summaries } = state;
    return selectedId ? summaries[selectedId] || null : null;
  });
};

export const useServerHealth = (serverId: string) => {
  return useServers((state) => state.health[serverId] || null);
};

export const useServerSettings = (serverId: string) => {
  return useServers((state) => state.settings[serverId] || null);
};

export const useServersList = () => {
  return useServers((state) => Object.values(state.summaries));
};
