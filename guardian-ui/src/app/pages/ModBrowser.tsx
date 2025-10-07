import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Search, 
  Filter, 
  Download, 
  Package, 
  ExternalLink,
  Loader2,
  AlertTriangle,
  CheckCircle,
  X,
  Plus,
  Star,
  Eye,
  Heart,
  Calendar,
  User,
  Tag,
  Zap,
  Shield,
  RefreshCw
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useToast } from '@/hooks/use-toast';
import { apiClient as api } from '@/lib/api';
import { useServers } from '@/store/servers-new';
import { ServerSelectionModal } from '@/components/ModBrowser/ServerSelectionModal';
import { ModsGridLoading, ModpacksGridLoading } from '@/components/ui/LoadingStates';
import { NoModsEmptyState, NoModpacksEmptyState, SearchEmptyState } from '@/components/ui/EmptyState';

interface ModInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  downloads: number;
  author: string;
  logo_url?: string;
  categories: string[];
  server_safe: boolean;
  source: 'curseforge' | 'modrinth';
  minecraft_version: string;
  loader: string;
  created_at: string;
  updated_at: string;
  rating?: number;
  file_size?: number;
  dependencies?: string[];
}

interface ModpackInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  downloads: number;
  author: string;
  logo_url?: string;
  server_mods: number;
  client_mods: number;
  total_mods: number;
  source: 'curseforge' | 'modrinth';
  minecraft_version: string;
  loader: string;
  created_at: string;
  updated_at: string;
  rating?: number;
}

interface SearchFilters {
  query: string;
  source: 'all' | 'curseforge' | 'modrinth';
  minecraft_version: string;
  loader: string;
  category: string;
  server_safe: boolean;
  sort_by: 'relevance' | 'downloads' | 'updated' | 'created' | 'rating';
  sort_order: 'asc' | 'desc';
}

interface PaginationInfo {
  page: number;
  limit: number;
  total: number;
  total_pages: number;
}

const defaultFilters: SearchFilters = {
  query: '',
  source: 'all',
  minecraft_version: '1.21.1',
  loader: 'all',
  category: 'all',
  server_safe: true,
  sort_by: 'relevance',
  sort_order: 'desc'
};

const categories = [
  'all', 'adventure', 'cursed', 'decoration', 'economy', 'equipment', 'food', 'game-mechanics',
  'library', 'magic', 'management', 'minigame', 'mobs', 'optimization', 'social', 'storage',
  'technology', 'transportation', 'utility', 'worldgen'
];

const loaders = ['all', 'forge', 'fabric', 'quilt', 'neoforge'];

const minecraftVersions = [
  '1.21.1', '1.21', '1.20.6', '1.20.4', '1.20.2', '1.20.1', '1.20',
  '1.19.4', '1.19.3', '1.19.2', '1.19.1', '1.19', '1.18.2', '1.18.1', '1.18'
];

