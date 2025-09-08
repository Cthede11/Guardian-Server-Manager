import React, { useState, useEffect, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
// Unused import removed
// import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
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
import { api } from '@/lib/api';

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
  const { id: serverId } = useParams<{ id: string }>();
  const [topology, setTopology] = useState<TopologyData>({
    nodes: [],
    connections: []
  });
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [canvasSize] = useState({ width: 800, height: 600 });

  const fetchTopology = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      // Real API call to get sharding topology
      const response = await api.getShardingTopology();
      if (response.ok && response.data) {
        const topologyData = response.data as any;
        
        // Transform the API response to match our interface
        const nodes: ShardNode[] = (topologyData.nodes || []).map((node: any, index: number) => ({
          id: node.id || `shard-${index}`,
          name: node.name || 'Unknown Shard',
          status: node.status || 'offline',
          players: node.players || 0,
          maxPlayers: node.maxPlayers || 100,
          tps: node.tps || 0,
          memory: node.memory || 0,
          cpu: node.cpu || 0,
          position: node.position || { x: 200 + (index * 100), y: 150 + (index * 50) },
          connections: node.connections || [],
          lastSeen: node.lastSeen || new Date().toISOString()
        }));

        const connections = topologyData.connections || [];

        setTopology({ nodes, connections });
      } else {
        // If no data available, show empty state
        setTopology({ nodes: [], connections: [] });
      }
    } catch (error) {
      console.error('Failed to fetch topology:', error);
      setTopology({ nodes: [], connections: [] });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (serverId) {
      fetchTopology();
      
      // Auto-refresh every 15 seconds
      const interval = setInterval(fetchTopology, 15000);
      return () => clearInterval(interval);
    }
  }, [serverId]);

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
