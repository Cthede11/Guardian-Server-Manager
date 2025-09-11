import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useServers } from '@/store/servers-new';
import { useConnectionStatus } from '@/store/live';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import TpsChart from '@/components/Charts/TpsChart';
import HeapChart from '@/components/Charts/HeapChart';
import PhaseChart from '@/components/Charts/PhaseChart';
import LatencyChart from '@/components/Charts/LatencyChart';
import { subscribeMetrics, type MetricsPoint } from '@/features/performance/useMetrics';
import { apiClient as api } from '@/lib/api';

export const Performance: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServers();
  const server = serverId ? getServerById(serverId) : null;
  const { connected } = useConnectionStatus();
  
  const [metrics, setMetrics] = useState<MetricsPoint[]>([]);
  const [currentMetrics, setCurrentMetrics] = useState<MetricsPoint | null>(null);

  useEffect(() => {
    if (!serverId) return;

    // Load historical metrics
    const loadHistory = async () => {
      try {
        const history = await api.call<MetricsPoint[]>(`/servers/${serverId}/metrics`);
        setMetrics(history);
      } catch (error) {
        console.error('Failed to load metrics history:', error);
      }
    };

    loadHistory();

    // Subscribe to live metrics
    const unsubscribe = subscribeMetrics(serverId, (point) => {
      setMetrics(prev => [...prev.slice(-599), point]); // Keep last 600 points
      setCurrentMetrics(point);
    });

    return unsubscribe;
  }, [serverId]);

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its performance metrics."
        />
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
          <div className="text-2xl font-bold text-success">
            {currentMetrics ? currentMetrics.tps.toFixed(1) : '—'}
          </div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Memory Usage</div>
          <div className="text-2xl font-bold text-warning">
            {currentMetrics?.heap_mb ? `${currentMetrics.heap_mb.toFixed(1)} GB` : '—'}
          </div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Tick Time (P95)</div>
          <div className="text-2xl font-bold text-accent">
            {currentMetrics ? `${currentMetrics.tick_p95_ms.toFixed(1)} ms` : '—'}
          </div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">GPU Latency</div>
          <div className="text-2xl font-bold text-danger">
            {currentMetrics?.gpu_latency_ms ? `${currentMetrics.gpu_latency_ms.toFixed(1)} ms` : '—'}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Performance;
