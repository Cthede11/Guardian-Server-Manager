import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  TrendingUp, 
  TrendingDown,
  Cpu,
  HardDrive,
  Wifi,
  MemoryStick,
  Clock,
  BarChart3,
  RefreshCw,
  Calendar,
  Filter
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { apiClient as api } from '@/lib/api';

interface PerformanceMetrics {
  timestamp: string;
  tps: number;
  tick_ms: number;
  memory_usage_mb: number;
  memory_max_mb: number;
  memory_utilization_percent: number;
  cpu_usage_percent: number;
  disk_read_bytes_per_sec: number;
  disk_write_bytes_per_sec: number;
  network_in_bytes_per_sec: number;
  network_out_bytes_per_sec: number;
  disk_usage_mb: number;
}

interface PerformanceSummary {
  server_id: string;
  avg_tps: number;
  max_tps: number;
  min_tps: number;
  avg_tick_ms: number;
  max_tick_ms: number;
  min_tick_ms: number;
  avg_memory_utilization_percent: number;
  max_memory_utilization_percent: number;
  avg_cpu_usage_percent: number;
  max_cpu_usage_percent: number;
  total_disk_read_mb: number;
  total_disk_write_mb: number;
  total_network_in_mb: number;
  total_network_out_mb: number;
  current_disk_usage_mb: number;
}

interface AnalyticsPageProps {
  serverId: string;
  className?: string;
}

