import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { 
  Send, 
  Filter, 
  Download, 
  Trash2, 
  Play, 
  Pause, 
  Square,
  AlertTriangle,
  Info,
  Bug,
  CheckCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { useServersStore } from '@/store/servers';
import { ConsoleMessage } from '@/lib/types';

interface ConsoleStreamProps {
  className?: string;
}

export const ConsoleStream: React.FC<ConsoleStreamProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [messages, setMessages] = useState<ConsoleMessage[]>([]);
  const [command, setCommand] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [filters, setFilters] = useState({
    level: 'all' as 'all' | 'info' | 'warn' | 'error' | 'debug',
    search: '',
    showTimestamps: true,
    showLevels: true,
    autoScroll: true
  });
  
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const commandInputRef = useRef<HTMLInputElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const sseRef = useRef<EventSource | null>(null);

  // Auto-scroll to bottom when new messages arrive
  const scrollToBottom = useCallback(() => {
    if (filters.autoScroll && messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [filters.autoScroll]);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Connect to console stream
  useEffect(() => {
    if (!serverId || !server) return;

    const connectToStream = () => {
      // Try WebSocket first
      try {
        const ws = new WebSocket(`ws://localhost:8080/api/v1/servers/${serverId}/console/stream`);
        
        ws.onopen = () => {
          console.log('WebSocket connected');
          setIsConnected(true);
          wsRef.current = ws;
        };

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            const message: ConsoleMessage = {
              id: Date.now() + Math.random(),
              timestamp: new Date(data.ts || Date.now()),
              level: data.level || 'info',
              message: data.msg || data.message || '',
              serverId: serverId
            };
            
            if (!isPaused) {
              setMessages(prev => [...prev.slice(-999), message]); // Keep last 1000 messages
            }
          } catch (error) {
            console.error('Error parsing WebSocket message:', error);
          }
        };

        ws.onclose = () => {
          console.log('WebSocket disconnected, trying SSE fallback');
          setIsConnected(false);
          wsRef.current = null;
          
          // Fallback to SSE
          trySSE();
        };

        ws.onerror = (error) => {
          console.error('WebSocket error:', error);
          setIsConnected(false);
          wsRef.current = null;
          trySSE();
        };

      } catch (error) {
        console.error('WebSocket connection failed:', error);
        trySSE();
      }
    };

    const trySSE = () => {
      try {
        const sse = new EventSource(`http://localhost:8080/api/v1/servers/${serverId}/console/stream`);
        
        sse.onopen = () => {
          console.log('SSE connected');
          setIsConnected(true);
          sseRef.current = sse;
        };

        sse.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            const message: ConsoleMessage = {
              id: Date.now() + Math.random(),
              timestamp: new Date(data.ts || Date.now()),
              level: data.level || 'info',
              message: data.msg || data.message || '',
              serverId: serverId
            };
            
            if (!isPaused) {
              setMessages(prev => [...prev.slice(-999), message]);
            }
          } catch (error) {
            console.error('Error parsing SSE message:', error);
          }
        };

        sse.onerror = (error) => {
          console.error('SSE error:', error);
          setIsConnected(false);
          sseRef.current = null;
        };

      } catch (error) {
        console.error('SSE connection failed:', error);
        setIsConnected(false);
      }
    };

    connectToStream();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      if (sseRef.current) {
        sseRef.current.close();
        sseRef.current = null;
      }
      setIsConnected(false);
    };
  }, [serverId, server, isPaused]);

  // Simulate console messages for demo
  useEffect(() => {
    if (!server || server.status !== 'running') return;

    const interval = setInterval(() => {
      const demoMessages = [
        { level: 'info', message: 'Player Steve joined the game' },
        { level: 'info', message: 'Player Alex joined the game' },
        { level: 'warn', message: 'Can\'t keep up! Is the server overloaded? Running 2000ms or 100 ticks behind' },
        { level: 'info', message: 'Player Steve left the game' },
        { level: 'error', message: 'Exception in thread "Server thread" java.lang.NullPointerException' },
        { level: 'debug', message: 'Chunk [0, 0] in world minecraft:overworld loaded' },
        { level: 'info', message: 'Saving chunks for level \'ServerLevel[world]\'' },
        { level: 'info', message: 'Saving chunks for level \'ServerLevel[world_nether]\'' },
        { level: 'info', message: 'Saving chunks for level \'ServerLevel[world_the_end]\'' }
      ];

      const randomMessage = demoMessages[Math.floor(Math.random() * demoMessages.length)];
      const message: ConsoleMessage = {
        id: Date.now() + Math.random(),
        timestamp: new Date(),
        level: randomMessage.level as any,
        message: randomMessage.message,
        serverId: serverId || ''
      };

      if (!isPaused) {
        setMessages(prev => [...prev.slice(-999), message]);
      }
    }, 3000 + Math.random() * 2000); // Random interval between 3-5 seconds

    return () => clearInterval(interval);
  }, [server, serverId, isPaused]);

  const handleSendCommand = async () => {
    if (!command.trim() || !serverId) return;

    try {
      const response = await fetch(`http://localhost:8080/api/v1/servers/${serverId}/console/command`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ cmd: command.trim() }),
      });

      if (response.ok) {
        // Add command to console
        const commandMessage: ConsoleMessage = {
          id: Date.now() + Math.random(),
          timestamp: new Date(),
          level: 'info',
          message: `> ${command.trim()}`,
          serverId: serverId
        };
        setMessages(prev => [...prev, commandMessage]);
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
    setMessages([]);
  };

  const downloadLogs = () => {
    const logContent = messages
      .map(msg => `[${msg.timestamp.toISOString()}] [${msg.level.toUpperCase()}] ${msg.message}`)
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

  const filteredMessages = messages.filter(msg => {
    if (filters.level !== 'all' && msg.level !== filters.level) return false;
    if (filters.search && !msg.message.toLowerCase().includes(filters.search.toLowerCase())) return false;
    return true;
  });

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

      {/* Console Output */}
      <div className="flex-1 overflow-auto bg-black text-green-400 font-mono text-sm p-4">
        {filteredMessages.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            No console messages to display
          </div>
        ) : (
          filteredMessages.map((msg) => (
            <div key={msg.id} className="flex items-start gap-2 py-1 hover:bg-white/5 rounded">
              {filters.showTimestamps && (
                <span className="text-gray-500 text-xs whitespace-nowrap">
                  {msg.timestamp.toLocaleTimeString()}
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
                {msg.message}
              </span>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
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
