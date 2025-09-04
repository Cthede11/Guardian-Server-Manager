import React, { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Search, Plus, Server, Settings, Users, MoreVertical, Copy, Trash2, FolderOpen } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
// import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { useServersStore } from '@/store/servers';
import { StatusPill } from '@/components/StatusPill';

// Add Server Wizard Component
const AddServerWizard: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const { createServer } = useServersStore();
  const [formData, setFormData] = useState({
    name: '',
    loader: 'forge',
    version: '1.20.1',
    paths: {
      world: '/opt/minecraft/world',
      mods: '/opt/minecraft/mods',
      config: '/opt/minecraft/config',
    }
  });
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    
    const success = await createServer(formData);
    if (success) {
      onClose();
    }
    setIsSubmitting(false);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
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
        <label className="block text-sm font-medium mb-2">Loader</label>
        <select
          value={formData.loader}
          onChange={(e) => setFormData({ ...formData, loader: e.target.value })}
          className="w-full px-3 py-2 border border-border rounded-md bg-background"
        >
          <option value="forge">Forge</option>
          <option value="fabric">Fabric</option>
          <option value="quilt">Quilt</option>
        </select>
      </div>
      
      <div>
        <label className="block text-sm font-medium mb-2">Version</label>
        <Input
          value={formData.version}
          onChange={(e) => setFormData({ ...formData, version: e.target.value })}
          placeholder="1.20.1"
          required
        />
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
  const { servers, selectedServerId, selectServer, loading, startServer, stopServer, promoteServer } = useServersStore();
  const { id: currentServerId } = useParams<{ id: string }>();
  const [searchQuery, setSearchQuery] = useState('');
  const [showAddServer, setShowAddServer] = useState(false);

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
          // Delete functionality would be implemented here
          console.log('Deleting server:', serverId);
        }
        break;
      case 'open-folder':
        // Open folder functionality would be implemented here
        console.log('Opening folder for server:', serverId);
        break;
    }
  };

  return (
    <nav className="w-64 bg-card border-r border-border flex flex-col shadow-xl" role="navigation">
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
          <div className="nav-section-header">
            <div className="nav-section-icon">
              <Server className="h-4 w-4 text-primary" />
            </div>
            <span className="nav-section-title">Servers</span>
            <div className="nav-section-badge">
              {servers.length}
            </div>
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
                         <DropdownMenuContent align="end" className="modern-card">
                           <DropdownMenuItem onClick={() => handleServerAction('start', server.id)}>
                             <Server className="h-4 w-4 mr-2" />
                             Start
                           </DropdownMenuItem>
                           <DropdownMenuItem onClick={() => handleServerAction('stop', server.id)}>
                             <Server className="h-4 w-4 mr-2" />
                             Stop
                           </DropdownMenuItem>
                           <DropdownMenuItem onClick={() => handleServerAction('promote', server.id)}>
                             <Server className="h-4 w-4 mr-2" />
                             Promote
                           </DropdownMenuItem>
                           <DropdownMenuItem onClick={() => handleServerAction('clone', server.id)}>
                             <Copy className="h-4 w-4 mr-2" />
                             Clone
                           </DropdownMenuItem>
                           <DropdownMenuItem onClick={() => handleServerAction('open-folder', server.id)}>
                             <FolderOpen className="h-4 w-4 mr-2" />
                             Open Folder
                           </DropdownMenuItem>
                           <DropdownMenuItem 
                             onClick={() => handleServerAction('delete', server.id)}
                             className="text-destructive"
                           >
                             <Trash2 className="h-4 w-4 mr-2" />
                             Delete
                           </DropdownMenuItem>
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

                {/* Workspace Section */}
        <div className="p-4 border-t border-border/30 bg-gradient-to-br from-muted/20 to-muted/40">
          <div className="nav-section-header">
            <div className="nav-section-icon">
              <Settings className="h-4 w-4 text-secondary" />
            </div>
            <span className="nav-section-title">Workspace</span>
          </div>

          <div className="nav-section">
            <Link
              to="/workspace/users-roles"
              className={`nav-link ${location.pathname === '/workspace/users-roles' ? 'active' : ''}`}
            >
              <div className="nav-link-icon">
                <Users className="h-3.5 w-3.5 text-primary" />
              </div>
              <span>Users & Roles</span>
            </Link>
            <Link
              to="/workspace/backup-targets"
              className={`nav-link ${location.pathname === '/workspace/backup-targets' ? 'active' : ''}`}
            >
              <div className="nav-link-icon">
                <Server className="h-3.5 w-3.5 text-primary" />
              </div>
              <span>Backup Targets</span>
            </Link>
            <Link
              to="/workspace/tokens"
              className={`nav-link ${location.pathname === '/workspace/tokens' ? 'active' : ''}`}
            >
              <div className="nav-link-icon">
                <Settings className="h-3.5 w-3.5 text-primary" />
              </div>
              <span>API Tokens</span>
            </Link>
            <Link
              to="/workspace/theme"
              className={`nav-link ${location.pathname === '/workspace/theme' ? 'active' : ''}`}
            >
              <div className="nav-link-icon">
                <Settings className="h-3.5 w-3.5 text-primary" />
              </div>
              <span>Theme</span>
            </Link>
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Sidebar;
