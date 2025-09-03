import { io, Socket } from 'socket.io-client';

export type SocketEvent = {
  type: string;
  data: any;
  timestamp: string;
};

class SocketManager {
  private socket: Socket | null = null;
  private eventSource: EventSource | null = null;
  private useSSE: boolean = false;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();

  constructor() {
    // Check if we should use SSE fallback
    this.useSSE = import.meta.env.VITE_USE_SSE === 'true' || !window.WebSocket;
  }

  connect(serverUrl: string = '') {
    if (this.useSSE) {
      this.connectSSE(serverUrl);
    } else {
      this.connectSocket(serverUrl);
    }
  }

  private connectSocket(serverUrl: string) {
    this.socket = io(serverUrl, {
      transports: ['websocket', 'polling'],
      timeout: 20000,
    });

    this.socket.on('connect', () => {
      console.log('Socket.IO connected');
    });

    this.socket.on('disconnect', () => {
      console.log('Socket.IO disconnected');
    });

    this.socket.on('error', (error) => {
      console.error('Socket.IO error:', error);
      // Fallback to SSE on error
      this.fallbackToSSE(serverUrl);
    });

    // Forward all events to our listeners
    this.socket.onAny((eventName, data) => {
      this.emitToListeners(eventName, data);
    });
  }

  private connectSSE(serverUrl: string) {
    this.eventSource = new EventSource(`${serverUrl}/events`);

    this.eventSource.onopen = () => {
      console.log('SSE connected');
    };

    this.eventSource.onerror = (error) => {
      console.error('SSE error:', error);
    };

    this.eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emitToListeners(data.type, data.data);
      } catch (error) {
        console.error('Failed to parse SSE message:', error);
      }
    };
  }

  private fallbackToSSE(serverUrl: string) {
    console.log('Falling back to SSE');
    this.disconnect();
    this.useSSE = true;
    this.connectSSE(serverUrl);
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
export default socketManager;
