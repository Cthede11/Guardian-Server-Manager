import React from 'react';
import { useParams } from 'react-router-dom';
import { useServersStore } from '@/store/servers';
import { useConnectionStatus } from '@/store/live';
import TpsChart from '@/components/Charts/TpsChart';
import HeapChart from '@/components/Charts/HeapChart';
import PhaseChart from '@/components/Charts/PhaseChart';
import LatencyChart from '@/components/Charts/LatencyChart';

export const Performance: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  const { connected } = useConnectionStatus();

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view performance metrics</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Performance Metrics</h2>
          <p className="text-muted-foreground">
            Real-time performance monitoring for {server.name}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${connected ? 'bg-success' : 'bg-danger'}`} />
          <span className="text-sm text-muted-foreground">
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <TpsChart serverId={serverId!} />
        <HeapChart serverId={serverId!} />
        <PhaseChart serverId={serverId!} />
        <LatencyChart serverId={serverId!} />
      </div>

      {/* Performance Summary */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Current TPS</div>
          <div className="text-2xl font-bold text-success">20.0</div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Memory Usage</div>
          <div className="text-2xl font-bold text-warning">2.1 GB</div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Tick Time (P95)</div>
          <div className="text-2xl font-bold text-accent">45.2 ms</div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">GPU Latency</div>
          <div className="text-2xl font-bold text-danger">12.8 ms</div>
        </div>
      </div>
    </div>
  );
};

export default Performance;