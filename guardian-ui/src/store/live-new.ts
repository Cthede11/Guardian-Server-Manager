import { create } from "zustand";
import type { ConsoleLine, Player, FreezeTicket, PregenJob, Metrics, ServerHealth } from "@/lib/types.gen";

type Line = { ts: number; level: string; msg: string };
type Series = number[];

interface LiveState {
  // Console streams - rolling buffer of 5000 lines per server
  console: Record<string, Line[]>;
  
  // Player data
  players: Record<string, Player[]>;
  
  // World data
  freezes: Record<string, FreezeTicket[]>;
  
  // Performance data - time-series with max 120 points
  metrics: Record<string, { 
    tps: Series; 
    p95: Series; 
    heap: Series; 
    gpu: Series; 
    ppl: Series 
  }>;
  
  // Pregen data
  pregenJobs: Record<string, PregenJob[]>;
  
  // Server health
  health: Record<string, ServerHealth>;
  
  // Actions
  appendConsole: (id: string, lines: Line[]) => void;
  updatePlayers: (id: string, players: Player[]) => void;
  updateFreezes: (id: string, freezes: FreezeTicket[]) => void;
  applyMetrics: (id: string, m: { 
    tps: number; 
    tick_p95_ms: number; 
    heap_mb: number; 
    gpu_queue_ms: number; 
    players_online: number 
  }) => void;
  updatePregenJobs: (id: string, jobs: PregenJob[]) => void;
  updateHealth: (id: string, health: ServerHealth) => void;
  clearConsole: (id: string) => void;
}

// RequestAnimationFrame batching system
let queue: Array<() => void> = [];
let scheduled = false;

function rafBatch(fn: () => void) {
  queue.push(fn);
  if (!scheduled) {
    scheduled = true;
    requestAnimationFrame(() => {
      const q = queue;
      queue = [];
      scheduled = false;
      q.forEach(f => f());
    });
  }
}

function pushWindow(arr: number[], v: number, max = 120) {
  const next = arr.length >= max ? arr.slice(1) : arr.slice();
  next.push(v);
  return next;
}

export const useLive = create<LiveState>((set) => ({
  // Initial state
  console: {},
  players: {},
  freezes: {},
  metrics: {},
  pregenJobs: {},
  health: {},

  // Console actions with rolling buffer
  appendConsole: (id: string, lines: Line[]) => {
    rafBatch(() => {
      set((state) => {
        const currentLines = state.console[id] || [];
        const newLines = [...currentLines, ...lines].slice(-5000); // Keep last 5000 lines
        return {
          console: {
            ...state.console,
            [id]: newLines,
          },
        };
      });
    });
  },

  updatePlayers: (id: string, players: Player[]) => {
    rafBatch(() => {
      set((state) => ({
        players: {
          ...state.players,
          [id]: players,
        },
      }));
    });
  },

  updateFreezes: (id: string, freezes: FreezeTicket[]) => {
    rafBatch(() => {
      set((state) => ({
        freezes: {
          ...state.freezes,
          [id]: freezes,
        },
      }));
    });
  },

  // Metrics actions with time-series management
  applyMetrics: (id: string, m: { 
    tps: number; 
    tick_p95_ms: number; 
    heap_mb: number; 
    gpu_queue_ms: number; 
    players_online: number 
  }) => {
    rafBatch(() => {
      set((state) => {
        const currentMetrics = state.metrics[id] || { 
          tps: [], 
          p95: [], 
          heap: [], 
          gpu: [], 
          ppl: [] 
        };

        const updatedMetrics = {
          tps: pushWindow(currentMetrics.tps, m.tps),
          p95: pushWindow(currentMetrics.p95, m.tick_p95_ms),
          heap: pushWindow(currentMetrics.heap, m.heap_mb),
          gpu: pushWindow(currentMetrics.gpu, m.gpu_queue_ms),
          ppl: pushWindow(currentMetrics.ppl, m.players_online),
        };

        return {
          metrics: {
            ...state.metrics,
            [id]: updatedMetrics,
          },
        };
      });
    });
  },

  updatePregenJobs: (id: string, jobs: PregenJob[]) => {
    rafBatch(() => {
      set((state) => ({
        pregenJobs: {
          ...state.pregenJobs,
          [id]: jobs,
        },
      }));
    });
  },

  updateHealth: (id: string, health: ServerHealth) => {
    rafBatch(() => {
      set((state) => ({
        health: {
          ...state.health,
          [id]: health,
        },
      }));
    });
  },

  clearConsole: (id: string) => {
    set((state) => ({
      console: {
        ...state.console,
        [id]: [],
      },
    }));
  },
}));

// Performance-optimized selectors
export const useConsoleStream = (serverId: string) => {
  return useLive((state) => state.console[serverId] || []);
};

export const usePlayerData = (serverId: string) => {
  return useLive((state) => state.players[serverId] || []);
};

export const useWorldData = (serverId: string) => {
  return useLive((state) => state.freezes[serverId] || []);
};

export const useMetrics = (serverId: string) => {
  return useLive((state) => state.metrics[serverId] || null);
};

export const usePregenJobs = (serverId: string) => {
  return useLive((state) => state.pregenJobs[serverId] || []);
};

export const useServerHealth = (serverId: string) => {
  return useLive((state) => state.health[serverId] || null);
};
