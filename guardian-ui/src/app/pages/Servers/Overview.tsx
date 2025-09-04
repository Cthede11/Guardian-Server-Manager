import React from 'react';
import { useParams } from 'react-router-dom';
import { useServersStore } from '@/store/servers';
import { useMetrics } from '@/store/live';
import { StatCard } from '@/components/StatCard';
import { StatusPill } from '@/components/StatusPill';
import { StatsGridLoading } from '@/components/ui/LoadingStates';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { CheckCircle, XCircle, AlertTriangle, Clock, Users, Zap, HardDrive } from 'lucide-react';
import { useLoadingState } from '@/components/ui/LoadingStates';

export const Overview: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const selectedServer = serverId ? getServerById(serverId) : null;
  const metrics = useMetrics(serverId || '');
  const { isLoading, error, startLoading, stopLoading, setLoadingError } = useLoadingState();

  // Mock health data for now
  const health = {
    rcon: true,
    query: true,
    crashTickets: 0,
    freezeTickets: 0,
  };

  React.useEffect(() => {
    if (selectedServer) {
      startLoading();
      // Simulate loading
      setTimeout(() => {
        stopLoading();
      }, 1000);
    }
  }, [selectedServer, startLoading, stopLoading]);

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
        <StatusPill status={selectedServer.status} />
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="TPS"
          value={metrics?.tps?.[metrics.tps.length - 1]?.value?.toFixed(1) || '20.0'}
          subtitle="Ticks per second"
          icon={<Zap className="h-4 w-4" />}
        />
        <StatCard
          title="Players"
          value="12"
          subtitle="20 max"
          icon={<Users className="h-4 w-4" />}
        />
        <StatCard
          title="Memory"
          value={`${metrics?.heap?.[metrics.heap.length - 1]?.value ? Math.round(metrics.heap[metrics.heap.length - 1].value / 1024 / 1024) : 2048}MB`}
          subtitle="4096MB max"
          icon={<HardDrive className="h-4 w-4" />}
        />
        <StatCard
          title="Tick Time"
          value={`${metrics?.tickP95?.[metrics.tickP95.length - 1]?.value?.toFixed(1) || 45.2}ms`}
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
              {health?.crashTickets === 0 ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <AlertTriangle className="h-4 w-4 text-yellow-500" />
              )}
              <span className="text-sm">Crashes</span>
              {health?.crashTickets > 0 && (
                <Badge variant="secondary">{health.crashTickets}</Badge>
              )}
            </div>
            <div className="flex items-center space-x-2">
              {health?.freezeTickets === 0 ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <AlertTriangle className="h-4 w-4 text-yellow-500" />
              )}
              <span className="text-sm">Freezes</span>
              {health?.freezeTickets > 0 && (
                <Badge variant="secondary">{health.freezeTickets}</Badge>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Blue/Green Deployment Status */}
      {selectedServer.blueGreen && (
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
                    selectedServer.blueGreen.active === 'blue' ? 'bg-blue-500' : 'bg-gray-300'
                  }`} />
                  <span className="text-sm font-medium">Blue</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className={`w-3 h-3 rounded-full ${
                    selectedServer.blueGreen.active === 'green' ? 'bg-green-500' : 'bg-gray-300'
                  }`} />
                  <span className="text-sm font-medium">Green</span>
                </div>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-sm text-muted-foreground">Active:</span>
                <Badge variant={selectedServer.blueGreen.active === 'blue' ? 'default' : 'secondary'}>
                  {selectedServer.blueGreen.active}
                </Badge>
              </div>
            </div>
            <div className="mt-4 flex items-center space-x-2">
              <span className="text-sm text-muted-foreground">Candidate Health:</span>
              {selectedServer.blueGreen.candidateHealthy ? (
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
