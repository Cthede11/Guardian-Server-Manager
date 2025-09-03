import React from 'react';
import { create } from 'zustand';
import { socketManager } from '@/lib/socket';
import { ConsoleMessage, Player, FreezeTicket, PregenJob } from '@/lib/types';

interface LiveState {
  // Connection state
  isConnected: boolean;
  connectionType: 'socket' | 'sse' | 'disconnected';
  
  // Console streams
  consoleStreams: Record<string, ConsoleMessage[]>;
  
  // Player data
  players: Record<string, Player[]>;
  
  // World data
  freezes: Record<string, FreezeTicket[]>;
  
  // Performance data
  metrics: Record<string, any>;
  
  // Pregen data
  pregenJobs: Record<string, PregenJob[]>;
  
  // Actions
  connect: () => void;
  disconnect: () => void;
  subscribeToConsole: (serverId: string) => void;
  subscribeToPlayers: (serverId: string) => void;
  subscribeToWorld: (serverId: string) => void;
  subscribeToMetrics: (serverId: string) => void;
  subscribeToPregen: (serverId: string) => void;
  clearConsole: (serverId: string) => void;
  addConsoleMessage: (serverId: string, message: ConsoleMessage) => void;
  updatePlayers: (serverId: string, players: Player[]) => void;
  updateFreezes: (serverId: string, freezes: FreezeTicket[]) => void;
  updateMetrics: (serverId: string, metrics: any) => void;
  updatePregenJobs: (serverId: string, jobs: PregenJob[]) => void;
}

export const useLiveStore = create<LiveState>((set, get) => ({
  // Initial state
  isConnected: false,
  connectionType: 'disconnected',
  consoleStreams: {},
  players: {},
  freezes: {},
  metrics: {},
  pregenJobs: {},

  // Actions
  connect: () => {
    socketManager.connect();
    
    // Update connection state periodically
    const updateConnectionState = () => {
      set({
        isConnected: socketManager.isConnected(),
        connectionType: socketManager.getConnectionType(),
      });
    };

    updateConnectionState();
    const interval = setInterval(updateConnectionState, 1000);

    // Clean up interval when component unmounts
    return () => clearInterval(interval);
  },

  disconnect: () => {
    socketManager.disconnect();
    set({
      isConnected: false,
      connectionType: 'disconnected',
    });
  },

  subscribeToConsole: (serverId: string) => {
    const unsubscribe = socketManager.subscribeToConsole(serverId, (message: ConsoleMessage) => {
      set((state) => ({
        consoleStreams: {
          ...state.consoleStreams,
          [serverId]: [...(state.consoleStreams[serverId] || []), message].slice(-1000), // Keep last 1000 messages
        },
      }));
    });

    return unsubscribe;
  },

  subscribeToPlayers: (serverId: string) => {
    const unsubscribe = socketManager.subscribeToPlayers(serverId, (players: Player[]) => {
      set((state) => ({
        players: {
          ...state.players,
          [serverId]: players,
        },
      }));
    });

    return unsubscribe;
  },

  subscribeToWorld: (serverId: string) => {
    const unsubscribe = socketManager.subscribeToWorld(serverId, (worldData: any) => {
      if (worldData.freezes) {
        set((state) => ({
          freezes: {
            ...state.freezes,
            [serverId]: worldData.freezes,
          },
        }));
      }
    });

    return unsubscribe;
  },

  subscribeToMetrics: (serverId: string) => {
    const unsubscribe = socketManager.subscribeToMetrics(serverId, (metrics: any) => {
      set((state) => ({
        metrics: {
          ...state.metrics,
          [serverId]: metrics,
        },
      }));
    });

    return unsubscribe;
  },

  subscribeToPregen: (serverId: string) => {
    const unsubscribe = socketManager.subscribeToPregen(serverId, (pregenData: any) => {
      if (pregenData.jobs) {
        set((state) => ({
          pregenJobs: {
            ...state.pregenJobs,
            [serverId]: pregenData.jobs,
          },
        }));
      }
    });

    return unsubscribe;
  },

  clearConsole: (serverId: string) => {
    set((state) => ({
      consoleStreams: {
        ...state.consoleStreams,
        [serverId]: [],
      },
    }));
  },

  addConsoleMessage: (serverId: string, message: ConsoleMessage) => {
    set((state) => ({
      consoleStreams: {
        ...state.consoleStreams,
        [serverId]: [...(state.consoleStreams[serverId] || []), message].slice(-1000),
      },
    }));
  },

  updatePlayers: (serverId: string, players: Player[]) => {
    set((state) => ({
      players: {
        ...state.players,
        [serverId]: players,
      },
    }));
  },

  updateFreezes: (serverId: string, freezes: FreezeTicket[]) => {
    set((state) => ({
      freezes: {
        ...state.freezes,
        [serverId]: freezes,
      },
    }));
  },

  updateMetrics: (serverId: string, metrics: any) => {
    set((state) => ({
      metrics: {
        ...state.metrics,
        [serverId]: metrics,
      },
    }));
  },

  updatePregenJobs: (serverId: string, jobs: PregenJob[]) => {
    set((state) => ({
      pregenJobs: {
        ...state.pregenJobs,
        [serverId]: jobs,
      },
    }));
  },
}));

// Helper hooks for common subscriptions
export const useConsoleStream = (serverId: string) => {
  const { consoleStreams, subscribeToConsole, clearConsole } = useLiveStore();
  
  React.useEffect(() => {
    const unsubscribe = subscribeToConsole(serverId);
    return unsubscribe;
  }, [serverId, subscribeToConsole]);

  return {
    messages: consoleStreams[serverId] || [],
    clear: () => clearConsole(serverId),
  };
};

export const usePlayerData = (serverId: string) => {
  const { players, subscribeToPlayers } = useLiveStore();
  
  React.useEffect(() => {
    const unsubscribe = subscribeToPlayers(serverId);
    return unsubscribe;
  }, [serverId, subscribeToPlayers]);

  return players[serverId] || [];
};

export const useWorldData = (serverId: string) => {
  const { freezes, subscribeToWorld } = useLiveStore();
  
  React.useEffect(() => {
    const unsubscribe = subscribeToWorld(serverId);
    return unsubscribe;
  }, [serverId, subscribeToWorld]);

  return {
    freezes: freezes[serverId] || [],
  };
};

export const useMetrics = (serverId: string) => {
  const { metrics, subscribeToMetrics } = useLiveStore();
  
  React.useEffect(() => {
    const unsubscribe = subscribeToMetrics(serverId);
    return unsubscribe;
  }, [serverId, subscribeToMetrics]);

  return metrics[serverId] || null;
};

export const usePregenJobs = (serverId: string) => {
  const { pregenJobs, subscribeToPregen } = useLiveStore();
  
  React.useEffect(() => {
    const unsubscribe = subscribeToPregen(serverId);
    return unsubscribe;
  }, [serverId, subscribeToPregen]);

  return pregenJobs[serverId] || [];
};
