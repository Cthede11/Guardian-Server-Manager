import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useServers } from '@/store/servers-new';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { RefreshCw, AlertTriangle, CheckCircle, XCircle, Plus, Settings } from 'lucide-react';
import { ShardingTopology } from '@/components/Sharding/ShardingTopology';
import { ShardingAssignment } from '@/components/Sharding/ShardingAssignment';
import { ShardingWarnings } from '@/components/Sharding/ShardingWarnings';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import { apiClient as api } from '@/lib/api';
import { safeTimeString } from '@/lib/formatters';

interface ShardingStats {
  totalShards: number;
  activeShards: number;
  healthyShards: number;
  unhealthyShards: number;
  totalPlayers: number;
  averageLoad: number;
  lastHealthCheck: string;
}

export const Sharding: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServers();
  const server = serverId ? getServerById(serverId) : null;
  const [activeTab, setActiveTab] = useState('topology');
  const [stats, setStats] = useState<ShardingStats>({
    totalShards: 0,
    activeShards: 0,
    healthyShards: 0,
    unhealthyShards: 0,
    totalPlayers: 0,
    averageLoad: 0,
    lastHealthCheck: 'Never'
  });
  const [isLoading, setIsLoading] = useState(false);
  // const [lastRefresh, setLastRefresh] = useState<Date>(new Date());

  const fetchStats = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      // Real API call to get sharding stats
      const response = await api.getShardingTopology();
      if (response.ok && response.data) {
        const topologyData = response.data as any;
        const nodes = topologyData.nodes || [];
        
        const stats: ShardingStats = {
          totalShards: nodes.length,
          activeShards: nodes.filter((node: any) => node.status !== 'offline').length,
          healthyShards: nodes.filter((node: any) => node.status === 'healthy').length,
          unhealthyShards: nodes.filter((node: any) => node.status === 'critical' || node.status === 'warning').length,
          totalPlayers: nodes.reduce((sum: number, node: any) => sum + (node.players || 0), 0),
          averageLoad: nodes.length > 0 ? nodes.reduce((sum: number, node: any) => sum + (node.cpu || 0), 0) / nodes.length : 0,
          lastHealthCheck: new Date().toISOString()
        };
        
        setStats(stats);
      } else {
        // If no data available, show empty state
        setStats({
          totalShards: 0,
          activeShards: 0,
          healthyShards: 0,
          unhealthyShards: 0,
          totalPlayers: 0,
          averageLoad: 0,
          lastHealthCheck: 'Never'
        });
      }
    } catch (error) {
      console.error('Failed to fetch sharding stats:', error);
      setStats({
        totalShards: 0,
        activeShards: 0,
        healthyShards: 0,
        unhealthyShards: 0,
        totalPlayers: 0,
        averageLoad: 0,
        lastHealthCheck: 'Never'
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (serverId) {
      fetchStats();
      
      // Auto-refresh every 30 seconds
      const interval = setInterval(fetchStats, 30000);
      return () => clearInterval(interval);
    }
  }, [serverId]);

  const handleRefresh = () => {
    fetchStats();
  };

  const getHealthStatus = () => {
    if (stats.unhealthyShards === 0) return 'healthy';
    if (stats.unhealthyShards <= 2) return 'warning';
    return 'critical';
  };

  const healthStatus = getHealthStatus();

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its sharding configuration."
        />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Sharding</h1>
          <p className="text-muted-foreground">
            Manage server sharding topology and player distribution
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
          <Button size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Add Shard
          </Button>
        </div>
      </div>

      {/* Health Status Alert */}
      {healthStatus !== 'healthy' && (
        <Alert variant={healthStatus === 'critical' ? 'destructive' : 'default'}>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {healthStatus === 'critical' 
              ? `${stats.unhealthyShards} shards are unhealthy. Immediate attention required.`
              : `${stats.unhealthyShards} shards have issues. Monitor closely.`
            }
          </AlertDescription>
        </Alert>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Shards</CardTitle>
            <Settings className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalShards}</div>
            <p className="text-xs text-muted-foreground">
              {stats.activeShards} active
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Health Status</CardTitle>
            {healthStatus === 'healthy' ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : (
              <XCircle className="h-4 w-4 text-red-500" />
            )}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">{stats.healthyShards}</div>
            <p className="text-xs text-muted-foreground">
              {stats.unhealthyShards} unhealthy
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Players</CardTitle>
            <Badge variant="secondary">{stats.totalPlayers}</Badge>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalPlayers}</div>
            <p className="text-xs text-muted-foreground">
              Across all shards
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Average Load</CardTitle>
            <Badge variant={stats.averageLoad > 80 ? 'destructive' : stats.averageLoad > 60 ? 'default' : 'secondary'}>
              {stats.averageLoad.toFixed(1)}%
            </Badge>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.averageLoad.toFixed(1)}%</div>
            <p className="text-xs text-muted-foreground">
              Last updated: {safeTimeString(stats.lastHealthCheck)}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="topology">Topology</TabsTrigger>
          <TabsTrigger value="assignment">Assignment</TabsTrigger>
          <TabsTrigger value="warnings">Warnings</TabsTrigger>
        </TabsList>

        <TabsContent value="topology" className="flex-1">
          <ShardingTopology />
        </TabsContent>

        <TabsContent value="assignment" className="flex-1">
          <ShardingAssignment />
        </TabsContent>

        <TabsContent value="warnings" className="flex-1">
          <ShardingWarnings />
        </TabsContent>
      </Tabs>
    </div>
  );
};
