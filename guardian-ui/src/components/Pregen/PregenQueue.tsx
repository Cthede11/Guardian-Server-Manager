import React, { useState } from 'react';
import { 
  Map, 
  Play, 
  Pause,
  Square,
  MoreHorizontal,
  Edit,
  Trash2,
  Clock,
  Zap,
  Cpu,
  HardDrive,
  CheckCircle,
  XCircle,
  Target,
  Layers,
  Activity,
  Database
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';

interface PregenQueueProps {
  jobs: any[];
  searchQuery: string;
  filterStatus: string;
  filterDimension: string;
  onJobAction: (jobId: string, action: string) => void;
  onDelete: (jobId: string) => void;
  onEdit: (jobId: string) => void;
  className?: string;
}

export const PregenQueue: React.FC<PregenQueueProps> = ({
  jobs,
  searchQuery,
  filterStatus,
  filterDimension,
  onJobAction,
  onDelete,
  onEdit,
}) => {
  const [sortBy] = useState('createdAt');
  const [sortOrder] = useState<'asc' | 'desc'>('desc');

  // Filter and sort jobs
  const filteredJobs = jobs
    .filter(job => {
      const matchesSearch = job.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           job.dimension.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesStatus = filterStatus === 'all' || job.status === filterStatus;
      const matchesDimension = filterDimension === 'all' || job.dimension === filterDimension;
      
      return matchesSearch && matchesStatus && matchesDimension;
    })
    .sort((a, b) => {
      let aValue = a[sortBy];
      let bValue = b[sortBy];
      
      if (sortBy === 'createdAt' || sortBy === 'startTime') {
        aValue = new Date(aValue).getTime();
        bValue = new Date(bValue).getTime();
      } else if (sortBy === 'name') {
        aValue = aValue.toLowerCase();
        bValue = bValue.toLowerCase();
      } else if (sortBy === 'progress') {
        aValue = a.progress;
        bValue = b.progress;
      }
      
      if (sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

  // Unused function removed
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
        return <XCircle className="h-4 w-4" />;
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

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'low':
        return 'bg-gray-500/20 text-gray-400';
      case 'normal':
        return 'bg-blue-500/20 text-blue-400';
      case 'high':
        return 'bg-yellow-500/20 text-yellow-400';
      case 'critical':
        return 'bg-red-500/20 text-red-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const formatDuration = (minutes: number) => {
    if (minutes < 60) {
      return `${minutes}m`;
    } else {
      const hours = Math.floor(minutes / 60);
      const mins = minutes % 60;
      return `${hours}h ${mins}m`;
    }
  };

  const formatChunks = (chunks: number) => {
    if (chunks >= 1000000) {
      return `${(chunks / 1000000).toFixed(1)}M`;
    } else if (chunks >= 1000) {
      return `${(chunks / 1000).toFixed(1)}K`;
    } else {
      return chunks.toString();
    }
  };

  // Unused function removed
  // const getProgressColor = (progress: number) => {
  //   if (progress >= 100) return 'bg-green-500';
  //   if (progress >= 75) return 'bg-blue-500';
  //   if (progress >= 50) return 'bg-yellow-500';
  //   if (progress >= 25) return 'bg-orange-500';
  //   return 'bg-red-500';
  // };

  if (filteredJobs.length === 0) {
    return (
      <Card>
        <CardContent className="text-center py-12">
          <Map className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p className="text-muted-foreground">
            {searchQuery || filterStatus !== 'all' || filterDimension !== 'all'
              ? 'No pregen jobs found matching your criteria' 
              : 'No pregen jobs scheduled'}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            Create your first pregen job to get started
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Layers className="h-5 w-5" />
            Pregen Queue ({filteredJobs.length})
          </CardTitle>
          
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline">
              <Map className="h-4 w-4 mr-2" />
              Export
            </Button>
            <Button size="sm" variant="outline">
              <Layers className="h-4 w-4 mr-2" />
              Bulk Actions
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {filteredJobs.map((job) => (
            <div
              key={job.id}
              className="p-4 border rounded-lg hover:bg-muted/50 transition-colors"
            >
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-4 flex-1">
                  {/* Job Icon/Status */}
                  <div className="flex items-center gap-2">
                    <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
                      <Map className="h-5 w-5" />
                    </div>
                    
                    {getStatusIcon(job.status)}
                  </div>

                  {/* Job Info */}
                  <div className="flex-1 space-y-1">
                    <div className="flex items-center gap-2">
                      <h3 className="font-medium">{job.name}</h3>
                      <Badge 
                        variant="outline" 
                        className={`text-xs ${getDimensionColor(job.dimension)}`}
                      >
                        {job.dimension}
                      </Badge>
                      <Badge 
                        variant="outline" 
                        className={`text-xs ${getPriorityColor(job.priority)}`}
                      >
                        {job.priority}
                      </Badge>
                      {job.gpuAccelerated && (
                        <Badge variant="outline" className="text-xs text-blue-400">
                          <Zap className="h-3 w-3 mr-1" />
                          GPU
                        </Badge>
                      )}
                      {job.tags.length > 0 && (
                        <Badge variant="outline" className="text-xs">
                          {job.tags.join(', ')}
                        </Badge>
                      )}
                    </div>
                    
                    <div className="flex items-center gap-4 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Target className="h-3 w-3" />
                        {job.region.centerX}, {job.region.centerZ} (r{job.region.radius})
                      </span>
                      <span className="flex items-center gap-1">
                        <Database className="h-3 w-3" />
                        {formatChunks(job.totalChunks)} chunks
                      </span>
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatDate(job.createdAt)}
                      </span>
                      {job.status === 'running' && (
                        <span className="flex items-center gap-1">
                          <Activity className="h-3 w-3" />
                          {job.chunksPerSecond} chunks/s
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2">
                  {job.status === 'queued' && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => onJobAction(job.id, 'start')}
                    >
                      <Play className="h-4 w-4 mr-1" />
                      Start
                    </Button>
                  )}
                  
                  {job.status === 'running' && (
                    <>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => onJobAction(job.id, 'pause')}
                      >
                        <Pause className="h-4 w-4 mr-1" />
                        Pause
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => onJobAction(job.id, 'stop')}
                      >
                        <Square className="h-4 w-4 mr-1" />
                        Stop
                      </Button>
                    </>
                  )}
                  
                  {job.status === 'paused' && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => onJobAction(job.id, 'resume')}
                    >
                      <Play className="h-4 w-4 mr-1" />
                      Resume
                    </Button>
                  )}
                  
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button size="sm" variant="ghost">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => onEdit(job.id)}>
                        <Edit className="h-4 w-4 mr-2" />
                        Edit
                      </DropdownMenuItem>
                      
                      <DropdownMenuItem onClick={() => onJobAction(job.id, 'stop')}>
                        <Square className="h-4 w-4 mr-2" />
                        Cancel
                      </DropdownMenuItem>
                      
                      <DropdownMenuSeparator />
                      
                      <DropdownMenuItem 
                        onClick={() => onDelete(job.id)}
                        className="text-red-600"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Delete
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>

              {/* Progress Bar */}
              {job.status === 'running' || job.status === 'paused' || job.status === 'completed' ? (
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">
                      Progress: {formatChunks(job.completedChunks)} / {formatChunks(job.totalChunks)} chunks
                    </span>
                    <span className="font-medium">
                      {job.progress.toFixed(1)}%
                    </span>
                  </div>
                  <Progress 
                    value={job.progress} 
                    className="h-2"
                  />
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span>
                      {job.status === 'running' ? 'Processing...' : 
                       job.status === 'paused' ? 'Paused' : 
                       job.status === 'completed' ? 'Completed' : 'Unknown'}
                    </span>
                    {job.status === 'running' && (
                      <span>
                        ETA: {formatDuration(job.estimatedTime - Math.floor((Date.now() - job.startTime) / 60000))}
                      </span>
                    )}
                  </div>
                </div>
              ) : null}

              {/* Performance Metrics */}
              {job.status === 'running' && (
                <div className="mt-3 pt-3 border-t">
                  <div className="grid grid-cols-4 gap-4 text-xs">
                    <div className="flex items-center gap-1">
                      <Cpu className="h-3 w-3 text-blue-400" />
                      <span className="text-muted-foreground">CPU:</span>
                      <span className="font-medium">{job.chunksPerSecond} chunks/s</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <HardDrive className="h-3 w-3 text-green-400" />
                      <span className="text-muted-foreground">Memory:</span>
                      <span className="font-medium">{job.memoryUsage} MB</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Zap className="h-3 w-3 text-yellow-400" />
                      <span className="text-muted-foreground">GPU:</span>
                      <span className="font-medium">{job.gpuAccelerated ? 'Active' : 'Inactive'}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Clock className="h-3 w-3 text-purple-400" />
                      <span className="text-muted-foreground">Runtime:</span>
                      <span className="font-medium">
                        {formatDuration(Math.floor((Date.now() - job.startTime) / 60000))}
                      </span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

export default PregenQueue;
