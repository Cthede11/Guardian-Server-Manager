import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Trash2, Send } from 'lucide-react';
import { useSelectedServer } from '@/store/servers-new';
import { useConsoleStream, useLive } from '@/store/live-new';
import { useServerStreams } from '@/app/hooks/useServerStreams';
import { api } from '@/lib/client';
import ConsoleStreamNew from '@/components/Console/ConsoleStreamNew';

export default function ConsoleNew() {
  const [command, setCommand] = useState('');
  const [isRunning, setIsRunning] = useState(false);
  const selectedServer = useSelectedServer();
  const consoleLines = useConsoleStream(selectedServer?.id || '');
  const clearConsole = useLive((state) => state.clearConsole);

  // Attach streams for the selected server
  useServerStreams(selectedServer?.id);

  // Check if server is running
  useEffect(() => {
    if (selectedServer) {
      setIsRunning(selectedServer.status === 'running');
    }
  }, [selectedServer]);

  const handleSendCommand = async () => {
    if (!selectedServer || !command.trim()) return;

    try {
      await api.sendRcon(selectedServer.id, command.trim());
      setCommand('');
    } catch (error) {
      console.error('Failed to send command:', error);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendCommand();
    }
  };

  const handleClearConsole = () => {
    if (selectedServer) {
      clearConsole(selectedServer.id);
    }
  };

  if (!selectedServer) {
    return (
      <div className="h-full flex items-center justify-center">
        <Card className="w-96">
          <CardContent className="pt-6">
            <div className="text-center">
              <h3 className="text-lg font-semibold mb-2">No Server Selected</h3>
              <p className="text-muted-foreground">
                Please select a server from the sidebar to view its console.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <Card className="flex-1 flex flex-col">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${
                isRunning ? 'bg-green-500 animate-pulse' : 'bg-red-500'
              }`}></div>
              Console - {selectedServer.name}
              <span className="text-sm text-muted-foreground">
                ({selectedServer.status})
              </span>
            </CardTitle>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleClearConsole}
                className="flex items-center gap-2"
              >
                <Trash2 className="w-4 h-4" />
                Clear
              </Button>
            </div>
          </div>
        </CardHeader>
        
        <CardContent className="flex-1 flex flex-col p-0">
          {/* Console Output */}
          <div className="flex-1 min-h-0">
            <ConsoleStreamNew serverId={selectedServer.id} />
          </div>
          
          {/* Command Input */}
          <div className="border-t p-4">
            <div className="flex gap-2">
              <Input
                value={command}
                onChange={(e) => setCommand(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder={isRunning ? "Enter command..." : "Server is not running"}
                disabled={!isRunning}
                className="font-mono"
              />
              <Button
                onClick={handleSendCommand}
                disabled={!isRunning || !command.trim()}
                className="flex items-center gap-2"
              >
                <Send className="w-4 h-4" />
                Send
              </Button>
            </div>
            {!isRunning && (
              <p className="text-sm text-muted-foreground mt-2">
                Start the server to send commands.
              </p>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
