import React, { useState, useEffect, useRef } from 'react';
import { 
  MapPin, 
  Eye, 
  Thermometer,
  Users,
  Zap,
  Grid3X3
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

interface WorldHeatmapProps {
  dimension: string;
  heatmapType: string;
  onChunkClick: (chunk: any) => void;
  frozenChunks: any[];
  className?: string;
}

export const WorldHeatmap: React.FC<WorldHeatmapProps> = ({
  dimension,
  heatmapType,
  onChunkClick,
  frozenChunks,
  className = ''
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hoveredChunk, setHoveredChunk] = useState<any>(null);
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Generate mock chunk data for demo
  const generateMockChunks = () => {
    const chunks = [];
    const size = 20; // 20x20 grid
    
    for (let x = -size/2; x < size/2; x++) {
      for (let z = -size/2; z < size/2; z++) {
        const distance = Math.sqrt(x * x + z * z);
        const isFrozen = frozenChunks.some(fc => fc.x === x && fc.z === z);
        
        let value = 0;
        switch (heatmapType) {
          case 'tps':
            value = Math.max(0, Math.min(5, 2 + Math.sin(x * 0.3) * Math.cos(z * 0.3) + (Math.random() - 0.5) * 0.5));
            break;
          case 'entities':
            value = Math.max(0, Math.min(100, 20 + Math.sin(x * 0.2) * Math.cos(z * 0.2) * 30 + (Math.random() - 0.5) * 20));
            break;
          case 'redstone':
            value = Math.max(0, Math.min(10, 1 + Math.sin(x * 0.4) * Math.cos(z * 0.4) * 3 + (Math.random() - 0.5) * 2));
            break;
          case 'chunks':
            value = Math.max(0, Math.min(1, 0.3 + Math.sin(x * 0.1) * Math.cos(z * 0.1) * 0.3 + (Math.random() - 0.5) * 0.2));
            break;
        }
        
        chunks.push({
          x,
          z,
          value,
          isFrozen,
          dimension,
          entities: Math.floor(Math.random() * 50),
          tileEntities: Math.floor(Math.random() * 20),
          lastUpdate: new Date(Date.now() - Math.random() * 300000)
        });
      }
    }
    
    return chunks;
  };

  const [chunks, setChunks] = useState<any[]>([]);

  useEffect(() => {
    setChunks(generateMockChunks());
  }, [dimension, heatmapType, frozenChunks]);

  // Draw heatmap
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = 600;
    canvas.height = 600;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw grid
    const cellSize = 30 * zoom;
    const startX = (canvas.width / 2) + offset.x;
    const startY = (canvas.height / 2) + offset.y;

    chunks.forEach(chunk => {
      const x = startX + (chunk.x * cellSize);
      const y = startY + (chunk.z * cellSize);

      // Skip if outside viewport
      if (x < -cellSize || x > canvas.width + cellSize || y < -cellSize || y > canvas.height + cellSize) {
        return;
      }

      // Get color based on value and heatmap type
      let color = getHeatmapColor(chunk.value, heatmapType);
      
      // Override color for frozen chunks
      if (chunk.isFrozen) {
        color = '#3b82f6'; // Blue for frozen
      }

      // Draw chunk
      ctx.fillStyle = color;
      ctx.fillRect(x, y, cellSize - 1, cellSize - 1);

      // Draw border
      ctx.strokeStyle = chunk.isFrozen ? '#1e40af' : '#374151';
      ctx.lineWidth = 1;
      ctx.strokeRect(x, y, cellSize - 1, cellSize - 1);

      // Draw coordinates for frozen chunks
      if (chunk.isFrozen) {
        ctx.fillStyle = '#ffffff';
        ctx.font = '10px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(`${chunk.x},${chunk.z}`, x + cellSize/2, y + cellSize/2 + 3);
      }
    });

    // Draw center crosshair
    ctx.strokeStyle = '#ef4444';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(startX - 10, startY);
    ctx.lineTo(startX + 10, startY);
    ctx.moveTo(startX, startY - 10);
    ctx.lineTo(startX, startY + 10);
    ctx.stroke();

  }, [chunks, zoom, offset, heatmapType]);

  const getHeatmapColor = (value: number, type: string) => {
    let normalizedValue = value;
    
    switch (type) {
      case 'tps':
        normalizedValue = value / 5; // Max 5 TPS impact
        break;
      case 'entities':
        normalizedValue = value / 100; // Max 100 entities
        break;
      case 'redstone':
        normalizedValue = value / 10; // Max 10 redstone
        break;
      case 'chunks':
        normalizedValue = value; // Already 0-1
        break;
    }

    // Create heatmap color (green -> yellow -> red)
    const r = Math.min(255, Math.floor(normalizedValue * 255));
    const g = Math.min(255, Math.floor((1 - normalizedValue) * 255));
    const b = 0;
    
    return `rgb(${r}, ${g}, ${b})`;
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (isDragging) {
      setOffset(prev => ({
        x: prev.x + (x - dragStart.x),
        y: prev.y + (y - dragStart.y)
      }));
      setDragStart({ x, y });
      return;
    }

    // Find hovered chunk
    const cellSize = 30 * zoom;
    const startX = (canvas.width / 2) + offset.x;
    const startY = (canvas.height / 2) + offset.y;

    const chunkX = Math.floor((x - startX) / cellSize);
    const chunkZ = Math.floor((y - startY) / cellSize);

    const hovered = chunks.find(chunk => chunk.x === chunkX && chunk.z === chunkZ);
    setHoveredChunk(hovered);
  };

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    setIsDragging(true);
    const rect = canvasRef.current?.getBoundingClientRect();
    if (rect) {
      setDragStart({
        x: e.clientX - rect.left,
        y: e.clientY - rect.top
      });
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (isDragging) return;
    
    if (hoveredChunk) {
      onChunkClick(hoveredChunk);
    }
  };

  const handleWheel = (e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom(prev => Math.max(0.5, Math.min(3, prev * delta)));
  };

  const getHeatmapIcon = () => {
    switch (heatmapType) {
      case 'tps':
        return <Thermometer className="h-4 w-4" />;
      case 'entities':
        return <Users className="h-4 w-4" />;
      case 'redstone':
        return <Zap className="h-4 w-4" />;
      case 'chunks':
        return <Grid3X3 className="h-4 w-4" />;
      default:
        return <MapPin className="h-4 w-4" />;
    }
  };

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Heatmap Legend */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {getHeatmapIcon()}
          <span className="text-sm font-medium">
            {heatmapType === 'tps' && 'TPS Impact'}
            {heatmapType === 'entities' && 'Entity Count'}
            {heatmapType === 'redstone' && 'Redstone Activity'}
            {heatmapType === 'chunks' && 'Chunk Load Time'}
          </span>
        </div>
        
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="text-xs">
            <div className="w-3 h-3 bg-green-500 rounded mr-1" />
            Low
          </Badge>
          <Badge variant="outline" className="text-xs">
            <div className="w-3 h-3 bg-yellow-500 rounded mr-1" />
            Medium
          </Badge>
          <Badge variant="outline" className="text-xs">
            <div className="w-3 h-3 bg-red-500 rounded mr-1" />
            High
          </Badge>
          <Badge variant="outline" className="text-xs">
            <div className="w-3 h-3 bg-blue-500 rounded mr-1" />
            Frozen
          </Badge>
        </div>
      </div>

      {/* Canvas */}
      <div className="relative border rounded-lg overflow-hidden bg-gray-900">
        <canvas
          ref={canvasRef}
          className="cursor-crosshair"
          onMouseMove={handleMouseMove}
          onMouseDown={handleMouseDown}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onClick={handleClick}
          onWheel={handleWheel}
        />
        
        {/* Hover Info */}
        {hoveredChunk && (
          <div className="absolute top-2 left-2 bg-black/80 text-white p-2 rounded text-xs">
            <div>Chunk: {hoveredChunk.x}, {hoveredChunk.z}</div>
            <div>Value: {hoveredChunk.value.toFixed(2)}</div>
            <div>Entities: {hoveredChunk.entities}</div>
            <div>Tile Entities: {hoveredChunk.tileEntities}</div>
            {hoveredChunk.isFrozen && <div className="text-blue-400">FROZEN</div>}
          </div>
        )}
      </div>

      {/* Controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setZoom(1)}
          >
            Reset Zoom
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => setOffset({ x: 0, y: 0 })}
          >
            Center
          </Button>
        </div>
        
        <div className="text-xs text-muted-foreground">
          Zoom: {(zoom * 100).toFixed(0)}% | Click chunks to inspect
        </div>
      </div>
    </div>
  );
};

export default WorldHeatmap;