export const AnalyticsPage: React.FC<AnalyticsPageProps> = ({
  serverId,
  className = ''
}) => {
  const [metrics, setMetrics] = useState<PerformanceMetrics[]>([]);
  const [summary, setSummary] = useState<PerformanceSummary | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [timeRange, setTimeRange] = useState('1h');
  const [selectedMetric, setSelectedMetric] = useState('tps');

  // Fetch performance data
  const fetchPerformanceData = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const [metricsRes, summaryRes] = await Promise.all([
        api.call(`/api/performance/${serverId}/metrics`),
        api.call(`/api/performance/${serverId}/summary`)
      ]);

      if (metricsRes && typeof metricsRes === 'object' && 'ok' in metricsRes && metricsRes.ok && 'data' in metricsRes && metricsRes.data) {
        setMetrics(metricsRes.data as PerformanceMetrics[]);
      }
      if (summaryRes && typeof summaryRes === 'object' && 'ok' in summaryRes && summaryRes.ok && 'data' in summaryRes && summaryRes.data) {
        setSummary(summaryRes.data as PerformanceSummary);
      }
    } catch (error) {
      console.error('Error fetching performance data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchPerformanceData();
    
    // Refresh data every 30 seconds
    const interval = setInterval(fetchPerformanceData, 30000);
    return () => clearInterval(interval);
  }, [serverId]);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatBytesPerSec = (bytes: number) => {
    return formatBytes(bytes) + '/s';
  };

  const getPerformanceColor = (value: number, type: 'tps' | 'tick' | 'memory' | 'cpu') => {
    switch (type) {
      case 'tps':
        if (value >= 18) return 'text-green-400';
        if (value >= 15) return 'text-yellow-400';
        return 'text-red-400';
      case 'tick':
        if (value <= 50) return 'text-green-400';
        if (value <= 100) return 'text-yellow-400';
        return 'text-red-400';
      case 'memory':
        if (value <= 70) return 'text-green-400';
        if (value <= 85) return 'text-yellow-400';
        return 'text-red-400';
      case 'cpu':
        if (value <= 50) return 'text-green-400';
        if (value <= 80) return 'text-yellow-400';
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  const getTrendIcon = (current: number, previous: number) => {
    if (current > previous) return <TrendingUp className="h-4 w-4 text-green-400" />;
    if (current < previous) return <TrendingDown className="h-4 w-4 text-red-400" />;
    return <Activity className="h-4 w-4 text-gray-400" />;
  };

  const getTrendColor = (current: number, previous: number) => {
    if (current > previous) return 'text-green-400';
    if (current < previous) return 'text-red-400';
    return 'text-gray-400';
  };

  // Calculate trend data
  const getTrendData = () => {
    if (metrics.length < 2) return null;
    const current = metrics[metrics.length - 1];
    const previous = metrics[metrics.length - 2];
    
    return {
      tps: { current: current.tps, previous: previous.tps },
      tick_ms: { current: current.tick_ms, previous: previous.tick_ms },
      memory: { current: current.memory_utilization_percent, previous: previous.memory_utilization_percent },
      cpu: { current: current.cpu_usage_percent, previous: previous.cpu_usage_percent }
    };
  };

  const trendData = getTrendData();

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Performance Analytics</h2>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <Activity className="h-3 w-3" />
              {metrics.length} Data Points
            </Badge>
            {summary && (
              <Badge variant="outline" className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                Last Updated: {new Date().toLocaleTimeString()}
              </Badge>
            )}
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Select value={timeRange} onValueChange={setTimeRange}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="15m">Last 15m</SelectItem>
              <SelectItem value="1h">Last 1h</SelectItem>
              <SelectItem value="6h">Last 6h</SelectItem>
              <SelectItem value="24h">Last 24h</SelectItem>
            </SelectContent>
          </Select>
          
          <Button
            size="sm"
            variant="outline"
            onClick={fetchPerformanceData}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Summary Cards */}
      {summary && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {/* TPS Card */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm flex items-center gap-2">
                <Activity className="h-4 w-4" />
                TPS Performance
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-2xl font-bold text-green-400">
                    {summary.avg_tps.toFixed(1)}
                  </span>
                  {trendData && getTrendIcon(trendData.tps.current, trendData.tps.previous)}
                </div>
                <div className="text-xs text-muted-foreground">
                  Avg: {summary.avg_tps.toFixed(1)} | Max: {summary.max_tps.toFixed(1)} | Min: {summary.min_tps.toFixed(1)}
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Tick Time Card */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm flex items-center gap-2">
                <Clock className="h-4 w-4" />
                Tick Time
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className={`text-2xl font-bold ${getPerformanceColor(summary.avg_tick_ms, 'tick')}`}>
                    {summary.avg_tick_ms.toFixed(1)}ms
                  </span>
                  {trendData && getTrendIcon(trendData.tick_ms.current, trendData.tick_ms.previous)}
                </div>
                <div className="text-xs text-muted-foreground">
                  Avg: {summary.avg_tick_ms.toFixed(1)}ms | Max: {summary.max_tick_ms.toFixed(1)}ms
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Memory Usage Card */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm flex items-center gap-2">
                <MemoryStick className="h-4 w-4" />
                Memory Usage
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className={`text-2xl font-bold ${getPerformanceColor(summary.avg_memory_utilization_percent, 'memory')}`}>
                    {summary.avg_memory_utilization_percent.toFixed(1)}%
                  </span>
                  {trendData && getTrendIcon(trendData.memory.current, trendData.memory.previous)}
                </div>
                <div className="text-xs text-muted-foreground">
                  Max: {summary.max_memory_utilization_percent.toFixed(1)}% | Current: {formatBytes(summary.current_disk_usage_mb * 1024 * 1024)}
                </div>
              </div>
            </CardContent>
          </Card>

          {/* CPU Usage Card */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm flex items-center gap-2">
                <Cpu className="h-4 w-4" />
                CPU Usage
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className={`text-2xl font-bold ${getPerformanceColor(summary.avg_cpu_usage_percent, 'cpu')}`}>
                    {summary.avg_cpu_usage_percent.toFixed(1)}%
                  </span>
                  {trendData && getTrendIcon(trendData.cpu.current, trendData.cpu.previous)}
                </div>
                <div className="text-xs text-muted-foreground">
                  Max: {summary.max_cpu_usage_percent.toFixed(1)}%
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Detailed Analytics */}
      <Tabs defaultValue="performance" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="performance" className="flex items-center gap-2">
            <Activity className="h-4 w-4" />
            Performance
          </TabsTrigger>
          <TabsTrigger value="network" className="flex items-center gap-2">
            <Wifi className="h-4 w-4" />
            Network
          </TabsTrigger>
          <TabsTrigger value="storage" className="flex items-center gap-2">
            <HardDrive className="h-4 w-4" />
            Storage
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            History
          </TabsTrigger>
        </TabsList>

        {/* Performance Tab */}
        <TabsContent value="performance" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {/* TPS Chart Placeholder */}
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">TPS Over Time</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64 flex items-center justify-center border-2 border-dashed border-muted-foreground/25 rounded-lg">
                  <div className="text-center">
                    <BarChart3 className="h-12 w-12 text-muted-foreground mx-auto mb-2" />
                    <p className="text-muted-foreground">TPS Chart Component</p>
                    <p className="text-xs text-muted-foreground">Integration with chart library needed</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Tick Time Chart Placeholder */}
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Tick Time Over Time</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64 flex items-center justify-center border-2 border-dashed border-muted-foreground/25 rounded-lg">
                  <div className="text-center">
                    <Clock className="h-12 w-12 text-muted-foreground mx-auto mb-2" />
                    <p className="text-muted-foreground">Tick Time Chart Component</p>
                    <p className="text-xs text-muted-foreground">Integration with chart library needed</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Network Tab */}
        <TabsContent value="network" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Network I/O</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Download Rate</span>
                    <span className="text-sm text-blue-400">
                      {summary ? formatBytesPerSec(summary.total_network_in_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Upload Rate</span>
                    <span className="text-sm text-green-400">
                      {summary ? formatBytesPerSec(summary.total_network_out_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Total Downloaded</span>
                    <span className="text-sm text-muted-foreground">
                      {summary ? formatBytes(summary.total_network_in_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Total Uploaded</span>
                    <span className="text-sm text-muted-foreground">
                      {summary ? formatBytes(summary.total_network_out_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Network Chart</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-64 flex items-center justify-center border-2 border-dashed border-muted-foreground/25 rounded-lg">
                  <div className="text-center">
                    <Wifi className="h-12 w-12 text-muted-foreground mx-auto mb-2" />
                    <p className="text-muted-foreground">Network Chart Component</p>
                    <p className="text-xs text-muted-foreground">Integration with chart library needed</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Storage Tab */}
        <TabsContent value="storage" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Disk I/O</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Read Rate</span>
                    <span className="text-sm text-blue-400">
                      {summary ? formatBytesPerSec(summary.total_disk_read_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Write Rate</span>
                    <span className="text-sm text-green-400">
                      {summary ? formatBytesPerSec(summary.total_disk_write_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Total Read</span>
                    <span className="text-sm text-muted-foreground">
                      {summary ? formatBytes(summary.total_disk_read_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Total Written</span>
                    <span className="text-sm text-muted-foreground">
                      {summary ? formatBytes(summary.total_disk_write_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Disk Usage</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">Current Usage</span>
                    <span className="text-sm text-muted-foreground">
                      {summary ? formatBytes(summary.current_disk_usage_mb * 1024 * 1024) : 'N/A'}
                    </span>
                  </div>
                  <div className="w-full bg-muted rounded-full h-2">
                    <div 
                      className="bg-blue-400 h-2 rounded-full" 
                      style={{ width: summary ? `${Math.min((summary.current_disk_usage_mb / 1024) * 100, 100)}%` : '0%' }}
                    ></div>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Server directory size
                  </p>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Performance History</CardTitle>
            </CardHeader>
            <CardContent>
              {metrics.length === 0 ? (
                <div className="text-center py-8">
                  <BarChart3 className="h-12 w-12 text-muted-foreground mx-auto mb-2" />
                  <p className="text-muted-foreground">No performance data available</p>
                  <p className="text-xs text-muted-foreground mt-1">
                    Data will appear once the server starts collecting metrics
                  </p>
                </div>
              ) : (
                <div className="space-y-2">
                  {metrics.slice(-10).reverse().map((metric, index) => (
                    <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center gap-4">
                        <span className="text-sm text-muted-foreground">
                          {new Date(metric.timestamp).toLocaleTimeString()}
                        </span>
                        <Badge variant="outline" className={getPerformanceColor(metric.tps, 'tps')}>
                          {metric.tps.toFixed(1)} TPS
                        </Badge>
                        <Badge variant="outline" className={getPerformanceColor(metric.tick_ms, 'tick')}>
                          {metric.tick_ms.toFixed(1)}ms
                        </Badge>
                      </div>
                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span>Mem: {metric.memory_utilization_percent.toFixed(1)}%</span>
                        <span>CPU: {metric.cpu_usage_percent.toFixed(1)}%</span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default AnalyticsPage;
