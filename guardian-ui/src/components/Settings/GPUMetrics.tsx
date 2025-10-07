import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { apiClient } from '@/lib/api';
import { 
  Activity, 
  Cpu, 
  MemoryStick, 
  Thermometer, 
  Zap,
  AlertTriangle,
  CheckCircle,
  Info
} from 'lucide-react';

interface GPUMetrics {
  utilization: number;
  memory_used: number;
  memory_total: number;
  temperature: number;
  power_usage: number;
  last_update: string;
}

interface GPUStatus {
  enabled: boolean;
  healthy: boolean;
  worker_available: boolean;
}

export const GPUMetrics: React.FC = () => {
  const [metrics, setMetrics] = useState<GPUMetrics | null>(null);
  const [status, setStatus] = useState<GPUStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMetrics = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Fetch GPU status
      const statusData = await apiClient.call<{ data: GPUStatus }>('/api/gpu/status');
      setStatus(statusData.data);

      // Only fetch metrics if GPU is enabled
      if (statusData.data?.enabled) {
        const metricsData = await apiClient.call<{ data: GPUMetrics }>('/api/gpu/metrics');
        setMetrics(metricsData.data);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch GPU data');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchMetrics();
    
    // Refresh metrics every 5 seconds if GPU is enabled
    const interval = setInterval(() => {
      if (status?.enabled) {
        fetchMetrics();
      }
    }, 5000);

    return () => clearInterval(interval);
  }, [status?.enabled]);

  const getStatusColor = (healthy: boolean, enabled: boolean) => {
    if (!enabled) return 'bg-gray-500';
    if (healthy) return 'bg-green-500';
    return 'bg-red-500';
  };

  const getStatusText = (healthy: boolean, enabled: boolean) => {
    if (!enabled) return 'Disabled';
    if (healthy) return 'Healthy';
    return 'Unhealthy';
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatTemperature = (temp: number) => {
    return `${temp.toFixed(1)}°C`;
  };

  const formatPower = (power: number) => {
    return `${power.toFixed(1)}W`;
  };

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            GPU Metrics
          </CardTitle>
          <CardDescription>Loading GPU metrics...</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            GPU Metrics
          </CardTitle>
          <CardDescription>Error loading GPU metrics</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2 text-red-600">
            <AlertTriangle className="h-4 w-4" />
            <span>{error}</span>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!status?.enabled) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            GPU Metrics
          </CardTitle>
          <CardDescription>GPU acceleration is disabled</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2 text-gray-500">
            <Info className="h-4 w-4" />
            <span>Enable GPU acceleration in settings to view metrics</span>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          GPU Metrics
          <Badge 
            variant={status?.healthy ? "default" : "destructive"}
            className="ml-auto"
          >
            {getStatusText(status?.healthy || false, status?.enabled || false)}
          </Badge>
        </CardTitle>
        <CardDescription>
          Real-time GPU performance metrics
          {metrics && (
            <span className="text-xs text-gray-500 ml-2">
              Last updated: {new Date(metrics.last_update).toLocaleTimeString()}
            </span>
          )}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {metrics ? (
          <>
            {/* GPU Utilization */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Cpu className="h-4 w-4" />
                  <span className="text-sm font-medium">GPU Utilization</span>
                </div>
                <span className="text-sm text-gray-600">
                  {(metrics.utilization * 100).toFixed(1)}%
                </span>
              </div>
              <Progress 
                value={metrics.utilization * 100} 
                className="h-2"
              />
            </div>

            {/* Memory Usage */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <MemoryStick className="h-4 w-4" />
                  <span className="text-sm font-medium">Memory Usage</span>
                </div>
                <span className="text-sm text-gray-600">
                  {formatBytes(metrics.memory_used)} / {formatBytes(metrics.memory_total)}
                </span>
              </div>
              <Progress 
                value={(metrics.memory_used / metrics.memory_total) * 100} 
                className="h-2"
              />
              <div className="text-xs text-gray-500">
                {((metrics.memory_used / metrics.memory_total) * 100).toFixed(1)}% used
              </div>
            </div>

            {/* Temperature and Power */}
            <div className="grid grid-cols-2 gap-4">
              <div className="flex items-center gap-2">
                <Thermometer className="h-4 w-4 text-blue-500" />
                <div>
                  <div className="text-sm font-medium">Temperature</div>
                  <div className="text-xs text-gray-600">
                    {formatTemperature(metrics.temperature)}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Zap className="h-4 w-4 text-yellow-500" />
                <div>
                  <div className="text-sm font-medium">Power Usage</div>
                  <div className="text-xs text-gray-600">
                    {formatPower(metrics.power_usage)}
                  </div>
                </div>
              </div>
            </div>

            {/* Status Indicators */}
            <div className="grid grid-cols-2 gap-4 pt-4 border-t">
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${getStatusColor(status?.healthy || false, status?.enabled || false)}`}></div>
                <span className="text-sm">Worker Status</span>
              </div>
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${status?.worker_available ? 'bg-green-500' : 'bg-red-500'}`}></div>
                <span className="text-sm">Availability</span>
              </div>
            </div>
          </>
        ) : (
          <div className="flex items-center justify-center py-8 text-gray-500">
            <div className="text-center">
              <Activity className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No metrics available</p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};
