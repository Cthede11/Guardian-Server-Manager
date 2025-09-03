import React, { useState, useEffect, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { 
  Server, 
  Users, 
  Activity, 
  AlertTriangle, 
  CheckCircle, 
  XCircle,
  Zap,
  Cpu,
  HardDrive
} from 'lucide-react';

interface ShardNode {
  id: string;
  name: string;
  status: 'healthy' | 'warning' | 'critical' | 'offline';
  players: number;
  maxPlayers: number;
  tps: number;
  memory: number;
  cpu: number;
  position: { x: number; y: number };
  connections: string[];
  lastSeen: string;
}

interface TopologyData {
  nodes: ShardNode[];
  connections: Array<{
    from: string;
    to: string;
    type: 'primary' | 'secondary' | 'backup';
    latency: number;
  }>;
}

export const ShardingTopology: React.FC = () => {
  const [topology, setTopology] = useState<TopologyData>({
    nodes: [],
    connections: []
  });
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [canvasSize, setCanvasSize] = useState({ width: 800, height: 600 });

  const fetchTopology = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const mockNodes: ShardNode[] = [
        {
          id: 'shard-1',
          name: 'Main World',
          status: 'healthy',
          players: 45,
          maxPlayers: 100,
          tps: 19.8,
          memory: 65,
          cpu: 45,
          position: { x: 200, y: 150 },
          connections: ['shard-2', 'shard-3'],
          lastSeen: new Date().toISOString()
        },
        {
          id: 'shard-2',
          name: 'Nether',
          status: 'healthy',
          players: 23,
          maxPlayers: 50,
          tps: 20.0,
          memory: 42,
          cpu: 38,
          position: { x: 400, y: 100 },
          connections: ['shard-1', 'shard-4'],
          lastSeen: new Date().toISOString()
        },
        {
          id: 'shard-3',
          name: 'End',
          status: 'warning',
          players: 12,
          maxPlayers: 30,
          tps: 18.5,
          memory: 78,
          cpu: 62,
          position: { x: 200, y: 300 },
          connections: ['shard-1', 'shard-5'],
          lastSeen: new Date().toISOString()
        },
        {
          id: 'shard-4',
          name: 'Creative',
          status: 'healthy',
          players: 8,
          maxPlayers: 20,
          tps: 20.0,
          memory: 35,
          cpu: 28,
          position: { x: 500, y: 200 },
          connections: ['shard-2'],
          lastSeen: new Date().toISOString()
        },
        {
          id: 'shard-5',
          name: 'Minigames',
          status: 'critical',
          players: 0,
          maxPlayers: 50,
          tps: 5.2,
          memory: 95,
          cpu: 89,
          position: { x: 300, y: 400 },
          connections: ['shard-3'],
          lastSeen: new Date(Date.now() - 300000).toISOString()
        },
        {
          id: 'shard-6',
          name: 'Backup',
          status: 'offline',
          players: 0,
          maxPlayers: 100,
          tps: 0,
          memory: 0,
          cpu: 0,
          position: { x: 600, y: 350 },
          connections: [],
          lastSeen: new Date(Date.now() - 3600000).toISOString()
        }
      ];

      const mockConnections = [
        { from: 'shard-1', to: 'shard-2', type: 'primary' as const, latency: 12 },
        { from: 'shard-1', to: 'shard-3', type: 'primary' as const, latency: 8 },
        { from: 'shard-2', to: 'shard-4', type: 'secondary' as const, latency: 15 },
        { from: 'shard-3', to: 'shard-5', type: 'secondary' as const, latency: 22 },
        { from: 'shard-6', to: 'shard-1', type: 'backup' as const, latency: 0 }
      ];

      setTopology({ nodes: mockNodes, connections: mockConnections });
    } catch (error) {
      console.error('Failed to fetch topology:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchTopology();
    
    // Auto-refresh every 15 seconds
    const interval = setInterval(fetchTopology, 15000);
    return () => clearInterval(interval);
  }, []);

  const getStatusColor = (status: ShardNode['status']) => {
    switch (status) {
      case 'healthy': return '#10b981';
      case 'warning': return '#f59e0b';
      case 'critical': return '#ef4444';
      case 'offline': return '#6b7280';
      default: return '#6b7280';
    }
  };

  const getStatusIcon = (status: ShardNode['status']) => {
    switch (status) {
      case 'healthy': return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'warning': return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'critical': return <XCircle className="h-4 w-4 text-red-500" />;
      case 'offline': return <XCircle className="h-4 w-4 text-gray-500" />;
      default: return <XCircle className="h-4 w-4 text-gray-500" />;
    }
  };

  const getConnectionColor = (type: string) => {
    switch (type) {
      case 'primary': return '#3b82f6';
      case 'secondary': return '#8b5cf6';
      case 'backup': return '#6b7280';
      default: return '#6b7280';
    }
  };

  const drawTopology = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw connections
    topology.connections.forEach(conn => {
      const fromNode = topology.nodes.find(n => n.id === conn.from);
      const toNode = topology.nodes.find(n => n.id === conn.to);
      
      if (!fromNode || !toNode) return;

      ctx.strokeStyle = getConnectionColor(conn.type);
      ctx.lineWidth = conn.type === 'primary' ? 3 : 2;
      ctx.setLineDash(conn.type === 'backup' ? [5, 5] : []);
      
      ctx.beginPath();
      ctx.moveTo(fromNode.position.x, fromNode.position.y);
      ctx.lineTo(toNode.position.x, toNode.position.y);
      ctx.stroke();

      // Draw latency label
      const midX = (fromNode.position.x + toNode.position.x) / 2;
      const midY = (fromNode.position.y + toNode.position.y) / 2;
      
      ctx.fillStyle = '#374151';
      ctx.fillRect(midX - 15, midY - 8, 30, 16);
      ctx.fillStyle = '#ffffff';
      ctx.font = '10px sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(`${conn.latency}ms`, midX, midY + 3);
    });

    // Draw nodes
    topology.nodes.forEach(node => {
      const isSelected = selectedNode === node.id;
      const radius = isSelected ? 35 : 30;
      
      // Node background
      ctx.fillStyle = getStatusColor(node.status);
      ctx.beginPath();
      ctx.arc(node.position.x, node.position.y, radius, 0, 2 * Math.PI);
      ctx.fill();

      // Node border
      ctx.strokeStyle = isSelected ? '#ffffff' : '#1f2937';
      ctx.lineWidth = isSelected ? 3 : 2;
      ctx.stroke();

      // Node text
      ctx.fillStyle = '#ffffff';
      ctx.font = 'bold 12px sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(node.name, node.position.x, node.position.y - 5);
      
      ctx.font = '10px sans-serif';
      ctx.fillText(`${node.players}/${node.maxPlayers}`, node.position.x, node.position.y + 10);
    });
  };

  useEffect(() => {
    drawTopology();
  }, [topology, selectedNode]);

  const handleCanvasClick = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Check if click is on a node
    const clickedNode = topology.nodes.find(node => {
      const distance = Math.sqrt(
        Math.pow(x - node.position.x, 2) + Math.pow(y - node.position.y, 2)
      );
      return distance <= 35;
    });

    setSelectedNode(clickedNode?.id || null);
  };

  const selectedNodeData = topology.nodes.find(n => n.id === selectedNode);

  return (
    <div className="h-full flex space-x-6">
      {/* Topology Canvas */}
      <div className="flex-1">
        <Card className="h-full">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Server className="h-5 w-5" />
              <span>Shard Topology</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="h-full">
            <div className="relative h-full">
              <canvas
                ref={canvasRef}
                width={canvasSize.width}
                height={canvasSize.height}
                className="border border-border rounded-lg cursor-pointer"
                onClick={handleCanvasClick}
                style={{ maxWidth: '100%', height: 'auto' }}
              />
              
              {isLoading && (
                <div className="absolute inset-0 flex items-center justify-center bg-background/80">
                  <div className="text-center">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-2"></div>
                    <p className="text-sm text-muted-foreground">Loading topology...</p>
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Node Details */}
      <div className="w-80">
        <Card className="h-full">
          <CardHeader>
            <CardTitle>Node Details</CardTitle>
          </CardHeader>
          <CardContent>
            {selectedNodeData ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold">{selectedNodeData.name}</h3>
                  {getStatusIcon(selectedNodeData.status)}
                </div>

                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Status</span>
                    <Badge variant={
                      selectedNodeData.status === 'healthy' ? 'default' :
                      selectedNodeData.status === 'warning' ? 'secondary' : 'destructive'
                    }>
                      {selectedNodeData.status}
                    </Badge>
                  </div>

                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Players</span>
                    <div className="flex items-center space-x-1">
                      <Users className="h-4 w-4" />
                      <span>{selectedNodeData.players}/{selectedNodeData.maxPlayers}</span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">TPS</span>
                    <div className="flex items-center space-x-1">
                      <Activity className="h-4 w-4" />
                      <span className={selectedNodeData.tps < 15 ? 'text-red-500' : 'text-green-500'}>
                        {selectedNodeData.tps.toFixed(1)}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Memory</span>
                    <div className="flex items-center space-x-1">
                      <HardDrive className="h-4 w-4" />
                      <span>{selectedNodeData.memory}%</span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">CPU</span>
                    <div className="flex items-center space-x-1">
                      <Cpu className="h-4 w-4" />
                      <span>{selectedNodeData.cpu}%</span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Last Seen</span>
                    <span className="text-xs">
                      {new Date(selectedNodeData.lastSeen).toLocaleTimeString()}
                    </span>
                  </div>
                </div>

                <div className="pt-4 border-t">
                  <h4 className="font-medium mb-2">Connections</h4>
                  <div className="space-y-2">
                    {selectedNodeData.connections.map(connId => {
                      const conn = topology.connections.find(c => 
                        c.from === selectedNodeData.id && c.to === connId
                      );
                      const targetNode = topology.nodes.find(n => n.id === connId);
                      
                      return (
                        <div key={connId} className="flex items-center justify-between text-sm">
                          <span>{targetNode?.name}</span>
                          {conn && (
                            <Badge variant="outline" className="text-xs">
                              {conn.type} ({conn.latency}ms)
                            </Badge>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>

                <div className="pt-4 border-t">
                  <Button size="sm" className="w-full">
                    <Zap className="h-4 w-4 mr-2" />
                    Restart Shard
                  </Button>
                </div>
              </div>
            ) : (
              <div className="text-center text-muted-foreground">
                <Server className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>Click on a shard node to view details</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
};
