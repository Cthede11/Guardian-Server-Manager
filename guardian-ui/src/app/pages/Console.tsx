import React, { useEffect, useRef } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
// import { ScrollArea } from '@/components/ui/scroll-area';
import { Trash2, Play } from 'lucide-react';
import { useConsoleStore } from '@/store/console';
import type { ConsoleLine } from '@/lib/types.gen';

export default function Console() {
  const { logs, addLog, clearLogs } = useConsoleStore();
  // const [isRunning] = useState(false);
  const scrollAreaRef = useRef<HTMLDivElement>(null);

  // Override console methods to capture logs
  useEffect(() => {
    const originalLog = console.log;
    const originalWarn = console.warn;
    const originalError = console.error;
    const originalInfo = console.info;

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

    // Add initial log if no logs exist
    if (logs.length === 0) {
      addLog('info', 'Console initialized - Ready for debugging');
    }

    return () => {
      console.log = originalLog;
      console.warn = originalWarn;
      console.error = originalError;
      console.info = originalInfo;
    };
  }, [addLog, logs.length]);

  // Auto-scroll to bottom when new logs are added
  useEffect(() => {
    if (scrollAreaRef.current) {
      const scrollContainer = scrollAreaRef.current.querySelector('[data-radix-scroll-area-viewport]');
      if (scrollContainer) {
        scrollContainer.scrollTop = scrollContainer.scrollHeight;
      }
    }
  }, [logs]);

  const handleClearLogs = () => {
    clearLogs();
    console.log('Console cleared');
  };

  const testAPI = async () => {
    console.log('Testing API connection...');
    try {
      // Use the API client instead of direct fetch
      const { apiClient } = await import('../../lib/api');
      
      // Test health endpoint
      const healthResponse = await apiClient.call('/api/healthz');
      console.log('API Health Check Response:', healthResponse);
      
      if (healthResponse) {
        console.log('✅ API is working correctly');
        console.log(`Backend health status: ${healthResponse}`);
      } else {
        console.error('❌ API Health Check Failed - No response');
      }
      
      // Test servers endpoint
      try {
        const servers = await apiClient.getServers();
        console.log('✅ Servers endpoint working');
        console.log(`Found ${servers?.length || 0} servers`);
        if (servers && servers.length > 0) {
          console.log('Existing servers:', servers.map((s: any) => ({ id: s.id, name: s.name, status: s.status })));
        }
      } catch (serverError) {
        console.error('❌ Servers endpoint failed:', serverError);
      }
      
      // Test Java detection endpoint
      try {
        const javaResponse = await apiClient.call('/api/server/detect-java');
        console.log('✅ Java detection endpoint working');
        console.log('Java detection response:', javaResponse);
      } catch (javaError) {
        console.error('❌ Java detection endpoint failed:', javaError);
      }
      
    } catch (error) {
      console.error('❌ API test failed:', error);
      console.error('Error details:', {
        name: error instanceof Error ? error.name : 'Unknown',
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined
      });
    }
  };

  const testServerCreation = async () => {
    console.log('Testing server creation with updated API...');
    try {
      // Test data using the correct structure that matches the backend expectations
      const serverData = {
        name: 'Test Server from Console',
        loader: 'vanilla',
        version: '1.21.1',
        minecraft_version: '1.21.1', // Backend expects minecraft_version, not mc_version
        paths: {
          world: './world',
          mods: './mods',
          config: './config',
          java_path: 'java' // Added java_path with fallback
        },
        max_players: 20,
        memory: 4096, // Memory in MB (4GB)
        world_settings: {
          world_name: 'Test Server from Console',
          difficulty: 'normal',
          gamemode: 'survival'
        },
        // Optional fields that the backend supports
        port: 25565,
        rcon_port: 25575,
        query_port: 25566,
        auto_start: false,
        auto_restart: true,
        pvp: true,
        online_mode: true,
        whitelist: false,
        enable_command_block: false,
        view_distance: 10,
        simulation_distance: 10,
        motd: 'A Test Minecraft Server'
      };

      console.log('Sending server creation request:', serverData);
      
      // Use the updated API client instead of direct fetch
      const { apiClient } = await import('../../lib/api');
      
      try {
        const newServer = await apiClient.createServer(serverData);
        console.log('✅ Server created successfully!');
        console.log('Server details:', newServer);
        console.log('Server ID:', newServer?.id);
        console.log('Server name:', newServer?.name);
        console.log('Server status:', newServer?.status);
        console.log('Server version:', newServer?.version);
        console.log('Server memory:', newServer?.memory_usage, 'MB');
      } catch (apiError) {
        console.error('❌ API client error:', apiError);
        
        // Fallback to direct fetch for debugging
        console.log('Falling back to direct fetch for debugging...');
        const { getAPI_BASE } = await import('../../lib/api');
        const base = await getAPI_BASE();
        
        const response = await fetch(`${base}/api/servers`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(serverData)
        });

        console.log('Direct fetch response status:', response.status, response.statusText);
        
        if (!response.ok) {
          const errorText = await response.text();
          console.error(`❌ Direct fetch failed with HTTP ${response.status} ${response.statusText}`);
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
        } else {
          const data = await response.json();
          console.log('Direct fetch response data:', data);
          
          if (data.success) {
            console.log('✅ Server created successfully via direct fetch!');
            console.log('Server ID:', data.data?.id);
            console.log('Server name:', data.data?.name);
            console.log('Server status:', data.data?.status);
          } else {
            console.error('❌ Server creation failed:', data.error);
          }
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

  const getLevelColor = (level: ConsoleLine['level']) => {
    switch (level) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'info': return 'text-blue-400';
      case 'debug': return 'text-gray-400';
      default: return 'text-white';
    }
  };

  const getLevelBadge = (level: ConsoleLine['level']) => {
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
                onClick={handleClearLogs}
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
