import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Server, 
  CheckCircle, 
  AlertTriangle, 
  Loader2,
  X,
  Search,
  Filter,
  Package,
  Download
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { useServers } from '@/store/servers-new';
import { useToast } from '@/hooks/use-toast';
import { apiClient as api } from '@/lib/api';

interface ServerInfo {
  id: string;
  name: string;
  status: string;
  version: string;
  loader: string;
  memory_usage: number;
  players_online: number;
  created_at: string;
}

interface ServerSelectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onInstall: (serverId: string) => Promise<void>;
  title: string;
  description: string;
  itemType: 'mod' | 'modpack';
  itemName: string;
  itemVersion: string;
  itemSource: 'curseforge' | 'modrinth';
}

export const ServerSelectionModal: React.FC<ServerSelectionModalProps> = ({
  isOpen,
  onClose,
  onInstall,
  title,
  description,
  itemType,
  itemName,
  itemVersion,
  itemSource
}) => {
  const [servers, setServers] = useState<ServerInfo[]>([]);
  const [filteredServers, setFilteredServers] = useState<ServerInfo[]>([]);
  const [selectedServer, setSelectedServer] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [compatibilityChecks, setCompatibilityChecks] = useState<Record<string, {
    compatible: boolean;
    warnings: string[];
    errors: string[];
  }>>({});

  const { summaries, getServerById } = useServers();
  const serverStore = Object.values(summaries);
  const { toast } = useToast();

  // Load servers
  useEffect(() => {
    if (isOpen) {
      loadServers();
    }
  }, [isOpen]);

  // Filter servers based on search
  useEffect(() => {
    if (!searchQuery.trim()) {
      setFilteredServers(servers);
    } else {
      setFilteredServers(servers.filter(server =>
        server.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        server.version.toLowerCase().includes(searchQuery.toLowerCase()) ||
        server.loader.toLowerCase().includes(searchQuery.toLowerCase())
      ));
    }
  }, [servers, searchQuery]);

  const loadServers = async () => {
    setIsLoading(true);
    try {
      const response = await api.call<{
        success: boolean;
        data: ServerInfo[];
        error: string;
      }>('/api/servers');

      if (response.success && response.data) {
        setServers(response.data);
        // Check compatibility for each server
        checkCompatibility(response.data);
      } else {
        console.error('Failed to load servers:', response.error);
        toast({
          title: "Error",
          description: "Failed to load servers. Please try again.",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Error loading servers:', error);
      toast({
        title: "Error",
        description: "An error occurred while loading servers.",
        variant: "destructive"
      });
    } finally {
      setIsLoading(false);
    }
  };

  const checkCompatibility = async (servers: ServerInfo[]) => {
    const checks: Record<string, { compatible: boolean; warnings: string[]; errors: string[] }> = {};
    
    for (const server of servers) {
      try {
        const response = await api.call<{
          success: boolean;
          data: {
            compatible: boolean;
            warnings: string[];
            errors: string[];
          };
          error: string;
        }>(`/api/servers/${server.id}/compatibility`, {
          method: 'POST',
          body: JSON.stringify({
            item_type: itemType,
            item_name: itemName,
            item_version: itemVersion,
            item_source: itemSource
          })
        });

        if (response.success && response.data) {
          checks[server.id] = response.data;
        } else {
          checks[server.id] = {
            compatible: true, // Assume compatible if check fails
            warnings: ['Compatibility check failed'],
            errors: []
          };
        }
      } catch (error) {
        console.error(`Compatibility check failed for server ${server.id}:`, error);
        checks[server.id] = {
          compatible: true, // Assume compatible if check fails
          warnings: ['Compatibility check failed'],
          errors: []
        };
      }
    }

    setCompatibilityChecks(checks);
  };

  const handleInstall = async () => {
    if (!selectedServer) return;

    setIsInstalling(true);
    try {
      await onInstall(selectedServer);
      toast({
        title: "Success",
        description: `${itemType === 'mod' ? 'Mod' : 'Modpack'} installed successfully!`,
        variant: "default"
      });
      onClose();
    } catch (error) {
      console.error('Installation failed:', error);
      toast({
        title: "Installation Failed",
        description: `Failed to install ${itemType}. Please try again.`,
        variant: "destructive"
      });
    } finally {
      setIsInstalling(false);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'running':
        return 'text-green-600 bg-green-100';
      case 'stopped':
        return 'text-gray-600 bg-gray-100';
      case 'starting':
        return 'text-yellow-600 bg-yellow-100';
      case 'stopping':
        return 'text-orange-600 bg-orange-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  const getCompatibilityStatus = (serverId: string) => {
    const check = compatibilityChecks[serverId];
    if (!check) return { status: 'checking', color: 'text-gray-600 bg-gray-100' };
    
    if (check.errors.length > 0) {
      return { status: 'incompatible', color: 'text-red-600 bg-red-100' };
    } else if (check.warnings.length > 0) {
      return { status: 'warning', color: 'text-yellow-600 bg-yellow-100' };
    } else {
      return { status: 'compatible', color: 'text-green-600 bg-green-100' };
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center space-x-2">
            <Package className="h-5 w-5" />
            <span>{title}</span>
          </DialogTitle>
          <p className="text-muted-foreground">{description}</p>
        </DialogHeader>

        <div className="space-y-6">
          {/* Item Info */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">{itemName}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center space-x-4 text-sm text-muted-foreground">
                <span>Version: {itemVersion}</span>
                <Badge variant={itemSource === 'curseforge' ? 'default' : 'secondary'}>
                  {itemSource}
                </Badge>
                <Badge variant="outline">
                  {itemType === 'mod' ? 'Mod' : 'Modpack'}
                </Badge>
              </div>
            </CardContent>
          </Card>

          {/* Search */}
          <div className="flex items-center space-x-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search servers..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="flex-1"
            />
          </div>

          {/* Servers List */}
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin" />
              <span className="ml-2">Loading servers...</span>
            </div>
          ) : filteredServers.length === 0 ? (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <Server className="h-12 w-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">No servers found</h3>
                <p className="text-muted-foreground text-center">
                  {searchQuery ? 'No servers match your search criteria.' : 'No servers available.'}
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {filteredServers.map((server) => {
                const compatibility = getCompatibilityStatus(server.id);
                const isSelected = selectedServer === server.id;
                
                return (
                  <Card
                    key={server.id}
                    className={`cursor-pointer transition-all ${
                      isSelected ? 'ring-2 ring-primary' : 'hover:shadow-md'
                    } ${!compatibilityChecks[server.id]?.compatible ? 'opacity-50' : ''}`}
                    onClick={() => {
                      if (compatibilityChecks[server.id]?.compatible) {
                        setSelectedServer(server.id);
                      }
                    }}
                  >
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <CardTitle className="text-lg">{server.name}</CardTitle>
                          <p className="text-sm text-muted-foreground">
                            {server.loader} • {server.version}
                          </p>
                        </div>
                        <div className="flex items-center space-x-2">
                          <Badge className={getStatusColor(server.status)}>
                            {server.status}
                          </Badge>
                          <Badge className={compatibility.color}>
                            {compatibility.status}
                          </Badge>
                        </div>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <span className="text-muted-foreground">Memory:</span>
                          <span className="ml-2">{server.memory_usage}MB</span>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Players:</span>
                          <span className="ml-2">{server.players_online}</span>
                        </div>
                      </div>

                      {/* Compatibility Warnings/Errors */}
                      {compatibilityChecks[server.id] && (
                        <div className="space-y-2">
                          {compatibilityChecks[server.id].errors.length > 0 && (
                            <Alert variant="destructive">
                              <AlertTriangle className="h-4 w-4" />
                              <AlertDescription>
                                <div className="space-y-1">
                                  {compatibilityChecks[server.id].errors.map((error, index) => (
                                    <div key={index} className="text-sm">{error}</div>
                                  ))}
                                </div>
                              </AlertDescription>
                            </Alert>
                          )}
                          
                          {compatibilityChecks[server.id].warnings.length > 0 && (
                            <Alert>
                              <AlertTriangle className="h-4 w-4" />
                              <AlertDescription>
                                <div className="space-y-1">
                                  {compatibilityChecks[server.id].warnings.map((warning, index) => (
                                    <div key={index} className="text-sm">{warning}</div>
                                  ))}
                                </div>
                              </AlertDescription>
                            </Alert>
                          )}
                        </div>
                      )}

                      {isSelected && (
                        <div className="flex items-center space-x-2 text-green-600">
                          <CheckCircle className="h-4 w-4" />
                          <span className="text-sm font-medium">Selected</span>
                        </div>
                      )}
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          )}

          {/* Actions */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="text-sm text-muted-foreground">
              {selectedServer ? `Selected: ${servers.find(s => s.id === selectedServer)?.name}` : 'No server selected'}
            </div>
            <div className="flex items-center space-x-2">
              <Button variant="outline" onClick={onClose}>
                Cancel
              </Button>
              <Button
                onClick={handleInstall}
                disabled={!selectedServer || isInstalling}
                className="flex items-center space-x-2"
              >
                {isInstalling ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span>Installing...</span>
                  </>
                ) : (
                  <>
                    <Download className="h-4 w-4" />
                    <span>Install {itemType === 'mod' ? 'Mod' : 'Modpack'}</span>
                  </>
                )}
              </Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
