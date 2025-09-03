import React, { useState } from 'react';
import { useParams, useLocation, useNavigate } from 'react-router-dom';
import { Play, Square, RotateCcw, ArrowUpDown, Circle, Loader2, AlertTriangle, CheckCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useServersStore, useServerHealth } from '@/store/servers';
import { StatusPill } from '@/components/StatusPill';

const serverTabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'console', label: 'Console' },
  { id: 'players', label: 'Players' },
  { id: 'world', label: 'World' },
  { id: 'mods-rules', label: 'Mods & Rules' },
  { id: 'performance', label: 'Performance' },
  { id: 'backups', label: 'Backups' },
  { id: 'events', label: 'Events' },
  { id: 'pregen', label: 'Pregen' },
  { id: 'sharding', label: 'Sharding' },
  { id: 'diagnostics', label: 'Diagnostics' },
  { id: 'settings', label: 'Settings' },
];

export const ServerHeader: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const location = useLocation();
  const { 
    getServerById, 
    startServer, 
    stopServer, 
    restartServer, 
    promoteServer,
    error,
    clearError
  } = useServersStore();

  const server = serverId ? getServerById(serverId) : null;
  const serverHealth = serverId ? useServerHealth(serverId) : null;
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  
  // Determine current tab from pathname
  const currentTab = React.useMemo(() => {
    const pathParts = location.pathname.split('/');
    const tabIndex = pathParts.findIndex(part => part === serverId) + 1;
    return pathParts[tabIndex] || 'overview';
  }, [location.pathname, serverId]);

  const handleServerAction = async (action: 'start' | 'stop' | 'restart' | 'promote') => {
    if (!serverId) return;
    
    setActionLoading(action);
    clearError();
    
    try {
      let success = false;
      switch (action) {
        case 'start':
          success = await startServer(serverId);
          break;
        case 'stop':
          success = await stopServer(serverId);
          break;
        case 'restart':
          success = await restartServer(serverId);
          break;
        case 'promote':
          success = await promoteServer(serverId);
          break;
      }
      
      if (!success && error) {
        // Error is already set in the store
        console.error(`Failed to ${action} server:`, error);
      }
    } catch (err) {
      console.error(`Error during ${action} action:`, err);
    } finally {
      setActionLoading(null);
    }
  };

  const getActionButtonState = (action: 'start' | 'stop' | 'restart' | 'promote') => {
    if (!server) return { disabled: true, loading: false };
    
    const isLoading = actionLoading === action;
    
    switch (action) {
      case 'start':
        return {
          disabled: server.status === 'running' || server.status === 'starting' || isLoading,
          loading: server.status === 'starting' || isLoading,
        };
      case 'stop':
        return {
          disabled: server.status === 'stopped' || server.status === 'stopping' || isLoading,
          loading: server.status === 'stopping' || isLoading,
        };
      case 'restart':
        return {
          disabled: server.status === 'stopped' || server.status === 'starting' || server.status === 'stopping' || isLoading,
          loading: server.status === 'starting' || server.status === 'stopping' || isLoading,
        };
      case 'promote':
        return {
          disabled: server.status !== 'running' || !server.blueGreen.candidateHealthy || isLoading,
          loading: isLoading,
        };
      default:
        return { disabled: true, loading: false };
    }
  };

  if (!server) {
    return (
      <div className="h-16 bg-card border-b border-border flex items-center justify-center">
        <p className="text-muted-foreground">Select a server to view details</p>
      </div>
    );
  }

  return (
    <div className="bg-card border-b border-border">
      {/* Error Display */}
      {error && (
        <div className="px-6 py-2 bg-destructive/10 border-b border-destructive/20">
          <div className="flex items-center gap-2 text-destructive text-sm">
            <AlertTriangle className="h-4 w-4" />
            <span>{error}</span>
            <Button
              size="sm"
              variant="ghost"
              onClick={clearError}
              className="ml-auto h-6 w-6 p-0"
            >
              ×
            </Button>
          </div>
        </div>
      )}
      
      <div className="h-16 flex items-center justify-between px-6">
      {/* Server Info */}
      <div className="flex items-center gap-4">
        <div>
          <h1 className="text-lg font-semibold">{server.name}</h1>
          <div className="flex items-center gap-2">
            <StatusPill status={server.status} />
            {server.blueGreen && (
              <Badge variant="outline" className="text-xs">
                {server.blueGreen.active === 'blue' ? 'Blue' : 'Green'}
                {server.blueGreen.candidateHealthy && (
                  <Circle className="h-2 w-2 ml-1 text-green-400" />
                )}
              </Badge>
            )}
            {serverHealth && (
              <div className="flex items-center gap-1">
                {serverHealth.rcon ? (
                  <CheckCircle className="h-3 w-3 text-green-400" title="RCON Connected" />
                ) : (
                  <AlertTriangle className="h-3 w-3 text-yellow-400" title="RCON Disconnected" />
                )}
                {serverHealth.query ? (
                  <CheckCircle className="h-3 w-3 text-green-400" title="Query Connected" />
                ) : (
                  <AlertTriangle className="h-3 w-3 text-yellow-400" title="Query Disconnected" />
                )}
                {serverHealth.crashTickets > 0 && (
                  <Badge variant="destructive" className="text-xs">
                    {serverHealth.crashTickets} crashes
                  </Badge>
                )}
                {serverHealth.freezeTickets > 0 && (
                  <Badge variant="secondary" className="text-xs">
                    {serverHealth.freezeTickets} freezes
                  </Badge>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex items-center gap-2">
        <Button
          size="sm"
          onClick={() => handleServerAction('start')}
          disabled={getActionButtonState('start').disabled}
          className="bg-green-600 hover:bg-green-700"
        >
          {getActionButtonState('start').loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Play className="h-4 w-4" />
          )}
          Start
        </Button>
        
        <Button
          size="sm"
          variant="destructive"
          onClick={() => handleServerAction('stop')}
          disabled={getActionButtonState('stop').disabled}
        >
          {getActionButtonState('stop').loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Square className="h-4 w-4" />
          )}
          Stop
        </Button>
        
        <Button
          size="sm"
          variant="outline"
          onClick={() => handleServerAction('restart')}
          disabled={getActionButtonState('restart').disabled}
        >
          {getActionButtonState('restart').loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RotateCcw className="h-4 w-4" />
          )}
          Restart
        </Button>
        
        <Button
          size="sm"
          variant="outline"
          onClick={() => handleServerAction('promote')}
          disabled={getActionButtonState('promote').disabled}
        >
          {getActionButtonState('promote').loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <ArrowUpDown className="h-4 w-4" />
          )}
          Promote
        </Button>
      </div>
      </div>
    </div>
  );
};

export const ServerTabs: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const location = useLocation();
  const navigate = useNavigate();
  
  // Determine current tab from pathname
  const currentTab = React.useMemo(() => {
    const pathParts = location.pathname.split('/');
    const tabIndex = pathParts.findIndex(part => part === serverId) + 1;
    return pathParts[tabIndex] || 'overview';
  }, [location.pathname, serverId]);

  if (!serverId) return null;

  const handleTabChange = (tabId: string) => {
    navigate(`/servers/${serverId}/${tabId}`);
  };

  return (
    <div className="bg-card border-b border-border px-6">
      <Tabs value={currentTab} onValueChange={handleTabChange} className="w-full">
        <TabsList className="grid w-full grid-cols-12">
          {serverTabs.map((tab) => (
            <TabsTrigger
              key={tab.id}
              value={tab.id}
            >
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
    </div>
  );
};

export default ServerHeader;
