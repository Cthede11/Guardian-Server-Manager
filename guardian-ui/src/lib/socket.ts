import { io, Socket } from 'socket.io-client';
import { liveStore } from '../store/live';
import { useServers } from '../store/servers-new';

export type SocketEvent = {
  type: string;
  data: any;
  timestamp: string;
};

// rAF batcher for preventing re-render storms
let updateQueue: Array<() => void> = [];
let scheduled = false;

function batchUpdate(fn: () => void) {
  updateQueue.push(fn);
  if (!scheduled) {
    scheduled = true;
    requestAnimationFrame(() => {
      const queue = updateQueue;
      updateQueue = [];
      scheduled = false;
      
      // Execute all updates in a single batch
      liveStore.getState().batch(queue);
    });
  }
}

class SocketManager {
  private socket: Socket | null = null;
  private eventSource: EventSource | null = null;
  private useSSE: boolean = false;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();

  constructor() {
    // Check if we should use SSE fallback
    this.useSSE = import.meta.env.VITE_USE_SSE === 'true' || !window.WebSocket;
  }

  async connect(serverUrl: string = '') {
    // Get the current API base URL dynamically if not provided
    if (!serverUrl) {
      const { getAPI_BASE } = await import('./api');
      serverUrl = await getAPI_BASE();
    }
    
    if (this.useSSE) {
      this.connectSSE(serverUrl);
    } else {
      this.connectSocket(serverUrl);
    }
  }

  private connectSocket(serverUrl: string) {
    this.socket = io(serverUrl, {
      transports: ['websocket'],
      path: '/ws',
      timeout: 20000,
    });

    this.socket.on('connect', () => {
      console.log('Socket.IO connected');
      batchUpdate(() => {
        liveStore.getState().setConnected(true);
        liveStore.getState().setConnectionType('socket');
      });
    });

    this.socket.on('disconnect', () => {
      console.log('Socket.IO disconnected');
      batchUpdate(() => {
        liveStore.getState().setConnected(false);
        liveStore.getState().setConnectionType('disconnected');
      });
    });

    this.socket.on('error', (error) => {
      console.error('Socket.IO error:', error);
      // Fallback to SSE on error
      this.fallbackToSSE(serverUrl).catch(console.error);
    });

    // Handle server-specific events with batching
    this.socket.on('metrics', (payload: { serverId: string; data: any }) => {
      const selectedId = useServers.getState().selectedServerId;
      if (payload.serverId === selectedId) {
        batchUpdate(() => {
          liveStore.getState().applyMetrics(payload.serverId, payload.data);
        });
      }
    });

    this.socket.on('console', (payload: { serverId: string; lines: any[] }) => {
      const selectedId = useServers.getState().selectedServerId;
      if (payload.serverId === selectedId) {
        batchUpdate(() => {
          liveStore.getState().appendConsole(payload.serverId, payload.lines);
        });
      }
    });

    this.socket.on('players', (payload: { serverId: string; players: any[] }) => {
      const selectedId = useServers.getState().selectedServerId;
      if (payload.serverId === selectedId) {
        batchUpdate(() => {
          liveStore.getState().updatePlayers(payload.serverId, payload.players);
        });
      }
    });

    this.socket.on('freezes', (payload: { serverId: string; freezes: any[] }) => {
      const selectedId = useServers.getState().selectedServerId;
      if (payload.serverId === selectedId) {
        batchUpdate(() => {
          liveStore.getState().updateFreezes(payload.serverId, payload.freezes);
        });
      }
    });

    this.socket.on('pregen', (payload: { serverId: string; jobs: any[] }) => {
      const selectedId = useServers.getState().selectedServerId;
      if (payload.serverId === selectedId) {
        batchUpdate(() => {
          liveStore.getState().updatePregenJobs(payload.serverId, payload.jobs);
        });
      }
    });

    // Forward other events to listeners
    this.socket.onAny((eventName, data) => {
      this.emitToListeners(eventName, data);
    });
  }

