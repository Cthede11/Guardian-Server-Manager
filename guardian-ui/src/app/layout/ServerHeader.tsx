import React, { useState } from 'react';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
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
  // const location = useLocation();
  const { 
    getServerById, 
    startServer, 
    stopServer, 
    restartServer, 
    promoteServer,
    error,
    clearError
  } = useServersStore();

  // Always call hooks at the top level
  const server = serverId ? getServerById(serverId) : null;
  const serverHealth = useServerHealth(serverId || '');
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  
  // Determine current tab from pathname
  // const currentTab = React.useMemo(() => {
  //   const pathParts = location.pathname.split('/');
  //   const tabIndex = pathParts.findIndex(part => part === serverId) + 1;
  //   return pathParts[tabIndex] || 'overview';
  // }, [location.pathname, serverId]);

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

  if (!serverId || !server) {
    return null;
  }

  return (
    <div className="app-header border-b border-border/50 shadow-lg">
      {/* Error Display */}
      {error && (
        <div className="px-6 py-3 bg-destructive/10 border-b border-destructive/20">
          <div className="flex items-center gap-3 text-destructive text-sm">
            <AlertTriangle className="h-4 w-4 flex-shrink-0" />
            <span className="flex-1">{error}</span>
            <Button
              size="sm"
              variant="ghost"
              onClick={clearError}
              className="h-6 w-6 p-0 hover:bg-destructive/20"
            >
              ×
            </Button>
          </div>
        </div>
      )}
      
      <div className="h-20 flex items-center justify-between px-6">
      {/* Server Info */}
      <div className="flex items-center gap-4">
        <div className="w-14 h-14 bg-gradient-to-br from-primary/25 to-secondary/25 rounded-2xl flex items-center justify-center ring-1 ring-primary/20">
          <Play className="h-7 w-7 text-primary" />
        </div>
        <div>
          <h1 className="text-2xl font-bold gradient-text">{server.name}</h1>
          <div className="flex items-center gap-4 mt-2">
            <StatusPill status={server.status} />
            {server.blueGreen && (
              <Badge variant="outline" className="text-xs bg-primary/15 border-primary/40 text-primary px-3 py-1">
                {server.blueGreen.active === 'blue' ? 'Blue' : 'Green'}
                {server.blueGreen.candidateHealthy && (
                  <Circle className="h-2 w-2 ml-2 text-success" />
                )}
              </Badge>
            )}
            {serverHealth && typeof serverHealth === 'object' && (
              <div className="flex items-center gap-3">
                {(serverHealth as any).rcon ? (
                  <CheckCircle className="h-4 w-4 text-success" />
                ) : (
                  <AlertTriangle className="h-4 w-4 text-warning" />
                )}
                {(serverHealth as any).query ? (
                  <CheckCircle className="h-4 w-4 text-success" />
                ) : (
                  <AlertTriangle className="h-4 w-4 text-warning" />
                )}
                {(serverHealth as any).crashTickets > 0 && (
                  <Badge variant="destructive" className="text-xs px-2 py-1">
                    {(serverHealth as any).crashTickets} crashes
                  </Badge>
                )}
                {(serverHealth as any).freezeTickets > 0 && (
                  <Badge variant="secondary" className="text-xs px-2 py-1">
                    {(serverHealth as any).freezeTickets} freezes
                  </Badge>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex items-center gap-3">
        <Button
          size="sm"
          onClick={() => handleServerAction('start')}
          disabled={getActionButtonState('start').disabled}
          className="btn-primary"
        >
          {getActionButtonState('start').loading ? (
            <Loader2 className="h-4 w-4 animate-spin mr-2" />
          ) : (
            <Play className="h-4 w-4 mr-2" />
          )}
          Start
        </Button>
        
        <Button
          size="sm"
          variant="destructive"
          onClick={() => handleServerAction('stop')}
          disabled={getActionButtonState('stop').disabled}
          className="px-4 py-2 rounded-xl font-semibold text-sm transition-all duration-300 ease-out bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-md hover:shadow-lg hover:-translate-y-0.5"
        >
          {getActionButtonState('stop').loading ? (
            <Loader2 className="h-4 w-4 animate-spin mr-2" />
          ) : (
            <Square className="h-4 w-4 mr-2" />
          )}
          Stop
        </Button>
        
        <Button
          size="sm"
          variant="outline"
          onClick={() => handleServerAction('restart')}
          disabled={getActionButtonState('restart').disabled}
          className="border-border hover:bg-accent/50 shadow-sm hover:shadow-md transition-all duration-200"
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
          className="border-secondary/50 hover:bg-secondary/10 shadow-sm hover:shadow-md transition-all duration-200"
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
    <div className="bg-card border-b border-border px-6 shadow-sm">
      <Tabs value={currentTab} onValueChange={handleTabChange} className="w-full">
        <TabsList className="grid w-full grid-cols-12 bg-muted/30 h-12 rounded-lg p-1">
          {serverTabs.map((tab) => (
            <TabsTrigger
              key={tab.id}
              value={tab.id}
              className="text-sm font-medium data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-md rounded-md transition-all duration-200 hover:bg-accent/50"
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
