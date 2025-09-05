import React, { useState, useEffect } from 'react';
import { Link, useParams, useLocation, useNavigate } from 'react-router-dom';
import { 
  Search, 
  Plus, 
  Server, 
  Settings, 
  Users, 
  MoreVertical, 
  Copy, 
  Trash2, 
  FolderOpen,
  Database,
  Key,
  Palette,
  Building2,
  ChevronDown,
  ChevronRight,
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
import { useServersStore } from '@/store/servers';
import { openDevTools } from '@/lib/tauri-api';
import { StatusPill } from '@/components/StatusPill';

// Add Server Wizard Component
const AddServerWizard: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const { createServer } = useServersStore();
  const [formData, setFormData] = useState({
    // Step 1: Basic Info
    name: '',
    type: 'vanilla' as 'vanilla' | 'forge' | 'fabric' | 'paper' | 'purpur' | 'spigot' | 'bukkit',
    version: '1.21.1',
    
    // Step 2: Java Configuration
    javaPath: '',
    javaArgs: '-Xmx4G -Xms2G -XX:+UseG1GC',
    memory: 4096,
    
    // Step 3: Network Configuration
    serverPort: 25565,
    rconPort: 25575,
    rconPassword: '',
    queryPort: 25565,
    
    // Step 4: File Paths
    paths: {
      world: './world',
      mods: './mods',
      config: './config',
      logs: './logs',
      backups: './backups'
    },
    
    // Step 5: Advanced Settings
    settings: {
      autoStart: false,
      autoRestart: true,
      maxRestarts: 3,
      backupInterval: 24,
      backupRetention: 7
    }
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [currentStep, setCurrentStep] = useState(1);
  const [javaInstallations, setJavaInstallations] = useState<any[]>([]);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setError(null);
    
    try {
      const success = await createServer({
        name: formData.name,
        loader: formData.type,
        version: formData.version,
        paths: formData.paths
      });
      if (success) {
        console.log('Server created successfully:', formData);
        onClose();
      } else {
        setError('Failed to create server. Please check the console for details.');
        console.error('Failed to create server');
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      setError(`Error creating server: ${errorMessage}`);
      console.error('Error creating server:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}
      <div>
        <label className="block text-sm font-medium mb-2">Server Name</label>
        <Input
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          placeholder="My Server"
          required
        />
      </div>
      
      <div>
        <label className="block text-sm font-medium mb-2">Mod Manager</label>
        <select
          value={formData.type}
          onChange={(e) => setFormData({ ...formData, type: e.target.value as any })}
          className="w-full px-3 py-2 border border-border rounded-md bg-background"
        >
          <option value="vanilla">Vanilla</option>
          <option value="forge">Forge</option>
          <option value="neoforge">NeoForge</option>
          <option value="fabric">Fabric</option>
          <option value="quilt">Quilt</option>
          <option value="paper">Paper</option>
          <option value="purpur">Purpur</option>
          <option value="spigot">Spigot</option>
          <option value="bukkit">Bukkit</option>
        </select>
      </div>
      
      <div>
        <label className="block text-sm font-medium mb-2">Minecraft Version</label>
        <select
          value={formData.version}
          onChange={(e) => setFormData({ ...formData, version: e.target.value })}
          className="w-full px-3 py-2 border border-border rounded-md bg-background"
        >
          <option value="1.21.1">1.21.1 (Latest)</option>
          <option value="1.21">1.21</option>
          <option value="1.20.6">1.20.6</option>
          <option value="1.20.4">1.20.4</option>
          <option value="1.20.2">1.20.2</option>
          <option value="1.20.1">1.20.1</option>
          <option value="1.20">1.20</option>
          <option value="1.19.4">1.19.4</option>
          <option value="1.19.2">1.19.2</option>
          <option value="1.18.2">1.18.2</option>
          <option value="1.17.1">1.17.1</option>
          <option value="1.16.5">1.16.5</option>
        </select>
      </div>
      
      <div className="flex gap-2 pt-4">
        <Button type="submit" disabled={isSubmitting} className="flex-1">
          {isSubmitting ? 'Creating...' : 'Create Server'}
        </Button>
        <Button type="button" variant="outline" onClick={onClose}>
          Cancel
        </Button>
      </div>
    </form>
  );
};

export const Sidebar: React.FC = () => {
  const { servers, selectedServerId, selectServer, loading, startServer, stopServer, promoteServer, deleteServer } = useServersStore();
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

  // Load servers on mount
  useEffect(() => {
    const { fetchServers } = useServersStore.getState();
    fetchServers();
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
          await deleteServer(serverId);
        }
        break;
      case 'open-folder':
        // Open folder functionality would be implemented here
        console.log('Opening folder for server:', serverId);
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
                                    <StatusPill status={server.status} />
                                    <span className="text-xs text-muted-foreground font-medium">
                                      {server.playersOnline} players
                                    </span>
                                  </div>
                                </div>
                              </div>
                            </Link>
                     
                     <div className="flex items-center gap-1">
                       {server.blueGreen && (
                         <div className={`w-2 h-2 rounded-full ${
                           server.blueGreen.active === 'blue' ? 'bg-blue-500' : 'bg-green-500'
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
                const response = await fetch('http://localhost:8080/api/health');
                const data = await response.json();
                console.log('API Health Check:', data);
                alert(`API Status: ${data.success ? 'OK' : 'Error'}\nData: ${JSON.stringify(data)}`);
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
