import React, { useState, useEffect } from 'react';
import { Link, useParams, useLocation, useNavigate } from 'react-router-dom';
import { 
  Search, 
  Plus, 
  Server, 
  Users, 
  MoreVertical, 
  Copy, 
  Trash2, 
  Database,
  Key,
  Palette,
  Building2,
  Play,
  Square,
  ArrowUp,
  Folder,
  Package
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useServers } from '@/store/servers-new';
import { openDevTools, openServerFolder, ensureBackend } from '@/lib/tauri-api';
import { StatusPill } from '@/components/StatusPill';
import { getVersionsForModpack } from '@/lib/constants/minecraft-versions';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { ServerCreationWizard } from '@/components/ServerCreationWizard';

// Add Server Wizard Component - now using the comprehensive ServerCreationWizard
const AddServerWizard: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const { fetchServers } = useServers();
  const navigate = useNavigate();

  const handleServerCreated = async (createdServer: any) => {
    try {
      console.log('Server created successfully:', createdServer);
      
      // Refresh the servers list to show the new server
      await fetchServers();
      
      // Close the wizard
      onClose();
      
      // Navigate to the new server if it has an ID
      if (createdServer?.id) {
        navigate(`/servers/${createdServer.id}`);
      }
    } catch (error) {
      console.error('Error handling server creation:', error);
      alert(`Error handling server creation: ${error instanceof Error ? error.message : 'Unknown error occurred'}`);
    }
  };

  return (
    <ServerCreationWizard 
      open={true}
      onOpenChange={() => onClose()}
      onClose={onClose} 
      onServerCreated={handleServerCreated} 
    />
  );
};

