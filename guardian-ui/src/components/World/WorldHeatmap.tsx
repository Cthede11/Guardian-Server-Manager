import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useServersStore } from '@/store/servers';
import { useWorldData } from '@/store/live';

interface HeatmapCell {
  x: number;
  z: number;
  intensity: number;
  lastUpdate: number;
}

interface WorldHeatmapProps {
  className?: string;
}

export const WorldHeatmap: React.FC<WorldHeatmapProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  const worldData = useWorldData(serverId || '');
  const freezes = worldData?.freezes || [];
  
  const [heatmapData, setHeatmapData] = useState<HeatmapCell[]>([]);
  const [isVisible, setIsVisible] = useState(true);
  const [throttleRate, setThrottleRate] = useState(10); // 10 Hz max
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastRenderRef = useRef<number>(0);
  const animationFrameRef = useRef<number | undefined>(undefined);

  // Throttled rendering function
  const renderHeatmap = useCallback(() => {
    const now = Date.now();
    const timeSinceLastRender = now - lastRenderRef.current;
    const minInterval = 1000 / throttleRate; // Convert Hz to ms

    if (timeSinceLastRender < minInterval) {
      animationFrameRef.current = requestAnimationFrame(renderHeatmap);
      return;
    }

    const canvas = canvasRef.current;
    if (!canvas || !isVisible) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Render heatmap cells
    heatmapData.forEach(cell => {
      const alpha = Math.min(cell.intensity, 1);
      const color = `rgba(255, 0, 0, ${alpha * 0.7})`;
      
      ctx.fillStyle = color;
      ctx.fillRect(cell.x * 4, cell.z * 4, 4, 4);
    });

    lastRenderRef.current = now;
  }, [heatmapData, throttleRate, isVisible]);

  // Generate mock heatmap data based on freezes
  const generateHeatmapData = useCallback((): HeatmapCell[] => {
    const cells: HeatmapCell[] = [];
    const now = Date.now();
    
    // Generate some mock data based on freeze locations
    for (let i = 0; i < 100; i++) {
      cells.push({
        x: Math.floor(Math.random() * 100),
        z: Math.floor(Math.random() * 100),
        intensity: Math.random(),
        lastUpdate: now,
      });
    }

    // Add freeze locations with higher intensity
    freezes.forEach((_freeze: any) => {
      cells.push({
        x: Math.floor(Math.random() * 100),
        z: Math.floor(Math.random() * 100),
        intensity: 0.8 + Math.random() * 0.2,
        lastUpdate: now,
      });
    });

    return cells;
  }, [freezes]);

  // Update heatmap data periodically
  useEffect(() => {
    if (!serverId || !server) return;

    const updateData = () => {
      setHeatmapData(generateHeatmapData());
    };

    // Initial data
    updateData();

    // Update every 5 seconds
    const interval = setInterval(updateData, 5000);
    return () => clearInterval(interval);
  }, [serverId, server, generateHeatmapData]);

  // Visibility change handler
  useEffect(() => {
    const handleVisibilityChange = () => {
      setIsVisible(!document.hidden);
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, []);

  // Start/stop rendering based on visibility
  useEffect(() => {
    if (isVisible) {
      animationFrameRef.current = requestAnimationFrame(renderHeatmap);
    } else {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [isVisible, renderHeatmap]);

  // Memoized canvas dimensions
  const canvasDimensions = useMemo(() => ({
    width: 400,
    height: 400,
  }), []);

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view world heatmap</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Controls */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">World Heatmap</h3>
        <div className="flex items-center gap-4">
          <label className="text-sm">
            Render Rate: {throttleRate} Hz
            <input
              type="range"
              min="1"
              max="60"
              value={throttleRate}
              onChange={(e) => setThrottleRate(Number(e.target.value))}
              className="ml-2"
            />
          </label>
          <div className="text-sm text-muted-foreground">
            {heatmapData.length} cells
          </div>
        </div>
      </div>

      {/* Heatmap Canvas */}
      <div className="panel p-4">
        <canvas
          ref={canvasRef}
          width={canvasDimensions.width}
          height={canvasDimensions.height}
          className="border border-border rounded"
          style={{ imageRendering: 'pixelated' }}
        />
        
        {/* Legend */}
        <div className="mt-4 flex items-center gap-4 text-sm">
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 bg-red-500/30 border border-red-500"></div>
            <span>Low Activity</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 bg-red-500/70 border border-red-500"></div>
            <span>Medium Activity</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 bg-red-500 border border-red-500"></div>
            <span>High Activity</span>
          </div>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4">
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Total Cells</div>
          <div className="text-2xl font-bold">{heatmapData.length}</div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Active Freezes</div>
          <div className="text-2xl font-bold">{freezes.length}</div>
        </div>
        <div className="panel p-4">
          <div className="text-sm text-muted-foreground">Render Rate</div>
          <div className="text-2xl font-bold">{throttleRate} Hz</div>
        </div>
      </div>
    </div>
  );
};

export default WorldHeatmap;