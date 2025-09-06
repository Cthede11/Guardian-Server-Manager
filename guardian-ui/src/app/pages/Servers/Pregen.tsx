import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import { 
  Map, 
  Plus, 
  Play,
  Pause,
  Square,
  RefreshCw,
  Search,
  // Filter,
  Settings,
  Zap,
  Cpu,
  // HardDrive,
  Clock,
  CheckCircle,
  // AlertTriangle,
  // XCircle,
  // MoreHorizontal,
  // Edit,
  // Trash2,
  // Download,
  // Upload,
  // Target,
  Layers,
  Activity,
  Gauge,
  // Database,
  // Server,
  // Monitor
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
// import { 
//   DropdownMenu, 
//   DropdownMenuContent, 
//   DropdownMenuItem, 
//   DropdownMenuTrigger,
//   DropdownMenuSeparator
// } from '@/components/ui/dropdown-menu';
import { useServersStore } from '@/store/servers';
import { usePregenJobs, liveStore } from '@/store/live';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import { PregenQueue } from '@/components/Pregen/PregenQueue';
import { RegionSelector } from '@/components/Pregen/RegionSelector';
import { PregenStats } from '@/components/Pregen/PregenStats';

interface PregenPageProps {
  className?: string;
}

export const Pregen: React.FC<PregenPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  // Use live store for pregen jobs data
  const pregenJobs = usePregenJobs(serverId || '');
  
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState('all');
  const [filterDimension, setFilterDimension] = useState('all');
  // const [selectedRegion, setSelectedRegion] = useState<any>(null);
  const [regionSelectorOpen, setRegionSelectorOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  // No need for fetchPregenJobs since we're using the live store

  // No need for mock data generation since we're using the live store

  const handleCreatePregenJob = async (regionData: any) => {
    try {
      // Simulate creating a pregen job
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const newJob = {
        id: `pregen-${Date.now()}`,
        ...regionData,
        status: 'queued',
        createdAt: Date.now(),
        createdBy: 'admin',
        completedChunks: 0,
        progress: 0
      };
      
      // Update live store instead of local state
      liveStore.getState().addPregenJob(serverId || '', newJob);
      setRegionSelectorOpen(false);
    } catch (error) {
      console.error('Error creating pregen job:', error);
    }
  };

  const handleJobAction = async (jobId: string, action: string) => {
    try {
      // Simulate job action
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      let updates: any = {};
      switch (action) {
        case 'start':
          updates = { status: 'running', startTime: Date.now() };
          break;
        case 'pause':
          updates = { status: 'paused' };
          break;
        case 'stop':
          updates = { status: 'cancelled', endTime: Date.now() };
          break;
        case 'resume':
          updates = { status: 'running' };
          break;
        default:
          return;
      }
      
      // Update live store instead of local state
      liveStore.getState().updatePregenJob(serverId || '', jobId, updates);
    } catch (error) {
      console.error('Error performing job action:', error);
    }
  };

  const handleDeleteJob = async (jobId: string) => {
    try {
      // Simulate deletion
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Update live store instead of local state
      const currentJobs = liveStore.getState().pregenJobs[serverId || ''] || [];
      const updatedJobs = currentJobs.filter(job => job.id !== jobId);
      liveStore.setState(state => ({
        pregenJobs: {
          ...state.pregenJobs,
          [serverId || '']: updatedJobs,
        },
      }));
    } catch (error) {
      console.error('Error deleting job:', error);
    }
  };

  // const getStatusColor = (status: string) => {
  //   switch (status) {
  //     case 'queued':
  //       return 'text-blue-400';
  //     case 'running':
  //       return 'text-green-400';
  //     case 'paused':
  //       return 'text-yellow-400';
  //     case 'completed':
  //       return 'text-gray-400';
  //     case 'failed':
  //       return 'text-red-400';
  //     case 'cancelled':
  //       return 'text-orange-400';
  //     default:
  //       return 'text-gray-400';
  //   }
  // };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'queued':
        return <Clock className="h-4 w-4" />;
      case 'running':
        return <Play className="h-4 w-4" />;
      case 'paused':
        return <Pause className="h-4 w-4" />;
      case 'completed':
        return <CheckCircle className="h-4 w-4" />;
      case 'failed':
        return <CheckCircle className="h-4 w-4" />;
      case 'cancelled':
        return <Square className="h-4 w-4" />;
      default:
        return <Clock className="h-4 w-4" />;
    }
  };

  const getDimensionColor = (dimension: string) => {
    switch (dimension) {
      case 'overworld':
        return 'bg-green-500/20 text-green-400';
      case 'nether':
        return 'bg-red-500/20 text-red-400';
      case 'end':
        return 'bg-purple-500/20 text-purple-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  // const getPriorityColor = (priority: string) => {
  //   switch (priority) {
  //     case 'low':
  //       return 'bg-gray-500/20 text-gray-400';
  //     case 'normal':
  //       return 'bg-blue-500/20 text-blue-400';
  //     case 'high':
  //       return 'bg-yellow-500/20 text-yellow-400';
  //     case 'critical':
  //       return 'bg-red-500/20 text-red-400';
  //     default:
  //       return 'bg-gray-500/20 text-gray-400';
  //   }
  // };

  // const formatDate = (timestamp: number) => {
  //   return new Date(timestamp).toLocaleString();
  // };

  // const formatDuration = (minutes: number) => {
  //   if (minutes < 60) {
  //     return `${minutes}m`;
  //   } else {
  //     const hours = Math.floor(minutes / 60);
  //     const mins = minutes % 60;
  //     return `${hours}h ${mins}m`;
  //   }
  // };

  const formatChunks = (chunks: number) => {
    if (chunks >= 1000000) {
      return `${(chunks / 1000000).toFixed(1)}M`;
    } else if (chunks >= 1000) {
      return `${(chunks / 1000).toFixed(1)}K`;
    } else {
      return chunks.toString();
    }
  };

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its pregen jobs."
        />
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Pregen</h2>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <Map className="h-3 w-3" />
              {pregenJobs.filter(j => j.status === 'running').length} Running
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {pregenJobs.filter(j => j.status === 'queued').length} Queued
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <Zap className="h-3 w-3" />
              {pregenJobs.filter(j => j.gpuAssist).length} GPU Assisted
            </Badge>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setRegionSelectorOpen(true)}
          >
            <Plus className="h-4 w-4 mr-2" />
            New Region
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={async () => {
              setIsLoading(true);
              try {
                // Simulate a refresh operation
                await new Promise(resolve => setTimeout(resolve, 1000));
                
                // Refresh pregen jobs from live store
                const currentJobs = liveStore.getState().pregenJobs[serverId || ''] || [];
                // Trigger a refresh by updating the store
                liveStore.setState(state => ({
                  pregenJobs: {
                    ...state.pregenJobs,
                    [serverId || '']: [...currentJobs],
                  },
                }));
              } finally {
                setIsLoading(false);
              }
            }}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search pregen jobs..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="queued">Queued</SelectItem>
            <SelectItem value="running">Running</SelectItem>
            <SelectItem value="paused">Paused</SelectItem>
            <SelectItem value="completed">Completed</SelectItem>
            <SelectItem value="failed">Failed</SelectItem>
            <SelectItem value="cancelled">Cancelled</SelectItem>
          </SelectContent>
        </Select>
        
        <Select value={filterDimension} onValueChange={setFilterDimension}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Dimensions</SelectItem>
            <SelectItem value="overworld">Overworld</SelectItem>
            <SelectItem value="nether">Nether</SelectItem>
            <SelectItem value="end">End</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="queue" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="queue" className="flex items-center gap-2">
            <Layers className="h-4 w-4" />
            Queue
          </TabsTrigger>
          <TabsTrigger value="stats" className="flex items-center gap-2">
            <Gauge className="h-4 w-4" />
            Stats
          </TabsTrigger>
          <TabsTrigger value="regions" className="flex items-center gap-2">
            <Map className="h-4 w-4" />
            Regions
          </TabsTrigger>
          <TabsTrigger value="settings" className="flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Settings
          </TabsTrigger>
        </TabsList>

        <TabsContent value="queue" className="space-y-4">
          <PregenQueue
            jobs={pregenJobs}
            searchQuery={searchQuery}
            filterStatus={filterStatus}
            filterDimension={filterDimension}
            onJobAction={handleJobAction}
            onDelete={handleDeleteJob}
            onEdit={(jobId) => {
              console.log('Edit job:', jobId);
            }}
          />
        </TabsContent>

        <TabsContent value="stats" className="space-y-4">
          <PregenStats jobs={pregenJobs} />
        </TabsContent>

        <TabsContent value="regions" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Map className="h-5 w-5" />
                  Active Regions
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {pregenJobs
                    .filter(j => j.status === 'running' || j.status === 'queued')
                    .slice(0, 5)
                    .map((job) => (
                      <div key={job.id} className="flex items-center justify-between p-3 border rounded-lg">
                        <div className="flex items-center gap-3">
                          <div className="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center">
                            {getStatusIcon(job.status)}
                          </div>
                          <div>
                            <p className="font-medium text-sm">Region {job.id}</p>
                            <p className="text-xs text-muted-foreground">
                              {job.region.x}, {job.region.z} (r{job.region.radius})
                            </p>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <Badge className={`text-xs ${getDimensionColor(job.dimension)}`}>
                            {job.dimension}
                          </Badge>
                          {job.gpuAssist && (
                            <Badge variant="outline" className="text-xs text-blue-400">
                              <Zap className="h-3 w-3 mr-1" />
                              GPU
                            </Badge>
                          )}
                        </div>
                      </div>
                    ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Performance
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Total Chunks Processed</span>
                    <span className="text-sm font-medium">
                      {formatChunks(pregenJobs.reduce((sum, j) => sum + (j.progress || 0), 0))}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Average Progress</span>
                    <span className="text-sm font-medium">
                      {Math.floor(pregenJobs.reduce((sum, j) => sum + (j.progress || 0), 0) / pregenJobs.length)}%
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">GPU Acceleration</span>
                    <span className="text-sm font-medium">
                      {pregenJobs.length > 0 ? Math.floor((pregenJobs.filter(j => j.gpuAssist).length / pregenJobs.length) * 100) : 0}%
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Active Jobs</span>
                    <span className="text-sm font-medium">
                      {pregenJobs.filter(j => j.status === 'running').length}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="settings" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Cpu className="h-5 w-5" />
                  Performance Settings
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Max Concurrent Jobs</label>
                  <Select defaultValue="2">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">1 Job</SelectItem>
                      <SelectItem value="2">2 Jobs</SelectItem>
                      <SelectItem value="3">3 Jobs</SelectItem>
                      <SelectItem value="4">4 Jobs</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">Chunk Processing Rate</label>
                  <Select defaultValue="medium">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="low">Low (10 chunks/s)</SelectItem>
                      <SelectItem value="medium">Medium (25 chunks/s)</SelectItem>
                      <SelectItem value="high">High (50 chunks/s)</SelectItem>
                      <SelectItem value="unlimited">Unlimited</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">Memory Limit</label>
                  <Select defaultValue="2048">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1024">1 GB</SelectItem>
                      <SelectItem value="2048">2 GB</SelectItem>
                      <SelectItem value="4096">4 GB</SelectItem>
                      <SelectItem value="8192">8 GB</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Zap className="h-5 w-5" />
                  GPU Settings
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">GPU Acceleration</label>
                  <Select defaultValue="enabled">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="enabled">Enabled</SelectItem>
                      <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">GPU Memory Limit</label>
                  <Select defaultValue="1024">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="512">512 MB</SelectItem>
                      <SelectItem value="1024">1 GB</SelectItem>
                      <SelectItem value="2048">2 GB</SelectItem>
                      <SelectItem value="4096">4 GB</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">GPU Queue Size</label>
                  <Select defaultValue="1000">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="500">500 chunks</SelectItem>
                      <SelectItem value="1000">1000 chunks</SelectItem>
                      <SelectItem value="2000">2000 chunks</SelectItem>
                      <SelectItem value="5000">5000 chunks</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>

      {/* Region Selector Modal */}
      <RegionSelector
        isOpen={regionSelectorOpen}
        onClose={() => setRegionSelectorOpen(false)}
        onCreate={handleCreatePregenJob}
      />
    </div>
  );
};

export default Pregen;
