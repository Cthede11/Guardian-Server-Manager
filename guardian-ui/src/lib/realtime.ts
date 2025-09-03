import { socketManager } from './socket';
import type { ConsoleMessage, Player, FreezeTicket, PregenJob, ServerSummary, Event, Snapshot, ModInfo, Rule, Shard } from './types';

// Real-time data types
export interface RealtimeMetrics {
  tps: number;
  tickP95: number;
  heapMb: number;
  playersOnline: number;
  gpuQueueMs: number;
  lastSnapshotAt: string | null;
  blueGreen: {
    active: 'blue' | 'green';
    candidateHealthy: boolean;
  };
}

export interface RealtimeHealth {
  rcon: boolean;
  query: boolean;
  crashTickets: number;
  freezeTickets: number;
}

export interface RealtimeWorldData {
  freezes: FreezeTicket[];
  heatmap: {
    x: number;
    z: number;
    intensity: number;
  }[];
  chunks: {
    x: number;
    z: number;
    loaded: boolean;
    entities: number;
    tileEntities: number;
  }[];
}

export interface RealtimePerformanceData {
  tps: number[];
  tickP95: number[];
  heapMb: number[];
  gpuQueueMs: number[];
  phaseBreakdown: {
    entity: number;
    tileEntity: number;
    block: number;
    fluid: number;
    chunk: number;
  };
}

export interface RealtimeBackupData {
  snapshots: Snapshot[];
  activeBackup: {
    id: string;
    progress: number;
    status: 'running' | 'completed' | 'failed';
  } | null;
}

export interface RealtimeEventData {
  events: Event[];
  activeEvents: Event[];
}

export interface RealtimePregenData {
  jobs: PregenJob[];
  activeJob: PregenJob | null;
  stats: {
    totalChunks: number;
    completedChunks: number;
    chunksPerSecond: number;
  };
}

export interface RealtimeShardingData {
  assignments: ShardAssignment[];
  topology: {
    nodes: Array<{
      id: string;
      x: number;
      y: number;
      status: 'active' | 'inactive' | 'error';
      playerCount: number;
    }>;
    connections: Array<{
      from: string;
      to: string;
      weight: number;
    }>;
  };
}

export interface RealtimeDiagnosticsData {
  crashSignatures: CrashSignature[];
  systemHealth: {
    cpu: number;
    memory: number;
    disk: number;
    network: number;
  };
  alerts: Array<{
    id: string;
    type: 'warning' | 'error' | 'info';
    message: string;
    timestamp: string;
  }>;
}

export interface RealtimeModsData {
  mods: Mod[];
  conflicts: Array<{
    id: string;
    mods: string[];
    severity: 'low' | 'medium' | 'high';
    description: string;
  }>;
  rules: Rule[];
}

// Subscription manager class
export class RealtimeSubscriptionManager {
  private subscriptions: Map<string, () => void> = new Map();
  private serverId: string | null = null;

  constructor(serverId?: string) {
    this.serverId = serverId || null;
  }

  // Generic subscription method
  private subscribe<T>(
    eventName: string,
    callback: (data: T) => void,
    key?: string
  ): () => void {
    const subscriptionKey = key || `${eventName}_${Date.now()}`;
    
    const unsubscribe = socketManager.subscribe(eventName, callback);
    this.subscriptions.set(subscriptionKey, unsubscribe);
    
    return () => {
      unsubscribe();
      this.subscriptions.delete(subscriptionKey);
    };
  }

  // Server-specific subscription method
  private subscribeToServer<T>(
    eventName: string,
    callback: (data: T) => void,
    key?: string
  ): () => void {
    if (!this.serverId) {
      throw new Error('Server ID is required for server-specific subscriptions');
    }
    
    const fullEventName = `${eventName}:${this.serverId}`;
    return this.subscribe(fullEventName, callback, key);
  }

  // Console subscriptions
  subscribeToConsole(callback: (message: ConsoleMessage) => void): () => void {
    return this.subscribeToServer('console', callback, 'console');
  }

  // Metrics subscriptions
  subscribeToMetrics(callback: (metrics: RealtimeMetrics) => void): () => void {
    return this.subscribeToServer('metrics', callback, 'metrics');
  }

  subscribeToHealth(callback: (health: RealtimeHealth) => void): () => void {
    return this.subscribeToServer('health', callback, 'health');
  }