export const Sidebar: React.FC = () => {
  const { summaries, selectedId, select, loading, startServer, stopServer, promoteServer, deleteServer, fetchServers } = useServers();
  const servers = Object.values(summaries);
  const selectedServerId = selectedId;
  const selectServer = select;
  const { id: currentServerId } = useParams<{ id: string }>();
  const location = useLocation();
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState('');
  const [showAddServer, setShowAddServer] = useState(false);
  const [workspaceExpanded, setWorkspaceExpanded] = useState(false);

  // Filter servers based on search query
  const filteredServers = servers.filter(server =>
    server.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  // Set selected server when route changes
  useEffect(() => {
    if (currentServerId && currentServerId !== selectedServerId) {
      selectServer(currentServerId);
    }
  }, [currentServerId, selectedServerId, selectServer]);

  // Ensure backend then load servers on mount
  useEffect(() => {
    (async () => {
      try {
        await ensureBackend();
      } catch (e) {
        console.error('Failed to ensure backend:', e);
      }
      const { fetchServers } = useServers.getState();
      fetchServers();
    })();
  }, []);

  const handleContextMenu = (e: React.MouseEvent, _serverId: string) => {
    e.preventDefault();
    // Context menu will be handled by the dropdown menu
  };

  const handleServerAction = async (action: string, serverId: string) => {
    switch (action) {
      case 'start':
        await startServer(serverId);
        break;
      case 'stop':
        await stopServer(serverId);
        break;
      case 'promote':
        await promoteServer(serverId);
        break;
      case 'clone':
        const server = servers.find(s => s.id === serverId);
        if (server) {
          const newName = prompt('Enter name for cloned server:', `${server.name} (Copy)`);
          if (newName) {
            // Clone functionality would be implemented here
            console.log('Cloning server:', serverId, 'to:', newName);
          }
        }
        break;
      case 'delete':
        if (confirm('Are you sure you want to delete this server?')) {
          console.log('Deleting server:', serverId);
          const success = await deleteServer(serverId);
          if (success) {
            // Server is already removed from local state by deleteServer
            // Navigate to servers list if we're on the deleted server
            if (currentServerId === serverId) {
              navigate('/servers');
            }
          }
        }
        break;
      case 'open-folder':
        try {
          console.log('Attempting to open folder for server:', serverId);
          await openServerFolder(serverId);
          console.log('Successfully opened folder for server:', serverId);
        } catch (error) {
          console.error('Failed to open server folder:', error);
          // Show user-friendly error message
          alert(`Failed to open server folder: ${error}`);
        }
        break;
    }
  };

  return (
    <TooltipProvider delayDuration={500}>
      <nav className="w-72 bg-card border-r border-border flex flex-col shadow-xl" role="navigation">
      {/* Header */}
      <div className="p-6 border-b border-border/30">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-br from-primary via-primary/80 to-secondary rounded-xl flex items-center justify-center shadow-lg ring-2 ring-primary/20">
              <span className="text-white text-lg font-bold">G</span>
            </div>
            <h1 className="text-xl font-bold text-foreground tracking-tight">Guardian</h1>
          </div>
          <Dialog open={showAddServer} onOpenChange={setShowAddServer}>
            <DialogTrigger asChild>
              <Button size="sm" className="bg-primary hover:bg-primary/90 text-primary-foreground shadow-md hover:shadow-lg transition-all duration-200">
                <Plus className="h-4 w-4 mr-2" />
                Add Server
              </Button>
            </DialogTrigger>
            <DialogContent className="modern-card">
              <DialogHeader>
                <DialogTitle>Add New Server</DialogTitle>
              </DialogHeader>
              <AddServerWizard onClose={() => setShowAddServer(false)} />
            </DialogContent>
          </Dialog>
        </div>

        {/* Search */}
                  <div className="relative group">
            <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors duration-200" />
            <Input
              placeholder="Search servers..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-12 pr-4 py-3 bg-background/60 border-border/40 focus:bg-background focus:border-primary/60 focus:ring-2 focus:ring-primary/20 transition-all duration-300 rounded-xl shadow-sm hover:shadow-md focus:shadow-lg"
            />
          </div>
      </div>

      {/* Navigation */}
      <div className="flex-1 overflow-y-auto">
        {/* Servers Section */}
        <div className="p-4">
          <div className="flex items-center gap-3 mb-4 px-2">
            <div className="w-8 h-8 bg-gradient-to-br from-primary/20 to-secondary/20 rounded-lg flex items-center justify-center border border-primary/20">
              <Server className="h-4 w-4 text-primary" />
            </div>
            <div className="flex-1">
              <h3 className="text-sm font-semibold text-foreground">Servers</h3>
              <p className="text-xs text-muted-foreground">Manage your Minecraft servers</p>
            </div>
            <Badge variant="secondary" className="bg-primary/15 text-primary border-primary/20">
              {servers.length}
            </Badge>
          </div>
          
          {loading ? (
            <div className="space-y-2">
              {[...Array(3)].map((_, i) => (
                <div key={i} className="server-card animate-pulse">
                  <div className="h-4 bg-muted rounded w-3/4 mb-2"></div>
                  <div className="h-3 bg-muted rounded w-1/2"></div>
                </div>
              ))}
            </div>
          ) : (
                         <div className="space-y-2">
                                     {filteredServers.map((server) => (
                        <div
                          key={server.id}
                          className={`server-card group ${
                            selectedServerId === server.id ? 'selected' : ''
                          }`}
                          onContextMenu={(e) => handleContextMenu(e, server.id)}
                        >
                          <div className="flex items-center justify-between">
                            <Link
                              to={`/servers/${server.id}/overview`}
                              className="flex-1 min-w-0"
                            >
                              <div className="flex items-center gap-4">
                                <div className="server-icon">
                                  <Server className="h-4 w-4 text-primary" />
                                </div>
                                <div className="min-w-0 flex-1">
                                  <h3 className="server-name">
                                    {server.name}
                                  </h3>
                                  <div className="server-meta">
                                    <StatusPill status={server.status as "stopped" | "starting" | "running" | "stopping"} />
                                    <span className="text-xs text-muted-foreground font-medium">
                                      {server.players_online} players
                                    </span>
                                  </div>
                                </div>
                              </div>
                            </Link>
                     
                     <div className="flex items-center gap-1">
                        {server.blueGreen && (
                         <div className={`w-2 h-2 rounded-full ${
                           server.blue_green.active === 'blue' ? 'bg-blue-500' : 'bg-green-500'
                         }`} />
                       )}
                       
                       <DropdownMenu>
                         <DropdownMenuTrigger asChild>
                           <Button
                             variant="ghost"
                             size="sm"
                             className="h-7 w-7 p-0 hover:bg-accent/50"
                           >
                             <MoreVertical className="h-3 w-3" />
                           </Button>
                         </DropdownMenuTrigger>
                         <DropdownMenuContent align="end" className="w-56 p-1 bg-card border border-border shadow-xl">
                           <Tooltip>
                             <TooltipTrigger asChild>
                               <DropdownMenuItem 
                                 onClick={() => handleServerAction('start', server.id)}
                                 className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-muted/60 hover:text-foreground focus:bg-muted/60 focus:text-foreground"
                               >
                                 <Play className="h-4 w-4 text-muted-foreground" />
                                 <span>Start Server</span>
                               </DropdownMenuItem>
                             </TooltipTrigger>
                             <TooltipContent side="left" className="bg-card text-foreground border border-border shadow-lg backdrop-blur-sm">
                               <p>Start the Minecraft server</p>
                             </TooltipContent>
                           </Tooltip>

                           <Tooltip>
                             <TooltipTrigger asChild>
                               <DropdownMenuItem 
                                 onClick={() => handleServerAction('stop', server.id)}
                                 className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-muted/60 hover:text-foreground focus:bg-muted/60 focus:text-foreground"
                               >
                                 <Square className="h-4 w-4 text-muted-foreground" />
                                 <span>Stop Server</span>
                               </DropdownMenuItem>
                             </TooltipTrigger>
                             <TooltipContent side="left" className="bg-card text-foreground border border-border shadow-lg backdrop-blur-sm">
                               <p>Gracefully stop the server</p>
                             </TooltipContent>
                           </Tooltip>

                           <Tooltip>
                             <TooltipTrigger asChild>
                               <DropdownMenuItem 
                                 onClick={() => handleServerAction('promote', server.id)}
                                 className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-muted/60 hover:text-foreground focus:bg-muted/60 focus:text-foreground"
                               >
                                 <ArrowUp className="h-4 w-4 text-muted-foreground" />
                                 <span>Promote Server</span>
                               </DropdownMenuItem>
                             </TooltipTrigger>
                             <TooltipContent side="left" className="bg-card text-foreground border border-border shadow-lg backdrop-blur-sm">
                               <p>Promote to active deployment (Blue/Green)</p>
                             </TooltipContent>
                           </Tooltip>

                           <Tooltip>
                             <TooltipTrigger asChild>
                               <DropdownMenuItem 
                                 onClick={() => handleServerAction('clone', server.id)}
                                 className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-muted/60 hover:text-foreground focus:bg-muted/60 focus:text-foreground"
                               >
                                 <Copy className="h-4 w-4 text-muted-foreground" />
                                 <span>Clone Server</span>
                               </DropdownMenuItem>
                             </TooltipTrigger>
                             <TooltipContent side="left" className="bg-card text-foreground border border-border shadow-lg backdrop-blur-sm">
                               <p>Create a copy of this server configuration</p>
                             </TooltipContent>
                           </Tooltip>

                           <Tooltip>
                             <TooltipTrigger asChild>
                               <DropdownMenuItem 
                                 onClick={() => handleServerAction('open-folder', server.id)}
                                 className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-muted/60 hover:text-foreground focus:bg-muted/60 focus:text-foreground"
                               >
                                 <Folder className="h-4 w-4 text-muted-foreground" />
                                 <span>Open Folder</span>
                               </DropdownMenuItem>
                             </TooltipTrigger>
                             <TooltipContent side="left" className="bg-card text-foreground border border-border shadow-lg backdrop-blur-sm">
                               <p>Open server directory in file explorer</p>
                             </TooltipContent>
                           </Tooltip>

                           <Tooltip>
                             <TooltipTrigger asChild>
                               <DropdownMenuItem 
                                 onClick={() => handleServerAction('delete', server.id)}
                                 className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-muted/60 hover:text-foreground focus:bg-muted/60 focus:text-foreground"
                               >
                                 <Trash2 className="h-4 w-4 text-muted-foreground" />
                                 <span>Delete Server</span>
                               </DropdownMenuItem>
                             </TooltipTrigger>
                             <TooltipContent side="left" className="bg-card text-foreground border border-border shadow-lg backdrop-blur-sm">
                               <p>Permanently delete this server (cannot be undone)</p>
                             </TooltipContent>
                           </Tooltip>
                         </DropdownMenuContent>
                       </DropdownMenu>
                     </div>
                   </div>
                 </div>
               ))}
              
              {filteredServers.length === 0 && !loading && (
                <div className="text-center py-8 text-muted-foreground">
                  <Server className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">No servers found</p>
                  {searchQuery && (
                    <p className="text-xs">Try adjusting your search</p>
                  )}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Modpacks Section */}
        <div className="p-4 border-t border-border/30">
          <Link
            to="/modpacks"
            className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 ${
              location.pathname === '/modpacks' 
                ? 'bg-primary/15 text-primary border border-primary/20 shadow-sm' 
                : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
            }`}
          >
            <div className={`w-6 h-6 rounded-md flex items-center justify-center ${
              location.pathname === '/modpacks' 
                ? 'bg-primary/20 text-primary' 
                : 'bg-muted/50 text-muted-foreground'
            }`}>
              <Package className="h-3.5 w-3.5" />
            </div>
            <span>Modpacks</span>
          </Link>
        </div>

        {/* Workspace Section */}
        <div className="p-4 border-t border-border/30 bg-gradient-to-br from-muted/10 to-muted/20">
          {/* Workspace Header - Non-clickable section title */}
          <div className="flex items-center gap-3 mb-4 px-2">
            <div className="w-8 h-8 bg-gradient-to-br from-primary/20 to-secondary/20 rounded-lg flex items-center justify-center border border-primary/20">
              <Building2 className="h-4 w-4 text-primary" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-foreground">Workspace</h3>
              <p className="text-xs text-muted-foreground">System settings & configuration</p>
            </div>
          </div>

          {/* Workspace Navigation Links */}
          <div className="space-y-1">
            <Link
              to="/workspace/users-roles"
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 ${
                location.pathname === '/workspace/users-roles' 
                  ? 'bg-primary/15 text-primary border border-primary/20 shadow-sm' 
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
              }`}
            >
              <div className={`w-6 h-6 rounded-md flex items-center justify-center ${
                location.pathname === '/workspace/users-roles' 
                  ? 'bg-primary/20 text-primary' 
                  : 'bg-muted/50 text-muted-foreground'
              }`}>
                <Users className="h-3.5 w-3.5" />
              </div>
              <span>Users & Roles</span>
            </Link>
            
            <Link
              to="/workspace/backup-targets"
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 ${
                location.pathname === '/workspace/backup-targets' 
                  ? 'bg-primary/15 text-primary border border-primary/20 shadow-sm' 
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
              }`}
            >
              <div className={`w-6 h-6 rounded-md flex items-center justify-center ${
                location.pathname === '/workspace/backup-targets' 
                  ? 'bg-primary/20 text-primary' 
                  : 'bg-muted/50 text-muted-foreground'
              }`}>
                <Database className="h-3.5 w-3.5" />
              </div>
              <span>Backup Targets</span>
            </Link>
            
            <Link
              to="/workspace/tokens"
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 ${
                location.pathname === '/workspace/tokens' 
                  ? 'bg-primary/15 text-primary border border-primary/20 shadow-sm' 
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
              }`}
            >
              <div className={`w-6 h-6 rounded-md flex items-center justify-center ${
                location.pathname === '/workspace/tokens' 
                  ? 'bg-primary/20 text-primary' 
                  : 'bg-muted/50 text-muted-foreground'
              }`}>
                <Key className="h-3.5 w-3.5" />
              </div>
              <span>API Tokens</span>
            </Link>
            
            <Link
              to="/workspace/theme"
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 ${
                location.pathname === '/workspace/theme' 
                  ? 'bg-primary/15 text-primary border border-primary/20 shadow-sm' 
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
              }`}
            >
              <div className={`w-6 h-6 rounded-md flex items-center justify-center ${
                location.pathname === '/workspace/theme' 
                  ? 'bg-primary/20 text-primary' 
                  : 'bg-muted/50 text-muted-foreground'
              }`}>
                <Palette className="h-3.5 w-3.5" />
              </div>
              <span>Theme Settings</span>
            </Link>
            
          </div>
        </div>
        
        {/* Console Button - Bottom Left */}
        <div className="p-4 border-t border-border/50">
          <Button
            variant="outline"
            size="sm"
            className="w-full justify-start gap-2 text-muted-foreground hover:text-foreground"
            onClick={() => {
              console.log('Opening live console page...');
              navigate('/console');
            }}
          >
            <div className="w-4 h-4 rounded bg-muted/50 flex items-center justify-center">
              <span className="text-xs font-mono">{'>'}</span>
            </div>
            <span>Open Console</span>
          </Button>
          
          {/* Test API Button */}
          <Button
            variant="outline"
            size="sm"
            className="w-full justify-start gap-2 text-muted-foreground hover:text-foreground mt-2"
            onClick={async () => {
              console.log('Testing API connection...');
              try {
                const { api } = await import('../../lib/api');
                const response = await api('/healthz') as { success: boolean; data: string; error?: string; timestamp: string };
                console.log('API Health Check:', response);
                alert(`API Status: ${response.success ? 'OK' : 'Error'}\nHealth: ${response.data}\nTimestamp: ${response.timestamp}`);
              } catch (error) {
                console.error('API Test Failed:', error);
                alert(`API Test Failed: ${error}`);
              }
            }}
          >
            <div className="w-4 h-4 rounded bg-muted/50 flex items-center justify-center">
              <span className="text-xs font-mono">?</span>
            </div>
            <span>Test API</span>
          </Button>
        </div>
      </div>
    </nav>
    </TooltipProvider>
  );
};

export default Sidebar;
