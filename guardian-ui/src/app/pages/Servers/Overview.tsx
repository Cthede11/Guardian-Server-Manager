import React from 'react';
import { useParams } from 'react-router-dom';
import { useServers } from '@/store/servers-new';
import { useMetrics, usePlayerData, useServerHealth } from '@/store/live-new';
import { useServerStreams } from '@/app/hooks/useServerStreams';
import { StatCard } from '@/components/StatCard';
import { StatusPill } from '@/components/StatusPill';
import { StatsGridLoading } from '@/components/ui/LoadingStates';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { CheckCircle, XCircle, AlertTriangle, Clock, Users, Zap, HardDrive } from 'lucide-react';
import { useLoadingState } from '@/components/ui/LoadingStates';
import { LoadingWrapper } from '@/components/LoadingWrapper';
import { useStartupDelay } from '@/hooks/useStartupDelay';
import { api } from '@/lib/client';

export const Overview: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById, select } = useServers();
  const selectedServer = serverId ? getServerById(serverId) : null;
  const metrics = useMetrics(serverId || '');
  const players = usePlayerData(serverId || '');
  const health = useServerHealth(serverId || '');
  const { isLoading, error, startLoading, stopLoading, setLoadingError } = useLoadingState();
  const isStartupReady = useStartupDelay(1000); // 1 second delay

  // Select the server when the component mounts
  React.useEffect(() => {
    if (serverId) {
      select(serverId);
    }
  }, [serverId, select]);

  // Attach streams for the selected server
  useServerStreams(serverId);

  // Fetch real server data
  React.useEffect(() => {
    if (!selectedServer || !serverId || !isStartupReady) return;

    const fetchServerData = async () => {
      startLoading();
      try {
        // Fetch server health - now handled by streams
        // const healthResponse = await api.getServerHealth(serverId);

        // Fetch players - now handled by streams
        // const playersResponse = await api.getPlayers(serverId);

        // Fetch real-time metrics - now handled by streams
        // const metricsResponse = await api.getRealtimeMetrics(serverId);

        stopLoading();
      } catch (err) {
        console.error('Failed to fetch server data:', err);
        setLoadingError(err as Error);
        stopLoading();
      }
    };

    // Add a small delay to prevent rapid API calls
    const timeoutId = setTimeout(fetchServerData, 100);

    // Set up polling for real-time data
    const interval = setInterval(fetchServerData, 10000); // Poll every 10 seconds

    return () => {
      clearTimeout(timeoutId);
      clearInterval(interval);
    };
  }, [selectedServer, serverId, isStartupReady, startLoading, stopLoading, setLoadingError]);

  if (!isStartupReady) {
    return (
      <LoadingWrapper
        isLoading={true}
        error={null}
        className="p-6"
      />
    );
  }

  if (!selectedServer) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its overview."
        />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="Failed to load server data"
          description={error.message}
          onRetry={() => {
            setLoadingError(null as any);
            startLoading();
            setTimeout(() => stopLoading(), 1000);
          }}
        />
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-6 space-y-6">
        <div>
          <h1 className="text-2xl font-bold mb-2">{selectedServer.name}</h1>
          <p className="text-muted-foreground">Server Overview</p>
        </div>
        <StatsGridLoading count={4} />
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">{selectedServer.name}</h1>
          <p className="text-muted-foreground">Server Overview</p>
        </div>
        <StatusPill status={selectedServer.status as any} />
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="TPS"
          value={selectedServer.status === 'running' && selectedServer.tps
            ? selectedServer.tps.toFixed(1)
            : '0.0'}
          subtitle="Ticks per second"
          icon={<Zap className="h-4 w-4" />}
        />
        <StatCard
          title="Players"
          value={selectedServer.status === 'running' 
            ? `${selectedServer.players_online || 0}` 
            : '0'}
          subtitle={`${selectedServer.max_players || 20} max`}
          icon={<Users className="h-4 w-4" />}
        />
        <StatCard
          title="Memory"
          value={selectedServer.status === 'running' && selectedServer.heap_mb
            ? `${selectedServer.heap_mb}MB`
            : '0MB'}
          subtitle={`${selectedServer.memory || 4096}MB max`}
          icon={<HardDrive className="h-4 w-4" />}
        />
        <StatCard
          title="Tick Time"
          value={selectedServer.status === 'running' && selectedServer.tick_p95_ms
            ? `${selectedServer.tick_p95_ms.toFixed(1)}ms`
            : '0ms'}
          subtitle="95th percentile"
          icon={<Clock className="h-4 w-4" />}
        />
      </div>

      {/* Health Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5" />
            Health Status
          </CardTitle>
          <CardDescription>
            Current health indicators for {selectedServer.name}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="flex items-center space-x-2">
              {health?.rcon ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <XCircle className="h-4 w-4 text-red-500" />
              )}
              <span className="text-sm">RCON</span>
            </div>
            <div className="flex items-center space-x-2">
              {health?.query ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <XCircle className="h-4 w-4 text-red-500" />
              )}
              <span className="text-sm">Query</span>
            </div>
            <div className="flex items-center space-x-2">
              {(health?.crash_tickets || 0) === 0 ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <AlertTriangle className="h-4 w-4 text-yellow-500" />
              )}
              <span className="text-sm">Crashes</span>
              {(health?.crash_tickets || 0) > 0 && (
                <Badge variant="secondary">{health?.crash_tickets || 0}</Badge>
              )}
            </div>
            <div className="flex items-center space-x-2">
              {(health?.freeze_tickets || 0) === 0 ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <AlertTriangle className="h-4 w-4 text-yellow-500" />
              )}
              <span className="text-sm">Freezes</span>
              {(health?.freeze_tickets || 0) > 0 && (
                <Badge variant="secondary">{health?.freeze_tickets || 0}</Badge>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Blue/Green Deployment Status */}
      {selectedServer.blue_green && (
        <Card>
          <CardHeader>
            <CardTitle>Blue/Green Deployment</CardTitle>
            <CardDescription>
              Current deployment status and health
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-4">
                <div className="flex items-center space-x-2">
                  <div className={`w-3 h-3 rounded-full ${
                    selectedServer.blue_green.active === 'blue' ? 'bg-blue-500' : 'bg-gray-300'
                  }`} />
                  <span className="text-sm font-medium">Blue</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className={`w-3 h-3 rounded-full ${
                    selectedServer.blue_green.active === 'green' ? 'bg-green-500' : 'bg-gray-300'
                  }`} />
                  <span className="text-sm font-medium">Green</span>
                </div>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-sm text-muted-foreground">Active:</span>
                <Badge variant={selectedServer.blue_green.active === 'blue' ? 'default' : 'secondary'}>
                  {selectedServer.blue_green.active}
                </Badge>
              </div>
            </div>
            <div className="mt-4 flex items-center space-x-2">
              <span className="text-sm text-muted-foreground">Candidate Health:</span>
              {selectedServer.blue_green.candidate_healthy ? (
                <Badge variant="default" className="bg-green-500">
                  Healthy
                </Badge>
              ) : (
                <Badge variant="destructive">
                  Unhealthy
                </Badge>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default Overview;
