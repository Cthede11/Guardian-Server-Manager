import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { 
  Map, 
  Snowflake, 
  Eye, 
  RefreshCw,
  Search,
  Filter,
  Grid3X3,
  Layers,
  Thermometer,
  AlertTriangle,
  CheckCircle,
  X
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useServersStore } from '@/store/servers';
import { WorldHeatmap } from '@/components/World/WorldHeatmap';
import { FrozenChunksList } from '@/components/World/FrozenChunksList';
import { ChunkInspector } from '@/components/Drawers/ChunkInspector';

interface WorldPageProps {
  className?: string;
}

export const World: React.FC<WorldPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [selectedDimension, setSelectedDimension] = useState('overworld');
  const [heatmapType, setHeatmapType] = useState('tps');
  const [frozenChunks, setFrozenChunks] = useState<any[]>([]);
  const [selectedChunk, setSelectedChunk] = useState<any>(null);
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  // Fetch frozen chunks data
  const fetchFrozenChunks = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const response = await fetch(`http://localhost:8080/api/v1/servers/${serverId}/world/frozen-chunks`);
      if (response.ok) {
        const data = await response.json();
        setFrozenChunks(data);
      } else {
        // Use mock data for demo
        setFrozenChunks(generateMockFrozenChunks());
      }
    } catch (error) {
      console.error('Error fetching frozen chunks:', error);
      // Use mock data for demo
      setFrozenChunks(generateMockFrozenChunks());
    } finally {
      setIsLoading(false);
    }
  };

  // Generate mock frozen chunks for demo
  const generateMockFrozenChunks = () => {
    return [
      {
        id: '1',
        x: 0,
        z: 0,
        dimension: 'overworld',
        frozenAt: new Date(Date.now() - 300000), // 5 minutes ago
        reason: 'High TPS impact',
        tpsImpact: 2.3,
        entities: 45,
        tileEntities: 12,
        blocks: 65536
      },
      {
        id: '2',
        x: 1,
        z: 0,
        dimension: 'overworld',
        frozenAt: new Date(Date.now() - 180000), // 3 minutes ago
        reason: 'Entity overflow',
        tpsImpact: 1.8,
        entities: 67,
        tileEntities: 8,
        blocks: 65536
      },
      {
        id: '3',
        x: -1,
        z: 1,
        dimension: 'nether',
        frozenAt: new Date(Date.now() - 600000), // 10 minutes ago
        reason: 'Redstone lag',
        tpsImpact: 3.1,
        entities: 23,
        tileEntities: 15,
        blocks: 65536
      },
      {
        id: '4',
        x: 2,
        z: -1,
        dimension: 'overworld',
        frozenAt: new Date(Date.now() - 120000), // 2 minutes ago
        reason: 'Chunk loading issue',
        tpsImpact: 1.2,
        entities: 12,
        tileEntities: 3,
        blocks: 65536
      }
    ];
  };

  useEffect(() => {
    fetchFrozenChunks();
    
    // Refresh frozen chunks every 30 seconds
    const interval = setInterval(fetchFrozenChunks, 30000);
    return () => clearInterval(interval);
  }, [serverId]);

  const handleChunkClick = (chunk: any) => {
    setSelectedChunk(chunk);
    setInspectorOpen(true);
  };

  const handleThawChunk = async (chunkId: string) => {
    if (!serverId) return;
    
    try {
      const response = await fetch(`http://localhost:8080/api/v1/servers/${serverId}/world/chunks/${chunkId}/thaw`, {
        method: 'POST',
      });

      if (response.ok) {
        // Remove from frozen chunks list
        setFrozenChunks(prev => prev.filter(chunk => chunk.id !== chunkId));
        console.log(`Chunk ${chunkId} thawed successfully`);
      } else {
        console.error('Failed to thaw chunk');
      }
    } catch (error) {
      console.error('Error thawing chunk:', error);
    }
  };

  const getDimensionColor = (dimension: string) => {
    switch (dimension) {
      case 'overworld':
        return 'text-green-400';
      case 'nether':
        return 'text-red-400';
      case 'end':
        return 'text-purple-400';
      default:
        return 'text-gray-400';
    }
  };

  const getTpsImpactColor = (impact: number) => {
    if (impact < 1.0) return 'text-green-400';
    if (impact < 2.0) return 'text-yellow-400';
    return 'text-red-400';
  };

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view world</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">World</h2>
          <Badge variant="outline" className="flex items-center gap-1">
            <Snowflake className="h-3 w-3" />
            {frozenChunks.length} Frozen
          </Badge>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={fetchFrozenChunks}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4" />
          <span className="text-sm font-medium">Dimension:</span>
          <Select value={selectedDimension} onValueChange={setSelectedDimension}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="overworld">Overworld</SelectItem>
              <SelectItem value="nether">Nether</SelectItem>
              <SelectItem value="end">End</SelectItem>
            </SelectContent>
          </Select>
        </div>
        
        <div className="flex items-center gap-2">
          <Thermometer className="h-4 w-4" />
          <span className="text-sm font-medium">Heatmap:</span>
          <Select value={heatmapType} onValueChange={setHeatmapType}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="tps">TPS Impact</SelectItem>
              <SelectItem value="entities">Entities</SelectItem>
              <SelectItem value="redstone">Redstone</SelectItem>
              <SelectItem value="chunks">Chunk Load</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* World Heatmap */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Map className="h-5 w-5" />
                World Heatmap - {selectedDimension}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <WorldHeatmap
                dimension={selectedDimension}
                heatmapType={heatmapType}
                onChunkClick={handleChunkClick}
                frozenChunks={frozenChunks}
              />
            </CardContent>
          </Card>
        </div>

        {/* Frozen Chunks List */}
        <div>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Snowflake className="h-5 w-5" />
                Frozen Chunks
              </CardTitle>
            </CardHeader>
            <CardContent>
              <FrozenChunksList
                chunks={frozenChunks}
                onThaw={handleThawChunk}
                onInspect={handleChunkClick}
              />
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Chunk Inspector Drawer */}
      <ChunkInspector
        chunk={selectedChunk}
        isOpen={inspectorOpen}
        onClose={() => setInspectorOpen(false)}
        onThaw={handleThawChunk}
      />
    </div>
  );
};

export default World;
