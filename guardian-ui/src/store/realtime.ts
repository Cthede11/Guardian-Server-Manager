import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { 
  createRealtimeManager, 
  realtimeHelpers
} from '@/lib/realtime';
import type { 
  RealtimeMetrics, 
  RealtimeHealth, 
  RealtimeWorldData, 
  RealtimePerformanceData, 
  RealtimeBackupData, 
  RealtimeEventData, 
  RealtimePregenData, 
  RealtimeShardingData, 
  RealtimeDiagnosticsData, 
  RealtimeModsData
} from '@/lib/realtime';
import type { ConsoleMessage, Player, ServerSummary } from '@/lib/types';

interface RealtimeState {
  // Connection state
  isConnected: boolean;
  connectionType: 'socket' | 'sse' | 'disconnected';
  lastConnected: string | null;
  connectionErrors: string[];

  // Server data
  servers: ServerSummary[];
  selectedServerId: string | null;

  // Per-server data
  serverData: Record<string, {
    console: ConsoleMessage[];
    metrics: RealtimeMetrics | null;
    health: RealtimeHealth | null;
    players: Player[];
    world: RealtimeWorldData | null;
    performance: RealtimePerformanceData | null;
    backups: RealtimeBackupData | null;
    events: RealtimeEventData | null;
    pregen: RealtimePregenData | null;
    mods: RealtimeModsData | null;
    diagnostics: RealtimeDiagnosticsData | null;
  }>;

  // Global data
  sharding: RealtimeShardingData | null;
  workspace: any;

  // Subscription managers
  managers: Record<string, ReturnType<typeof createRealtimeManager>>;

  // Actions
  connect: () => void;
  disconnect: () => void;
  selectServer: (serverId: string) => void;
  
  // Server data actions
  updateServers: (servers: ServerSummary[]) => void;
  updateConsole: (serverId: string, message: ConsoleMessage) => void;
  updateMetrics: (serverId: string, metrics: RealtimeMetrics) => void;
  updateHealth: (serverId: string, health: RealtimeHealth) => void;
  updatePlayers: (serverId: string, players: Player[]) => void;
  updateWorld: (serverId: string, world: RealtimeWorldData) => void;
  updatePerformance: (serverId: string, performance: RealtimePerformanceData) => void;
  updateBackups: (serverId: string, backups: RealtimeBackupData) => void;
  updateEvents: (serverId: string, events: RealtimeEventData) => void;
  updatePregen: (serverId: string, pregen: RealtimePregenData) => void;
  updateMods: (serverId: string, mods: RealtimeModsData) => void;
  updateDiagnostics: (serverId: string, diagnostics: RealtimeDiagnosticsData) => void;
  
  // Global data actions
  updateSharding: (sharding: RealtimeShardingData) => void;
  updateWorkspace: (workspace: any) => void;
  
  // Utility actions
  clearConsole: (serverId: string) => void;
  clearConnectionErrors: () => void;
  getServerData: (serverId: string) => any;
  getHealthScore: (serverId: string) => number;
  getPerformanceScore: (serverId: string) => number;
}

export const useRealtimeStore = create<RealtimeState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    isConnected: false,
    connectionType: 'disconnected',
    lastConnected: null,
    connectionErrors: [],
    servers: [],
    selectedServerId: null,
    serverData: {},
    sharding: null,
    workspace: null,
    managers: {},

    // Actions
    connect: () => {
      const { socketManager } = require('@/lib/socket');
      
      // Update connection state
      const updateConnectionState = () => {
        const isConnected = socketManager.isConnected();
        const connectionType = socketManager.getConnectionType();
        
        set((state) => ({
          isConnected,
          connectionType,
          lastConnected: isConnected ? new Date().toISOString() : state.lastConnected,
        }));
      };

      // Initial connection state
      updateConnectionState();
      
      // Subscribe to connection changes
      const interval = setInterval(updateConnectionState, 1000);
      
      // Clean up on disconnect
      return () => clearInterval(interval);
    },

    disconnect: () => {
      const { socketManager } = require('@/lib/socket');
      socketManager.disconnect();
      
      set({
        isConnected: false,
        connectionType: 'disconnected',
      });
    },

    selectServer: (serverId: string) => {
      set({ selectedServerId: serverId });
      
      // Initialize server data if not exists
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: state.serverData[serverId] || {
            console: [],
            metrics: null,
            health: null,
            players: [],
            world: null,
            performance: null,
            backups: null,
            events: null,
            pregen: null,
            mods: null,
            diagnostics: null,
          },
        },
      }));
    },

    // Server data actions
    updateServers: (servers: ServerSummary[]) => {
      set({ servers });
    },

    updateConsole: (serverId: string, message: ConsoleMessage) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            console: [...(state.serverData[serverId]?.console || []), message].slice(-1000), // Keep last 1000 messages
          },
        },
      }));
    },

    updateMetrics: (serverId: string, metrics: RealtimeMetrics) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            metrics,
          },
        },
      }));
    },

    updateHealth: (serverId: string, health: RealtimeHealth) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            health,
          },
        },
      }));
    },

    updatePlayers: (serverId: string, players: Player[]) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            players,
          },
        },
      }));
    },

    updateWorld: (serverId: string, world: RealtimeWorldData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            world,
          },
        },
      }));
    },

    updatePerformance: (serverId: string, performance: RealtimePerformanceData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            performance,
          },
        },
      }));
    },

    updateBackups: (serverId: string, backups: RealtimeBackupData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            backups,
          },
        },
      }));
    },

    updateEvents: (serverId: string, events: RealtimeEventData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            events,
          },
        },
      }));
    },

    updatePregen: (serverId: string, pregen: RealtimePregenData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            pregen,
          },
        },
      }));
    },

    updateMods: (serverId: string, mods: RealtimeModsData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            mods,
          },
        },
      }));
    },

    updateDiagnostics: (serverId: string, diagnostics: RealtimeDiagnosticsData) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            diagnostics,
          },
        },
      }));
    },

    // Global data actions
    updateSharding: (sharding: RealtimeShardingData) => {
      set({ sharding });
    },

    updateWorkspace: (workspace: any) => {
      set({ workspace });
    },

    // Utility actions
    clearConsole: (serverId: string) => {
      set((state) => ({
        serverData: {
          ...state.serverData,
          [serverId]: {
            ...state.serverData[serverId],
            console: [],
          },
        },
      }));
    },

    clearConnectionErrors: () => {
      set({ connectionErrors: [] });
    },

    getServerData: (serverId: string) => {
      const state = get();
      return state.serverData[serverId] || null;
    },

    getHealthScore: (serverId: string) => {
      const state = get();
      const health = state.serverData[serverId]?.health;
      return health ? realtimeHelpers.calculateHealthScore(health) : 0;
    },

    getPerformanceScore: (serverId: string) => {
      const state = get();
      const metrics = state.serverData[serverId]?.metrics;
      return metrics ? realtimeHelpers.calculatePerformanceScore(metrics) : 0;
    },
  }))
);

