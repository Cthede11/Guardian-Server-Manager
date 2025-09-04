import React, { useState, useEffect } from 'react';
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
  const [assignments, setAssignments] = useState<PlayerAssignment[]>([]);
  const [shards, setShards] = useState<ShardInfo[]>([]);
  const [selectedPlayers, setSelectedPlayers] = useState<string[]>([]);
  const [targetShard, setTargetShard] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [isAssigning, setIsAssigning] = useState(false);

  const fetchData = async () => {
    setIsLoading(true);
    try {
      // Mock API calls
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const mockAssignments: PlayerAssignment[] = [
        {
          id: 'player-1',
          name: 'PlayerOne',
          currentShard: 'shard-1',
          targetShard: 'shard-2',
          priority: 'high',
          reason: 'Load balancing',
          status: 'pending',
          estimatedTime: 30,
          lastSeen: new Date().toISOString()
        },
        {
          id: 'player-2',
          name: 'MinerPro',
          currentShard: 'shard-3',
          targetShard: 'shard-1',
          priority: 'medium',
          reason: 'Player request',
          status: 'in_progress',
          estimatedTime: 45,
          lastSeen: new Date().toISOString()
        },
        {
          id: 'player-3',
          name: 'BuilderBob',
          currentShard: 'shard-2',
          targetShard: 'shard-4',
          priority: 'low',
          reason: 'Maintenance',
          status: 'completed',
          estimatedTime: 20,
          lastSeen: new Date().toISOString()
        },
        {
          id: 'player-4',
          name: 'ExplorerEve',
          currentShard: 'shard-5',
          targetShard: 'shard-1',
          priority: 'high',
          reason: 'Shard failure',
          status: 'failed',
          estimatedTime: 60,
          lastSeen: new Date(Date.now() - 300000).toISOString()
        }
      ];

      const mockShards: ShardInfo[] = [
        {
          id: 'shard-1',
          name: 'Main World',
          currentPlayers: 45,
          maxPlayers: 100,
          load: 65,
          status: 'healthy'
        },
        {
          id: 'shard-2',
          name: 'Nether',
          currentPlayers: 23,
          maxPlayers: 50,
          load: 42,
          status: 'healthy'
        },
        {
          id: 'shard-3',
          name: 'End',
          currentPlayers: 12,
          maxPlayers: 30,
          load: 78,
          status: 'warning'
        },
        {
          id: 'shard-4',
          name: 'Creative',
          currentPlayers: 8,
          maxPlayers: 20,
          load: 35,
          status: 'healthy'
        },
        {
          id: 'shard-5',
          name: 'Minigames',
          currentPlayers: 0,
          maxPlayers: 50,
          load: 95,
          status: 'critical'
        }
      ];

      setAssignments(mockAssignments);
      setShards(mockShards);
    } catch (error) {
      console.error('Failed to fetch assignment data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    
    // Auto-refresh every 10 seconds
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, []);

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
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Update assignments
      setAssignments(prev => prev.map(assignment => 
        selectedPlayers.includes(assignment.id)
          ? { ...assignment, targetShard, status: 'pending' as const }
          : assignment
      ));
      
      setSelectedPlayers([]);
      setTargetShard('');
    } catch (error) {
      console.error('Failed to assign players:', error);
    } finally {
      setIsAssigning(false);
    }
  };

  const handleRetryAssignment = async (assignmentId: string) => {
    setIsAssigning(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setAssignments(prev => prev.map(assignment => 
        assignment.id === assignmentId
          ? { ...assignment, status: 'in_progress' as const }
          : assignment
      ));
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
