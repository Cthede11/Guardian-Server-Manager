import React, { useState, useEffect, useRef } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
// import { ScrollArea } from '@/components/ui/scroll-area';
import { Trash2, Play } from 'lucide-react';

interface LogEntry {
  id: string;
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
  data?: any;
}

export default function Console() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  // const [isRunning] = useState(false);
  const scrollAreaRef = useRef<HTMLDivElement>(null);
  const logIdRef = useRef(0);

  // Override console methods to capture logs
  useEffect(() => {
    const originalLog = console.log;
    const originalWarn = console.warn;
    const originalError = console.error;
    const originalInfo = console.info;

    const addLog = (level: LogEntry['level'], ...args: any[]) => {
      const id = `log-${++logIdRef.current}`;
      const timestamp = new Date().toLocaleTimeString();
      const message = args.map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
      ).join(' ');
      
      setLogs(prev => [...prev, { id, timestamp, level, message, data: args }]);
    };

    console.log = (...args) => {
      originalLog(...args);
      addLog('info', ...args);
    };

    console.warn = (...args) => {
      originalWarn(...args);
      addLog('warn', ...args);
    };

    console.error = (...args) => {
      originalError(...args);
      addLog('error', ...args);
    };

    console.info = (...args) => {
      originalInfo(...args);
      addLog('info', ...args);
    };

    // Add initial log
    addLog('info', 'Console initialized - Ready for debugging');

    return () => {
      console.log = originalLog;
      console.warn = originalWarn;
      console.error = originalError;
      console.info = originalInfo;
    };
  }, []);

  // Auto-scroll to bottom when new logs are added
  useEffect(() => {
    if (scrollAreaRef.current) {
      const scrollContainer = scrollAreaRef.current.querySelector('[data-radix-scroll-area-viewport]');
      if (scrollContainer) {
        scrollContainer.scrollTop = scrollContainer.scrollHeight;
      }
    }
  }, [logs]);

  const clearLogs = () => {
    setLogs([]);
    console.log('Console cleared');
  };

  const testAPI = async () => {
    console.log('Testing API connection...');
    try {
      // Use the API client instead of direct fetch
      const { api } = await import('../../lib/api');
      const { isSuccessResponse, logApiError, getErrorMessageWithSuggestions } = await import('../../lib/api-response-handler');
      
      const response = await api('/healthz') as { success: boolean; data: string; error?: string; timestamp: string };
      console.log('API Health Check Response:', response);
      
      if (isSuccessResponse(response)) {
        console.log('✅ API is working correctly');
        console.log(`Backend health status: ${response.data}`);
        console.log(`Response timestamp: ${response.timestamp}`);
      } else {
        const errorMessage = getErrorMessageWithSuggestions(response);
        console.error('❌ API Health Check Failed');
        console.error(errorMessage);
      }
    } catch (error) {
      const { logApiError, getErrorMessageWithSuggestions } = await import('../../lib/api-response-handler');
      logApiError(error, 'API Health Check Failed');
      console.error(getErrorMessageWithSuggestions(error));
    }
  };

  const testServerCreation = async () => {
    console.log('Testing server creation...');
    try {
      const serverData = {
        name: 'Test Server from Console',
        loader: 'vanilla',
        version: '1.21.1',
        mc_version: '1.21.1', // Backend expects both version and mc_version
        paths: {
          world: './world',
          mods: './mods',
          config: './config'
        },
        pregeneration_policy: {
          enabled: false,
          radius: 0,
          dimensions: ["overworld"],
          gpu_acceleration: true,
          efficiency_package: false
        }
      };

      console.log('Sending server creation request:', serverData);
      
      // Use dynamic API base discovery instead of hardcoded localhost:8080
      const { getAPI_BASE } = await import('../../lib/api');
      const base = await getAPI_BASE();
      
      const response = await fetch(`${base}/api/servers`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(serverData)
      });

      console.log('Server creation response status:', response.status, response.statusText);
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error(`❌ Server creation failed with HTTP ${response.status} ${response.statusText}`);
        console.error('Response body:', errorText);
        
        try {
          const errorData = JSON.parse(errorText);
          if (errorData.error) {
            console.error('Error message:', errorData.error);
          }
          if (errorData.details) {
            console.error('Error details:', errorData.details);
          }
        } catch (e) {
          console.error('Could not parse error response as JSON');
        }
        return;
      }

      const data = await response.json();
      console.log('Server creation response data:', data);
      
      if (data.success) {
        console.log('✅ Server created successfully!');
        console.log('Server ID:', data.data.id);
        console.log('Server Name:', data.data.name);
        console.log('Server Version:', data.data.version);
        console.log('Server Status:', data.data.status);
      } else {
        console.error('❌ Server creation failed');
        console.error('Error:', data.error || 'Unknown error');
        if (data.details) {
          console.error('Details:', data.details);
        }
      }
    } catch (error) {
      console.error('❌ Server creation error:', error);
      console.error('Error details:', {
        name: error instanceof Error ? error.name : 'Unknown',
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined
      });
    }
  };

  const getLevelColor = (level: LogEntry['level']) => {
    switch (level) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'info': return 'text-blue-400';
      case 'debug': return 'text-gray-400';
      default: return 'text-white';
    }
  };

  const getLevelBadge = (level: LogEntry['level']) => {
    switch (level) {
      case 'error': return <Badge variant="destructive">ERROR</Badge>;
      case 'warn': return <Badge variant="outline" className="text-yellow-400 border-yellow-400">WARN</Badge>;
      case 'info': return <Badge variant="outline" className="text-blue-400 border-blue-400">INFO</Badge>;
      case 'debug': return <Badge variant="outline" className="text-gray-400 border-gray-400">DEBUG</Badge>;
      default: return <Badge variant="outline">LOG</Badge>;
    }
  };

  return (
    <div className="h-full flex flex-col">
      <Card className="flex-1 flex flex-col">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              Live Console
            </CardTitle>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={testAPI}
                className="flex items-center gap-2"
              >
                <Play className="w-4 h-4" />
                Test API
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={testServerCreation}
                className="flex items-center gap-2"
              >
                <Play className="w-4 h-4" />
                Test Server Creation
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={clearLogs}
                className="flex items-center gap-2"
              >
                <Trash2 className="w-4 h-4" />
                Clear
              </Button>
            </div>
          </div>
        </CardHeader>
        
        <CardContent className="flex-1 p-0">
          <div ref={scrollAreaRef} className="h-full overflow-y-auto">
            <div className="p-4 space-y-2 font-mono text-sm">
              {logs.length === 0 ? (
                <div className="text-muted-foreground text-center py-8">
                  No logs yet. Try creating a server or testing the API.
                </div>
              ) : (
                logs.map((log) => (
                  <div key={log.id} className="flex items-start gap-3 py-1">
                    <div className="flex items-center gap-2 min-w-0 flex-shrink-0">
                      <span className="text-muted-foreground text-xs">
                        {log.timestamp}
                      </span>
                      {getLevelBadge(log.level)}
                    </div>
                    <div className={`flex-1 min-w-0 ${getLevelColor(log.level)}`}>
                      <div className="whitespace-pre-wrap break-words">
                        {log.message}
                      </div>
                      {log.data && log.data.length > 1 && (
                        <details className="mt-1">
                          <summary className="text-xs text-muted-foreground cursor-pointer">
                            Raw data
                          </summary>
                          <pre className="text-xs text-muted-foreground mt-1 p-2 bg-muted rounded">
                            {JSON.stringify(log.data, null, 2)}
                          </pre>
                        </details>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
