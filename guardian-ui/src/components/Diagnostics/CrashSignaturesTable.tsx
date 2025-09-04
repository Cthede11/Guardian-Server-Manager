import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { 
  Bug, 
  AlertTriangle, 
  CheckCircle, 
  XCircle, 
  RefreshCw,
  Search,
  TrendingUp,
  TrendingDown,
  Download,
  Eye,
  Clock,
  Activity
} from 'lucide-react';

interface CrashSignature {
  id: string;
  signature: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  frequency: number;
  lastOccurrence: string;
  firstSeen: string;
  affectedVersions: string[];
  stackTrace: string;
  rootCause: string;
  status: 'active' | 'investigating' | 'resolved' | 'ignored';
  assignedTo?: string;
  resolution?: string;
  tags: string[];
  impact: {
    playersAffected: number;
    downtimeMinutes: number;
    dataLoss: boolean;
  };
}

interface CrashStats {
  total: number;
  active: number;
  resolved: number;
  critical: number;
  trend: 'up' | 'down' | 'stable';
  averageResolutionTime: number;
}

export const CrashSignaturesTable: React.FC = () => {
  const [crashes, setCrashes] = useState<CrashSignature[]>([]);
  const [stats, setStats] = useState<CrashStats>({
    total: 0,
    active: 0,
    resolved: 0,
    critical: 0,
    trend: 'stable',
    averageResolutionTime: 0
  });
  const [isLoading, setIsLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [severityFilter, setSeverityFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [selectedCrash, setSelectedCrash] = useState<CrashSignature | null>(null);

  const fetchCrashes = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const mockCrashes: CrashSignature[] = [
        {
          id: 'crash-1',
          signature: 'NullPointerException in EntityTick',
          description: 'Null pointer exception occurs during entity tick processing, likely due to unloaded chunks',
          severity: 'high',
          frequency: 15,
          lastOccurrence: new Date(Date.now() - 3600000).toISOString(),
          firstSeen: new Date(Date.now() - 86400000 * 7).toISOString(),
          affectedVersions: ['1.20.1', '1.20.2'],
          stackTrace: 'java.lang.NullPointerException\n  at net.minecraft.world.entity.Entity.tick(Entity.java:123)\n  at net.minecraft.server.level.ServerLevel.tickEntities(ServerLevel.java:456)',
          rootCause: 'Entity references unloaded chunk data',
          status: 'active',
          assignedTo: 'dev-team',
          tags: ['entity', 'chunk', 'tick'],
          impact: {
            playersAffected: 45,
            downtimeMinutes: 12,
            dataLoss: false
          }
        },
        {
          id: 'crash-2',
          signature: 'OutOfMemoryError in WorldGen',
          description: 'Memory exhaustion during world generation, particularly in large biomes',
          severity: 'critical',
          frequency: 8,
          lastOccurrence: new Date(Date.now() - 7200000).toISOString(),
          firstSeen: new Date(Date.now() - 86400000 * 14).toISOString(),
          affectedVersions: ['1.20.1', '1.20.2', '1.19.4'],
          stackTrace: 'java.lang.OutOfMemoryError: Java heap space\n  at net.minecraft.world.level.chunk.ChunkGenerator.generateChunk(ChunkGenerator.java:234)',
          rootCause: 'Insufficient heap space for large biome generation',
          status: 'investigating',
          assignedTo: 'performance-team',
          tags: ['memory', 'worldgen', 'biome'],
          impact: {
            playersAffected: 120,
            downtimeMinutes: 45,
            dataLoss: true
          }
        },
        {
          id: 'crash-3',
          signature: 'ConcurrentModificationException in PlayerList',
          description: 'Concurrent modification of player list during server shutdown',
          severity: 'medium',
          frequency: 3,
          lastOccurrence: new Date(Date.now() - 86400000 * 2).toISOString(),
          firstSeen: new Date(Date.now() - 86400000 * 30).toISOString(),
          affectedVersions: ['1.20.1'],
          stackTrace: 'java.util.ConcurrentModificationException\n  at java.util.ArrayList$Itr.checkForComodification(ArrayList.java:909)\n  at net.minecraft.server.players.PlayerList.tick(PlayerList.java:123)',
          rootCause: 'Race condition during server shutdown',
          status: 'resolved',
          resolution: 'Added synchronization to player list operations',
          tags: ['concurrency', 'shutdown', 'players'],
          impact: {
            playersAffected: 8,
            downtimeMinutes: 2,
            dataLoss: false
          }
        },
        {
          id: 'crash-4',
          signature: 'StackOverflowError in Recursive Block Update',
          description: 'Infinite recursion in block update chain, likely caused by mod conflicts',
          severity: 'high',
          frequency: 12,
          lastOccurrence: new Date(Date.now() - 1800000).toISOString(),
          firstSeen: new Date(Date.now() - 86400000 * 5).toISOString(),
          affectedVersions: ['1.20.1'],
          stackTrace: 'java.lang.StackOverflowError\n  at net.minecraft.world.level.block.Block.updateShape(Block.java:456)\n  at net.minecraft.world.level.block.Block.updateShape(Block.java:456)',
          rootCause: 'Mod conflict causing infinite block update loop',
          status: 'active',
          assignedTo: 'mod-team',
          tags: ['mod', 'block', 'recursion'],
          impact: {
            playersAffected: 67,
            downtimeMinutes: 8,
            dataLoss: false
          }
        },
        {
          id: 'crash-5',
          signature: 'IOException in World Save',
          description: 'Disk I/O error during world save operation',
          severity: 'critical',
          frequency: 2,
          lastOccurrence: new Date(Date.now() - 86400000 * 1).toISOString(),
          firstSeen: new Date(Date.now() - 86400000 * 3).toISOString(),
          affectedVersions: ['1.20.1', '1.20.2'],
          stackTrace: 'java.io.IOException: No space left on device\n  at java.io.FileOutputStream.writeBytes(Native Method)\n  at net.minecraft.world.level.storage.LevelStorage.save(LevelStorage.java:789)',
          rootCause: 'Insufficient disk space for world save',
          status: 'resolved',
          resolution: 'Increased disk space and added monitoring',
          tags: ['io', 'disk', 'save'],
          impact: {
            playersAffected: 200,
            downtimeMinutes: 30,
            dataLoss: true
          }
        }
      ];

      setCrashes(mockCrashes);

      // Calculate stats
      const newStats: CrashStats = {
        total: mockCrashes.length,
        active: mockCrashes.filter(c => c.status === 'active').length,
        resolved: mockCrashes.filter(c => c.status === 'resolved').length,
        critical: mockCrashes.filter(c => c.severity === 'critical').length,
        trend: 'down',
        averageResolutionTime: 2.5
      };

      setStats(newStats);
    } catch (error) {
      console.error('Failed to fetch crashes:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchCrashes();
    
    // Auto-refresh every 60 seconds
    const interval = setInterval(fetchCrashes, 60000);
    return () => clearInterval(interval);
  }, []);

  const getSeverityIcon = (severity: CrashSignature['severity']) => {
    switch (severity) {
      case 'critical': return <XCircle className="h-4 w-4 text-red-500" />;
      case 'high': return <AlertTriangle className="h-4 w-4 text-orange-500" />;
      case 'medium': return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'low': return <CheckCircle className="h-4 w-4 text-blue-500" />;
      default: return <AlertTriangle className="h-4 w-4 text-gray-500" />;
    }
  };

  const getSeverityColor = (severity: CrashSignature['severity']) => {
    switch (severity) {
      case 'critical': return 'destructive';
      case 'high': return 'default';
      case 'medium': return 'secondary';
      case 'low': return 'outline';
      default: return 'outline';
    }
  };

  const getStatusColor = (status: CrashSignature['status']) => {
    switch (status) {
      case 'active': return 'bg-red-100 text-red-800';
      case 'investigating': return 'bg-yellow-100 text-yellow-800';
      case 'resolved': return 'bg-green-100 text-green-800';
      case 'ignored': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getTrendIcon = () => {
    switch (stats.trend) {
      case 'up': return <TrendingUp className="h-4 w-4 text-red-500" />;
      case 'down': return <TrendingDown className="h-4 w-4 text-green-500" />;
      case 'stable': return <Activity className="h-4 w-4 text-blue-500" />;
      default: return <Activity className="h-4 w-4 text-gray-500" />;
    }
  };

  const filteredCrashes = crashes.filter(crash => {
    const matchesSearch = crash.signature.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         crash.description.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesSeverity = severityFilter === 'all' || crash.severity === severityFilter;
    const matchesStatus = statusFilter === 'all' || crash.status === statusFilter;
    
    return matchesSearch && matchesSeverity && matchesStatus;
  });

  const formatFrequency = (frequency: number) => {
    if (frequency >= 10) return `${frequency} (High)`;
    if (frequency >= 5) return `${frequency} (Medium)`;
    return `${frequency} (Low)`;
  };

  const formatImpact = (impact: CrashSignature['impact']) => {
    return `${impact.playersAffected} players, ${impact.downtimeMinutes}min downtime${impact.dataLoss ? ', data loss' : ''}`;
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Crashes</CardTitle>
            <Bug className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">
              {stats.resolved} resolved
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Issues</CardTitle>
            {stats.active > 0 ? (
              <XCircle className="h-4 w-4 text-red-500" />
            ) : (
              <CheckCircle className="h-4 w-4 text-green-500" />
            )}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">{stats.active}</div>
            <p className="text-xs text-muted-foreground">
              {stats.critical} critical
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Trend</CardTitle>
            {getTrendIcon()}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold capitalize">{stats.trend}</div>
            <p className="text-xs text-muted-foreground">
              Last 7 days
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Resolution</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.averageResolutionTime}d</div>
            <p className="text-xs text-muted-foreground">
              Time to resolve
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex items-center space-x-4">
        <div className="flex-1">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search crashes..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-10"
            />
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium">Severity:</label>
          <select 
            value={severityFilter} 
            onChange={(e) => setSeverityFilter(e.target.value)}
            className="px-3 py-1 border border-border rounded-md text-sm"
          >
            <option value="all">All</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium">Status:</label>
          <select 
            value={statusFilter} 
            onChange={(e) => setStatusFilter(e.target.value)}
            className="px-3 py-1 border border-border rounded-md text-sm"
          >
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="investigating">Investigating</option>
            <option value="resolved">Resolved</option>
            <option value="ignored">Ignored</option>
          </select>
        </div>
        <Button variant="outline" size="sm" onClick={fetchCrashes} disabled={isLoading}>
          <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Crashes Table */}
      <div className="flex-1 overflow-y-auto">
        <Card>
          <CardHeader>
            <CardTitle>Crash Signatures ({filteredCrashes.length})</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {filteredCrashes.map(crash => (
                <div key={crash.id} className="border rounded-lg p-4 hover:bg-muted/50 transition-colors">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-3 mb-2">
                        {getSeverityIcon(crash.severity)}
                        <h3 className="font-semibold">{crash.signature}</h3>
                        <Badge variant={getSeverityColor(crash.severity)}>
                          {crash.severity}
                        </Badge>
                        <Badge variant="outline" className={getStatusColor(crash.status)}>
                          {crash.status}
                        </Badge>
                      </div>
                      
                      <p className="text-sm text-muted-foreground mb-3">{crash.description}</p>
                      
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                        <div>
                          <span className="font-medium">Frequency:</span>
                          <span className="ml-2">{formatFrequency(crash.frequency)}</span>
                        </div>
                        <div>
                          <span className="font-medium">Last Occurrence:</span>
                          <span className="ml-2">{new Date(crash.lastOccurrence).toLocaleDateString()}</span>
                        </div>
                        <div>
                          <span className="font-medium">Impact:</span>
                          <span className="ml-2">{formatImpact(crash.impact)}</span>
                        </div>
                      </div>
                      
                      <div className="flex items-center space-x-2 mt-3">
                        {crash.tags.map(tag => (
                          <Badge key={tag} variant="outline" className="text-xs">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    </div>
                    
                    <div className="flex items-center space-x-2 ml-4">
                      <Button size="sm" variant="outline" onClick={() => setSelectedCrash(crash)}>
                        <Eye className="h-4 w-4 mr-2" />
                        View Details
                      </Button>
                      <Button size="sm" variant="outline">
                        <Download className="h-4 w-4 mr-2" />
                        Download
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
              
              {filteredCrashes.length === 0 && (
                <div className="text-center text-muted-foreground py-12">
                  <Bug className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No crashes match the current filters</p>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Crash Details Modal */}
      {selectedCrash && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg p-6 max-w-4xl max-h-[80vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold">Crash Details</h2>
              <Button variant="outline" size="sm" onClick={() => setSelectedCrash(null)}>
                Close
              </Button>
            </div>
            
            <div className="space-y-4">
              <div>
                <h3 className="font-semibold mb-2">Signature</h3>
                <p className="text-sm bg-muted p-3 rounded">{selectedCrash.signature}</p>
              </div>
              
              <div>
                <h3 className="font-semibold mb-2">Description</h3>
                <p className="text-sm">{selectedCrash.description}</p>
              </div>
              
              <div>
                <h3 className="font-semibold mb-2">Root Cause</h3>
                <p className="text-sm">{selectedCrash.rootCause}</p>
              </div>
              
              <div>
                <h3 className="font-semibold mb-2">Stack Trace</h3>
                <pre className="text-xs bg-muted p-3 rounded overflow-x-auto">
                  {selectedCrash.stackTrace}
                </pre>
              </div>
              
              {selectedCrash.resolution && (
                <div>
                  <h3 className="font-semibold mb-2">Resolution</h3>
                  <p className="text-sm">{selectedCrash.resolution}</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
