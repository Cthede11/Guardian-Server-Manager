import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
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
  CheckCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { useServersStore } from '@/store/servers';
import { useConsoleStream, useConnectionStatus, liveStore } from '@/store/live';
import type { ConsoleMessage } from '@/lib/types';

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

  const handleSendCommand = async () => {
    if (!command.trim() || !serverId) return;

    try {
      const response = await fetch(`/api/v1/servers/${serverId}/console/command`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ cmd: command.trim() }),
      });

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
        console.error('Failed to send command');
      }
    } catch (error) {
      console.error('Error sending command:', error);
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
        className="flex-1 overflow-auto bg-black text-green-400 font-mono text-sm"
        style={{ height: '100%' }}
      >
        {filteredMessages.length === 0 ? (
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