  // Player subscriptions
  subscribeToPlayers(callback: (players: Player[]) => void): () => void {
    return this.subscribeToServer('players', callback, 'players');
  }

  // World subscriptions
  subscribeToWorld(callback: (worldData: RealtimeWorldData) => void): () => void {
    return this.subscribeToServer('world', callback, 'world');
  }

  // Performance subscriptions
  subscribeToPerformance(callback: (performance: RealtimePerformanceData) => void): () => void {
    return this.subscribeToServer('performance', callback, 'performance');
  }

  // Backup subscriptions
  subscribeToBackups(callback: (backupData: RealtimeBackupData) => void): () => void {
    return this.subscribeToServer('backups', callback, 'backups');
  }

  // Event subscriptions
  subscribeToEvents(callback: (eventData: RealtimeEventData) => void): () => void {
    return this.subscribeToServer('events', callback, 'events');
  }

  // Pregen subscriptions
  subscribeToPregen(callback: (pregenData: RealtimePregenData) => void): () => void {
    return this.subscribeToServer('pregen', callback, 'pregen');
  }

  // Mods and Rules subscriptions
  subscribeToMods(callback: (modsData: RealtimeModsData) => void): () => void {
    return this.subscribeToServer('mods', callback, 'mods');
  }

  // Diagnostics subscriptions
  subscribeToDiagnostics(callback: (diagnostics: RealtimeDiagnosticsData) => void): () => void {
    return this.subscribeToServer('diagnostics', callback, 'diagnostics');
  }

  // Global subscriptions (not server-specific)
  subscribeToServers(callback: (servers: ServerSummary[]) => void): () => void {
    return this.subscribe('servers', callback, 'servers');
  }

  subscribeToSharding(callback: (shardingData: RealtimeShardingData) => void): () => void {
    return this.subscribe('sharding', callback, 'sharding');
  }

  subscribeToWorkspace(callback: (workspaceData: any) => void): () => void {
    return this.subscribe('workspace', callback, 'workspace');
  }

  // Batch subscription methods
  subscribeToAllServerData(callbacks: {
    console?: (message: ConsoleMessage) => void;
    metrics?: (metrics: RealtimeMetrics) => void;
    health?: (health: RealtimeHealth) => void;
    players?: (players: Player[]) => void;
    world?: (worldData: RealtimeWorldData) => void;
    performance?: (performance: RealtimePerformanceData) => void;
    backups?: (backupData: RealtimeBackupData) => void;
    events?: (eventData: RealtimeEventData) => void;
    pregen?: (pregenData: RealtimePregenData) => void;
    mods?: (modsData: RealtimeModsData) => void;
    diagnostics?: (diagnostics: RealtimeDiagnosticsData) => void;
  }): () => void {
    const unsubscribers: (() => void)[] = [];

    if (callbacks.console) {
      unsubscribers.push(this.subscribeToConsole(callbacks.console));
    }
    if (callbacks.metrics) {
      unsubscribers.push(this.subscribeToMetrics(callbacks.metrics));
    }
    if (callbacks.health) {
      unsubscribers.push(this.subscribeToHealth(callbacks.health));
    }
    if (callbacks.players) {
      unsubscribers.push(this.subscribeToPlayers(callbacks.players));
    }
    if (callbacks.world) {
      unsubscribers.push(this.subscribeToWorld(callbacks.world));
    }
    if (callbacks.performance) {
      unsubscribers.push(this.subscribeToPerformance(callbacks.performance));
    }
    if (callbacks.backups) {
      unsubscribers.push(this.subscribeToBackups(callbacks.backups));
    }
    if (callbacks.events) {
      unsubscribers.push(this.subscribeToEvents(callbacks.events));
    }
    if (callbacks.pregen) {
      unsubscribers.push(this.subscribeToPregen(callbacks.pregen));
    }
    if (callbacks.mods) {
      unsubscribers.push(this.subscribeToMods(callbacks.mods));
    }
    if (callbacks.diagnostics) {
      unsubscribers.push(this.subscribeToDiagnostics(callbacks.diagnostics));
    }

    return () => {
      unsubscribers.forEach(unsubscribe => unsubscribe());
    };
  }

  // Cleanup all subscriptions
  unsubscribeAll(): void {
    this.subscriptions.forEach(unsubscribe => unsubscribe());
    this.subscriptions.clear();
  }

