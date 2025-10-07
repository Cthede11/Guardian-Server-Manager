import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Package, 
  Search, 
  Filter, 
  Download, 
  Trash2, 
  Settings, 
  Eye, 
  EyeOff,
  AlertTriangle,
  CheckCircle,
  Loader2,
  RefreshCw,
  ExternalLink,
  Info,
  Zap,
  Shield
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { useToast } from '@/hooks/use-toast';
import { apiClient as api } from '@/lib/api';

interface InstalledMod {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  logo_url?: string;
  categories: string[];
  source: 'curseforge' | 'modrinth';
  minecraft_version: string;
  loader: string;
  file_size: number;
  installed_at: string;
  enabled: boolean;
  dependencies: string[];
  conflicts: string[];
  server_safe: boolean;
  file_path: string;
}

interface ModManagerProps {
  serverId: string;
  serverName: string;
}

export const ModManager: React.FC<ModManagerProps> = ({ serverId, serverName }) => {
  const [mods, setMods] = useState<InstalledMod[]>([]);
  const [filteredMods, setFilteredMods] = useState<InstalledMod[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterEnabled, setFilterEnabled] = useState<'all' | 'enabled' | 'disabled'>('all');
  const [filterSource, setFilterSource] = useState<'all' | 'curseforge' | 'modrinth'>('all');
  const [filterCategory, setFilterCategory] = useState<string>('all');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [showModDetails, setShowModDetails] = useState<string | null>(null);
  const [isUpdating, setIsUpdating] = useState<Set<string>>(new Set());
  const [conflicts, setConflicts] = useState<Record<string, string[]>>({});
  
  const { toast } = useToast();

  // Load mods
  useEffect(() => {
    loadMods();
  }, [serverId]);

  // Filter mods
  useEffect(() => {
    let filtered = mods;

    // Search filter
    if (searchQuery.trim()) {
      filtered = filtered.filter(mod =>
        mod.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        mod.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        mod.author.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    // Enabled filter
    if (filterEnabled !== 'all') {
      filtered = filtered.filter(mod => 
        filterEnabled === 'enabled' ? mod.enabled : !mod.enabled
      );
    }

    // Source filter
    if (filterSource !== 'all') {
      filtered = filtered.filter(mod => mod.source === filterSource);
    }

    // Category filter
    if (filterCategory !== 'all') {
      filtered = filtered.filter(mod => mod.categories.includes(filterCategory));
    }

    setFilteredMods(filtered);
  }, [mods, searchQuery, filterEnabled, filterSource, filterCategory]);

  const loadMods = async () => {
    setIsLoading(true);
    try {
      const response = await api.call<{
        success: boolean;
        data: InstalledMod[];
        error: string;
      }>(`/api/servers/${serverId}/mods`);

      if (response.success && response.data) {
        setMods(response.data);
        // Check for conflicts
        checkConflicts(response.data);
      } else {
        console.error('Failed to load mods:', response.error);
        toast({
          title: "Error",
          description: "Failed to load mods. Please try again.",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Error loading mods:', error);
      toast({
        title: "Error",
        description: "An error occurred while loading mods.",
        variant: "destructive"
      });
    } finally {
      setIsLoading(false);
    }
  };

  const checkConflicts = async (mods: InstalledMod[]) => {
    try {
      const response = await api.call<{
        success: boolean;
        data: Record<string, string[]>;
        error: string;
      }>(`/api/servers/${serverId}/mods/conflicts`, {
        method: 'POST',
        body: JSON.stringify({
          mod_ids: mods.map(mod => mod.id)
        })
      });

      if (response.success && response.data) {
        setConflicts(response.data);
      }
    } catch (error) {
      console.error('Error checking conflicts:', error);
    }
  };

  const toggleMod = async (modId: string, enabled: boolean) => {
    setIsUpdating(prev => new Set(prev).add(modId));
    try {
      const response = await api.call<{
        success: boolean;
        data: { message: string };
        error: string;
      }>(`/api/servers/${serverId}/mods/${modId}/toggle`, {
        method: 'POST',
        body: JSON.stringify({ enabled })
      });

      if (response.success) {
        setMods(prev => prev.map(mod => 
          mod.id === modId ? { ...mod, enabled } : mod
        ));
        toast({
          title: "Success",
          description: `Mod ${enabled ? 'enabled' : 'disabled'} successfully.`,
          variant: "default"
        });
      } else {
        throw new Error(response.error);
      }
    } catch (error) {
      console.error('Failed to toggle mod:', error);
      toast({
        title: "Error",
        description: `Failed to ${enabled ? 'enable' : 'disable'} mod.`,
        variant: "destructive"
      });
    } finally {
      setIsUpdating(prev => {
        const newSet = new Set(prev);
        newSet.delete(modId);
        return newSet;
      });
    }
  };

  const uninstallMod = async (modId: string) => {
    setIsUpdating(prev => new Set(prev).add(modId));
    try {
      const response = await api.call<{
        success: boolean;
        data: { message: string };
        error: string;
      }>(`/api/servers/${serverId}/mods/${modId}`, {
        method: 'DELETE'
      });

      if (response.success) {
        setMods(prev => prev.filter(mod => mod.id !== modId));
        toast({
          title: "Success",
          description: "Mod uninstalled successfully.",
          variant: "default"
        });
      } else {
        throw new Error(response.error);
      }
    } catch (error) {
      console.error('Failed to uninstall mod:', error);
      toast({
        title: "Error",
        description: "Failed to uninstall mod.",
        variant: "destructive"
      });
    } finally {
      setIsUpdating(prev => {
        const newSet = new Set(prev);
        newSet.delete(modId);
        return newSet;
      });
    }
  };

  const bulkToggle = async (enabled: boolean) => {
    const modIds = Array.from(selectedMods);
    setIsUpdating(prev => new Set([...prev, ...modIds]));
    
    try {
      const response = await api.call<{
        success: boolean;
        data: { message: string };
        error: string;
      }>(`/api/servers/${serverId}/mods/bulk-toggle`, {
        method: 'POST',
        body: JSON.stringify({ 
          mod_ids: modIds,
          enabled 
        })
      });

      if (response.success) {
        setMods(prev => prev.map(mod => 
          selectedMods.has(mod.id) ? { ...mod, enabled } : mod
        ));
        setSelectedMods(new Set());
        toast({
          title: "Success",
          description: `${modIds.length} mods ${enabled ? 'enabled' : 'disabled'} successfully.`,
          variant: "default"
        });
      } else {
        throw new Error(response.error);
      }
    } catch (error) {
      console.error('Failed to bulk toggle mods:', error);
      toast({
        title: "Error",
        description: "Failed to update mods.",
        variant: "destructive"
      });
    } finally {
      setIsUpdating(prev => {
        const newSet = new Set(prev);
        modIds.forEach(id => newSet.delete(id));
        return newSet;
      });
    }
  };

  const formatFileSize = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getCategories = () => {
    const allCategories = new Set<string>();
    mods.forEach(mod => {
      mod.categories.forEach(category => allCategories.add(category));
    });
    return Array.from(allCategories).sort();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Mod Manager</h2>
          <p className="text-muted-foreground">
            Manage mods for {serverName}
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            onClick={loadMods}
            disabled={isLoading}
            className="flex items-center space-x-2"
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            <span>Refresh</span>
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center space-x-2">
              <Package className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-sm text-muted-foreground">Total Mods</p>
                <p className="text-2xl font-bold">{mods.length}</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center space-x-2">
              <CheckCircle className="h-4 w-4 text-green-600" />
              <div>
                <p className="text-sm text-muted-foreground">Enabled</p>
                <p className="text-2xl font-bold text-green-600">
                  {mods.filter(mod => mod.enabled).length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center space-x-2">
              <EyeOff className="h-4 w-4 text-gray-600" />
              <div>
                <p className="text-sm text-muted-foreground">Disabled</p>
                <p className="text-2xl font-bold text-gray-600">
                  {mods.filter(mod => !mod.enabled).length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center space-x-2">
              <AlertTriangle className="h-4 w-4 text-yellow-600" />
              <div>
                <p className="text-sm text-muted-foreground">Conflicts</p>
                <p className="text-2xl font-bold text-yellow-600">
                  {Object.keys(conflicts).length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Filter className="h-5 w-5" />
            <span>Filters</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-4">
            <div className="flex-1">
              <Input
                placeholder="Search mods..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="flex-1"
              />
            </div>
            <Select value={filterEnabled} onValueChange={(value) => setFilterEnabled(value as any)}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All</SelectItem>
                <SelectItem value="enabled">Enabled</SelectItem>
                <SelectItem value="disabled">Disabled</SelectItem>
              </SelectContent>
            </Select>
            <Select value={filterSource} onValueChange={(value) => setFilterSource(value as any)}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Sources</SelectItem>
                <SelectItem value="curseforge">CurseForge</SelectItem>
                <SelectItem value="modrinth">Modrinth</SelectItem>
              </SelectContent>
            </Select>
            <Select value={filterCategory} onValueChange={setFilterCategory}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {getCategories().map(category => (
                  <SelectItem key={category} value={category}>
                    {category}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Bulk Actions */}
          {selectedMods.size > 0 && (
            <div className="flex items-center space-x-2 p-3 bg-muted rounded-lg">
              <span className="text-sm font-medium">
                {selectedMods.size} mod{selectedMods.size > 1 ? 's' : ''} selected
              </span>
              <Button
                size="sm"
                variant="outline"
                onClick={() => bulkToggle(true)}
                disabled={isUpdating.size > 0}
              >
                Enable All
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => bulkToggle(false)}
                disabled={isUpdating.size > 0}
              >
                Disable All
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => setSelectedMods(new Set())}
              >
                Clear Selection
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Mods List */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin" />
          <span className="ml-2">Loading mods...</span>
        </div>
      ) : filteredMods.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Package className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">No mods found</h3>
            <p className="text-muted-foreground text-center">
              {searchQuery || filterEnabled !== 'all' || filterSource !== 'all' || filterCategory !== 'all'
                ? 'No mods match your current filters.'
                : 'No mods are installed on this server.'}
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {filteredMods.map((mod) => {
            const isUpdatingMod = isUpdating.has(mod.id);
            const modConflicts = conflicts[mod.id] || [];
            const isSelected = selectedMods.has(mod.id);
            
            return (
              <Card key={mod.id} className={`transition-all ${isSelected ? 'ring-2 ring-primary' : ''}`}>
                <CardContent className="p-4">
                  <div className="flex items-start space-x-4">
                    {/* Checkbox */}
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setSelectedMods(prev => new Set(prev).add(mod.id));
                        } else {
                          setSelectedMods(prev => {
                            const newSet = new Set(prev);
                            newSet.delete(mod.id);
                            return newSet;
                          });
                        }
                      }}
                      className="mt-1"
                    />

                    {/* Mod Icon */}
                    {mod.logo_url ? (
                      <img
                        src={mod.logo_url}
                        alt={mod.name}
                        className="w-12 h-12 rounded-lg object-cover"
                      />
                    ) : (
                      <div className="w-12 h-12 bg-muted rounded-lg flex items-center justify-center">
                        <Package className="h-6 w-6 text-muted-foreground" />
                      </div>
                    )}

                    {/* Mod Info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between">
                        <div className="flex-1 min-w-0">
                          <h3 className="text-lg font-semibold truncate">{mod.name}</h3>
                          <p className="text-sm text-muted-foreground truncate">
                            by {mod.author} • v{mod.version}
                          </p>
                          <p className="text-sm text-muted-foreground mt-1 line-clamp-2">
                            {mod.description}
                          </p>
                        </div>
                        <div className="flex items-center space-x-2 ml-4">
                          <Badge variant={mod.source === 'curseforge' ? 'default' : 'secondary'}>
                            {mod.source}
                          </Badge>
                          {mod.server_safe && (
                            <Badge variant="outline" className="text-green-600 border-green-600">
                              <Shield className="h-3 w-3 mr-1" />
                              Server Safe
                            </Badge>
                          )}
                          {modConflicts.length > 0 && (
                            <Badge variant="destructive">
                              <AlertTriangle className="h-3 w-3 mr-1" />
                              Conflicts
                            </Badge>
                          )}
                        </div>
                      </div>

                      {/* Categories and Details */}
                      <div className="flex items-center space-x-4 mt-2 text-sm text-muted-foreground">
                        <div className="flex items-center space-x-1">
                          <span>Size:</span>
                          <span>{formatFileSize(mod.file_size)}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <span>Installed:</span>
                          <span>{formatDate(mod.installed_at)}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <span>Loader:</span>
                          <span>{mod.loader}</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <span>MC:</span>
                          <span>{mod.minecraft_version}</span>
                        </div>
                      </div>

                      {/* Categories */}
                      <div className="flex flex-wrap gap-1 mt-2">
                        {mod.categories.slice(0, 5).map((category) => (
                          <Badge key={category} variant="outline" className="text-xs">
                            {category}
                          </Badge>
                        ))}
                        {mod.categories.length > 5 && (
                          <Badge variant="outline" className="text-xs">
                            +{mod.categories.length - 5}
                          </Badge>
                        )}
                      </div>

                      {/* Conflicts */}
                      {modConflicts.length > 0 && (
                        <Alert variant="destructive" className="mt-2">
                          <AlertTriangle className="h-4 w-4" />
                          <AlertDescription>
                            <div className="space-y-1">
                              <div className="font-medium">Conflicts detected:</div>
                              {modConflicts.map((conflict, index) => (
                                <div key={index} className="text-sm">{conflict}</div>
                              ))}
                            </div>
                          </AlertDescription>
                        </Alert>
                      )}
                    </div>

                    {/* Actions */}
                    <div className="flex items-center space-x-2">
                      <Switch
                        checked={mod.enabled}
                        onCheckedChange={(enabled) => toggleMod(mod.id, enabled)}
                        disabled={isUpdatingMod}
                      />
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setShowModDetails(mod.id)}
                      >
                        <Info className="h-4 w-4" />
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => window.open(`https://${mod.source}.com/mod/${mod.id}`, '_blank')}
                      >
                        <ExternalLink className="h-4 w-4" />
                      </Button>
                      <Button
                        size="sm"
                        variant="destructive"
                        onClick={() => uninstallMod(mod.id)}
                        disabled={isUpdatingMod}
                      >
                        {isUpdatingMod ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <Trash2 className="h-4 w-4" />
                        )}
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Mod Details Modal */}
      <Dialog open={!!showModDetails} onOpenChange={() => setShowModDetails(null)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Mod Details</DialogTitle>
          </DialogHeader>
          {showModDetails && (() => {
            const mod = mods.find(m => m.id === showModDetails);
            if (!mod) return null;
            
            return (
              <div className="space-y-4">
                <div className="flex items-start space-x-4">
                  {mod.logo_url ? (
                    <img
                      src={mod.logo_url}
                      alt={mod.name}
                      className="w-16 h-16 rounded-lg object-cover"
                    />
                  ) : (
                    <div className="w-16 h-16 bg-muted rounded-lg flex items-center justify-center">
                      <Package className="h-8 w-8 text-muted-foreground" />
                    </div>
                  )}
                  <div className="flex-1">
                    <h3 className="text-xl font-bold">{mod.name}</h3>
                    <p className="text-muted-foreground">by {mod.author}</p>
                    <div className="flex items-center space-x-2 mt-2">
                      <Badge variant={mod.source === 'curseforge' ? 'default' : 'secondary'}>
                        {mod.source}
                      </Badge>
                      <Badge variant="outline">v{mod.version}</Badge>
                      {mod.server_safe && (
                        <Badge variant="outline" className="text-green-600 border-green-600">
                          <Shield className="h-3 w-3 mr-1" />
                          Server Safe
                        </Badge>
                      )}
                    </div>
                  </div>
                </div>

                <div>
                  <h4 className="font-semibold mb-2">Description</h4>
                  <p className="text-muted-foreground">{mod.description}</p>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold mb-2">Details</h4>
                    <div className="space-y-1 text-sm">
                      <div>File Size: {formatFileSize(mod.file_size)}</div>
                      <div>Installed: {formatDate(mod.installed_at)}</div>
                      <div>Loader: {mod.loader}</div>
                      <div>Minecraft: {mod.minecraft_version}</div>
                      <div>Status: {mod.enabled ? 'Enabled' : 'Disabled'}</div>
                    </div>
                  </div>
                  <div>
                    <h4 className="font-semibold mb-2">Categories</h4>
                    <div className="flex flex-wrap gap-1">
                      {mod.categories.map((category) => (
                        <Badge key={category} variant="outline" className="text-xs">
                          {category}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </div>

                {mod.dependencies.length > 0 && (
                  <div>
                    <h4 className="font-semibold mb-2">Dependencies</h4>
                    <div className="flex flex-wrap gap-1">
                      {mod.dependencies.map((dep) => (
                        <Badge key={dep} variant="secondary" className="text-xs">
                          {dep}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}

                {conflicts[mod.id] && conflicts[mod.id].length > 0 && (
                  <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertDescription>
                      <div className="font-medium mb-2">Conflicts:</div>
                      <div className="space-y-1">
                        {conflicts[mod.id].map((conflict, index) => (
                          <div key={index} className="text-sm">{conflict}</div>
                        ))}
                      </div>
                    </AlertDescription>
                  </Alert>
                )}
              </div>
            );
          })()}
        </DialogContent>
      </Dialog>
    </div>
  );
};
