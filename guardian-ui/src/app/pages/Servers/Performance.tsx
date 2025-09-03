import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { 
  Activity, 
  TrendingUp, 
  TrendingDown,
  RefreshCw,
  Settings,
  Zap,
  Cpu,
  MemoryStick,
  HardDrive,
  Network,
  AlertTriangle,
  CheckCircle,
  Clock,
  BarChart3,
  LineChart
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { useServersStore } from '@/store/servers';
import { TpsChart } from '@/components/Charts/TpsChart';
import { PhaseChart } from '@/components/Charts/PhaseChart';

interface PerformancePageProps {
  className?: string;
}

export const Performance: React.FC<PerformancePageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [performanceData, setPerformanceData] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [timeRange, setTimeRange] = useState('1h');
  const [autoRefresh, setAutoRefresh] = useState(true);

  // Performance budgets
  const [budgets, setBudgets] = useState({
    tps: 18,
    tickTime: 50,
    memory: 80,
    cpu: 70,
    disk: 90
  });

  // Performance policies
  const [policies, setPolicies] = useState({
    autoFreeze: true,
    autoRestart: false,
    autoScale: true,
    aggressiveGC: false,
    chunkUnloading: true,
    entityCulling: true
  });

  // Fetch performance data
  const fetchPerformanceData = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const response = await fetch(`http://localhost:8080/api/v1/servers/${serverId}/performance`);
      if (response.ok) {
        const data = await response.json();
        setPerformanceData(data);
      } else {
        // Use mock data for demo
        setPerformanceData(generateMockPerformanceData());
      }
    } catch (error) {
      console.error('Error fetching performance data:', error);
      // Use mock data for demo
      setPerformanceData(generateMockPerformanceData());
    } finally {
      setIsLoading(false);
    }
  };

  // Generate mock performance data
  const generateMockPerformanceData = () => {
    const now = Date.now();
    const dataPoints = 60; // 60 data points for 1 hour
    
    const tpsData = [];
    const tickTimeData = [];
    const memoryData = [];
    const cpuData = [];
    const phaseData = [];
    
    for (let i = 0; i < dataPoints; i++) {
      const timestamp = now - (dataPoints - i) * 60000; // 1 minute intervals
      
      // Generate realistic TPS data with some variation
      const baseTps = 19.5;
      const variation = Math.sin(i * 0.1) * 0.5 + (Math.random() - 0.5) * 0.3;
      const tps = Math.max(15, Math.min(20, baseTps + variation));
      
      // Generate tick time data (inverse relationship with TPS)
      const tickTime = Math.max(40, Math.min(80, (20 - tps) * 4 + 40 + (Math.random() - 0.5) * 10));
      
      // Generate memory usage data
      const memoryUsage = Math.max(60, Math.min(95, 75 + Math.sin(i * 0.05) * 10 + (Math.random() - 0.5) * 5));
      
      // Generate CPU usage data
      const cpuUsage = Math.max(20, Math.min(90, 45 + Math.sin(i * 0.08) * 15 + (Math.random() - 0.5) * 10));
      
      tpsData.push({ timestamp, value: tps });
      tickTimeData.push({ timestamp, value: tickTime });
      memoryData.push({ timestamp, value: memoryUsage });
      cpuData.push({ timestamp, value: cpuUsage });
      
      // Generate phase data
      const phases = ['world', 'entities', 'tileentities', 'chunks', 'network'];
      const phase = phases[Math.floor(Math.random() * phases.length)];
      const phaseTime = Math.max(1, Math.min(20, Math.random() * 15));
      phaseData.push({ timestamp, phase, time: phaseTime });
    }
    
    return {
      current: {
        tps: 19.2,
        tickTime: 52.3,
        memoryUsage: 78.5,
        cpuUsage: 42.1,
        diskUsage: 65.2,
        networkIn: 1024,
        networkOut: 2048,
        entities: 1250,
        chunks: 850,
        players: 12
      },
      history: {
        tps: tpsData,
        tickTime: tickTimeData,
        memory: memoryData,
        cpu: cpuData,
        phases: phaseData
      },
      alerts: [
        {
          id: '1',
          type: 'warning',
          message: 'TPS dropped below 18 for 2 minutes',
          timestamp: now - 300000,
          resolved: false
        },
        {
          id: '2',
          type: 'info',
          message: 'Memory usage approaching 80%',
          timestamp: now - 600000,
          resolved: true
        }
      ]
    };
  };

  useEffect(() => {
    fetchPerformanceData();
    
    // Auto-refresh every 30 seconds if enabled
    let interval: NodeJS.Timeout;
    if (autoRefresh) {
      interval = setInterval(fetchPerformanceData, 30000);
    }
    
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [serverId, autoRefresh]);

  const handleBudgetChange = (key: string, value: number[]) => {
    setBudgets(prev => ({
      ...prev,
      [key]: value[0]
    }));
  };

  const handlePolicyToggle = (key: string) => {
    setPolicies(prev => ({
      ...prev,
      [key]: !prev[key]
    }));
  };

  const getStatusColor = (value: number, threshold: number, reverse = false) => {
    const isGood = reverse ? value < threshold : value > threshold;
    return isGood ? 'text-green-400' : 'text-red-400';
  };

  const getStatusIcon = (value: number, threshold: number, reverse = false) => {
    const isGood = reverse ? value < threshold : value > threshold;
    return isGood ? <CheckCircle className="h-4 w-4" /> : <AlertTriangle className="h-4 w-4" />;
  };

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view performance</p>
        </div>
      </div>
    );
  }

  if (!performanceData) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-4" />
          <p className="text-muted-foreground">Loading performance data...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Performance</h2>
          <div className="flex items-center gap-2">
            <Badge 
              variant="outline" 
              className={`flex items-center gap-1 ${
                performanceData.current.tps >= 18 ? 'text-green-400' : 'text-red-400'
              }`}
            >
              {getStatusIcon(performanceData.current.tps, 18)}
              {performanceData.current.tps.toFixed(1)} TPS
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <MemoryStick className="h-3 w-3" />
              {performanceData.current.memoryUsage.toFixed(1)}% Memory
            </Badge>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-2">
            <Label htmlFor="auto-refresh" className="text-sm">Auto-refresh</Label>
            <Switch
              id="auto-refresh"
              checked={autoRefresh}
              onCheckedChange={setAutoRefresh}
            />
          </div>
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

      {/* Current Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm flex items-center gap-2">
              <Activity className="h-4 w-4" />
              TPS
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getStatusColor(performanceData.current.tps, 18)}`}>
              {performanceData.current.tps.toFixed(1)}
            </div>
            <p className="text-xs text-muted-foreground">
              Target: 20.0 TPS
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm flex items-center gap-2">
              <Clock className="h-4 w-4" />
              Tick Time
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getStatusColor(performanceData.current.tickTime, 50, true)}`}>
              {performanceData.current.tickTime.toFixed(1)}ms
            </div>
            <p className="text-xs text-muted-foreground">
              Target: &lt; 50ms
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm flex items-center gap-2">
              <MemoryStick className="h-4 w-4" />
              Memory
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getStatusColor(performanceData.current.memoryUsage, 80, true)}`}>
              {performanceData.current.memoryUsage.toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              Target: &lt; 80%
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm flex items-center gap-2">
              <Cpu className="h-4 w-4" />
              CPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getStatusColor(performanceData.current.cpuUsage, 70, true)}`}>
              {performanceData.current.cpuUsage.toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              Target: &lt; 70%
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="charts" className="space-y-4">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="charts" className="flex items-center gap-2">
            <LineChart className="h-4 w-4" />
            Charts
          </TabsTrigger>
          <TabsTrigger value="budgets" className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            Budgets
          </TabsTrigger>
          <TabsTrigger value="policies" className="flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Policies
          </TabsTrigger>
        </TabsList>

        <TabsContent value="charts" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* TPS Chart */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  TPS Over Time
                </CardTitle>
              </CardHeader>
              <CardContent>
                <TpsChart data={performanceData.history.tps} />
              </CardContent>
            </Card>

            {/* Phase Chart */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Zap className="h-5 w-5" />
                  Tick Phases
                </CardTitle>
              </CardHeader>
              <CardContent>
                <PhaseChart data={performanceData.history.phases} />
              </CardContent>
            </Card>
          </div>

          {/* Additional Metrics */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>System Resources</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Memory Usage</span>
                    <span className="text-sm font-medium">
                      {performanceData.current.memoryUsage.toFixed(1)}%
                    </span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div 
                      className="bg-blue-500 h-2 rounded-full" 
                      style={{ width: `${performanceData.current.memoryUsage}%` }}
                    />
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm">CPU Usage</span>
                    <span className="text-sm font-medium">
                      {performanceData.current.cpuUsage.toFixed(1)}%
                    </span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div 
                      className="bg-green-500 h-2 rounded-full" 
                      style={{ width: `${performanceData.current.cpuUsage}%` }}
                    />
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Disk Usage</span>
                    <span className="text-sm font-medium">
                      {performanceData.current.diskUsage.toFixed(1)}%
                    </span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div 
                      className="bg-yellow-500 h-2 rounded-full" 
                      style={{ width: `${performanceData.current.diskUsage}%` }}
                    />
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Server Stats</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Entities</span>
                    <span className="text-sm font-medium">
                      {performanceData.current.entities.toLocaleString()}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Loaded Chunks</span>
                    <span className="text-sm font-medium">
                      {performanceData.current.chunks.toLocaleString()}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Online Players</span>
                    <span className="text-sm font-medium">
                      {performanceData.current.players}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Network In</span>
                    <span className="text-sm font-medium">
                      {(performanceData.current.networkIn / 1024).toFixed(1)} KB/s
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Network Out</span>
                    <span className="text-sm font-medium">
                      {(performanceData.current.networkOut / 1024).toFixed(1)} KB/s
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="budgets" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Performance Budgets</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-4">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label htmlFor="tps-budget">TPS Threshold</Label>
                    <span className="text-sm font-medium">{budgets.tps} TPS</span>
                  </div>
                  <Slider
                    id="tps-budget"
                    min={15}
                    max={20}
                    step={0.1}
                    value={[budgets.tps]}
                    onValueChange={(value) => handleBudgetChange('tps', value)}
                    className="w-full"
                  />
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label htmlFor="tick-budget">Tick Time Threshold</Label>
                    <span className="text-sm font-medium">{budgets.tickTime}ms</span>
                  </div>
                  <Slider
                    id="tick-budget"
                    min={30}
                    max={100}
                    step={1}
                    value={[budgets.tickTime]}
                    onValueChange={(value) => handleBudgetChange('tickTime', value)}
                    className="w-full"
                  />
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label htmlFor="memory-budget">Memory Threshold</Label>
                    <span className="text-sm font-medium">{budgets.memory}%</span>
                  </div>
                  <Slider
                    id="memory-budget"
                    min={50}
                    max={95}
                    step={1}
                    value={[budgets.memory]}
                    onValueChange={(value) => handleBudgetChange('memory', value)}
                    className="w-full"
                  />
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label htmlFor="cpu-budget">CPU Threshold</Label>
                    <span className="text-sm font-medium">{budgets.cpu}%</span>
                  </div>
                  <Slider
                    id="cpu-budget"
                    min={30}
                    max={90}
                    step={1}
                    value={[budgets.cpu]}
                    onValueChange={(value) => handleBudgetChange('cpu', value)}
                    className="w-full"
                  />
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label htmlFor="disk-budget">Disk Threshold</Label>
                    <span className="text-sm font-medium">{budgets.disk}%</span>
                  </div>
                  <Slider
                    id="disk-budget"
                    min={60}
                    max={95}
                    step={1}
                    value={[budgets.disk]}
                    onValueChange={(value) => handleBudgetChange('disk', value)}
                    className="w-full"
                  />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="policies" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Performance Policies</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="auto-freeze">Auto-freeze Chunks</Label>
                    <p className="text-sm text-muted-foreground">
                      Automatically freeze chunks when performance drops
                    </p>
                  </div>
                  <Switch
                    id="auto-freeze"
                    checked={policies.autoFreeze}
                    onCheckedChange={() => handlePolicyToggle('autoFreeze')}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="auto-restart">Auto-restart Server</Label>
                    <p className="text-sm text-muted-foreground">
                      Restart server when critical performance issues occur
                    </p>
                  </div>
                  <Switch
                    id="auto-restart"
                    checked={policies.autoRestart}
                    onCheckedChange={() => handlePolicyToggle('autoRestart')}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="auto-scale">Auto-scale Resources</Label>
                    <p className="text-sm text-muted-foreground">
                      Automatically adjust resource allocation based on load
                    </p>
                  </div>
                  <Switch
                    id="auto-scale"
                    checked={policies.autoScale}
                    onCheckedChange={() => handlePolicyToggle('autoScale')}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="aggressive-gc">Aggressive Garbage Collection</Label>
                    <p className="text-sm text-muted-foreground">
                      Force garbage collection when memory usage is high
                    </p>
                  </div>
                  <Switch
                    id="aggressive-gc"
                    checked={policies.aggressiveGC}
                    onCheckedChange={() => handlePolicyToggle('aggressiveGC')}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="chunk-unloading">Chunk Unloading</Label>
                    <p className="text-sm text-muted-foreground">
                      Unload chunks when no players are nearby
                    </p>
                  </div>
                  <Switch
                    id="chunk-unloading"
                    checked={policies.chunkUnloading}
                    onCheckedChange={() => handlePolicyToggle('chunkUnloading')}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="entity-culling">Entity Culling</Label>
                    <p className="text-sm text-muted-foreground">
                      Remove entities that are too far from players
                    </p>
                  </div>
                  <Switch
                    id="entity-culling"
                    checked={policies.entityCulling}
                    onCheckedChange={() => handlePolicyToggle('entityCulling')}
                  />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default Performance;
