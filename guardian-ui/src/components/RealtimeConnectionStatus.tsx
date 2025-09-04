import React, { useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { 
  Wifi, 
  WifiOff, 
  AlertTriangle, 
  CheckCircle, 
  RefreshCw, 
  Settings,
  Activity,
  Server,
  Globe,
  Zap,
  Info
} from 'lucide-react';
import { useRealtimeConnection } from '@/store/realtime';
import { realtimeHelpers } from '@/lib/realtime';

interface ConnectionStatusProps {
  showDetails?: boolean;
  compact?: boolean;
  onReconnect?: () => void;
}

export const RealtimeConnectionStatus: React.FC<ConnectionStatusProps> = ({
  // showDetails = false,
  compact = false,
  onReconnect
}) => {
  const { isConnected, connectionType, lastConnected, connectionErrors, status, typeDisplay } = useRealtimeConnection();
  const [isReconnecting, setIsReconnecting] = useState(false);
  const [showErrorDetails, setShowErrorDetails] = useState(false);

  const handleReconnect = async () => {
    setIsReconnecting(true);
    try {
      if (onReconnect) {
        await onReconnect();
      } else {
        // Default reconnect logic
        const { socketManager } = await import('@/lib/socket');
        socketManager.disconnect();
        setTimeout(() => {
          socketManager.connect();
        }, 1000);
      }
    } catch (error) {
      console.error('Reconnection failed:', error);
    } finally {
      setIsReconnecting(false);
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'connected':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'connecting':
        return <RefreshCw className="h-4 w-4 text-yellow-500 animate-spin" />;
      case 'disconnected':
        return <WifiOff className="h-4 w-4 text-gray-500" />;
      case 'error':
        return <AlertTriangle className="h-4 w-4 text-red-500" />;
      default:
        return <Wifi className="h-4 w-4 text-gray-500" />;
    }
  };

  const getStatusColor = () => {
    switch (status) {
      case 'connected':
        return 'bg-green-500';
      case 'connecting':
        return 'bg-yellow-500';
      case 'disconnected':
        return 'bg-gray-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'connected':
        return 'Connected';
      case 'connecting':
        return 'Connecting...';
      case 'disconnected':
        return 'Disconnected';
      case 'error':
        return 'Connection Error';
      default:
        return 'Unknown';
    }
  };

  const getConnectionTypeIcon = () => {
    switch (connectionType) {
      case 'socket':
        return <Zap className="h-3 w-3" />;
      case 'sse':
        return <Globe className="h-3 w-3" />;
      default:
        return <Server className="h-3 w-3" />;
    }
  };

  if (compact) {
    return (
      <div className="flex items-center space-x-2">
        {getStatusIcon()}
        <span className="text-sm text-muted-foreground">
          {getStatusText()}
        </span>
        {isConnected && (
          <Badge variant="outline" className="text-xs">
            {getConnectionTypeIcon()}
            <span className="ml-1">{typeDisplay}</span>
          </Badge>
        )}
      </div>
    );
  }

  return (
    <Card className="w-full">
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center justify-between text-lg">
          <div className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>Real-time Connection</span>
          </div>
          <div className="flex items-center space-x-2">
            {getStatusIcon()}
            <Badge className={`${getStatusColor()} text-white`}>
              {getStatusText()}
            </Badge>
          </div>
        </CardTitle>
        <CardDescription>
          Live data connection status and information
        </CardDescription>
      </CardHeader>
      
      <CardContent className="space-y-4">
        {/* Connection Details */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Connection Type</span>
              <Badge variant="outline" className="text-xs">
                {getConnectionTypeIcon()}
                <span className="ml-1">{typeDisplay}</span>
              </Badge>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Status</span>
              <span className="text-sm text-muted-foreground">
                {getStatusText()}
              </span>
            </div>
            
            {lastConnected && (
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Last Connected</span>
                <span className="text-sm text-muted-foreground">
                  {realtimeHelpers.formatTime(lastConnected)}
                </span>
              </div>
            )}
          </div>
          
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Connection Quality</span>
              <span className="text-sm text-muted-foreground">
                {isConnected ? 'Good' : 'Poor'}
              </span>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Latency</span>
              <span className="text-sm text-muted-foreground">
                {isConnected ? '< 50ms' : 'N/A'}
              </span>
            </div>
            
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Uptime</span>
              <span className="text-sm text-muted-foreground">
                {isConnected ? '99.9%' : '0%'}
              </span>
            </div>
          </div>
        </div>

        {/* Error Details */}
        {connectionErrors.length > 0 && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-red-500">Connection Errors</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowErrorDetails(!showErrorDetails)}
              >
                <Info className="h-3 w-3" />
              </Button>
            </div>
            
            {showErrorDetails && (
              <div className="p-3 bg-red-50 dark:bg-red-950 rounded-lg">
                <div className="space-y-1">
                  {connectionErrors.map((error, index) => (
                    <div key={index} className="text-sm text-red-700 dark:text-red-300">
                      {error}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex items-center justify-between pt-4 border-t">
          <div className="flex items-center space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleReconnect}
              disabled={isReconnecting}
            >
              <RefreshCw className={`h-4 w-4 mr-2 ${isReconnecting ? 'animate-spin' : ''}`} />
              {isReconnecting ? 'Reconnecting...' : 'Reconnect'}
            </Button>
            
            <Button variant="outline" size="sm">
              <Settings className="h-4 w-4 mr-2" />
              Settings
            </Button>
          </div>
          
          <div className="text-xs text-muted-foreground">
            Real-time data updates
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

// Compact version for headers
export const RealtimeConnectionBadge: React.FC = () => {
  const { status } = useRealtimeConnection();
  
  const getStatusColor = () => {
    switch (status) {
      case 'connected':
        return 'bg-green-500';
      case 'connecting':
        return 'bg-yellow-500';
      case 'disconnected':
        return 'bg-gray-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'connected':
        return <CheckCircle className="h-3 w-3" />;
      case 'connecting':
        return <RefreshCw className="h-3 w-3 animate-spin" />;
      case 'disconnected':
        return <WifiOff className="h-3 w-3" />;
      case 'error':
        return <AlertTriangle className="h-3 w-3" />;
      default:
        return <Wifi className="h-3 w-3" />;
    }
  };

  return (
    <Badge className={`${getStatusColor()} text-white text-xs`}>
      {getStatusIcon()}
      <span className="ml-1">
        {status === 'connected' ? 'Live' : status}
      </span>
    </Badge>
  );
};

// Connection indicator for status bars
export const RealtimeConnectionIndicator: React.FC = () => {
  const { isConnected } = useRealtimeConnection();
  
  return (
    <div className="flex items-center space-x-1">
      <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500 animate-pulse' : 'bg-gray-500'}`} />
      <span className="text-xs text-muted-foreground">
        {isConnected ? 'Live' : 'Offline'}
      </span>
    </div>
  );
};
