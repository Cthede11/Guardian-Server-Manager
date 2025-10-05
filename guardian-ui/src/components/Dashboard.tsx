import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Activity, 
  Users, 
  MemoryStick, 
  Clock, 
  Server, 
  Play, 
  Square, 
  RotateCcw,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Loader2
} from 'lucide-react';
import { useServers } from '@/store/servers-new';
import { metricsCollector, type MetricData } from '@/lib/metrics-collector';
import { realtimeConnection } from '@/lib/websocket';
import { errorHandler } from '@/lib/error-handler';

interface DashboardProps {
  className?: string;
}

export const Dashboard: React.FC<DashboardProps> = ({ className }) => {
  const { summaries, selectedId, select, startServer, stopServer, restartServer } = useServers();
  const servers = Object.values(summaries);
  const selectedServerId = selectedId;
  const selectServer = select;
  const [metrics, setMetrics] = useState<MetricData | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [alerts, setAlerts] = useState<any[]>([]);

  useEffect(() => {
    if (selectedServerId) {
      // Start metrics collection
      metricsCollector.startCollection();
      
      // Connect to real-time updates
      realtimeConnection.connect().then(() => {
        setIsConnected(true);
      }).catch((error: any) => {
        errorHandler.handleError(error, 'WebSocket Connection');
      });

      // Set up real-time message handlers
      const unsubscribeMetrics = realtimeConnection.subscribe('metrics', (data: any) => {
        setMetrics(data);
      });

      const unsubscribeStatus = realtimeConnection.subscribe('server_status', (data: any) => {
        console.log('Server status update:', data);
      });

      return () => {
        metricsCollector.stopCollection();
        realtimeConnection.disconnect();
        unsubscribeMetrics();
        unsubscribeStatus();
        setIsConnected(false);
      };
    }
  }, [selectedServerId]);

  const selectedServer = servers.find(s => s.id === selectedServerId);

  const handleServerAction = async (action: 'start' | 'stop' | 'restart') => {
    if (!selectedServerId) return;

    try {
      switch (action) {
        case 'start':
          await startServer(selectedServerId);
          break;
        case 'stop':
          await stopServer(selectedServerId);
          break;
        case 'restart':
          await restartServer(selectedServerId);
          break;
      }
    } catch (error) {
      errorHandler.handleError(error as Error, `Server ${action}`, { serverId: selectedServerId });
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'text-green-500';
      case 'stopped': return 'text-red-500';
      case 'starting': return 'text-yellow-500';
      case 'stopping': return 'text-orange-500';
      default: return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running': return <CheckCircle className="h-4 w-4" />;
      case 'stopped': return <XCircle className="h-4 w-4" />;
      case 'starting': return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'stopping': return <Loader2 className="h-4 w-4 animate-spin" />;
      default: return <AlertTriangle className="h-4 w-4" />;
    }
  };

  if (!selectedServer) {
    return (
      <div className={`space-y-6 ${className}`}>
        <Card>
          <CardContent className="flex items-center justify-center h-64">
            <div className="text-center">
              <Server className="h-12 w-12 mx-auto text-gray-400 mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No Server Selected</h3>
              <p className="text-gray-500">Select a server from the sidebar to view its dashboard</p>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Server Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <Server className="h-6 w-6" />
              <div>
                <CardTitle className="text-xl">{selectedServer.name}</CardTitle>
                <div className="flex items-center space-x-2 mt-1">
                  <Badge variant="outline">{selectedServer.type || 'Unknown'}</Badge>
                  <Badge variant="outline">{selectedServer.version}</Badge>
                  <div className={`flex items-center space-x-1 ${getStatusColor(selectedServer.status)}`}>
                    {getStatusIcon(selectedServer.status)}
                    <span className="text-sm font-medium capitalize">{selectedServer.status}</span>
                  </div>
                </div>
              </div>
            </div>
            
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleServerAction('restart')}
                disabled={selectedServer.status === 'starting' || selectedServer.status === 'stopping'}
              >
                <RotateCcw className="h-4 w-4 mr-2" />
                Restart
              </Button>
              
              {selectedServer.status === 'running' ? (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => handleServerAction('stop')}
                  disabled={selectedServer.status !== 'running'}
                >
                  <Square className="h-4 w-4 mr-2" />
                  Stop
                </Button>
              ) : (
                <Button
                  size="sm"
                  onClick={() => handleServerAction('start')}
                  disabled={selectedServer.status !== 'stopped'}
                >
                  <Play className="h-4 w-4 mr-2" />
                  Start
                </Button>
              )}
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Connection Status */}
      {!isConnected && (
        <Alert>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Not connected to server. Real-time updates may not be available.
          </AlertDescription>
        </Alert>
      )}

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">TPS</p>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics?.tps ? metrics.tps.toFixed(1) : '0.0'}
                </p>
              </div>
              <Activity className="h-8 w-8 text-blue-500" />
            </div>
            <div className="mt-2">
              <Progress 
                value={metrics?.tps ? (metrics.tps / 20) * 100 : 0} 
                className="h-2"
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Players</p>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics?.playersOnline || 0}/{selectedServer?.maxPlayers || 20}
                </p>
              </div>
              <Users className="h-8 w-8 text-green-500" />
            </div>
            <div className="mt-2">
              <Progress 
                value={metrics?.playersOnline ? (metrics.playersOnline / (selectedServer?.maxPlayers || 20)) * 100 : 0} 
                className="h-2"
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Memory</p>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics?.heapMb ? `${Math.round(metrics.heapMb / 1024)}GB` : '0GB'}
                </p>
                <p className="text-xs text-gray-500">
                  {metrics?.memoryUsage ? `${Math.round(metrics.memoryUsage)}%` : '0%'} used
                </p>
              </div>
              <MemoryStick className="h-8 w-8 text-purple-500" />
            </div>
            <div className="mt-2">
              <Progress 
                value={metrics?.memoryUsage || 0} 
                className="h-2"
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Tick Time</p>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics?.tickP95 ? `${metrics.tickP95.toFixed(1)}ms` : '0ms'}
                </p>
              </div>
              <Clock className="h-8 w-8 text-orange-500" />
            </div>
            <div className="mt-2">
              <Progress 
                value={metrics?.tickP95 ? Math.min((metrics.tickP95 / 50) * 100, 100) : 0} 
                className="h-2"
              />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Performance Alerts */}
      {alerts.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              <span>Performance Alerts</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {alerts.map((alert) => (
                <Alert key={alert.id} variant={alert.severity === 'critical' ? 'destructive' : 'default'}>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>{alert.message}</AlertDescription>
                </Alert>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Server Information */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Server Information</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Type:</span>
              <span className="text-sm font-medium">{selectedServer.type || 'Unknown'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Version:</span>
              <span className="text-sm font-medium">{selectedServer.version}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Status:</span>
              <span className={`text-sm font-medium ${getStatusColor(selectedServer.status)}`}>
                {selectedServer.status}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Uptime:</span>
              <span className="text-sm font-medium">--</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Performance Information</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">GPU Queue:</span>
              <span className="text-sm font-medium">{metrics?.gpuQueueMs ? `${metrics.gpuQueueMs.toFixed(1)}ms` : '0ms'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">CPU Usage:</span>
              <span className="text-sm font-medium">{metrics?.cpuUsage ? `${metrics.cpuUsage.toFixed(1)}%` : '0%'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Disk Usage:</span>
              <span className="text-sm font-medium">{metrics?.diskUsage ? `${metrics.diskUsage.toFixed(1)}%` : '0%'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-600">Network In:</span>
              <span className="text-sm font-medium">
                {metrics?.networkIn ? `${(metrics.networkIn / 1024).toFixed(1)}KB/s` : '0KB/s'}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};