  // Get subscription count
  getSubscriptionCount(): number {
    return this.subscriptions.size;
  }

  // Check if subscribed to specific event
  isSubscribed(key: string): boolean {
    return this.subscriptions.has(key);
  }
}

// Factory function to create subscription manager
export const createRealtimeManager = (serverId?: string): RealtimeSubscriptionManager => {
  return new RealtimeSubscriptionManager(serverId);
};

// Global subscription manager for workspace-level data
export const globalRealtimeManager = createRealtimeManager();

// Helper functions for common real-time operations
export const realtimeHelpers = {
  // Format TPS for display
  formatTPS: (tps: number): string => {
    if (tps >= 20) return `${tps.toFixed(1)}`;
    if (tps >= 15) return `${tps.toFixed(1)} (Good)`;
    if (tps >= 10) return `${tps.toFixed(1)} (Fair)`;
    return `${tps.toFixed(1)} (Poor)`;
  },

  // Format memory for display
  formatMemory: (mb: number): string => {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)}GB`;
    }
    return `${mb.toFixed(0)}MB`;
  },

  // Format time for display
  formatTime: (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  },

  // Calculate health score
  calculateHealthScore: (health: RealtimeHealth): number => {
    let score = 0;
    if (health.rcon) score += 50;
    if (health.query) score += 50;
    if (health.crashTickets === 0) score += 20;
    if (health.freezeTickets === 0) score += 20;
    return Math.min(score, 100);
  },

  // Get health status
  getHealthStatus: (health: RealtimeHealth): 'healthy' | 'warning' | 'critical' => {
    const score = realtimeHelpers.calculateHealthScore(health);
    if (score >= 90) return 'healthy';
    if (score >= 70) return 'warning';
    return 'critical';
  },

  // Calculate performance score
  calculatePerformanceScore: (metrics: RealtimeMetrics): number => {
    let score = 100;
    
    // TPS penalty
    if (metrics.tps < 20) {
      score -= (20 - metrics.tps) * 2;
    }
    
    // Tick P95 penalty
    if (metrics.tickP95 > 50) {
      score -= (metrics.tickP95 - 50) * 0.5;
    }
    
    // Memory penalty
    if (metrics.heapMb > 4096) {
      score -= (metrics.heapMb - 4096) * 0.01;
    }
    
    // GPU queue penalty
    if (metrics.gpuQueueMs > 16) {
      score -= (metrics.gpuQueueMs - 16) * 2;
    }
    
    return Math.max(0, Math.min(100, score));
  },

  // Get performance status
  getPerformanceStatus: (metrics: RealtimeMetrics): 'excellent' | 'good' | 'fair' | 'poor' => {
    const score = realtimeHelpers.calculatePerformanceScore(metrics);
    if (score >= 90) return 'excellent';
    if (score >= 75) return 'good';
    if (score >= 50) return 'fair';
    return 'poor';
  },

  // Format player count
  formatPlayerCount: (count: number, max?: number): string => {
    if (max !== undefined) {
      return `${count}/${max}`;
    }
    return count.toString();
  },

  // Calculate chunk loading progress
  calculateChunkProgress: (completed: number, total: number): number => {
    if (total === 0) return 0;
    return Math.round((completed / total) * 100);
  },

  // Format chunk progress
  formatChunkProgress: (completed: number, total: number): string => {
    const progress = realtimeHelpers.calculateChunkProgress(completed, total);
    return `${progress}% (${completed.toLocaleString()}/${total.toLocaleString()})`;
  },

  // Get connection status
  getConnectionStatus: (): 'connected' | 'connecting' | 'disconnected' | 'error' => {
    const connectionType = socketManager.getConnectionType();
    switch (connectionType) {
      case 'socket':
      case 'sse':
        return 'connected';
      case 'disconnected':
        return 'disconnected';
      default:
        return 'error';
    }
  },

  // Get connection type display
  getConnectionTypeDisplay: (): string => {
    const connectionType = socketManager.getConnectionType();
    switch (connectionType) {
      case 'socket':
        return 'WebSocket';
      case 'sse':
        return 'Server-Sent Events';
      case 'disconnected':
        return 'Disconnected';
      default:
        return 'Unknown';
    }
  }
};

// Export types for use in components
export type {
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
};