  private async connectSSE(serverUrl: string) {
    // Get the current API base URL dynamically if not provided
    if (!serverUrl) {
      const { getAPI_BASE } = await import('./api');
      serverUrl = await getAPI_BASE();
    }
    this.eventSource = new EventSource(`${serverUrl}/events`);

    this.eventSource.onopen = () => {
      console.log('SSE connected');
      batchUpdate(() => {
        liveStore.getState().setConnected(true);
        liveStore.getState().setConnectionType('sse');
      });
    };

    this.eventSource.onerror = (error) => {
      console.error('SSE error:', error);
      batchUpdate(() => {
        liveStore.getState().setConnected(false);
        liveStore.getState().setConnectionType('disconnected');
      });
    };

    this.eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        
        // Handle server-specific events with batching
        if (data.type === 'metrics' && data.serverId) {
          const selectedId = useServers.getState().selectedServerId;
          if (data.serverId === selectedId) {
            batchUpdate(() => {
              liveStore.getState().applyMetrics(data.serverId, data.data);
            });
          }
        } else if (data.type === 'console' && data.serverId) {
          const selectedId = useServers.getState().selectedServerId;
          if (data.serverId === selectedId) {
            batchUpdate(() => {
              liveStore.getState().appendConsole(data.serverId, data.lines);
            });
          }
        } else if (data.type === 'players' && data.serverId) {
          const selectedId = useServers.getState().selectedServerId;
          if (data.serverId === selectedId) {
            batchUpdate(() => {
              liveStore.getState().updatePlayers(data.serverId, data.players);
            });
          }
        } else {
          // Forward other events to listeners
          this.emitToListeners(data.type, data.data);
        }
      } catch (error) {
        console.error('Failed to parse SSE message:', error);
      }
    };
  }

  private async fallbackToSSE(serverUrl: string) {
    console.log('Falling back to SSE');
    this.disconnect();
    this.useSSE = true;
    await this.connectSSE(serverUrl);
  }

  disconnect() {
    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
    }
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
    
    batchUpdate(() => {
      liveStore.getState().setConnected(false);
      liveStore.getState().setConnectionType('disconnected');
    });
  }

  subscribe(eventName: string, callback: (data: any) => void) {
    if (!this.listeners.has(eventName)) {
      this.listeners.set(eventName, new Set());
    }
    this.listeners.get(eventName)!.add(callback);

    // If using Socket.IO, join the room
    if (this.socket && !this.useSSE) {
      this.socket.emit('subscribe', eventName);
    }

    return () => this.unsubscribe(eventName, callback);
  }

  unsubscribe(eventName: string, callback: (data: any) => void) {
    const listeners = this.listeners.get(eventName);
    if (listeners) {
      listeners.delete(callback);
      if (listeners.size === 0) {
        this.listeners.delete(eventName);
        // If using Socket.IO, leave the room
        if (this.socket && !this.useSSE) {
          this.socket.emit('unsubscribe', eventName);
        }
      }
    }
  }

  private emitToListeners(eventName: string, data: any) {
    const listeners = this.listeners.get(eventName);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error('Error in event listener:', error);
        }
      });
    }
  }

  // Convenience methods for common subscriptions
  subscribeToConsole(serverId: string, callback: (message: any) => void) {
    return this.subscribe(`console:${serverId}`, callback);
  }

  subscribeToMetrics(serverId: string, callback: (metrics: any) => void) {
    return this.subscribe(`metrics:${serverId}`, callback);
  }

  subscribeToServerStatus(serverId: string, callback: (status: any) => void) {
    return this.subscribe(`server:${serverId}:status`, callback);
  }

  subscribeToPlayers(serverId: string, callback: (players: any) => void) {
    return this.subscribe(`players:${serverId}`, callback);
  }

  subscribeToWorld(serverId: string, callback: (world: any) => void) {
    return this.subscribe(`world:${serverId}`, callback);
  }

  subscribeToPerformance(serverId: string, callback: (performance: any) => void) {
    return this.subscribe(`performance:${serverId}`, callback);
  }

  subscribeToBackups(serverId: string, callback: (backups: any) => void) {
    return this.subscribe(`backups:${serverId}`, callback);
  }

  subscribeToEvents(serverId: string, callback: (events: any) => void) {
    return this.subscribe(`events:${serverId}`, callback);
  }

  subscribeToPregen(serverId: string, callback: (pregen: any) => void) {
    return this.subscribe(`pregen:${serverId}`, callback);
  }

  subscribeToSharding(callback: (sharding: any) => void) {
    return this.subscribe('sharding', callback);
  }

  subscribeToDiagnostics(serverId: string, callback: (diagnostics: any) => void) {
    return this.subscribe(`diagnostics:${serverId}`, callback);
  }

  isConnected(): boolean {
    if (this.useSSE) {
      return this.eventSource?.readyState === EventSource.OPEN;
    } else {
      return this.socket?.connected || false;
    }
  }

  getConnectionType(): 'socket' | 'sse' | 'disconnected' {
    if (this.useSSE) {
      return this.eventSource?.readyState === EventSource.OPEN ? 'sse' : 'disconnected';
    } else {
      return this.socket?.connected ? 'socket' : 'disconnected';
    }
  }
}

export const socketManager = new SocketManager();

// Export the connect function for easy initialization
export const connect = () => socketManager.connect();
export const disconnect = () => socketManager.disconnect();

export default socketManager;