export const ModBrowser: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'mods' | 'modpacks'>('mods');
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [modpacks, setModpacks] = useState<ModpackInfo[]>([]);
  const [filters, setFilters] = useState<SearchFilters>(defaultFilters);
  const [pagination, setPagination] = useState<PaginationInfo>({
    page: 1,
    limit: 20,
    total: 0,
    total_pages: 0
  });
  const [isLoading, setIsLoading] = useState(false);
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [selectedModpacks, setSelectedModpacks] = useState<Set<string>>(new Set());
  const [showFilters, setShowFilters] = useState(false);
  const [installingMods, setInstallingMods] = useState<Set<string>>(new Set());
  const [installingModpacks, setInstallingModpacks] = useState<Set<string>>(new Set());
  const [serverSelectionModal, setServerSelectionModal] = useState<{
    isOpen: boolean;
    type: 'mod' | 'modpack';
    item: ModInfo | ModpackInfo | null;
  }>({
    isOpen: false,
    type: 'mod',
    item: null
  });
  
  const { summaries, getServerById } = useServers();
  const servers = Object.values(summaries);
  const { toast } = useToast();

  // Search mods
  const searchMods = useCallback(async (page = 1) => {
    setIsLoading(true);
    try {
      const params = new URLSearchParams({
        query: filters.query,
        source: filters.source,
        minecraft_version: filters.minecraft_version,
        loader: filters.loader,
        category: filters.category,
        server_safe: filters.server_safe.toString(),
        sort_by: filters.sort_by,
        sort_order: filters.sort_order,
        page: page.toString(),
        limit: pagination.limit.toString()
      });

      const response = await api.call<{
        success: boolean;
        data: {
          mods: ModInfo[];
          pagination: PaginationInfo;
        };
        error: string;
      }>(`/api/mods/search?${params}`);

      if (response.success && response.data) {
        setMods(response.data.mods);
        setPagination(response.data.pagination);
      } else {
        console.error('Failed to search mods:', response.error);
        setMods([]);
        toast({
          title: "Search Error",
          description: "Failed to search mods. Please try again.",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Error searching mods:', error);
      setMods([]);
      toast({
        title: "Search Error",
        description: "An error occurred while searching mods.",
        variant: "destructive"
      });
    } finally {
      setIsLoading(false);
    }
  }, [filters, pagination.limit, toast]);

  // Search modpacks
  const searchModpacks = useCallback(async (page = 1) => {
    setIsLoading(true);
    try {
      const params = new URLSearchParams({
        query: filters.query,
        source: filters.source,
        minecraft_version: filters.minecraft_version,
        loader: filters.loader,
        category: filters.category,
        sort_by: filters.sort_by,
        sort_order: filters.sort_order,
        page: page.toString(),
        limit: pagination.limit.toString()
      });

      const response = await api.call<{
        success: boolean;
        data: {
          modpacks: ModpackInfo[];
          pagination: PaginationInfo;
        };
        error: string;
      }>(`/api/modpacks/search?${params}`);

      if (response.success && response.data) {
        setModpacks(response.data.modpacks);
        setPagination(response.data.pagination);
      } else {
        console.error('Failed to search modpacks:', response.error);
        setModpacks([]);
        toast({
          title: "Search Error",
          description: "Failed to search modpacks. Please try again.",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Error searching modpacks:', error);
      setModpacks([]);
      toast({
        title: "Search Error",
        description: "An error occurred while searching modpacks.",
        variant: "destructive"
      });
    } finally {
      setIsLoading(false);
    }
  }, [filters, pagination.limit, toast]);

  // Install mod to server
  const installModToServer = async (modId: string, serverId: string) => {
    setInstallingMods(prev => new Set(prev).add(modId));
    try {
      const response = await api.call<{
        success: boolean;
        data: { message: string };
        error: string;
      }>(`/api/servers/${serverId}/mods/install`, {
        method: 'POST',
        body: JSON.stringify({
          mod_id: modId,
          provider: mods.find(m => m.id === modId)?.source || 'modrinth'
        })
      });

      if (response.success) {
        toast({
          title: "Mod Installed",
          description: `Successfully installed mod to server.`,
          variant: "default"
        });
      } else {
        throw new Error(response.error);
      }
    } catch (error) {
      console.error('Failed to install mod:', error);
      toast({
        title: "Installation Failed",
        description: `Failed to install mod: ${error}`,
        variant: "destructive"
      });
    } finally {
      setInstallingMods(prev => {
        const newSet = new Set(prev);
        newSet.delete(modId);
        return newSet;
      });
    }
  };

  // Apply modpack to server
  const applyModpackToServer = async (modpackId: string, serverId: string) => {
    setInstallingModpacks(prev => new Set(prev).add(modpackId));
    try {
      const response = await api.call<{
        success: boolean;
        data: { message: string };
        error: string;
      }>(`/api/servers/${serverId}/modpacks/apply`, {
        method: 'POST',
        body: JSON.stringify({
          modpack_id: modpackId,
          provider: modpacks.find(m => m.id === modpackId)?.source || 'modrinth'
        })
      });

      if (response.success) {
        toast({
          title: "Modpack Applied",
          description: `Successfully applied modpack to server.`,
          variant: "default"
        });
      } else {
        throw new Error(response.error);
      }
    } catch (error) {
      console.error('Failed to apply modpack:', error);
      toast({
        title: "Application Failed",
        description: `Failed to apply modpack: ${error}`,
        variant: "destructive"
      });
    } finally {
      setInstallingModpacks(prev => {
        const newSet = new Set(prev);
        newSet.delete(modpackId);
        return newSet;
      });
    }
  };

  // Handle filter changes
  const handleFilterChange = (key: keyof SearchFilters, value: any) => {
    setFilters(prev => ({ ...prev, [key]: value }));
    setPagination(prev => ({ ...prev, page: 1 }));
  };

  // Handle search
  const handleSearch = () => {
    if (activeTab === 'mods') {
      searchMods(1);
    } else {
      searchModpacks(1);
    }
  };

  // Handle page change
  const handlePageChange = (page: number) => {
    setPagination(prev => ({ ...prev, page }));
    if (activeTab === 'mods') {
      searchMods(page);
    } else {
      searchModpacks(page);
    }
  };

  // Handle mod install
  const handleModInstall = (mod: ModInfo) => {
    setServerSelectionModal({
      isOpen: true,
      type: 'mod',
      item: mod
    });
  };

  // Handle modpack install
  const handleModpackInstall = (modpack: ModpackInfo) => {
    setServerSelectionModal({
      isOpen: true,
      type: 'modpack',
      item: modpack
    });
  };

  // Handle server selection modal install
  const handleServerSelectionInstall = async (serverId: string) => {
    if (!serverSelectionModal.item) return;

    if (serverSelectionModal.type === 'mod') {
      await installModToServer(serverSelectionModal.item.id, serverId);
    } else {
      await applyModpackToServer(serverSelectionModal.item.id, serverId);
    }
  };

  // Load initial data
  useEffect(() => {
    if (activeTab === 'mods') {
      searchMods();
    } else {
      searchModpacks();
    }
  }, [activeTab, searchMods, searchModpacks]);

  // Format file size
  const formatFileSize = (bytes?: number) => {
    if (!bytes) return 'Unknown';
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  // Format date
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  // Format downloads
  const formatDownloads = (downloads: number) => {
    if (downloads >= 1000000) {
      return `${(downloads / 1000000).toFixed(1)}M`;
    } else if (downloads >= 1000) {
      return `${(downloads / 1000).toFixed(1)}K`;
    }
    return downloads.toString();
  };

  return (
    <div className="container mx-auto px-4 py-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Mod Browser</h1>
          <p className="text-muted-foreground">
            Discover and install mods and modpacks for your Minecraft servers
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center space-x-2"
          >
            <Filter className="h-4 w-4" />
            <span>Filters</span>
          </Button>
          <Button
            variant="outline"
            onClick={handleSearch}
            disabled={isLoading}
            className="flex items-center space-x-2"
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            <span>Refresh</span>
          </Button>
        </div>
      </div>

      {/* Search and Filters */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Search className="h-5 w-5" />
            <span>Search & Filters</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Search Bar */}
          <div className="flex space-x-2">
            <Input
              placeholder="Search mods and modpacks..."
              value={filters.query}
              onChange={(e) => handleFilterChange('query', e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
              className="flex-1"
            />
            <Button onClick={handleSearch} disabled={isLoading}>
              {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
            </Button>
          </div>

          {/* Filters */}
          <AnimatePresence>
            {showFilters && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 pt-4 border-t"
              >
                <div>
                  <label className="text-sm font-medium mb-2 block">Source</label>
                  <Select
                    value={filters.source}
                    onValueChange={(value) => handleFilterChange('source', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Sources</SelectItem>
                      <SelectItem value="curseforge">CurseForge</SelectItem>
                      <SelectItem value="modrinth">Modrinth</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <label className="text-sm font-medium mb-2 block">Minecraft Version</label>
                  <Select
                    value={filters.minecraft_version}
                    onValueChange={(value) => handleFilterChange('minecraft_version', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {minecraftVersions.map(version => (
                        <SelectItem key={version} value={version}>{version}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <label className="text-sm font-medium mb-2 block">Loader</label>
                  <Select
                    value={filters.loader}
                    onValueChange={(value) => handleFilterChange('loader', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {loaders.map(loader => (
                        <SelectItem key={loader} value={loader}>
                          {loader === 'all' ? 'All Loaders' : loader.charAt(0).toUpperCase() + loader.slice(1)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <label className="text-sm font-medium mb-2 block">Category</label>
                  <Select
                    value={filters.category}
                    onValueChange={(value) => handleFilterChange('category', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {categories.map(category => (
                        <SelectItem key={category} value={category}>
                          {category === 'all' ? 'All Categories' : category.charAt(0).toUpperCase() + category.slice(1)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="flex items-center space-x-2">
                  <Switch
                    id="server-safe"
                    checked={filters.server_safe}
                    onCheckedChange={(checked) => handleFilterChange('server_safe', checked)}
                  />
                  <label htmlFor="server-safe" className="text-sm font-medium">
                    Server Safe Only
                  </label>
                </div>

                <div>
                  <label className="text-sm font-medium mb-2 block">Sort By</label>
                  <Select
                    value={filters.sort_by}
                    onValueChange={(value) => handleFilterChange('sort_by', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="relevance">Relevance</SelectItem>
                      <SelectItem value="downloads">Downloads</SelectItem>
                      <SelectItem value="updated">Last Updated</SelectItem>
                      <SelectItem value="created">Date Created</SelectItem>
                      <SelectItem value="rating">Rating</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </CardContent>
      </Card>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as 'mods' | 'modpacks')}>
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="mods" className="flex items-center space-x-2">
            <Package className="h-4 w-4" />
            <span>Mods ({mods.length})</span>
          </TabsTrigger>
          <TabsTrigger value="modpacks" className="flex items-center space-x-2">
            <Package className="h-4 w-4" />
            <span>Modpacks ({modpacks.length})</span>
          </TabsTrigger>
        </TabsList>

        {/* Mods Tab */}
        <TabsContent value="mods" className="space-y-4">
          {isLoading ? (
            <ModsGridLoading count={8} />
          ) : mods.length === 0 ? (
            <NoModsEmptyState 
              searchQuery={filters.query}
              onRefresh={() => searchMods()}
              onAdd={() => setShowFilters(true)}
            />
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
              {mods.map((mod) => (
                <Card key={mod.id} className="group hover:shadow-lg transition-shadow">
                  <CardHeader className="pb-3">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <CardTitle className="text-lg line-clamp-1">{mod.name}</CardTitle>
                        <p className="text-sm text-muted-foreground mt-1">by {mod.author}</p>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Badge variant={mod.source === 'curseforge' ? 'default' : 'secondary'}>
                          {mod.source}
                        </Badge>
                        {mod.server_safe && (
                          <Badge variant="outline" className="text-green-600 border-green-600">
                            <Shield className="h-3 w-3 mr-1" />
                            Server Safe
                          </Badge>
                        )}
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {mod.logo_url && (
                      <div className="aspect-video bg-muted rounded-lg overflow-hidden">
                        <img
                          src={mod.logo_url}
                          alt={mod.name}
                          className="w-full h-full object-cover"
                        />
                      </div>
                    )}
                    
                    <p className="text-sm text-muted-foreground line-clamp-2">
                      {mod.description}
                    </p>

                    <div className="flex items-center justify-between text-sm text-muted-foreground">
                      <div className="flex items-center space-x-4">
                        <span className="flex items-center space-x-1">
                          <Download className="h-3 w-3" />
                          <span>{formatDownloads(mod.downloads)}</span>
                        </span>
                        <span className="flex items-center space-x-1">
                          <Calendar className="h-3 w-3" />
                          <span>{formatDate(mod.updated_at)}</span>
                        </span>
                      </div>
                      <span className="text-xs bg-muted px-2 py-1 rounded">
                        v{mod.version}
                      </span>
                    </div>

                    <div className="flex flex-wrap gap-1">
                      {mod.categories.slice(0, 3).map((category) => (
                        <Badge key={category} variant="outline" className="text-xs">
                          {category}
                        </Badge>
                      ))}
                      {mod.categories.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{mod.categories.length - 3}
                        </Badge>
                      )}
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="flex items-center space-x-2">
                        <Badge variant="outline" className="text-xs">
                          {mod.loader}
                        </Badge>
                        <Badge variant="outline" className="text-xs">
                          {mod.minecraft_version}
                        </Badge>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => window.open(`https://${mod.source}.com/mod/${mod.id}`, '_blank')}
                        >
                          <ExternalLink className="h-3 w-3" />
                        </Button>
                        <Button
                          size="sm"
                          onClick={() => handleModInstall(mod)}
                          disabled={installingMods.has(mod.id)}
                        >
                          {installingMods.has(mod.id) ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <Plus className="h-3 w-3" />
                          )}
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}

          {/* Pagination */}
          {pagination.total_pages > 1 && (
            <div className="flex items-center justify-center space-x-2">
              <Button
                variant="outline"
                onClick={() => handlePageChange(pagination.page - 1)}
                disabled={pagination.page <= 1}
              >
                Previous
              </Button>
              <span className="text-sm text-muted-foreground">
                Page {pagination.page} of {pagination.total_pages}
              </span>
              <Button
                variant="outline"
                onClick={() => handlePageChange(pagination.page + 1)}
                disabled={pagination.page >= pagination.total_pages}
              >
                Next
              </Button>
            </div>
          )}
        </TabsContent>

        {/* Modpacks Tab */}
        <TabsContent value="modpacks" className="space-y-4">
          {isLoading ? (
            <ModpacksGridLoading count={6} />
          ) : modpacks.length === 0 ? (
            <NoModpacksEmptyState 
              searchQuery={filters.query}
              onRefresh={() => searchModpacks()}
              onAdd={() => setShowFilters(true)}
            />
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
              {modpacks.map((modpack) => (
                <Card key={modpack.id} className="group hover:shadow-lg transition-shadow">
                  <CardHeader className="pb-3">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <CardTitle className="text-lg line-clamp-1">{modpack.name}</CardTitle>
                        <p className="text-sm text-muted-foreground mt-1">by {modpack.author}</p>
                      </div>
                      <Badge variant={modpack.source === 'curseforge' ? 'default' : 'secondary'}>
                        {modpack.source}
                      </Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {modpack.logo_url && (
                      <div className="aspect-video bg-muted rounded-lg overflow-hidden">
                        <img
                          src={modpack.logo_url}
                          alt={modpack.name}
                          className="w-full h-full object-cover"
                        />
                      </div>
                    )}
                    
                    <p className="text-sm text-muted-foreground line-clamp-2">
                      {modpack.description}
                    </p>

                    <div className="flex items-center justify-between text-sm text-muted-foreground">
                      <div className="flex items-center space-x-4">
                        <span className="flex items-center space-x-1">
                          <Download className="h-3 w-3" />
                          <span>{formatDownloads(modpack.downloads)}</span>
                        </span>
                        <span className="flex items-center space-x-1">
                          <Calendar className="h-3 w-3" />
                          <span>{formatDate(modpack.updated_at)}</span>
                        </span>
                      </div>
                      <span className="text-xs bg-muted px-2 py-1 rounded">
                        v{modpack.version}
                      </span>
                    </div>

                    <div className="grid grid-cols-3 gap-2 text-center">
                      <div className="bg-muted rounded-lg p-2">
                        <div className="text-lg font-semibold">{modpack.total_mods}</div>
                        <div className="text-xs text-muted-foreground">Total Mods</div>
                      </div>
                      <div className="bg-muted rounded-lg p-2">
                        <div className="text-lg font-semibold">{modpack.server_mods}</div>
                        <div className="text-xs text-muted-foreground">Server</div>
                      </div>
                      <div className="bg-muted rounded-lg p-2">
                        <div className="text-lg font-semibold">{modpack.client_mods}</div>
                        <div className="text-xs text-muted-foreground">Client</div>
                      </div>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="flex items-center space-x-2">
                        <Badge variant="outline" className="text-xs">
                          {modpack.loader}
                        </Badge>
                        <Badge variant="outline" className="text-xs">
                          {modpack.minecraft_version}
                        </Badge>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => window.open(`https://${modpack.source}.com/modpack/${modpack.id}`, '_blank')}
                        >
                          <ExternalLink className="h-3 w-3" />
                        </Button>
                        <Button
                          size="sm"
                          onClick={() => handleModpackInstall(modpack)}
                          disabled={installingModpacks.has(modpack.id)}
                        >
                          {installingModpacks.has(modpack.id) ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <Plus className="h-3 w-3" />
                          )}
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}

          {/* Pagination */}
          {pagination.total_pages > 1 && (
            <div className="flex items-center justify-center space-x-2">
              <Button
                variant="outline"
                onClick={() => handlePageChange(pagination.page - 1)}
                disabled={pagination.page <= 1}
              >
                Previous
              </Button>
              <span className="text-sm text-muted-foreground">
                Page {pagination.page} of {pagination.total_pages}
              </span>
              <Button
                variant="outline"
                onClick={() => handlePageChange(pagination.page + 1)}
                disabled={pagination.page >= pagination.total_pages}
              >
                Next
              </Button>
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* Server Selection Modal */}
      <ServerSelectionModal
        isOpen={serverSelectionModal.isOpen}
        onClose={() => setServerSelectionModal(prev => ({ ...prev, isOpen: false }))}
        onInstall={handleServerSelectionInstall}
        title={`Install ${serverSelectionModal.type === 'mod' ? 'Mod' : 'Modpack'}`}
        description={`Select a server to install ${serverSelectionModal.item?.name || ''} to.`}
        itemType={serverSelectionModal.type}
        itemName={serverSelectionModal.item?.name || ''}
        itemVersion={serverSelectionModal.item?.version || ''}
        itemSource={serverSelectionModal.item?.source || 'modrinth'}
      />
    </div>
  );
};

export default ModBrowser;
