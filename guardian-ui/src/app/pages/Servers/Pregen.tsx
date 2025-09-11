import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import { apiClient as api } from '@/lib/api';

type PregenStatus = {
  state: "idle"|"running"|"paused"|"failed"|"done";
  progress: number; 
  eta_seconds: number|null;
  capabilities: { gpu: boolean; fallback: boolean };
  last_error: string|null;
};
import { Button } from '@/components/ui/button';
import { useServers } from '@/store/servers-new';

interface PregenPageProps {
  className?: string;
}

export const Pregen: React.FC<PregenPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServers();
  const server = serverId ? getServerById(serverId) : null;
  
  const [s, setS] = useState<PregenStatus | null>(null);
  const [err, setErr] = useState<string | null>(null);

  async function refresh() {
    if (!serverId) return;
    try { 
      const status = await api.call<PregenStatus>(`/servers/${serverId}/pregen/status`); 
      setS(status); 
      setErr(null); 
    }
    catch (e: any) { 
      setErr(e.message ?? "unknown"); 
    }
  }

  useEffect(() => { 
    refresh(); 
    const t = setInterval(refresh, 2000); 
    return () => clearInterval(t); 
  }, [serverId]);

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view pregen jobs."
        />
      </div>
    );
  }

  if (err) {
    return (
      <div className="p-6">
        <div className="text-center">
          <h3 className="text-lg font-semibold mb-2">Pregen unavailable</h3>
          <p className="text-muted-foreground">{String(err)}</p>
        </div>
      </div>
    );
  }

  if (!s) {
    return (
      <div className="p-6">
        <div className="text-center">
          <p className="text-muted-foreground">Loading pregen status…</p>
        </div>
      </div>
    );
  }

  const canGpu = s.capabilities?.gpu;

  return (
    <div className={`p-6 space-y-4 ${className}`}>
      {!canGpu && (
        <div className="text-yellow-500 p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
          GPU worker not available—using server-driven fallback.
        </div>
      )}
      
      <div className="space-y-4">
        <div>
          <span className="font-medium">State:</span> <span className="font-bold">{s.state}</span> • 
          <span className="font-medium ml-2">Progress:</span> <span className="font-bold">{(s.progress*100).toFixed(1)}%</span>
        </div>
        
        <div className="space-x-2">
          <Button 
            onClick={() => api.call(`/servers/${serverId}/pregen/start`, { method:"POST" }).then(refresh)}
            disabled={s.state === "running"}
          >
            Start
          </Button>
          <Button 
            onClick={() => api.call(`/servers/${serverId}/pregen/pause`, { method:"POST" }).then(refresh)}
            disabled={s.state !== "running"}
          >
            Pause
          </Button>
          <Button 
            onClick={() => api.call(`/servers/${serverId}/pregen/resume`, { method:"POST" }).then(refresh)}
            disabled={s.state !== "paused"}
          >
            Resume
          </Button>
          <Button 
            onClick={() => api.call(`/servers/${serverId}/pregen/cancel`, { method:"POST" }).then(refresh)}
            disabled={s.state === "idle"}
          >
            Cancel
          </Button>
        </div>
        
        {s.last_error && (
          <div className="text-red-500 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
            Last error: {s.last_error}
          </div>
        )}
      </div>
    </div>
  );
};

export default Pregen;
