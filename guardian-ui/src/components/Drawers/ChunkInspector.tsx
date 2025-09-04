import React from 'react';
import { 
  X, 
  Snowflake, 
  Thermometer, 
  Users, 
  Grid3X3, 
  MapPin,
  AlertTriangle,
  Activity,
  // Database,
  Zap
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

interface ChunkInspectorProps {
  chunk: any;
  isOpen: boolean;
  onClose: () => void;
  onThaw: (chunkId: string) => void;
  className?: string;
}

export const ChunkInspector: React.FC<ChunkInspectorProps> = ({
  chunk,
  isOpen,
  onClose,
  onThaw,
  className = ''
}) => {
  if (!isOpen || !chunk) return null;

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
        return <Thermometer className="h-4 w-4" />;
      case 'entity overflow':
        return <Users className="h-4 w-4" />;
      case 'redstone lag':
        return <Zap className="h-4 w-4" />;
      case 'chunk loading issue':
        return <Grid3X3 className="h-4 w-4" />;
      default:
        return <AlertTriangle className="h-4 w-4" />;
    }
  };

  return (
    <div className={`fixed inset-0 z-50 ${className}`}>
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-black/50"
        onClick={onClose}
      />
      
      {/* Drawer */}
      <div className="absolute right-0 top-0 h-full w-96 bg-background border-l border-border shadow-lg">
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-border">
            <h3 className="text-lg font-semibold">Chunk Inspector</h3>
            <Button
              size="sm"
              variant="ghost"
              onClick={onClose}
              className="h-8 w-8 p-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-auto p-4 space-y-4">
            {/* Chunk Info */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center gap-2">
                  <MapPin className="h-4 w-4" />
                  Chunk Information
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Coordinates</span>
                  <span className="font-mono text-sm">{chunk.x}, {chunk.z}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Dimension</span>
                  <Badge variant="outline" className="text-xs">
                    <span className={getDimensionColor(chunk.dimension)}>
                      {chunk.dimension}
                    </span>
                  </Badge>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Status</span>
                  <Badge variant="secondary" className="text-xs">
                    <Snowflake className="h-3 w-3 mr-1" />
                    Frozen
                  </Badge>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Frozen Since</span>
                  <span className="text-sm">{formatTimeAgo(chunk.frozenAt)}</span>
                </div>
              </CardContent>
            </Card>

            {/* Performance Impact */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center gap-2">
                  <Activity className="h-4 w-4" />
                  Performance Impact
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">TPS Impact</span>
                  <span className={`text-sm font-medium ${getTpsImpactColor(chunk.tpsImpact)}`}>
                    {chunk.tpsImpact.toFixed(2)} TPS
                  </span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Entities</span>
                  <span className="text-sm">{chunk.entities}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Tile Entities</span>
                  <span className="text-sm">{chunk.tileEntities}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Blocks</span>
                  <span className="text-sm">{chunk.blocks.toLocaleString()}</span>
                </div>
              </CardContent>
            </Card>

            {/* Freeze Reason */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center gap-2">
                  {getReasonIcon(chunk.reason)}
                  Freeze Reason
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  {chunk.reason}
                </p>
              </CardContent>
            </Card>

            {/* Entity Breakdown */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center gap-2">
                  <Users className="h-4 w-4" />
                  Entity Breakdown
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Mobs</span>
                  <span className="text-sm">{Math.floor(chunk.entities * 0.6)}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Items</span>
                  <span className="text-sm">{Math.floor(chunk.entities * 0.3)}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Projectiles</span>
                  <span className="text-sm">{Math.floor(chunk.entities * 0.1)}</span>
                </div>
              </CardContent>
            </Card>

            {/* Tile Entity Breakdown */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center gap-2">
                  <Grid3X3 className="h-4 w-4" />
                  Tile Entity Breakdown
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Chests</span>
                  <span className="text-sm">{Math.floor(chunk.tileEntities * 0.4)}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Furnaces</span>
                  <span className="text-sm">{Math.floor(chunk.tileEntities * 0.3)}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Redstone</span>
                  <span className="text-sm">{Math.floor(chunk.tileEntities * 0.2)}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Other</span>
                  <span className="text-sm">{Math.floor(chunk.tileEntities * 0.1)}</span>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Footer */}
          <div className="p-4 border-t border-border">
            <Button
              onClick={() => onThaw(chunk.id)}
              className="w-full text-green-400 border-green-400 hover:bg-green-400 hover:text-white"
            >
              <Snowflake className="h-4 w-4 mr-2" />
              Thaw Chunk
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ChunkInspector;
