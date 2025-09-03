import React from 'react';
import { 
  Snowflake, 
  Eye, 
  Thermometer, 
  Users, 
  Grid3X3, 
  Clock,
  AlertTriangle,
  MapPin
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';

interface FrozenChunksListProps {
  chunks: any[];
  onThaw: (chunkId: string) => void;
  onInspect: (chunk: any) => void;
  className?: string;
}

export const FrozenChunksList: React.FC<FrozenChunksListProps> = ({
  chunks,
  onThaw,
  onInspect,
  className = ''
}) => {
  const formatTimeAgo = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m ago`;
    }
    return `${minutes}m ago`;
  };

  const getTpsImpactColor = (impact: number) => {
    if (impact < 1.0) return 'text-green-400';
    if (impact < 2.0) return 'text-yellow-400';
    return 'text-red-400';
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

  const getReasonIcon = (reason: string) => {
    switch (reason.toLowerCase()) {
      case 'high tps impact':
        return <Thermometer className="h-3 w-3" />;
      case 'entity overflow':
        return <Users className="h-3 w-3" />;
      case 'redstone lag':
        return <AlertTriangle className="h-3 w-3" />;
      case 'chunk loading issue':
        return <Grid3X3 className="h-3 w-3" />;
      default:
        return <AlertTriangle className="h-3 w-3" />;
    }
  };

  if (chunks.length === 0) {
    return (
      <div className={`text-center py-8 ${className}`}>
        <Snowflake className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
        <p className="text-muted-foreground">No frozen chunks</p>
        <p className="text-xs text-muted-foreground mt-1">
          Chunks will appear here when they are frozen due to performance issues
        </p>
      </div>
    );
  }

  return (
    <div className={`space-y-2 ${className}`}>
      {chunks.map((chunk) => (
        <Card key={chunk.id} className="p-3">
          <CardContent className="p-0">
            <div className="space-y-2">
              {/* Header */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <MapPin className="h-4 w-4 text-blue-400" />
                  <span className="font-mono text-sm">
                    {chunk.x}, {chunk.z}
                  </span>
                  <Badge variant="outline" className="text-xs">
                    <span className={getDimensionColor(chunk.dimension)}>
                      {chunk.dimension}
                    </span>
                  </Badge>
                </div>
                
                <div className="flex items-center gap-1">
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => onInspect(chunk)}
                    className="h-6 w-6 p-0"
                  >
                    <Eye className="h-3 w-3" />
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => onThaw(chunk.id)}
                    className="h-6 w-6 p-0 text-green-400 hover:text-green-300"
                  >
                    <Snowflake className="h-3 w-3" />
                  </Button>
                </div>
              </div>

              {/* Reason */}
              <div className="flex items-center gap-2">
                {getReasonIcon(chunk.reason)}
                <span className="text-xs text-muted-foreground">
                  {chunk.reason}
                </span>
              </div>

              {/* Stats */}
              <div className="grid grid-cols-2 gap-2 text-xs">
                <div className="flex items-center gap-1">
                  <Thermometer className="h-3 w-3" />
                  <span className={getTpsImpactColor(chunk.tpsImpact)}>
                    {chunk.tpsImpact.toFixed(1)} TPS
                  </span>
                </div>
                
                <div className="flex items-center gap-1">
                  <Users className="h-3 w-3" />
                  <span className="text-muted-foreground">
                    {chunk.entities} entities
                  </span>
                </div>
                
                <div className="flex items-center gap-1">
                  <Grid3X3 className="h-3 w-3" />
                  <span className="text-muted-foreground">
                    {chunk.tileEntities} tiles
                  </span>
                </div>
                
                <div className="flex items-center gap-1">
                  <Clock className="h-3 w-3" />
                  <span className="text-muted-foreground">
                    {formatTimeAgo(chunk.frozenAt)}
                  </span>
                </div>
              </div>

              {/* Thaw Button */}
              <Button
                size="sm"
                variant="outline"
                onClick={() => onThaw(chunk.id)}
                className="w-full text-green-400 border-green-400 hover:bg-green-400 hover:text-white"
              >
                <Snowflake className="h-3 w-3 mr-1" />
                Thaw Chunk
              </Button>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
};

export default FrozenChunksList;
