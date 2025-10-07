import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { 
  Server, 
  Play, 
  Square, 
  RotateCcw,
  MoreVertical,
  Search,
  Filter,
  Plus,
  Activity,
  Users,
  MemoryStick,
  Clock,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Loader2,
  Eye,
  Settings,
  Trash2,
  Copy,
  FolderOpen,
  RefreshCw
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';
import { Progress } from '@/components/ui/progress';
import { useServers } from '@/store/servers-new';
import { metricsCollector, type MetricData } from '@/lib/metrics-collector';
import { realtimeConnection } from '@/lib/websocket';
import { errorHandler } from '@/lib/error-handler';
import { openServerFolder } from '@/lib/tauri-api';

interface ServerListDashboardProps {
  className?: string;
}

export const ServerListDashboard: React.FC<ServerListDashboardProps> = ({ className }) => {
  const { 
    summaries, 
    selectedId, 
    select, 
    loading, 
    startServer, 
    stopServer, 
    restartServer, 
    deleteServer, 
    fetchServers 
  } = useServers();
  
  const servers = Object.values(summaries);
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [sortBy, setSortBy] = useState('name');
  const [metrics, setMetrics] = useState<Record<string, MetricData>>({});
  const [isConnected, setIsConnected] = useState(false);

  // Debug: Log server list changes
  useEffect(() => {
    console.log('Server list updated:', { servers: servers.length, summaries: Object.keys(summaries) });
  }, [servers, summaries]);

  useEffect(() => {
    // Connect to real-time updates
    realtimeConnection.connect().then(() => {
      setIsConnected(true);
    }).catch((error: any) => {
      errorHandler.handleError(error, 'WebSocket Connection');
    });

    // Set up real-time message handlers
    const unsubscribeMetrics = realtimeConnection.subscribe('metrics', (data: any) => {
      if (data.serverId) {
        setMetrics(prev => ({
          ...prev,
          [data.serverId]: data
        }));
      }
    });

    return () => {
      realtimeConnection.disconnect();
      unsubscribeMetrics();
      setIsConnected(false);
    };
  }, []);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'text-green-500';
      case 'stopped': return 'text-red-500';
      case 'starting': return 'text-yellow-500';
      case 'stopping': return 'text-orange-500';
      default: return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running': return <CheckCircle className="h-4 w-4" />;
      case 'stopped': return <XCircle className="h-4 w-4" />;
      case 'starting': return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'stopping': return <Loader2 className="h-4 w-4 animate-spin" />;
      default: return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const handleServerAction = async (serverId: string, action: 'start' | 'stop' | 'restart') => {
    try {
      switch (action) {
        case 'start':
          await startServer(serverId);
          break;
        case 'stop':
          await stopServer(serverId);
          break;
        case 'restart':
          await restartServer(serverId);
          break;
      }
    } catch (error) {
      errorHandler.handleError(error as Error, `Server ${action}`, { serverId });
    }
  };

  const handleDeleteServer = async (serverId: string) => {
    if (window.confirm('Are you sure you want to delete this server? This action cannot be undone.')) {
      try {
        await deleteServer(serverId);
      } catch (error) {
        errorHandler.handleError(error as Error, 'Delete Server', { serverId });
      }
    }
  };

  const filteredAndSortedServers = servers
    .filter(server => {
      const matchesSearch = server.name.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesStatus = statusFilter === 'all' || server.status === statusFilter;
      return matchesSearch && matchesStatus;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'status':
          return a.status.localeCompare(b.status);
        case 'version':
          return (a.version || '').localeCompare(b.version || '');
        case 'players':
          const aPlayers = metrics[a.id]?.playersOnline || 0;
          const bPlayers = metrics[b.id]?.playersOnline || 0;
          return bPlayers - aPlayers;
        default:
          return 0;
      }
    });

  if (loading) {
    return (
      <div className={`flex items-center justify-center h-64 ${className}`}>
        <div className="text-center">
          <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Loading servers...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Server Dashboard</h1>
          <p className="text-muted-foreground">
            Manage and monitor your Minecraft servers
          </p>
        </div>
        
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="flex items-center gap-1">
            <Server className="h-3 w-3" />
            {servers.length} Servers
          </Badge>
          {isConnected && (
            <Badge variant="outline" className="flex items-center gap-1 text-green-400">
              <CheckCircle className="h-3 w-3" />
              Connected
            </Badge>
          )}
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search servers..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={statusFilter} onValueChange={setStatusFilter}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="running">Running</SelectItem>
            <SelectItem value="stopped">Stopped</SelectItem>
            <SelectItem value="starting">Starting</SelectItem>
            <SelectItem value="stopping">Stopping</SelectItem>
          </SelectContent>
        </Select>

        <Select value={sortBy} onValueChange={setSortBy}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="name">Name</SelectItem>
            <SelectItem value="status">Status</SelectItem>
            <SelectItem value="version">Version</SelectItem>
            <SelectItem value="players">Players</SelectItem>
          </SelectContent>
        </Select>

        <Button onClick={() => {
          console.log('Manual refresh clicked');
          fetchServers();
        }}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh
        </Button>
        
        <Button 
          variant="outline" 
          onClick={async () => {
            console.log('Testing API connection...');
            try {
              const response = await fetch('http://127.0.0.1:52100/api/servers');
              const data = await response.json();
              console.log('API test response:', data);
              alert(`API Test: Found ${data.data?.length || 0} servers`);
            } catch (error) {
              console.error('API test failed:', error);
              alert(`API Test Failed: ${error}`);
            }
          }}
        >
          Test API
        </Button>
      </div>

      {/* Server Grid */}
      {filteredAndSortedServers.length === 0 ? (
        <Card>
          <CardContent className="text-center py-12">
            <Server className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
            <h3 className="text-lg font-medium mb-2">No servers found</h3>
            <p className="text-muted-foreground mb-4">
              {searchQuery ? 'No servers match your search criteria' : 'Get started by creating your first server'}
            </p>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              Create Server
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredAndSortedServers.map((server) => {
            const serverMetrics = metrics[server.id];
            const isSelected = selectedId === server.id;
            
            return (
              <Card 
                key={server.id} 
                className={`hover:shadow-lg transition-all duration-200 ${
                  isSelected ? 'ring-2 ring-blue-500' : ''
                }`}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <Server className="h-5 w-5 text-blue-400" />
                      <div>
                        <CardTitle className="text-lg">{server.name}</CardTitle>
                        <div className="flex items-center gap-2 mt-1">
                          <Badge variant="outline" className="text-xs">
                            {server.version || 'Unknown'}
                          </Badge>
                          <div className={`flex items-center gap-1 ${getStatusColor(server.status)}`}>
                            {getStatusIcon(server.status)}
                            <span className="text-sm font-medium capitalize">
                              {server.status}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm">
                          <MoreVertical className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem asChild>
                          <Link to={`/servers/${server.id}`}>
                            <Eye className="h-4 w-4 mr-2" />
                            View Details
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem asChild>
                          <Link to={`/servers/${server.id}/settings`}>
                            <Settings className="h-4 w-4 mr-2" />
                            Settings
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => openServerFolder(server.id)}>
                          <FolderOpen className="h-4 w-4 mr-2" />
                          Open Folder
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem 
                          onClick={() => handleDeleteServer(server.id)}
                          className="text-red-400"
                        >
                          <Trash2 className="h-4 w-4 mr-2" />
                          Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </CardHeader>
                
                <CardContent className="space-y-4">
                  {/* Live Metrics */}
                  {serverMetrics && (
                    <div className="grid grid-cols-2 gap-4">
                      <div className="text-center">
                        <div className="flex items-center justify-center gap-1 mb-1">
                          <Activity className="h-3 w-3 text-blue-400" />
                          <span className="text-xs text-muted-foreground">TPS</span>
                        </div>
                        <div className="text-lg font-bold text-blue-400">
                          {serverMetrics.tps?.toFixed(1) || '0.0'}
                        </div>
                        <Progress 
                          value={serverMetrics.tps ? (serverMetrics.tps / 20) * 100 : 0} 
                          className="h-1 mt-1"
                        />
                      </div>
                      
                      <div className="text-center">
                        <div className="flex items-center justify-center gap-1 mb-1">
                          <Users className="h-3 w-3 text-green-400" />
                          <span className="text-xs text-muted-foreground">Players</span>
                        </div>
                        <div className="text-lg font-bold text-green-400">
                          {serverMetrics.playersOnline || 0}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          / {server.maxPlayers || 20}
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Server Actions */}
                  <div className="flex items-center gap-2">
                    {server.status === 'running' ? (
                      <>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => handleServerAction(server.id, 'restart')}
                          disabled={server.status !== 'running'}
                          className="flex-1"
                        >
                          <RotateCcw className="h-4 w-4 mr-1" />
                          Restart
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          onClick={() => handleServerAction(server.id, 'stop')}
                          disabled={server.status !== 'running'}
                          className="flex-1"
                        >
                          <Square className="h-4 w-4 mr-1" />
                          Stop
                        </Button>
                      </>
                    ) : (
                      <Button
                        size="sm"
                        onClick={() => handleServerAction(server.id, 'start')}
                        disabled={server.status !== 'stopped'}
                        className="flex-1"
                      >
                        <Play className="h-4 w-4 mr-1" />
                        Start
                      </Button>
                    )}
                  </div>

                  {/* Quick Stats */}
                  <div className="text-xs text-muted-foreground space-y-1">
                    <div className="flex justify-between">
                      <span>Memory:</span>
                      <span>{serverMetrics?.heapMb ? `${Math.round(serverMetrics.heapMb / 1024)}GB` : 'N/A'}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Status:</span>
                      <span className="capitalize">{server.status}</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default ServerListDashboard;
