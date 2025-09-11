import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { useServers } from '@/store/servers-new';
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
  const { getServerById } = useServers();
  const server = serverId ? getServerById(serverId) : null;
  const worldData = useWorldData(serverId || '');
  const freezes = (worldData as any)?.freezes || [];
  
  const [heatmapData, setHeatmapData] = useState<HeatmapCell[]>([]);
  const [isVisible, setIsVisible] = useState(true);
  const [isPaused, setIsPaused] = useState(false);
  const [throttleRate, setThrottleRate] = useState(2);
  const [zoom, setZoom] = useState(1);
  const [panX, setPanX] = useState(0);
  const [panZ, setPanZ] = useState(0);
  const [filterType, setFilterType] = useState<'all' | 'high' | 'medium' | 'low'>('all');
  const [showGrid, setShowGrid] = useState(true);
  const [selectedCell, setSelectedCell] = useState<HeatmapCell | null>(null);
  const [lastUpdateTime, setLastUpdateTime] = useState<number>(Date.now());
  const [updateCount, setUpdateCount] = useState<number>(0);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastRenderRef = useRef<number>(0);
  const animationFrameRef = useRef<number | undefined>(undefined);
  const isRenderingRef = useRef<boolean>(false);

  // Optimized rendering function with zoom, pan, and filtering
  const renderHeatmap = useCallback(() => {
    if (isRenderingRef.current) return;
    
    const canvas = canvasRef.current;
    if (!canvas || !isVisible) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    isRenderingRef.current = true;

    // Get container dimensions
    const container = canvas.parentElement;
    if (!container) {
      isRenderingRef.current = false;
      return;
    }

    const containerRect = container.getBoundingClientRect();
    const canvasWidth = containerRect.width - 32; // Account for padding
    const canvasHeight = 500; // Fixed height

    // Set canvas size
    canvas.width = canvasWidth;
    canvas.height = canvasHeight;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw update indicator in top-right corner
    ctx.fillStyle = 'rgba(0, 255, 0, 0.7)';
    ctx.fillRect(canvas.width - 20, 10, 10, 10);
    ctx.fillStyle = 'white';
    ctx.font = '10px monospace';
    ctx.fillText(updateCount.toString(), canvas.width - 35, 20);

    // Calculate cell size based on zoom
    const baseCellSize = 6;
    const cellSize = baseCellSize * zoom;
    const offsetX = (canvas.width / 2) + panX;
    const offsetZ = (canvas.height / 2) + panZ;

    // Render grid if enabled
    if (showGrid) {
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
      ctx.lineWidth = 0.5;
      const gridSpacing = cellSize * 5;
      
      for (let x = 0; x < canvas.width; x += gridSpacing) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, canvas.height);
        ctx.stroke();
      }
      for (let z = 0; z < canvas.height; z += gridSpacing) {
        ctx.beginPath();
        ctx.moveTo(0, z);
        ctx.lineTo(canvas.width, z);
        ctx.stroke();
      }
    }

    // Filter and render heatmap cells
    const filteredData = heatmapData.filter(cell => {
      if (filterType === 'all') return true;
      const intensity = cell.intensity;
      if (filterType === 'high') return intensity >= 0.5;
      if (filterType === 'medium') return intensity >= 0.25 && intensity < 0.5;
      if (filterType === 'low') return intensity < 0.25;
      return true;
    });

    filteredData.forEach(cell => {
      const intensity = Math.min(cell.intensity, 1);
      let color;
      
      if (intensity < 0.25) {
        color = `rgba(107, 114, 128, ${intensity * 0.3})`;
      } else if (intensity < 0.5) {
        color = `rgba(249, 115, 22, ${intensity * 0.5})`;
      } else if (intensity < 0.8) {
        color = `rgba(239, 68, 68, ${intensity * 0.7})`;
      } else {
        color = `rgba(220, 38, 38, ${intensity})`;
      }
      
      const x = offsetX + (cell.x - 25) * cellSize;
      const z = offsetZ + (cell.z - 25) * cellSize;
      
      // Only render if cell is visible on canvas
      if (x + cellSize > 0 && x < canvas.width && z + cellSize > 0 && z < canvas.height) {
        ctx.fillStyle = color;
        ctx.fillRect(x, z, cellSize, cellSize);
      }
    });

    isRenderingRef.current = false;
  }, [heatmapData, isVisible, zoom, panX, panZ, filterType, showGrid, updateCount]);

  // Generate heatmap data from real world data
  const generateHeatmapData = useCallback((): HeatmapCell[] => {
    const cells: HeatmapCell[] = [];
    const now = Date.now();
    
    // Use real freeze data if available
    if (freezes && freezes.length > 0) {
      freezes.forEach((freeze: any) => {
        cells.push({
          x: freeze.x || 0,
          z: freeze.z || 0,
          intensity: freeze.intensity || 0.8,
          lastUpdate: freeze.timestamp || now,
        });
      });
    }

    return cells;
  }, [freezes]);

  // Update heatmap data periodically - controlled by throttleRate
  useEffect(() => {
    if (!serverId || !server || isPaused) return;

    const updateData = () => {
      if (!isPaused) {
        setHeatmapData(generateHeatmapData());
        setLastUpdateTime(Date.now());
        setUpdateCount(prev => prev + 1);
      }
    };

    // Initial data
    updateData();

    // Update based on throttleRate (convert Hz to milliseconds)
    const updateInterval = 1000 / throttleRate;
    console.log(`Setting update interval to ${updateInterval}ms (${throttleRate} Hz)`);
    const interval = setInterval(updateData, updateInterval);
    return () => {
      console.log('Clearing update interval');
      clearInterval(interval);
    };
  }, [serverId, server, generateHeatmapData, isPaused, throttleRate]);

  // Visibility change handler
  useEffect(() => {
    const handleVisibilityChange = () => {
      setIsVisible(!document.hidden);
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, []);

  // Render when data changes, visibility changes, or view parameters change
  useEffect(() => {
    if (isVisible && heatmapData.length > 0 && !isPaused) {
      // Use setTimeout to avoid blocking the main thread
      const timeoutId = setTimeout(() => {
        renderHeatmap();
      }, 0);
      
      return () => clearTimeout(timeoutId);
    }
  }, [isVisible, heatmapData, isPaused, renderHeatmap, zoom, panX, panZ, filterType, showGrid]);

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      if (isVisible && !isPaused) {
        setTimeout(() => renderHeatmap(), 100);
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [isVisible, isPaused, renderHeatmap]);

  // Mouse interaction handlers
  const handleMouseWheel = useCallback((e: WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom(prev => Math.max(0.5, Math.min(3, prev * delta)));
  }, []);

  const handleMouseDown = useCallback((e: MouseEvent) => {
    const startX = e.clientX - panX;
    const startZ = e.clientY - panZ;
    
    const handleMouseMove = (moveEvent: MouseEvent) => {
      setPanX(moveEvent.clientX - startX);
      setPanZ(moveEvent.clientY - startZ);
    };
    
    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [panX, panZ]);

  const handleCanvasClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const z = e.clientY - rect.top;
    
    const cellSize = Math.max(2, Math.min(8, (canvas.width / 50) * zoom));
    const offsetX = (canvas.width / 2) + panX;
    const offsetZ = (canvas.height / 2) + panZ;
    
    const cellX = Math.floor((x - offsetX) / cellSize) + 25;
    const cellZ = Math.floor((z - offsetZ) / cellSize) + 25;
    
    const clickedCell = heatmapData.find(cell => 
      Math.abs(cell.x - cellX) < 1 && Math.abs(cell.z - cellZ) < 1
    );
    
    setSelectedCell(clickedCell || null);
  }, [heatmapData, zoom, panX, panZ]);

  // Add event listeners
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    canvas.addEventListener('wheel', handleMouseWheel);
    canvas.addEventListener('mousedown', handleMouseDown);
    
    return () => {
      canvas.removeEventListener('wheel', handleMouseWheel);
      canvas.removeEventListener('mousedown', handleMouseDown);
    };
  }, [handleMouseWheel, handleMouseDown]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = undefined;
      }
    };
  }, []);

  // Canvas dimensions are now handled dynamically in renderHeatmap

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
    <div className={`space-y-6 ${className}`}>
      {/* Header and Main Controls */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">World Heatmap</h3>
        <div className="flex items-center gap-4">
          <button
            onClick={() => setIsPaused(!isPaused)}
            className={`px-3 py-1 text-sm rounded-md transition-colors ${
              isPaused 
                ? 'bg-green-500 text-white hover:bg-green-600' 
                : 'bg-red-500 text-white hover:bg-red-600'
            }`}
          >
            {isPaused ? 'Resume' : 'Pause'}
          </button>
          <button
            onClick={() => {
              setZoom(1);
              setPanX(0);
              setPanZ(0);
            }}
            className="px-3 py-1 text-sm bg-blue-500 text-white rounded-md hover:bg-blue-600 transition-colors"
          >
            Reset View
          </button>
        </div>
      </div>

      {/* Control Panel */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
        {/* View Controls */}
        <div className="panel p-4">
          <h4 className="text-sm font-medium mb-3">View Controls</h4>
          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground">Zoom: {zoom.toFixed(1)}x</label>
              <input
                type="range"
                min="0.5"
                max="3"
                step="0.1"
                value={zoom}
                onChange={(e) => setZoom(Number(e.target.value))}
                className="w-full mt-1"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground">Update Rate: {throttleRate} Hz</label>
              <input
                type="range"
                min="1"
                max="10"
                value={throttleRate}
                onChange={(e) => setThrottleRate(Number(e.target.value))}
                className="w-full mt-1"
                disabled={isPaused}
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="showGrid"
                checked={showGrid}
                onChange={(e) => setShowGrid(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="showGrid" className="text-xs text-muted-foreground">Show Grid</label>
            </div>
          </div>
        </div>

        {/* Filter Controls */}
        <div className="panel p-4">
          <h4 className="text-sm font-medium mb-3">Filters</h4>
          <div className="space-y-2">
            {[
              { value: 'all', label: 'All Activity', color: 'bg-gray-500' },
              { value: 'high', label: 'High Activity', color: 'bg-red-500' },
              { value: 'medium', label: 'Medium Activity', color: 'bg-orange-500' },
              { value: 'low', label: 'Low Activity', color: 'bg-gray-400' }
            ].map((filter) => (
              <button
                key={filter.value}
                onClick={() => setFilterType(filter.value as any)}
                className={`w-full text-left px-2 py-1 text-xs rounded transition-colors ${
                  filterType === filter.value 
                    ? 'bg-primary text-primary-foreground' 
                    : 'hover:bg-accent'
                }`}
              >
                <div className="flex items-center gap-2">
                  <div className={`w-3 h-3 rounded ${filter.color}`}></div>
                  {filter.label}
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Stats */}
        <div className="panel p-4">
          <h4 className="text-sm font-medium mb-3">Statistics</h4>
          <div className="space-y-2">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Total Cells:</span>
              <span className="font-medium">{heatmapData.length}</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Active Freezes:</span>
              <span className="font-medium">{freezes.length}</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Filtered Cells:</span>
              <span className="font-medium">
                {heatmapData.filter(cell => {
                  if (filterType === 'all') return true;
                  const intensity = cell.intensity;
                  if (filterType === 'high') return intensity >= 0.5;
                  if (filterType === 'medium') return intensity >= 0.25 && intensity < 0.5;
                  if (filterType === 'low') return intensity < 0.25;
                  return true;
                }).length}
              </span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Zoom Level:</span>
              <span className="font-medium">{zoom.toFixed(1)}x</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Last Update:</span>
              <span className="font-medium">
                {new Date(lastUpdateTime).toLocaleTimeString()}
              </span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Update Count:</span>
              <span className="font-medium">{updateCount}</span>
            </div>
          </div>
        </div>

        {/* Selected Cell Info */}
        <div className="panel p-4">
          <h4 className="text-sm font-medium mb-3">Cell Information</h4>
          {selectedCell ? (
            <div className="space-y-2 text-xs">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Position:</span>
                <span className="font-medium">({selectedCell.x}, {selectedCell.z})</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Intensity:</span>
                <span className="font-medium">{(selectedCell.intensity * 100).toFixed(1)}%</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Last Update:</span>
                <span className="font-medium">
                  {new Date(selectedCell.lastUpdate).toLocaleTimeString()}
                </span>
              </div>
            </div>
          ) : (
            <p className="text-xs text-muted-foreground">Click on a cell to view details</p>
          )}
        </div>
      </div>

      {/* Heatmap Canvas */}
      <div className="panel p-4">
        <div className="mb-4">
          <h4 className="text-sm font-medium mb-2">Interactive Heatmap</h4>
          <p className="text-xs text-muted-foreground">
            Use mouse wheel to zoom, drag to pan, and click cells for details
          </p>
        </div>
        <div className="relative w-full">
          <canvas
            ref={canvasRef}
            onClick={handleCanvasClick}
            className="border border-border rounded cursor-crosshair w-full"
            style={{ 
              imageRendering: 'pixelated',
              width: '100%',
              height: '500px',
              display: 'block'
            }}
          />
        </div>
        
        {/* Legend */}
        <div className="mt-4 space-y-3">
          <h4 className="text-sm font-medium text-foreground">Activity Legend</h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 bg-gray-500/30 border border-gray-500 rounded"></div>
              <span className="text-muted-foreground">Low Activity (0-25%)</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 bg-orange-500/50 border border-orange-500 rounded"></div>
              <span className="text-muted-foreground">Medium Activity (25-50%)</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 bg-red-500/70 border border-red-500 rounded"></div>
              <span className="text-muted-foreground">High Activity (50-80%)</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 bg-red-500 border border-red-500 rounded"></div>
              <span className="text-muted-foreground">Critical Issues (80-100%)</span>
            </div>
          </div>
          <p className="text-xs text-muted-foreground">
            Each cell represents a 4x4 chunk area. Red areas may indicate performance issues or high player activity.
          </p>
        </div>
      </div>
    </div>
  );
};

export default WorldHeatmap;
