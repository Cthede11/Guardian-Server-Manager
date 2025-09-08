import React, { useState, useEffect, useRef, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useVirtualizer } from '@tanstack/react-virtual';
import { 
  Send, 
  Filter, 
  Download, 
  Trash2, 
  Play, 
  Pause, 
  AlertTriangle,
  Info,
  Bug,
  CheckCircle,
  Loader2
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { useServersStore } from '@/store/servers';
import { useConsoleStream, useConnectionStatus, liveStore } from '@/store/live';
import type { ConsoleMessage } from '@/lib/types';
import { api } from '@/lib/api';

interface ConsoleStreamProps {
  className?: string;
}

export const ConsoleStream: React.FC<ConsoleStreamProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  // Use the live store for console messages
  const messages = useConsoleStream(serverId || '');
  
  const [command, setCommand] = useState('');
  const [isPaused, setIsPaused] = useState(false);
  const [eulaStatus, setEulaStatus] = useState<'accepted' | 'pending' | 'missing' | null>(null);
  const [eulaBusy, setEulaBusy] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [filters, setFilters] = useState({
    level: 'all' as 'all' | 'info' | 'warn' | 'error' | 'debug',
    search: '',
    showTimestamps: true,
    showLevels: true,
    autoScroll: true
  });
  
  const parentRef = useRef<HTMLDivElement>(null);
  const commandInputRef = useRef<HTMLInputElement>(null);

  // Filter messages with memoization for performance
  const filteredMessages = useMemo(() => {
    return messages.filter(msg => {
      if (filters.level !== 'all' && msg.level !== filters.level) return false;
      if (filters.search && !msg.msg.toLowerCase().includes(filters.search.toLowerCase())) return false;
      return true;
    });
  }, [messages, filters.level, filters.search]);

  // Virtualizer for performance with large message lists
  const rowVirtualizer = useVirtualizer({
    count: filteredMessages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24, // Compact line height
    overscan: 10,
  });

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (filters.autoScroll && filteredMessages.length > 0) {
      rowVirtualizer.scrollToIndex(filteredMessages.length - 1, { align: 'end' });
    }
  }, [filters.autoScroll, filteredMessages.length, rowVirtualizer]);

  // Connection status from live store
  const { connected: isConnected } = useConnectionStatus();

  // Load initial console data and poll EULA status
  useEffect(() => {
    if (!serverId) return;
    
    const loadInitialData = async () => {
      setIsLoading(true);
      try {
        // Load initial console messages
        const consoleResponse = await api.getConsoleMessages(serverId);
        if (consoleResponse.ok && consoleResponse.data) {
          liveStore.getState().appendConsole(serverId, consoleResponse.data as any);
        }
        
        // Load EULA status
        const eulaResponse = await api.getEulaStatus(serverId);
        if (eulaResponse.ok && eulaResponse.data) {
          const status = (eulaResponse.data as any).status as 'accepted' | 'pending' | 'missing';
          setEulaStatus(status);
        }
      } catch (error) {
        console.error('Failed to load initial console data:', error);
      } finally {
        setIsLoading(false);
      }
    };
    
    loadInitialData();
    
    // Poll EULA status
    let timer: any;
    const poll = async () => {
      try {
        const res = await api.getEulaStatus(serverId);
        if (res.ok && res.data) {
          const status = (res.data as any).status as 'accepted' | 'pending' | 'missing';
          setEulaStatus(status);
        }
      } catch (error) {
        console.error('Failed to fetch EULA status:', error);
      }
      timer = setTimeout(poll, 5000);
    };
    
    // Start polling after initial load
    const pollTimer = setTimeout(poll, 5000);
    
    return () => {
      if (timer) clearTimeout(timer);
      if (pollTimer) clearTimeout(pollTimer);
    };
  }, [serverId]);

  const handleAcceptEula = async () => {
    if (!serverId) return;
    setEulaBusy(true);
    const res = await api.acceptEula(serverId);
    setEulaBusy(false);
    if (res.ok) {
      setEulaStatus('accepted');
      // Append confirmation to console
      const msg: ConsoleMessage = {
        ts: new Date().toISOString(),
        level: 'info',
        msg: 'EULA accepted. Restarting server...'
      };
      liveStore.getState().appendConsole(serverId, [msg]);
      // Auto start server if not running
      await api.startServer(serverId);
    }
  };

  const handleSendCommand = async () => {
    if (!command.trim() || !serverId) return;

    try {
      const response = await api.sendConsoleCommand(serverId, command.trim());

      if (response.ok) {
        // Add command to console via live store
        const commandMessage: ConsoleMessage = {
          ts: new Date().toISOString(),
          level: 'info',
          msg: `> ${command.trim()}`,
        };
        liveStore.getState().appendConsole(serverId, [commandMessage]);
        setCommand('');
      } else {
        console.error('Failed to send command:', response.error);
        // Add error message to console
        const errorMessage: ConsoleMessage = {
          ts: new Date().toISOString(),
          level: 'error',
          msg: `Failed to send command: ${response.error}`,
        };
        liveStore.getState().appendConsole(serverId, [errorMessage]);
      }
    } catch (error) {
      console.error('Error sending command:', error);
      // Add error message to console
      const errorMessage: ConsoleMessage = {
        ts: new Date().toISOString(),
        level: 'error',
        msg: `Error sending command: ${error instanceof Error ? error.message : 'Unknown error'}`,
      };
      liveStore.getState().appendConsole(serverId, [errorMessage]);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSendCommand();
    }
  };

  const clearMessages = () => {
    if (serverId) {
      liveStore.getState().clearConsole(serverId);
    }
  };

  const downloadLogs = () => {
    const logContent = messages
      .map(msg => `[${msg.ts}] [${msg.level.toUpperCase()}] ${msg.msg}`)
      .join('\n');
    
    const blob = new Blob([logContent], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `server-${serverId}-console-${new Date().toISOString().split('T')[0]}.log`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const getLevelIcon = (level: string) => {
    switch (level) {
      case 'error':
        return <AlertTriangle className="h-3 w-3 text-red-400" />;
      case 'warn':
        return <AlertTriangle className="h-3 w-3 text-yellow-400" />;
      case 'info':
        return <Info className="h-3 w-3 text-blue-400" />;
      case 'debug':
        return <Bug className="h-3 w-3 text-gray-400" />;
      default:
        return <CheckCircle className="h-3 w-3 text-green-400" />;
    }
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return 'text-red-400';
      case 'warn':
        return 'text-yellow-400';
      case 'info':
        return 'text-blue-400';
      case 'debug':
        return 'text-gray-400';
      default:
        return 'text-green-400';
    }
  };

  // Get virtual items for rendering
  const virtualItems = rowVirtualizer.getVirtualItems();
  const paddingTop = virtualItems.length ? virtualItems[0].start : 0;
  // const paddingBottom = virtualItems.length ? rowVirtualizer.getTotalSize() - virtualItems[virtualItems.length - 1].end : 0;

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view console</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Console Header */}
      <div className="flex items-center justify-between p-4 border-b border-border">
        <div className="flex items-center gap-4">
          <h3 className="text-lg font-semibold">Console</h3>
          <div className="flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-400' : 'bg-red-400'}`} />
            <span className="text-sm text-muted-foreground">
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setIsPaused(!isPaused)}
          >
            {isPaused ? <Play className="h-4 w-4" /> : <Pause className="h-4 w-4" />}
            {isPaused ? 'Resume' : 'Pause'}
          </Button>
          
          <Button
            size="sm"
            variant="outline"
            onClick={downloadLogs}
            disabled={messages.length === 0}
          >
            <Download className="h-4 w-4" />
            Download
          </Button>
          
          <Button
            size="sm"
            variant="outline"
            onClick={clearMessages}
            disabled={messages.length === 0}
          >
            <Trash2 className="h-4 w-4" />
            Clear
          </Button>
        </div>
      </div>

      {/* EULA Banner */}
      {eulaStatus && eulaStatus !== 'accepted' && (
        <div className="p-4 bg-yellow-500/10 border-b border-yellow-500/30 text-yellow-200 flex items-center justify-between">
          <div className="text-sm">
            {eulaStatus === 'missing' ? (
              <span>The server EULA has not been created yet. Start the server once to generate files, then accept the EULA to continue.</span>
            ) : (
              <span>You must accept the Minecraft EULA to run the server.</span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="default" onClick={handleAcceptEula} disabled={eulaBusy}>
              {eulaBusy ? 'Applying…' : 'I Agree'}
            </Button>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="flex items-center gap-4 p-4 border-b border-border bg-muted/30">
        <div className="flex items-center gap-2">
          <Filter className="h-4 w-4" />
          <span className="text-sm font-medium">Filters:</span>
        </div>
        
        <Select value={filters.level} onValueChange={(value: any) => setFilters(prev => ({ ...prev, level: value }))}>
          <SelectTrigger className="w-32">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Levels</SelectItem>
            <SelectItem value="error">Errors</SelectItem>
            <SelectItem value="warn">Warnings</SelectItem>
            <SelectItem value="info">Info</SelectItem>
            <SelectItem value="debug">Debug</SelectItem>
          </SelectContent>
        </Select>
        
        <Input
          placeholder="Search messages..."
          value={filters.search}
          onChange={(e) => setFilters(prev => ({ ...prev, search: e.target.value }))}
          className="w-64"
        />
        
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Checkbox
              id="timestamps"
              checked={filters.showTimestamps}
              onCheckedChange={(checked) => setFilters(prev => ({ ...prev, showTimestamps: !!checked }))}
            />
            <label htmlFor="timestamps" className="text-sm">Timestamps</label>
          </div>
          
          <div className="flex items-center gap-2">
            <Checkbox
              id="levels"
              checked={filters.showLevels}
              onCheckedChange={(checked) => setFilters(prev => ({ ...prev, showLevels: !!checked }))}
            />
            <label htmlFor="levels" className="text-sm">Levels</label>
          </div>
          
          <div className="flex items-center gap-2">
            <Checkbox
              id="autoscroll"
              checked={filters.autoScroll}
              onCheckedChange={(checked) => setFilters(prev => ({ ...prev, autoScroll: !!checked }))}
            />
            <label htmlFor="autoscroll" className="text-sm">Auto-scroll</label>
          </div>
        </div>
      </div>

      {/* Console Output - Virtualized */}
      <div 
        ref={parentRef}
        className="flex-1 overflow-auto bg-black text-green-400 font-mono text-sm relative"
        style={{ height: '100%' }}
      >
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span>Loading console...</span>
            </div>
          </div>
        ) : filteredMessages.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground p-4">
            No console messages to display
          </div>
        ) : (
          <div
            style={{
              height: rowVirtualizer.getTotalSize(),
              position: 'relative',
            }}
          >
            <div style={{ transform: `translateY(${paddingTop}px)` }}>
              {virtualItems.map((virtualItem) => {
                const msg = filteredMessages[virtualItem.index];
                return (
                  <div
                    key={virtualItem.key}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      height: `${virtualItem.size}px`,
                      transform: `translateY(${virtualItem.start - paddingTop}px)`,
                    }}
                    className="flex items-start gap-2 px-4 py-1 hover:bg-white/5 rounded"
                  >
                    {filters.showTimestamps && (
                      <span className="text-gray-500 text-xs whitespace-nowrap">
                        {new Date(msg.ts).toLocaleTimeString()}
                      </span>
                    )}
                    
                    {filters.showLevels && (
                      <div className="flex items-center gap-1 min-w-0">
                        {getLevelIcon(msg.level)}
                        <Badge 
                          variant="outline" 
                          className={`text-xs ${getLevelColor(msg.level)} border-current`}
                        >
                          {msg.level.toUpperCase()}
                        </Badge>
                      </div>
                    )}
                    
                    <span className="flex-1 break-words">
                      {msg.msg}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* Command Input */}
      <div className="flex items-center gap-2 p-4 border-t border-border bg-muted/30">
        <Input
          ref={commandInputRef}
          placeholder="Enter command..."
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          onKeyPress={handleKeyPress}
          className="flex-1 font-mono"
          disabled={!isConnected || server.status !== 'running'}
        />
        <Button
          onClick={handleSendCommand}
          disabled={!command.trim() || !isConnected || server.status !== 'running'}
          size="sm"
        >
          <Send className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
};

export default ConsoleStream;
