import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useServers } from '@/store/servers-new';
import { apiClient as api } from '@/lib/api';
import { safeDateShort, healthLabel, healthStatus } from '@/lib/formatters';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Bug, 
  AlertTriangle, 
  CheckCircle, 
  XCircle, 
  RefreshCw,
  Download,
  FileText,
  Clock,
  Activity,
  HardDrive,
  Cpu,
  MemoryStick,
  Network,
  Zap,
  Search,
  Filter,
  Calendar,
  TrendingUp,
  TrendingDown
} from 'lucide-react';
import { CrashSignaturesTable } from '@/components/Diagnostics/CrashSignaturesTable';
import { CreateBundleModal } from '@/components/Diagnostics/CreateBundleModal';
import { SystemHealth } from '@/components/Diagnostics/SystemHealth';
import { ErrorEmptyState } from '@/components/ui/EmptyState';

interface DiagnosticStats {
  totalCrashes: number;
  criticalCrashes: number;
  resolvedCrashes: number;
  systemHealth: number;
  lastCrash: string;
  averageUptime: number;
  memoryLeaks: number;
  performanceIssues: number;
}

export const Diagnostics: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServers();
  const server = serverId ? getServerById(serverId) : null;
  const [activeTab, setActiveTab] = useState('crashes');
  const [stats, setStats] = useState<DiagnosticStats>({
    totalCrashes: 0,
    criticalCrashes: 0,
    resolvedCrashes: 0,
    systemHealth: 0,
    lastCrash: 'Never',
    averageUptime: 0,
    memoryLeaks: 0,
    performanceIssues: 0
  });
  const [isLoading, setIsLoading] = useState(false);
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date());

  const fetchStats = async () => {
    setIsLoading(true);
    try {
      const response = await api.getDiagnostics(serverId || '');
      if (response.ok && response.data) {
        setStats(response.data as DiagnosticStats);
      } else {
        console.error('Failed to fetch diagnostic stats:', response.error);
        setStats({
          totalCrashes: 0,
          criticalCrashes: 0,
          resolvedCrashes: 0,
          systemHealth: 0,
          lastCrash: 'Never',
          averageUptime: 0,
          memoryLeaks: 0,
          performanceIssues: 0
        });
      }
      
      setLastRefresh(new Date());
    } catch (error) {
      console.error('Failed to fetch diagnostic stats:', error);
      setStats({
        totalCrashes: 0,
        criticalCrashes: 0,
        resolvedCrashes: 0,
        systemHealth: 0,
        lastCrash: 'Never',
        averageUptime: 0,
        memoryLeaks: 0,
        performanceIssues: 0
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
    
    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchStats, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleRefresh = () => {
    fetchStats();
  };

  const getHealthStatus = () => {
    // For now, use a simple health calculation based on systemHealth
    if (stats.systemHealth === 0) return 'healthy';
    if (stats.systemHealth < 5) return 'warning';
    return 'critical';
  };

  const currentHealthStatus = getHealthStatus();

  const getUptimeColor = (uptime: number) => {
    if (uptime >= 99) return 'text-green-500';
    if (uptime >= 95) return 'text-yellow-500';
    return 'text-red-500';
  };

  const getUptimeTrend = (uptime: number) => {
    if (uptime >= 99.5) return { icon: TrendingUp, color: 'text-green-500' };
    if (uptime <= 95) return { icon: TrendingDown, color: 'text-red-500' };
    return null;
  };

  const uptimeTrend = getUptimeTrend(stats.averageUptime);

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its diagnostics."
        />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Diagnostics</h1>
          <p className="text-muted-foreground">
            Monitor system health, crash analysis, and performance diagnostics
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <CreateBundleModal />
        </div>
      </div>

      {/* Health Status Alert */}
      {currentHealthStatus !== 'healthy' && (
        <Alert variant={currentHealthStatus === 'critical' ? 'destructive' : 'default'}>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {currentHealthStatus === 'critical' 
              ? `System health is critical (${stats.systemHealth} tickets). Immediate attention required.`
              : `System health is degraded (${stats.systemHealth} tickets). Monitor closely.`
            }
          </AlertDescription>
        </Alert>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Crashes</CardTitle>
            <Bug className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalCrashes}</div>
            <p className="text-xs text-muted-foreground">
              {stats.resolvedCrashes} resolved
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Critical Issues</CardTitle>
            {stats.criticalCrashes > 0 ? (
              <XCircle className="h-4 w-4 text-red-500" />
            ) : (
              <CheckCircle className="h-4 w-4 text-green-500" />
            )}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">{stats.criticalCrashes}</div>
            <p className="text-xs text-muted-foreground">
              Immediate action required
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">System Health</CardTitle>
            {currentHealthStatus === 'healthy' ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : (
              <AlertTriangle className="h-4 w-4 text-yellow-500" />
            )}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.systemHealth}%</div>
            <p className="text-xs text-muted-foreground">
              Overall system status
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Average Uptime</CardTitle>
            {uptimeTrend && (
              <uptimeTrend.icon className={`h-4 w-4 ${uptimeTrend.color}`} />
            )}
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getUptimeColor(stats.averageUptime)}`}>
              {stats.averageUptime.toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              Last 30 days
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Additional Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center space-x-2">
              <MemoryStick className="h-4 w-4" />
              <span>Memory Leaks</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <span className="text-2xl font-bold">{stats.memoryLeaks}</span>
              <Badge variant={stats.memoryLeaks === 0 ? 'default' : 'destructive'}>
                {stats.memoryLeaks === 0 ? 'None' : 'Active'}
              </Badge>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center space-x-2">
              <Activity className="h-4 w-4" />
              <span>Performance Issues</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <span className="text-2xl font-bold">{stats.performanceIssues}</span>
              <Badge variant={stats.performanceIssues === 0 ? 'default' : 'secondary'}>
                {stats.performanceIssues === 0 ? 'None' : 'Active'}
              </Badge>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center space-x-2">
              <Clock className="h-4 w-4" />
              <span>Last Crash</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <span className="text-sm">
                {stats.lastCrash === 'Never' 
                  ? 'Never' 
                  : safeDateShort(stats.lastCrash)
                }
              </span>
              <Badge variant="outline">
                {stats.lastCrash === 'Never' ? 'Stable' : 'Recent'}
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="crashes">Crash Signatures</TabsTrigger>
          <TabsTrigger value="system">System Health</TabsTrigger>
          <TabsTrigger value="bundles">Diagnostic Bundles</TabsTrigger>
        </TabsList>

        <TabsContent value="crashes" className="flex-1">
          <CrashSignaturesTable />
        </TabsContent>

        <TabsContent value="system" className="flex-1">
          <SystemHealth />
        </TabsContent>

        <TabsContent value="bundles" className="flex-1">
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <FileText className="h-5 w-5" />
                  <span>Diagnostic Bundles</span>
                </CardTitle>
                <CardDescription>
                  Create and download diagnostic bundles for troubleshooting
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-4 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      <FileText className="h-8 w-8 text-blue-500" />
                      <div>
                        <h3 className="font-medium">System Diagnostic Bundle</h3>
                        <p className="text-sm text-muted-foreground">
                          Complete system logs, crash dumps, and performance metrics
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Badge variant="outline">Ready</Badge>
                      <Button size="sm" variant="outline">
                        <Download className="h-4 w-4 mr-2" />
                        Download
                      </Button>
                    </div>
                  </div>

                  <div className="flex items-center justify-between p-4 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      <Bug className="h-8 w-8 text-red-500" />
                      <div>
                        <h3 className="font-medium">Crash Analysis Bundle</h3>
                        <p className="text-sm text-muted-foreground">
                          Crash logs, stack traces, and memory dumps from recent crashes
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Badge variant="secondary">Generating</Badge>
                      <Button size="sm" variant="outline" disabled>
                        <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                        Processing
                      </Button>
                    </div>
                  </div>

                  <div className="flex items-center justify-between p-4 border rounded-lg">
                    <div className="flex items-center space-x-3">
                      <Activity className="h-8 w-8 text-green-500" />
                      <div>
                        <h3 className="font-medium">Performance Bundle</h3>
                        <p className="text-sm text-muted-foreground">
                          Performance metrics, profiling data, and optimization reports
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Badge variant="outline">Available</Badge>
                      <Button size="sm" variant="outline">
                        <Download className="h-4 w-4 mr-2" />
                        Download
                      </Button>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};
