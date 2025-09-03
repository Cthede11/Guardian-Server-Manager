import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { 
  Activity, 
  Cpu, 
  Memory, 
  Users, 
  HardDrive, 
  Zap, 
  Clock, 
  TrendingUp, 
  TrendingDown, 
  Minus,
  AlertTriangle,
  CheckCircle,
  Info
} from 'lucide-react';
import { useRealtimeServer } from '@/store/realtime';
import { realtimeHelpers } from '@/lib/realtime';

interface RealtimeMetricsProps {
  serverId: string;
  showDetails?: boolean;
  compact?: boolean;
  refreshInterval?: number;
}

export const RealtimeMetrics: React.FC<RealtimeMetricsProps> = ({
  serverId,
  showDetails = false,
  compact = false,
  refreshInterval = 1000
}) => {
  const { 
    metrics, 
    health, 
    healthScore, 
    performanceScore, 
    healthStatus, 
    performanceStatus 
  } = useRealtimeServer(serverId);
  
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [trends, setTrends] = useState<{
    tps: 'up' | 'down' | 'stable';
    memory: 'up' | 'down' | 'stable';
    players: 'up' | 'down' | 'stable';
  }>({
    tps: 'stable',
    memory: 'stable',
    players: 'stable'
  });

  // Update trends based on metrics changes
  useEffect(() => {
    if (metrics) {
      setLastUpdate(new Date());
      
      // Calculate trends (simplified - in real app, you'd compare with previous values)
      setTrends({
        tps: metrics.tps >= 20 ? 'up' : metrics.tps < 15 ? 'down' : 'stable',
        memory: metrics.heapMb > 2048 ? 'up' : metrics.heapMb < 1024 ? 'down' : 'stable',
        players: metrics.playersOnline > 0 ? 'up' : 'stable'
      });
    }
  }, [metrics]);

  const getTrendIcon = (trend: 'up' | 'down' | 'stable') => {
    switch (trend) {
      case 'up':
        return <TrendingUp className="h-3 w-3 text-green-500" />;
      case 'down':
        return <TrendingDown className="h-3 w-3 text-red-500" />;
      case 'stable':
        return <Minus className="h-3 w-3 text-gray-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'excellent':
      case 'healthy':
        return 'text-green-500';
      case 'good':
      case 'warning':
        return 'text-yellow-500';
      case 'fair':
      case 'critical':
        return 'text-red-500';
      default:
        return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'excellent':
      case 'healthy':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'good':
      case 'warning':
        return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'fair':
      case 'critical':
        return <AlertTriangle className="h-4 w-4 text-red-500" />;
      default:
        return <Info className="h-4 w-4 text-gray-500" />;
    }
  };

  if (compact) {
    return (
      <div className="flex items-center space-x-4">
        {metrics && (
          <>
            <div className="flex items-center space-x-1">
              <Activity className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">
                {realtimeHelpers.formatTPS(metrics.tps)}
              </span>
              {getTrendIcon(trends.tps)}
            </div>
            
            <div className="flex items-center space-x-1">
              <Users className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">
                {metrics.playersOnline}
              </span>
              {getTrendIcon(trends.players)}
            </div>
            
            <div className="flex items-center space-x-1">
              <Memory className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">
                {realtimeHelpers.formatMemory(metrics.heapMb)}
              </span>
              {getTrendIcon(trends.memory)}
            </div>
          </>
        )}
        
        <Badge className={`${getStatusColor(performanceStatus)} bg-transparent`}>
          {getStatusIcon(performanceStatus)}
          <span className="ml-1">{performanceStatus}</span>
        </Badge>
      </div>
    );
  }

  if (!metrics) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="text-center text-muted-foreground">
            <Activity className="h-8 w-8 mx-auto mb-2" />
            <p>No metrics data available</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>Real-time Metrics</span>
          </div>
          <div className="flex items-center space-x-2">
            <Badge className={`${getStatusColor(performanceStatus)} bg-transparent`}>
              {getStatusIcon(performanceStatus)}
              <span className="ml-1">{performanceStatus}</span>
            </Badge>
            <span className="text-xs text-muted-foreground">
              {performanceScore}%
            </span>
          </div>
        </CardTitle>
        <CardDescription>
          Live server performance metrics • Last updated: {lastUpdate.toLocaleTimeString()}
        </CardDescription>
      </CardHeader>
      
      <CardContent className="space-y-6">
        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Zap className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">TPS</span>
              </div>
              <div className="flex items-center space-x-1">
                {getTrendIcon(trends.tps)}
                <span className="text-sm font-mono">
                  {realtimeHelpers.formatTPS(metrics.tps)}
                </span>
              </div>
            </div>
            <Progress 
              value={Math.min((metrics.tps / 20) * 100, 100)} 
              className="h-2"
            />
          </div>
          
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Users className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Players</span>
              </div>
              <div className="flex items-center space-x-1">
                {getTrendIcon(trends.players)}
                <span className="text-sm font-mono">
                  {metrics.playersOnline}
                </span>
              </div>
            </div>
            <Progress 
              value={Math.min((metrics.playersOnline / 100) * 100, 100)} 
              className="h-2"
            />
          </div>
          
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Memory className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Memory</span>
              </div>
              <div className="flex items-center space-x-1">
                {getTrendIcon(trends.memory)}
                <span className="text-sm font-mono">
                  {realtimeHelpers.formatMemory(metrics.heapMb)}
                </span>
              </div>
            </div>
            <Progress 
              value={Math.min((metrics.heapMb / 8192) * 100, 100)} 
              className="h-2"
            />
          </div>
        </div>

        {/* Detailed Metrics */}
        {showDetails && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Tick P95</span>
                  <span className="text-sm font-mono">
                    {metrics.tickP95.toFixed(1)}ms
                  </span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">GPU Queue</span>
                  <span className="text-sm font-mono">
                    {metrics.gpuQueueMs.toFixed(1)}ms
                  </span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Last Snapshot</span>
                  <span className="text-sm font-mono">
                    {metrics.lastSnapshotAt 
                      ? realtimeHelpers.formatTime(metrics.lastSnapshotAt)
                      : 'Never'
                    }
                  </span>
                </div>
              </div>
              
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Health Score</span>
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-mono">{healthScore}%</span>
                    <Badge className={`${getStatusColor(healthStatus)} bg-transparent`}>
                      {getStatusIcon(healthStatus)}
                      <span className="ml-1">{healthStatus}</span>
                    </Badge>
                  </div>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Performance Score</span>
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-mono">{performanceScore}%</span>
                    <Badge className={`${getStatusColor(performanceStatus)} bg-transparent`}>
                      {getStatusIcon(performanceStatus)}
                      <span className="ml-1">{performanceStatus}</span>
                    </Badge>
                  </div>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Blue/Green</span>
                  <div className="flex items-center space-x-2">
                    <Badge className={`${metrics.blueGreen.active === 'blue' ? 'bg-blue-500' : 'bg-green-500'} text-white`}>
                      {metrics.blueGreen.active}
                    </Badge>
                    {metrics.blueGreen.candidateHealthy && (
                      <CheckCircle className="h-4 w-4 text-green-500" />
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Health Status */}
        {health && (
          <div className="pt-4 border-t">
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm font-medium">Health Status</span>
              <Badge className={`${getStatusColor(healthStatus)} bg-transparent`}>
                {getStatusIcon(healthStatus)}
                <span className="ml-1">{healthStatus}</span>
              </Badge>
            </div>
            
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${health.rcon ? 'bg-green-500' : 'bg-red-500'}`} />
                <span className="text-sm">RCON</span>
              </div>
              
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${health.query ? 'bg-green-500' : 'bg-red-500'}`} />
                <span className="text-sm">Query</span>
              </div>
              
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${health.crashTickets === 0 ? 'bg-green-500' : 'bg-red-500'}`} />
                <span className="text-sm">Crashes: {health.crashTickets}</span>
              </div>
              
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${health.freezeTickets === 0 ? 'bg-green-500' : 'bg-red-500'}`} />
                <span className="text-sm">Freezes: {health.freezeTickets}</span>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};

// Compact metrics display for headers
export const RealtimeMetricsBadge: React.FC<{ serverId: string }> = ({ serverId }) => {
  const { metrics, performanceStatus } = useRealtimeServer(serverId);
  
  if (!metrics) {
    return (
      <Badge variant="outline" className="text-xs">
        <Activity className="h-3 w-3 mr-1" />
        No Data
      </Badge>
    );
  }

  return (
    <Badge variant="outline" className="text-xs">
      <Activity className="h-3 w-3 mr-1" />
      {realtimeHelpers.formatTPS(metrics.tps)} • {metrics.playersOnline} players
    </Badge>
  );
};

// Metrics trend indicator
export const RealtimeMetricsTrend: React.FC<{ serverId: string; metric: 'tps' | 'memory' | 'players' }> = ({ 
  serverId, 
  metric 
}) => {
  const { metrics } = useRealtimeServer(serverId);
  const [trend, setTrend] = useState<'up' | 'down' | 'stable'>('stable');
  const [previousValue, setPreviousValue] = useState<number | null>(null);

  useEffect(() => {
    if (metrics) {
      const currentValue = metrics[metric];
      
      if (previousValue !== null) {
        if (currentValue > previousValue) {
          setTrend('up');
        } else if (currentValue < previousValue) {
          setTrend('down');
        } else {
          setTrend('stable');
        }
      }
      
      setPreviousValue(currentValue);
    }
  }, [metrics, metric, previousValue]);

  const getTrendIcon = () => {
    switch (trend) {
      case 'up':
        return <TrendingUp className="h-3 w-3 text-green-500" />;
      case 'down':
        return <TrendingDown className="h-3 w-3 text-red-500" />;
      case 'stable':
        return <Minus className="h-3 w-3 text-gray-500" />;
    }
  };

  return <>{getTrendIcon()}</>;
};
