import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { 
  Activity, 
  HardDrive, 
  Cpu, 
  MemoryStick, 
  Network,
  Zap,
  CheckCircle,
  AlertTriangle,
  XCircle,
  RefreshCw,
  TrendingUp,
  TrendingDown,
  Thermometer,
  Database,
  Server
} from 'lucide-react';
import { apiClient as api } from '@/lib/api';

interface SystemMetric {
  name: string;
  value: number;
  unit: string;
  status: 'healthy' | 'warning' | 'critical';
  trend: 'up' | 'down' | 'stable';
  threshold: {
    warning: number;
    critical: number;
  };
}

interface SystemHealthData {
  overall: number;
  metrics: SystemMetric[];
  uptime: string;
  lastRestart: string;
  version: string;
  javaVersion: string;
  os: string;
  architecture: string;
}

export const SystemHealth: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const [healthData, setHealthData] = useState<SystemHealthData>({
    overall: 0,
    metrics: [],
    uptime: '0d 0h 0m',
    lastRestart: 'Never',
    version: '1.20.1',
    javaVersion: 'OpenJDK 17.0.2',
    os: 'Linux Ubuntu 22.04',
    architecture: 'x86_64'
  });
  const [isLoading, setIsLoading] = useState(false);

  const fetchHealthData = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      // Real API call to get system health data
      const response = await api.getServerHealth(serverId);
      if (response.ok && response.data) {
        const healthData = response.data as any;
        
        // Transform the API response to match our interface
        const metrics: SystemMetric[] = healthData.metrics || [];
        const systemHealthData: SystemHealthData = {
          overall: healthData.overall || 0,
          metrics: metrics,
          uptime: healthData.uptime || '0d 0h 0m',
          lastRestart: healthData.lastRestart || 'Never',
          version: healthData.version || 'Unknown',
          javaVersion: healthData.javaVersion || 'Unknown',
          os: healthData.os || 'Unknown',
          architecture: healthData.architecture || 'Unknown'
        };

        setHealthData(systemHealthData);
      } else {
        // If no data available, show empty state
        setHealthData({
          overall: 0,
          metrics: [],
          uptime: '0d 0h 0m',
          lastRestart: 'Never',
          version: 'Unknown',
          javaVersion: 'Unknown',
          os: 'Unknown',
          architecture: 'Unknown'
        });
      }
    } catch (error) {
      console.error('Failed to fetch system health:', error);
      setHealthData({
        overall: 0,
        metrics: [],
        uptime: '0d 0h 0m',
        lastRestart: 'Never',
        version: 'Unknown',
        javaVersion: 'Unknown',
        os: 'Unknown',
        architecture: 'Unknown'
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (serverId) {
      fetchHealthData();
      
      // Auto-refresh every 30 seconds
      const interval = setInterval(fetchHealthData, 30000);
      return () => clearInterval(interval);
    }
  }, [serverId]);

  const getStatusIcon = (status: SystemMetric['status']) => {
    switch (status) {
      case 'healthy': return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'warning': return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'critical': return <XCircle className="h-4 w-4 text-red-500" />;
      default: return <AlertTriangle className="h-4 w-4 text-gray-500" />;
    }
  };

  // const getStatusColor = (status: SystemMetric['status']) => {
  //   switch (status) {
  //     case 'healthy': return 'text-green-500';
  //     case 'warning': return 'text-yellow-500';
  //     case 'critical': return 'text-red-500';
  //     default: return 'text-gray-500';
  //   }
  // };

  const getTrendIcon = (trend: SystemMetric['trend']) => {
    switch (trend) {
      case 'up': return <TrendingUp className="h-4 w-4 text-red-500" />;
      case 'down': return <TrendingDown className="h-4 w-4 text-green-500" />;
      case 'stable': return <Activity className="h-4 w-4 text-blue-500" />;
      default: return <Activity className="h-4 w-4 text-gray-500" />;
    }
  };

  const getMetricIcon = (name: string) => {
    switch (name.toLowerCase()) {
      case 'cpu usage': return <Cpu className="h-4 w-4" />;
      case 'memory usage': return <MemoryStick className="h-4 w-4" />;
      case 'disk usage': return <HardDrive className="h-4 w-4" />;
      case 'network latency': return <Network className="h-4 w-4" />;
      case 'tps': return <Zap className="h-4 w-4" />;
      case 'heap usage': return <Database className="h-4 w-4" />;
      case 'temperature': return <Thermometer className="h-4 w-4" />;
      case 'database connections': return <Server className="h-4 w-4" />;
      default: return <Activity className="h-4 w-4" />;
    }
  };

  const getOverallStatus = () => {
    if (healthData.overall >= 90) return 'healthy';
    if (healthData.overall >= 70) return 'warning';
    return 'critical';
  };

  const overallStatus = getOverallStatus();

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Overall Health */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center space-x-2">
              <Activity className="h-5 w-5" />
              <span>System Health Overview</span>
            </CardTitle>
            <Button variant="outline" size="sm" onClick={fetchHealthData} disabled={isLoading}>
              <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="text-center">
              <div className="text-4xl font-bold mb-2">{healthData.overall}%</div>
              <div className="flex items-center justify-center space-x-2">
                {getStatusIcon(overallStatus)}
                <span className="capitalize">{overallStatus}</span>
              </div>
              <Progress value={healthData.overall} className="mt-2" />
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Uptime</span>
                <span className="font-medium">{healthData.uptime}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Last Restart</span>
                <span className="font-medium">
                  {new Date(healthData.lastRestart).toLocaleDateString()}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Server Version</span>
                <span className="font-medium">{healthData.version}</span>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Java Version</span>
                <span className="font-medium">{healthData.javaVersion}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">OS</span>
                <span className="font-medium">{healthData.os}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Architecture</span>
                <span className="font-medium">{healthData.architecture}</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* System Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {healthData.metrics.map((metric, index) => (
          <Card key={index}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  {getMetricIcon(metric.name)}
                  <CardTitle className="text-sm">{metric.name}</CardTitle>
                </div>
                <div className="flex items-center space-x-1">
                  {getStatusIcon(metric.status)}
                  {getTrendIcon(metric.trend)}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-2xl font-bold">
                    {metric.value}{metric.unit}
                  </span>
                  <Badge variant={
                    metric.status === 'healthy' ? 'default' :
                    metric.status === 'warning' ? 'secondary' : 'destructive'
                  }>
                    {metric.status}
                  </Badge>
                </div>
                
                <div className="space-y-1">
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>Warning: {metric.threshold.warning}{metric.unit}</span>
                    <span>Critical: {metric.threshold.critical}{metric.unit}</span>
                  </div>
                  <Progress 
                    value={metric.value} 
                    className="h-2"
                  />
                </div>
                
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>Current: {metric.value}{metric.unit}</span>
                  <span className="capitalize">{metric.trend}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Health Alerts */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <AlertTriangle className="h-5 w-5" />
            <span>Health Alerts</span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {healthData.metrics.filter(m => m.status !== 'healthy').map((metric, index) => (
              <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                <div className="flex items-center space-x-3">
                  {getStatusIcon(metric.status)}
                  <div>
                    <div className="font-medium">{metric.name}</div>
                    <div className="text-sm text-muted-foreground">
                      Current: {metric.value}{metric.unit} | 
                      Threshold: {metric.status === 'warning' ? metric.threshold.warning : metric.threshold.critical}{metric.unit}
                    </div>
                  </div>
                </div>
                <Badge variant={metric.status === 'warning' ? 'secondary' : 'destructive'}>
                  {metric.status}
                </Badge>
              </div>
            ))}
            
            {healthData.metrics.filter(m => m.status !== 'healthy').length === 0 && (
              <div className="text-center text-muted-foreground py-8">
                <CheckCircle className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>No health alerts - all systems are healthy</p>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
