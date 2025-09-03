import React, { useEffect, useRef, useState } from 'react';
import { useRealtimeStore, useRealtimeConnection, useRealtimeGlobalSubscription } from '@/store/realtime';
import { RealtimeConnectionStatus } from './RealtimeConnectionStatus';

interface RealtimeDataProviderProps {
  children: React.ReactNode;
  serverId?: string;
  autoConnect?: boolean;
  showConnectionStatus?: boolean;
  onConnectionChange?: (isConnected: boolean) => void;
  onDataUpdate?: (data: any) => void;
}

export const RealtimeDataProvider: React.FC<RealtimeDataProviderProps> = ({
  children,
  serverId,
  autoConnect = true,
  showConnectionStatus = false,
  onConnectionChange,
  onDataUpdate
}) => {
  const { isConnected, connectionType } = useRealtimeConnection();
  const { connect, disconnect } = useRealtimeStore();
  const { unsubscribe: unsubscribeGlobal } = useRealtimeGlobalSubscription();
  const [isInitialized, setIsInitialized] = useState(false);
  const connectionIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const dataUpdateIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Initialize connection
  useEffect(() => {
    if (autoConnect && !isInitialized) {
      connect();
      setIsInitialized(true);
    }

    return () => {
      if (connectionIntervalRef.current) {
        clearInterval(connectionIntervalRef.current);
      }
      if (dataUpdateIntervalRef.current) {
        clearInterval(dataUpdateIntervalRef.current);
      }
    };
  }, [autoConnect, isInitialized, connect]);

  // Handle connection state changes
  useEffect(() => {
    if (onConnectionChange) {
      onConnectionChange(isConnected);
    }
  }, [isConnected, onConnectionChange]);

  // Set up data update monitoring
  useEffect(() => {
    if (onDataUpdate && isConnected) {
      dataUpdateIntervalRef.current = setInterval(() => {
        const store = useRealtimeStore.getState();
        onDataUpdate(store);
      }, 1000);
    }

    return () => {
      if (dataUpdateIntervalRef.current) {
        clearInterval(dataUpdateIntervalRef.current);
      }
    };
  }, [isConnected, onDataUpdate]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (unsubscribeGlobal) {
        unsubscribeGlobal();
      }
      disconnect();
    };
  }, [unsubscribeGlobal, disconnect]);

  return (
    <div className="relative">
      {showConnectionStatus && (
        <div className="absolute top-4 right-4 z-50">
          <RealtimeConnectionStatus compact />
        </div>
      )}
      {children}
    </div>
  );
};

// Server-specific data provider
interface ServerRealtimeProviderProps {
  children: React.ReactNode;
  serverId: string;
  autoConnect?: boolean;
  onServerDataUpdate?: (data: any) => void;
}

