import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { 
  Package, 
  Search, 
  Plus, 
  X, 
  Download,
  ExternalLink,
  Loader2,
  AlertTriangle,
  CheckCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { apiClient as api } from '@/lib/api';
import { useToast } from '@/hooks/use-toast';
import { type ServerFormData } from '@/lib/validation/server-schema';

interface StepModsProps {
  formData: ServerFormData;
  updateFormData: (updates: Partial<ServerFormData>) => void;
  errors: Record<string, string>;
  versions: string[];
  isLoadingVersions: boolean;
  onValidate: () => boolean;
}

interface Modpack {
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
}

interface Mod {
  id: string;
  name: string;
  description: string;
  version: string;
  downloads: number;
  author: string;
  logo_url?: string;
  categories: string[];
  server_safe: boolean;
}

export const StepMods: React.FC<StepModsProps> = ({
  formData,
  updateFormData,
  errors,
  versions,
  isLoadingVersions,
  onValidate
}) => {
  const [activeTab, setActiveTab] = useState<'modpack' | 'individual'>('modpack');
  const [modpackSearch, setModpackSearch] = useState('');
  const [modSearch, setModSearch] = useState('');
  const [modpackResults, setModpackResults] = useState<Modpack[]>([]);
  const [modResults, setModResults] = useState<Mod[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedModpack, setSelectedModpack] = useState<Modpack | null>(null);
  const [selectedMods, setSelectedMods] = useState<Mod[]>([]);
  const [searchProvider, setSearchProvider] = useState<'modrinth' | 'curseforge'>('modrinth');
  const { toast } = useToast();

  // Search modpacks
  const searchModpacks = async (query: string) => {
    if (!query.trim()) {
      setModpackResults([]);
      return;
    }

    setIsSearching(true);
    try {
      const response = await api.call<{success: boolean, data: {modpacks: Modpack[]}, error: string}>(`/api/modpacks?query=${encodeURIComponent(query)}&provider=${searchProvider}`);
      console.log('Modpack search response:', response);
      
      // Handle API response structure
      let modpacks: Modpack[] = [];
      if (response.success && response.data && response.data.modpacks) {
        modpacks = response.data.modpacks;
      }
      
      setModpackResults(modpacks);
      
      // Show message if no results found
      if (modpacks.length === 0) {
        toast({
          title: "No Modpacks Found",
          description: "No modpacks found for your search. Try a different search term or check back later.",
          variant: "default"
        });
      }
    } catch (error) {
      console.error('Failed to search modpacks:', error);
      toast({
        title: "Search Error",
        description: "Failed to search modpacks. The search feature is currently being developed.",
        variant: "destructive"
      });
    } finally {
      setIsSearching(false);
    }
  };

  // Search individual mods
  const searchMods = async (query: string) => {
    if (!query.trim()) {
      setModResults([]);
      return;
    }

    setIsSearching(true);
    try {
      const response = await api.call<{success: boolean, data: {mods: Mod[]}, error: string}>(`/api/modpacks/mods?query=${encodeURIComponent(query)}&provider=${searchProvider}&server_safe=true`);
      console.log('Mod search response:', response);
      
      // Handle API response structure
      let mods: Mod[] = [];
      if (response.success && response.data && response.data.mods) {
        mods = response.data.mods;
      }
      
      setModResults(mods);
      
      // Show message if no results found
      if (mods.length === 0) {
        toast({
          title: "No Mods Found",
          description: "No mods found for your search. Try a different search term or check back later.",
          variant: "default"
        });
      }
    } catch (error) {
      console.error('Failed to search mods:', error);
      toast({
        title: "Search Error",
        description: "Failed to search mods. The search feature is currently being developed.",
        variant: "destructive"
      });
    } finally {
      setIsSearching(false);
    }
  };

  // Debounced search
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (activeTab === 'modpack') {
        searchModpacks(modpackSearch);
      } else {
        searchMods(modSearch);
      }
    }, 500);

    return () => clearTimeout(timeoutId);
  }, [modpackSearch, modSearch, activeTab, searchProvider]);

  const handleModpackSelect = (modpack: Modpack) => {
    setSelectedModpack(modpack);
    updateFormData({
      modpack: {
        source: searchProvider,
        packId: modpack.id,
        packVersionId: 'latest', // Use latest since version is not available
        serverOnly: true
      },
      individualMods: [] // Clear individual mods when selecting modpack
    });
  };

  const handleModAdd = (mod: Mod) => {
    if (!selectedMods.find(m => m.id === mod.id)) {
      const newSelectedMods = [...selectedMods, mod];
      setSelectedMods(newSelectedMods);
      updateFormData({
        individualMods: newSelectedMods.map(m => ({
          provider: searchProvider,
          modId: m.id,
          fileId: m.version
        })),
        modpack: undefined // Clear modpack when selecting individual mods
      });
    }
  };

  const handleModRemove = (modId: string) => {
    const newSelectedMods = selectedMods.filter(m => m.id !== modId);
    setSelectedMods(newSelectedMods);
    updateFormData({
      individualMods: newSelectedMods.map(m => ({
        provider: searchProvider,
        modId: m.id,
        fileId: m.version
      }))
    });
  };

  const clearModpack = () => {
    setSelectedModpack(null);
    updateFormData({ modpack: undefined });
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-semibold mb-2">Mods & Modpacks</h3>
        <p className="text-muted-foreground">
          Add mods or install a modpack to enhance your server (optional)
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as 'modpack' | 'individual')}>
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="modpack">Modpack</TabsTrigger>
          <TabsTrigger value="individual">Individual Mods</TabsTrigger>
        </TabsList>

        <TabsContent value="modpack" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Package className="h-5 w-5" />
                Modpack Selection
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-3">
                <Select value={searchProvider} onValueChange={(value) => setSearchProvider(value as 'modrinth' | 'curseforge')}>
                  <SelectTrigger className="w-36 bg-background border-2 border-muted-foreground/20 hover:border-primary/50 focus:border-primary">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="modrinth">Modrinth</SelectItem>
                    <SelectItem value="curseforge">CurseForge</SelectItem>
                  </SelectContent>
                </Select>
                <div className="flex-1 relative">
                  <Input
                    placeholder="Search modpacks..."
                    value={modpackSearch}
                    onChange={(e) => setModpackSearch(e.target.value)}
                    className="w-full bg-background border-2 border-muted-foreground/20 hover:border-primary/50 focus:border-primary pr-10"
                  />
                  {isSearching && (
                    <Loader2 className="h-4 w-4 animate-spin absolute right-3 top-1/2 transform -translate-y-1/2 text-primary" />
                  )}
                </div>
              </div>

              {/* Selected Modpack */}
              {selectedModpack && (
                <Alert>
                  <CheckCircle className="h-4 w-4" />
                  <AlertDescription className="flex items-center justify-between">
                    <div>
                      <strong>{selectedModpack.name}</strong> by {selectedModpack.author}
                      <div className="text-sm text-muted-foreground">
                        {selectedModpack.total_mods} mods ({selectedModpack.server_mods} server, {selectedModpack.client_mods} client)
                      </div>
                    </div>
                    <Button variant="outline" size="sm" onClick={clearModpack}>
                      <X className="h-4 w-4" />
                    </Button>
                  </AlertDescription>
                </Alert>
              )}

              {/* Modpack Results */}
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {modpackResults.length === 0 && modpackSearch.trim() && !isSearching && (
                  <div className="text-center py-8 text-muted-foreground">
                    <Package className="h-12 w-12 mx-auto mb-4 opacity-50" />
                    <p className="text-sm">No modpacks found for "{modpackSearch}"</p>
                    <p className="text-xs mt-1">Try a different search term or check back later</p>
                  </div>
                )}
                {modpackResults.map((modpack) => (
                  <Card 
                    key={modpack.id} 
                    className={`cursor-pointer transition-colors hover:bg-accent ${
                      selectedModpack?.id === modpack.id ? 'ring-2 ring-primary' : ''
                    }`}
                    onClick={() => handleModpackSelect(modpack)}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-start gap-3">
                        {modpack.logo_url && (
                          <img 
                            src={modpack.logo_url} 
                            alt={modpack.name}
                            className="w-12 h-12 rounded object-cover"
                          />
                        )}
                        <div className="flex-1 min-w-0">
                          <h4 className="font-medium truncate">{modpack.name}</h4>
                          <p className="text-sm text-muted-foreground truncate">{modpack.description}</p>
                          <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                            <span>by {modpack.author}</span>
                            <span>{modpack.downloads.toLocaleString()} downloads</span>
                            <span>{modpack.total_mods} mods</span>
                          </div>
                        </div>
                        <Button variant="outline" size="sm" className="bg-primary/10 hover:bg-primary/20 border-primary/30 hover:border-primary/50">
                          <Plus className="h-4 w-4" />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>

              {selectedModpack && (
                <div className="space-y-2">
                  <Label>Installation Options</Label>
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="serverOnly"
                      checked={formData.modpack?.serverOnly || false}
                      onChange={(e) => updateFormData({
                        modpack: { 
                          ...formData.modpack, 
                          serverOnly: e.target.checked,
                          source: formData.modpack?.source || 'curseforge',
                          packId: formData.modpack?.packId || '',
                          packVersionId: formData.modpack?.packVersionId || ''
                        }
                      })}
                    />
                    <Label htmlFor="serverOnly" className="text-sm">
                      Install server subset only (recommended)
                    </Label>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="individual" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Search className="h-5 w-5" />
                Individual Mods
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-3">
                <Select value={searchProvider} onValueChange={(value) => setSearchProvider(value as 'modrinth' | 'curseforge')}>
                  <SelectTrigger className="w-36 bg-background border-2 border-muted-foreground/20 hover:border-primary/50 focus:border-primary">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="modrinth">Modrinth</SelectItem>
                    <SelectItem value="curseforge">CurseForge</SelectItem>
                  </SelectContent>
                </Select>
                <div className="flex-1 relative">
                  <Input
                    placeholder="Search mods..."
                    value={modSearch}
                    onChange={(e) => setModSearch(e.target.value)}
                    className="w-full bg-background border-2 border-muted-foreground/20 hover:border-primary/50 focus:border-primary pr-10"
                  />
                  {isSearching && (
                    <Loader2 className="h-4 w-4 animate-spin absolute right-3 top-1/2 transform -translate-y-1/2 text-primary" />
                  )}
                </div>
              </div>

              {/* Selected Mods */}
              {selectedMods.length > 0 && (
                <div className="space-y-2">
                  <Label>Selected Mods ({selectedMods.length})</Label>
                  <div className="flex flex-wrap gap-2">
                    {selectedMods.map((mod) => (
                      <Badge key={mod.id} variant="secondary" className="flex items-center gap-1">
                        {mod.name}
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-4 w-4 p-0 hover:bg-destructive hover:text-destructive-foreground"
                          onClick={() => handleModRemove(mod.id)}
                        >
                          <X className="h-3 w-3" />
                        </Button>
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {/* Mod Results */}
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {modResults.length === 0 && modSearch.trim() && !isSearching && (
                  <div className="text-center py-8 text-muted-foreground">
                    <Search className="h-12 w-12 mx-auto mb-4 opacity-50" />
                    <p className="text-sm">No mods found for "{modSearch}"</p>
                    <p className="text-xs mt-1">Try a different search term or check back later</p>
                  </div>
                )}
                {modResults.map((mod) => (
                  <Card 
                    key={mod.id} 
                    className={`cursor-pointer transition-colors hover:bg-accent ${
                      selectedMods.find(m => m.id === mod.id) ? 'ring-2 ring-primary' : ''
                    }`}
                    onClick={() => handleModAdd(mod)}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-start gap-3">
                        {mod.logo_url && (
                          <img 
                            src={mod.logo_url} 
                            alt={mod.name}
                            className="w-10 h-10 rounded object-cover"
                          />
                        )}
                        <div className="flex-1 min-w-0">
                          <h4 className="font-medium truncate">{mod.name}</h4>
                          <p className="text-sm text-muted-foreground truncate">{mod.description}</p>
                          <div className="flex items-center gap-2 mt-2">
                            <Badge variant="outline" className="text-xs">
                              {mod.version}
                            </Badge>
                            {mod.server_safe && (
                              <Badge variant="outline" className="text-xs text-green-600">
                                Server Safe
                              </Badge>
                            )}
                            <span className="text-xs text-muted-foreground">
                              {mod.downloads.toLocaleString()} downloads
                            </span>
                          </div>
                        </div>
                        <Button variant="outline" size="sm" className="bg-primary/10 hover:bg-primary/20 border-primary/30 hover:border-primary/50">
                          <Plus className="h-4 w-4" />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Summary */}
      {(selectedModpack || selectedMods.length > 0) && (
        <Card>
          <CardHeader>
            <CardTitle>Selection Summary</CardTitle>
          </CardHeader>
          <CardContent>
            {selectedModpack && (
              <div className="space-y-2">
                <h4 className="font-medium">Modpack: {selectedModpack.name}</h4>
                <p className="text-sm text-muted-foreground">
                  {selectedModpack.total_mods} mods • {selectedModpack.server_mods} server-side
                  {formData.modpack?.serverOnly && ' • Server subset only'}
                </p>
              </div>
            )}
            {selectedMods.length > 0 && (
              <div className="space-y-2">
                <h4 className="font-medium">Individual Mods: {selectedMods.length}</h4>
                <p className="text-sm text-muted-foreground">
                  {selectedMods.filter(m => m.server_safe).length} server-safe mods
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
};