// Helper hooks for common real-time data access
export const useRealtimeConnection = () => {
  const { isConnected, connectionType, lastConnected, connectionErrors } = useRealtimeStore();
  
  return {
    isConnected,
    connectionType,
    lastConnected,
    connectionErrors,
    status: realtimeHelpers.getConnectionStatus(),
    typeDisplay: realtimeHelpers.getConnectionTypeDisplay(),
  };
};

export const useRealtimeServer = (serverId: string) => {
  const { 
    serverData, 
    updateConsole, 
    updateMetrics, 
    updateHealth, 
    updatePlayers, 
    updateWorld, 
    updatePerformance, 
    updateBackups, 
    updateEvents, 
    updatePregen, 
    updateMods, 
    updateDiagnostics,
    clearConsole,
    getHealthScore,
    getPerformanceScore
  } = useRealtimeStore();
  
  const data = serverData[serverId] || {
    console: [],
    metrics: null,
    health: null,
    players: [],
    world: null,
    performance: null,
    backups: null,
    events: null,
    pregen: null,
    mods: null,
    diagnostics: null,
  };

  return {
    ...data,
    healthScore: getHealthScore(serverId),
    performanceScore: getPerformanceScore(serverId),
    healthStatus: data.health ? realtimeHelpers.getHealthStatus(data.health) : 'unknown',
    performanceStatus: data.metrics ? realtimeHelpers.getPerformanceStatus(data.metrics) : 'unknown',
    clearConsole: () => clearConsole(serverId),
  };
};

export const useRealtimeServers = () => {
  const { servers, updateServers } = useRealtimeStore();
  
  return {
    servers,
    updateServers,
  };
};

export const useRealtimeSharding = () => {
  const { sharding, updateSharding } = useRealtimeStore();
  
  return {
    sharding,
    updateSharding,
  };
};

export const useRealtimeWorkspace = () => {
  const { workspace, updateWorkspace } = useRealtimeStore();
  
  return {
    workspace,
    updateWorkspace,
  };
};

// Subscription hooks for automatic data updates
export const useRealtimeSubscription = (serverId: string) => {
  const { 
    updateConsole, 
    updateMetrics, 
    updateHealth, 
    updatePlayers, 
    updateWorld, 
    updatePerformance, 
    updateBackups, 
    updateEvents, 
    updatePregen, 
    updateMods, 
    updateDiagnostics 
  } = useRealtimeStore();
  
  const manager = createRealtimeManager(serverId);
  
  // Set up subscriptions
  const unsubscribe = manager.subscribeToAllServerData({
    console: updateConsole,
    metrics: updateMetrics,
    health: updateHealth,
    players: updatePlayers,
    world: updateWorld,
    performance: updatePerformance,
    backups: updateBackups,
    events: updateEvents,
    pregen: updatePregen,
    mods: updateMods,
    diagnostics: updateDiagnostics,
  });
  
  return {
    manager,
    unsubscribe,
  };
};

// Global subscription hook
export const useRealtimeGlobalSubscription = () => {
  const { updateServers, updateSharding, updateWorkspace } = useRealtimeStore();
  
  const manager = createRealtimeManager();
  
  // Set up global subscriptions
  const unsubscribe = manager.subscribeToAllServerData({
    // Global subscriptions would go here
  });
  
  // Add global subscriptions
  const unsubscribeServers = manager.subscribe('servers', updateServers);
  const unsubscribeSharding = manager.subscribe('sharding', updateSharding);
  const unsubscribeWorkspace = manager.subscribe('workspace', updateWorkspace);
  
  return {
    manager,
    unsubscribe: () => {
      unsubscribe();
      unsubscribeServers();
      unsubscribeSharding();
      unsubscribeWorkspace();
    },
  };
};