export const ServerRealtimeProvider: React.FC<ServerRealtimeProviderProps> = ({
  children,
  serverId,
  autoConnect = true,
  onServerDataUpdate
}) => {
  const { selectServer, getServerData } = useRealtimeStore();
  const [isSubscribed, setIsSubscribed] = useState(false);

  // Select server and set up subscriptions
  useEffect(() => {
    if (serverId) {
      selectServer(serverId);
      
      if (autoConnect) {
        // Set up server-specific subscriptions
        const { createRealtimeManager } = require('@/lib/realtime');
        const manager = createRealtimeManager(serverId);
        
        const unsubscribe = manager.subscribeToAllServerData({
          console: (message) => {
            // Console messages are handled by the store
          },
          metrics: (metrics) => {
            // Metrics are handled by the store
          },
          health: (health) => {
            // Health data is handled by the store
          },
          players: (players) => {
            // Player data is handled by the store
          },
          world: (world) => {
            // World data is handled by the store
          },
          performance: (performance) => {
            // Performance data is handled by the store
          },
          backups: (backups) => {
            // Backup data is handled by the store
          },
          events: (events) => {
            // Event data is handled by the store
          },
          pregen: (pregen) => {
            // Pregen data is handled by the store
          },
          mods: (mods) => {
            // Mods data is handled by the store
          },
          diagnostics: (diagnostics) => {
            // Diagnostics data is handled by the store
          },
        });
        
        setIsSubscribed(true);
        
        return () => {
          unsubscribe();
          setIsSubscribed(false);
        };
      }
    }
  }, [serverId, autoConnect, selectServer]);

  // Monitor server data changes
  useEffect(() => {
    if (onServerDataUpdate && isSubscribed) {
      const interval = setInterval(() => {
        const serverData = getServerData(serverId);
        if (serverData) {
          onServerDataUpdate(serverData);
        }
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [serverId, isSubscribed, onServerDataUpdate, getServerData]);

  return <>{children}</>;
};

// Workspace-level data provider
interface WorkspaceRealtimeProviderProps {
  children: React.ReactNode;
  autoConnect?: boolean;
  onWorkspaceDataUpdate?: (data: any) => void;
}

export const WorkspaceRealtimeProvider: React.FC<WorkspaceRealtimeProviderProps> = ({
  children,
  autoConnect = true,
  onWorkspaceDataUpdate
}) => {
  const { updateSharding, updateWorkspace } = useRealtimeStore();
  const [isSubscribed, setIsSubscribed] = useState(false);

  // Set up workspace-level subscriptions
  useEffect(() => {
    if (autoConnect) {
      const { createRealtimeManager } = require('@/lib/realtime');
      const manager = createRealtimeManager();
      
      const unsubscribe = manager.subscribeToAllServerData({
        // Global subscriptions
      });
      
      // Add workspace-specific subscriptions
      const unsubscribeSharding = manager.subscribe('sharding', updateSharding);
      const unsubscribeWorkspace = manager.subscribe('workspace', updateWorkspace);
      
      setIsSubscribed(true);
      
      return () => {
        unsubscribe();
        unsubscribeSharding();
        unsubscribeWorkspace();
        setIsSubscribed(false);
      };
    }
  }, [autoConnect, updateSharding, updateWorkspace]);

  // Monitor workspace data changes
  useEffect(() => {
    if (onWorkspaceDataUpdate && isSubscribed) {
      const interval = setInterval(() => {
        const store = useRealtimeStore.getState();
        onWorkspaceDataUpdate({
          sharding: store.sharding,
          workspace: store.workspace,
        });
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [isSubscribed, onWorkspaceDataUpdate]);

  return <>{children}</>;
};

// Hook for managing real-time subscriptions
export const useRealtimeSubscription = (serverId?: string) => {
  const [isSubscribed, setIsSubscribed] = useState(false);
  const [subscriptionManager, setSubscriptionManager] = useState<any>(null);

  useEffect(() => {
    if (serverId) {
      const { createRealtimeManager } = require('@/lib/realtime');
      const manager = createRealtimeManager(serverId);
      setSubscriptionManager(manager);
      setIsSubscribed(true);
    }

    return () => {
      if (subscriptionManager) {
        subscriptionManager.unsubscribeAll();
        setSubscriptionManager(null);
        setIsSubscribed(false);
      }
    };
  }, [serverId]);

  const subscribe = (eventName: string, callback: (data: any) => void) => {
    if (subscriptionManager) {
      return subscriptionManager.subscribe(eventName, callback);
    }
    return () => {};
  };

  const unsubscribe = (eventName: string, callback: (data: any) => void) => {
    if (subscriptionManager) {
      subscriptionManager.unsubscribe(eventName, callback);
    }
  };

  return {
    isSubscribed,
    subscribe,
    unsubscribe,
    manager: subscriptionManager,
  };
};

// Hook for real-time data with automatic updates
export const useRealtimeData = <T>(
  serverId: string,
  dataType: keyof ReturnType<typeof useRealtimeStore.getState>['serverData'][string]
): T | null => {
  const { serverData } = useRealtimeStore();
  const [data, setData] = useState<T | null>(null);

  useEffect(() => {
    const serverDataForId = serverData[serverId];
    if (serverDataForId && serverDataForId[dataType]) {
      setData(serverDataForId[dataType] as T);
    }
  }, [serverId, dataType, serverData]);

  return data;
};

// Export all providers and hooks
export {
  RealtimeDataProvider,
  ServerRealtimeProvider,
  WorkspaceRealtimeProvider,
  useRealtimeSubscription,
  useRealtimeData,
};
