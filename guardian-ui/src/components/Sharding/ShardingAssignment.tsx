import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  ArrowRight, 
  AlertTriangle, 
  CheckCircle, 
  XCircle,
  RefreshCw,
  Target,
  Loader2
} from 'lucide-react';
import { apiClient as api } from '@/lib/api';

interface PlayerAssignment {
  id: string;
  name: string;
  currentShard: string;
  targetShard: string;
  priority: 'high' | 'medium' | 'low';
  reason: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  estimatedTime: number;
  lastSeen: string;
}

interface ShardInfo {
  id: string;
  name: string;
  currentPlayers: number;
  maxPlayers: number;
  load: number;
  status: 'healthy' | 'warning' | 'critical' | 'offline';
}

export const ShardingAssignment: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const [assignments, setAssignments] = useState<PlayerAssignment[]>([]);
  const [shards, setShards] = useState<ShardInfo[]>([]);
  const [selectedPlayers, setSelectedPlayers] = useState<string[]>([]);
  const [targetShard, setTargetShard] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [isAssigning, setIsAssigning] = useState(false);

  const fetchData = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      // Real API calls to get sharding data
      const [assignmentsResponse, shardsResponse] = await Promise.all([
        api.getShardingAssignments ? api.getShardingAssignments() : Promise.resolve({ ok: false, data: [] }),
        api.getShardingTopology ? api.getShardingTopology() : Promise.resolve({ ok: false, data: { nodes: [] } })
      ]);

      if (assignmentsResponse.ok && assignmentsResponse.data) {
        setAssignments(assignmentsResponse.data as PlayerAssignment[]);
      } else {
        setAssignments([]);
      }

      if (shardsResponse.ok && shardsResponse.data) {
        const topologyData = shardsResponse.data as any;
        const shardNodes = topologyData.nodes || [];
        const shards: ShardInfo[] = shardNodes.map((node: any) => ({
          id: node.id,
          name: node.name,
          currentPlayers: node.players || 0,
          maxPlayers: node.maxPlayers || 100,
          load: node.cpu || 0,
          status: node.status || 'offline'
        }));
        setShards(shards);
      } else {
        setShards([]);
      }
    } catch (error) {
      console.error('Failed to fetch assignment data:', error);
      setAssignments([]);
      setShards([]);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (serverId) {
      fetchData();
      
      // Auto-refresh every 10 seconds
      const interval = setInterval(fetchData, 10000);
      return () => clearInterval(interval);
    }
  }, [serverId]);

  const getStatusIcon = (status: PlayerAssignment['status']) => {
    switch (status) {
      case 'pending': return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'in_progress': return <Loader2 className="h-4 w-4 text-blue-500 animate-spin" />;
      case 'completed': return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'failed': return <XCircle className="h-4 w-4 text-red-500" />;
      default: return <AlertTriangle className="h-4 w-4 text-gray-500" />;
    }
  };

  // Unused function removed
  // const getStatusColor = (status: PlayerAssignment['status']) => {
  //   switch (status) {
  //     case 'pending': return 'bg-yellow-100 text-yellow-800';
  //     case 'in_progress': return 'bg-blue-100 text-blue-800';
  //     case 'completed': return 'bg-green-100 text-green-800';
  //     case 'failed': return 'bg-red-100 text-red-800';
  //     default: return 'bg-gray-100 text-gray-800';
  //   }
  // };

  const getPriorityColor = (priority: PlayerAssignment['priority']) => {
    switch (priority) {
      case 'high': return 'destructive';
      case 'medium': return 'default';
      case 'low': return 'secondary';
      default: return 'secondary';
    }
  };

  const getShardName = (shardId: string) => {
    return shards.find(s => s.id === shardId)?.name || shardId;
  };

  // Unused function removed
  // const getShardStatus = (shardId: string) => {
  //   return shards.find(s => s.id === shardId)?.status || 'offline';
  // };

  // Unused function removed
  // const handlePlayerSelect = (playerId: string) => {
  //   setSelectedPlayers(prev => 
  //     prev.includes(playerId) 
  //       ? prev.filter(id => id !== playerId)
  //       : [...prev, playerId]
  //   );
  // };

  const handleBulkAssign = async () => {
    if (selectedPlayers.length === 0 || !targetShard) return;

    setIsAssigning(true);
    try {
      const response = await api.bulkAssignPlayers?.(selectedPlayers, targetShard);
      if (response?.ok) {
        // Update assignments
        setAssignments(prev => prev.map(assignment => 
          selectedPlayers.includes(assignment.id)
            ? { ...assignment, targetShard, status: 'pending' as const }
            : assignment
        ));
        
        setSelectedPlayers([]);
        setTargetShard('');
      } else {
        throw new Error('Failed to assign players');
      }
    } catch (error) {
      console.error('Failed to assign players:', error);
    } finally {
      setIsAssigning(false);
    }
  };

  const handleRetryAssignment = async (assignmentId: string) => {
    setIsAssigning(true);
    try {
      const response = await api.retryAssignment?.(assignmentId);
      if (response?.ok) {
        setAssignments(prev => prev.map(assignment => 
          assignment.id === assignmentId
            ? { ...assignment, status: 'in_progress' as const }
            : assignment
        ));
      } else {
        throw new Error('Failed to retry assignment');
      }
    } catch (error) {
      console.error('Failed to retry assignment:', error);
    } finally {
      setIsAssigning(false);
    }
  };

  const pendingAssignments = assignments.filter(a => a.status === 'pending' || a.status === 'in_progress');
  const completedAssignments = assignments.filter(a => a.status === 'completed');
  const failedAssignments = assignments.filter(a => a.status === 'failed');

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Assignment Controls */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Target className="h-5 w-5" />
            <span>Player Assignment</span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center space-x-4">
            <div className="flex-1">
              <Label htmlFor="target-shard">Target Shard</Label>
              <Select value={targetShard} onValueChange={setTargetShard}>
                <SelectTrigger>
                  <SelectValue placeholder="Select target shard" />
                </SelectTrigger>
                <SelectContent>
                  {shards.map(shard => (
                    <SelectItem key={shard.id} value={shard.id}>
                      <div className="flex items-center space-x-2">
                        <span>{shard.name}</span>
                        <Badge variant={
                          shard.status === 'healthy' ? 'default' :
                          shard.status === 'warning' ? 'secondary' : 'destructive'
                        }>
                          {shard.currentPlayers}/{shard.maxPlayers}
                        </Badge>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-center space-x-2">
              <Button
                onClick={handleBulkAssign}
                disabled={selectedPlayers.length === 0 || !targetShard || isAssigning}
                size="sm"
              >
                {isAssigning ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <ArrowRight className="h-4 w-4 mr-2" />
                )}
                Assign ({selectedPlayers.length})
              </Button>
              <Button variant="outline" size="sm" onClick={fetchData} disabled={isLoading}>
                <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Shard Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {shards.map(shard => (
          <Card key={shard.id}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm">{shard.name}</CardTitle>
                <Badge variant={
                  shard.status === 'healthy' ? 'default' :
                  shard.status === 'warning' ? 'secondary' : 'destructive'
                }>
                  {shard.status}
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Players</span>
                  <span>{shard.currentPlayers}/{shard.maxPlayers}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Load</span>
                  <span className={shard.load > 80 ? 'text-red-500' : shard.load > 60 ? 'text-yellow-500' : 'text-green-500'}>
                    {shard.load}%
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Assignment Queue */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1">
        {/* Pending/In Progress */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Loader2 className="h-5 w-5" />
              <span>Active Assignments ({pendingAssignments.length})</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {pendingAssignments.map(assignment => (
                <div key={assignment.id} className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center space-x-3">
                    {getStatusIcon(assignment.status)}
                    <div>
                      <div className="font-medium">{assignment.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {getShardName(assignment.currentShard)} → {getShardName(assignment.targetShard)}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Badge variant={getPriorityColor(assignment.priority)}>
                      {assignment.priority}
                    </Badge>
                    <span className="text-sm text-muted-foreground">
                      {assignment.estimatedTime}s
                    </span>
                  </div>
                </div>
              ))}
              {pendingAssignments.length === 0 && (
                <div className="text-center text-muted-foreground py-8">
                  <Target className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No active assignments</p>
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Completed/Failed */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <CheckCircle className="h-5 w-5" />
              <span>Recent Assignments ({completedAssignments.length + failedAssignments.length})</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {[...completedAssignments, ...failedAssignments].map(assignment => (
                <div key={assignment.id} className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center space-x-3">
                    {getStatusIcon(assignment.status)}
                    <div>
                      <div className="font-medium">{assignment.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {getShardName(assignment.currentShard)} → {getShardName(assignment.targetShard)}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {assignment.reason}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Badge variant={getPriorityColor(assignment.priority)}>
                      {assignment.priority}
                    </Badge>
                    {assignment.status === 'failed' && (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleRetryAssignment(assignment.id)}
                        disabled={isAssigning}
                      >
                        Retry
                      </Button>
                    )}
                  </div>
                </div>
              ))}
              {completedAssignments.length === 0 && failedAssignments.length === 0 && (
                <div className="text-center text-muted-foreground py-8">
                  <CheckCircle className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No recent assignments</p>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};
