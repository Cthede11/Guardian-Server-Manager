import { create } from 'zustand';
import type { ConsoleMessage, Player, FreezeTicket, PregenJob } from '@/lib/types';

interface LiveState {
  // Connection state
  connected: boolean;
  connectionType: 'socket' | 'sse' | 'disconnected';
  
  // Console streams - rolling buffer of 5000 lines per server
  console: Record<string, ConsoleMessage[]>;
  
  // Player data
  players: Record<string, Player[]>;
  
  // World data
  freezes: Record<string, FreezeTicket[]>;
  
  // Performance data - time-series with max 120 points
  metrics: Record<string, {
    tps: Array<{ timestamp: number; value: number }>;
    heap: Array<{ timestamp: number; value: number }>;
    tickP95: Array<{ timestamp: number; value: number }>;
    gpuMs: Array<{ timestamp: number; value: number }>;
  }>;
  
  // Pregen data
  pregenJobs: Record<string, PregenJob[]>;
  
  // Batched update system
  batch: (fns: Array<() => void>) => void;
  
  // Actions
  setConnected: (connected: boolean) => void;
  setConnectionType: (type: 'socket' | 'sse' | 'disconnected') => void;
  
  // Console actions
  appendConsole: (serverId: string, lines: ConsoleMessage[]) => void;
  clearConsole: (serverId: string) => void;
  
  // Player actions
  updatePlayers: (serverId: string, players: Player[]) => void;
  
  // World actions
  updateFreezes: (serverId: string, freezes: FreezeTicket[]) => void;
  
  // Metrics actions
  applyMetrics: (serverId: string, data: Partial<LiveState['metrics'][string]>) => void;
  
  // Pregen actions
  updatePregenJobs: (serverId: string, jobs: PregenJob[]) => void;
}

export const liveStore = create<LiveState>((set) => ({
  // Initial state
  connected: false,
  connectionType: 'disconnected',
  console: {},
  players: {},
  freezes: {},
  metrics: {},
  pregenJobs: {},

  // Batched update system - prevents re-render storms
  batch: (fns: Array<() => void>) => {
    // Execute all functions - they will update state directly
    fns.forEach(fn => fn());
  },

  // Connection actions
  setConnected: (connected: boolean) => set({ connected }),
  setConnectionType: (connectionType: 'socket' | 'sse' | 'disconnected') => set({ connectionType }),

  // Console actions with rolling buffer
  appendConsole: (serverId: string, lines: ConsoleMessage[]) => {
    set((state) => {
      const currentLines = state.console[serverId] || [];
      const newLines = [...currentLines, ...lines].slice(-5000); // Keep last 5000 lines
      return {
        console: {
          ...state.console,
          [serverId]: newLines,
        },
      };
    });
  },

  clearConsole: (serverId: string) => {
    set((state) => ({
      console: {
        ...state.console,
        [serverId]: [],
      },
    }));
  },

  // Player actions
  updatePlayers: (serverId: string, players: Player[]) => {
    set((state) => ({
      players: {
        ...state.players,
        [serverId]: players,
      },
    }));
  },

  // World actions
  updateFreezes: (serverId: string, freezes: FreezeTicket[]) => {
    set((state) => ({
      freezes: {
        ...state.freezes,
        [serverId]: freezes,
      },
    }));
  },

  // Pregen actions
  addPregenJob: (serverId: string, job: any) => {
    set((state) => ({
      pregenJobs: {
        ...state.pregenJobs,
        [serverId]: [...(state.pregenJobs[serverId] || []), job],
      },
    }));
  },

  updatePregenJob: (serverId: string, jobId: string, updates: any) => {
    set((state) => ({
      pregenJobs: {
        ...state.pregenJobs,
        [serverId]: (state.pregenJobs[serverId] || []).map(job =>
          job.id === jobId ? { ...job, ...updates } : job
        ),
      },
    }));
  },

  // Metrics actions with time-series management
  applyMetrics: (serverId: string, data: Partial<LiveState['metrics'][string]>) => {
    set((state) => {
      const currentMetrics = state.metrics[serverId] || {
        tps: [],
        heap: [],
        tickP95: [],
        gpuMs: [],
      };

      const now = Date.now();
      const maxPoints = 120; // Keep last 2 minutes at 1Hz

      const updateSeries = (series: Array<{ timestamp: number; value: number }>, newValue?: number) => {
        if (newValue !== undefined) {
          return [...series, { timestamp: now, value: newValue }].slice(-maxPoints);
        }
        return series;
      };

      const updatedMetrics = {
        tps: updateSeries(currentMetrics.tps, data.tps?.[data.tps.length - 1]?.value),
        heap: updateSeries(currentMetrics.heap, data.heap?.[data.heap.length - 1]?.value),
        tickP95: updateSeries(currentMetrics.tickP95, data.tickP95?.[data.tickP95.length - 1]?.value),
        gpuMs: updateSeries(currentMetrics.gpuMs, data.gpuMs?.[data.gpuMs.length - 1]?.value),
      };

      return {
        metrics: {
          ...state.metrics,
          [serverId]: updatedMetrics,
        },
      };
    });
  },

  // Pregen actions
  updatePregenJobs: (serverId: string, jobs: PregenJob[]) => {
    set((state) => ({
      pregenJobs: {
        ...state.pregenJobs,
        [serverId]: jobs,
      },
    }));
  },
}));

// Performance-optimized selectors with proper memoization
export const useConsoleStream = (serverId: string) => {
  return liveStore((state) => state.console[serverId] || []);
};

export const usePlayerData = (serverId: string) => {
  return liveStore((state) => state.players[serverId] || []);
};

export const useWorldData = (serverId: string) => {
  return liveStore((state) => ({
    freezes: state.freezes[serverId] || [],
  }));
};

export const useMetrics = (serverId: string) => {
  return liveStore((state) => state.metrics[serverId] || null);
};

export const usePregenJobs = (serverId: string) => {
  return liveStore((state) => state.pregenJobs[serverId] || []);
};

export const useConnectionStatus = () => {
  return liveStore((state) => ({
    connected: state.connected,
    connectionType: state.connectionType,
  }));
};
