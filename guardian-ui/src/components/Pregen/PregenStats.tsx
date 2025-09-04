import React from 'react';
import { 
  Map, 
  Zap,
  Clock,
  CheckCircle,
  Layers,
  Activity,
  Gauge,
  Database,
  BarChart3
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';

interface PregenStatsProps {
  jobs: any[];
  className?: string;
}

export const PregenStats: React.FC<PregenStatsProps> = ({
  jobs,
  className = ''
}) => {
  // Calculate statistics
  const totalJobs = jobs.length;
  const runningJobs = jobs.filter(j => j.status === 'running').length;
  // const queuedJobs = jobs.filter(j => j.status === 'queued').length;
  const completedJobs = jobs.filter(j => j.status === 'completed').length;
  // const failedJobs = jobs.filter(j => j.status === 'failed').length;
  const gpuAcceleratedJobs = jobs.filter(j => j.gpuAccelerated).length;

  const totalChunks = jobs.reduce((sum, j) => sum + j.totalChunks, 0);
  const completedChunks = jobs.reduce((sum, j) => sum + j.completedChunks, 0);
  const overallProgress = totalChunks > 0 ? (completedChunks / totalChunks) * 100 : 0;

  const averageSpeed = jobs.length > 0 ? 
    jobs.reduce((sum, j) => sum + j.chunksPerSecond, 0) / jobs.length : 0;
  
  const totalMemoryUsage = jobs.reduce((sum, j) => sum + j.memoryUsage, 0);
  const averageMemoryUsage = jobs.length > 0 ? totalMemoryUsage / jobs.length : 0;

  const totalRuntime = jobs.reduce((sum, j) => {
    if (j.status === 'running' && j.startTime) {
      return sum + (Date.now() - j.startTime);
    } else if (j.status === 'completed' && j.endTime && j.startTime) {
      return sum + (j.endTime - j.startTime);
    }
    return sum;
  }, 0);

  const formatChunks = (chunks: number) => {
    if (chunks >= 1000000) {
      return `${(chunks / 1000000).toFixed(1)}M`;
    } else if (chunks >= 1000) {
      return `${(chunks / 1000).toFixed(1)}K`;
    } else {
      return chunks.toString();
    }
  };

  const formatDuration = (milliseconds: number) => {
    const minutes = Math.floor(milliseconds / 60000);
    if (minutes < 60) {
      return `${minutes}m`;
    } else {
      const hours = Math.floor(minutes / 60);
      const mins = minutes % 60;
      return `${hours}h ${mins}m`;
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(1)} GB`;
    } else if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(1)} MB`;
    } else {
      return `${bytes} KB`;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
        return 'text-green-400';
      case 'queued':
        return 'text-blue-400';
      case 'completed':
        return 'text-gray-400';
      case 'failed':
        return 'text-red-400';
      case 'paused':
        return 'text-yellow-400';
      case 'cancelled':
        return 'text-orange-400';
      default:
        return 'text-gray-400';
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

  // Group jobs by dimension
  const jobsByDimension = jobs.reduce((acc, job) => {
    if (!acc[job.dimension]) {
      acc[job.dimension] = [];
    }
    acc[job.dimension].push(job);
    return acc;
  }, {} as Record<string, any[]>);

  // Group jobs by status
  const jobsByStatus = jobs.reduce((acc, job) => {
    if (!acc[job.status]) {
      acc[job.status] = [];
    }
    acc[job.status].push(job);
    return acc;
  }, {} as Record<string, any[]>);

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Total Jobs</p>
                <p className="text-2xl font-bold">{totalJobs}</p>
              </div>
              <Layers className="h-8 w-8 text-muted-foreground" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Running</p>
                <p className="text-2xl font-bold text-green-400">{runningJobs}</p>
              </div>
              <Activity className="h-8 w-8 text-green-400" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Completed</p>
                <p className="text-2xl font-bold text-gray-400">{completedJobs}</p>
              </div>
              <CheckCircle className="h-8 w-8 text-gray-400" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">GPU Accelerated</p>
                <p className="text-2xl font-bold text-blue-400">{gpuAcceleratedJobs}</p>
              </div>
              <Zap className="h-8 w-8 text-blue-400" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Progress Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            Overall Progress
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Total Progress</span>
            <span className="text-sm font-medium">{overallProgress.toFixed(1)}%</span>
          </div>
          <Progress value={overallProgress} className="h-2" />
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">Completed Chunks:</span>
              <span className="ml-2 font-medium">{formatChunks(completedChunks)}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Total Chunks:</span>
              <span className="ml-2 font-medium">{formatChunks(totalChunks)}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Performance Metrics */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Gauge className="h-5 w-5" />
              Performance Metrics
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">Average Speed</span>
              <span className="text-sm font-medium">{averageSpeed.toFixed(1)} chunks/s</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">Total Runtime</span>
              <span className="text-sm font-medium">{formatDuration(totalRuntime)}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">Memory Usage</span>
              <span className="text-sm font-medium">{formatBytes(averageMemoryUsage * 1024 * 1024)}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">GPU Utilization</span>
              <span className="text-sm font-medium">
                {totalJobs > 0 ? Math.round((gpuAcceleratedJobs / totalJobs) * 100) : 0}%
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              Job Distribution
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              {Object.entries(jobsByStatus).map(([status, statusJobs]) => (
                <div key={status} className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className={`w-3 h-3 rounded-full ${
                      status === 'running' ? 'bg-green-400' :
                      status === 'queued' ? 'bg-blue-400' :
                      status === 'completed' ? 'bg-gray-400' :
                      status === 'failed' ? 'bg-red-400' :
                      status === 'paused' ? 'bg-yellow-400' :
                      'bg-orange-400'
                    }`} />
                    <span className="text-sm capitalize">{status}</span>
                  </div>
                  <span className="text-sm font-medium">{(statusJobs as any[]).length}</span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Dimension Breakdown */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Map className="h-5 w-5" />
            Dimension Breakdown
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {Object.entries(jobsByDimension).map(([dimension, dimensionJobs]) => {
              const dimensionChunks = (dimensionJobs as any[]).reduce((sum: number, j: any) => sum + j.totalChunks, 0);
              const dimensionCompleted = (dimensionJobs as any[]).reduce((sum: number, j: any) => sum + j.completedChunks, 0);
              const dimensionProgress = dimensionChunks > 0 ? (dimensionCompleted / dimensionChunks) * 100 : 0;
              
              return (
                <div key={dimension} className="p-4 border rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <Badge className={`text-xs ${getDimensionColor(dimension)}`}>
                      {dimension}
                    </Badge>
                    <span className="text-sm font-medium">{(dimensionJobs as any[]).length} jobs</span>
                  </div>
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-xs">
                      <span>Progress</span>
                      <span>{dimensionProgress.toFixed(1)}%</span>
                    </div>
                    <Progress value={dimensionProgress} className="h-1" />
                    <div className="text-xs text-muted-foreground">
                      {formatChunks(dimensionCompleted)} / {formatChunks(dimensionChunks)} chunks
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Recent Activity */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Recent Activity
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {jobs
              .sort((a, b) => b.createdAt - a.createdAt)
              .slice(0, 5)
              .map((job) => (
                <div key={job.id} className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center">
                      <Map className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="font-medium text-sm">{job.name}</p>
                      <p className="text-xs text-muted-foreground">
                        {new Date(job.createdAt).toLocaleString()}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge className={`text-xs ${getDimensionColor(job.dimension)}`}>
                      {job.dimension}
                    </Badge>
                    <Badge className={`text-xs ${getStatusColor(job.status)}`}>
                      {job.status}
                    </Badge>
                    {job.gpuAccelerated && (
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
    </div>
  );
};

export default PregenStats;
