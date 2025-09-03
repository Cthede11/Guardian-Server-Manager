import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { 
  Package, 
  AlertTriangle, 
  Settings, 
  RefreshCw,
  Search,
  Filter,
  Plus,
  TestTube,
  FileText,
  Shield,
  Zap,
  Eye,
  MoreHorizontal
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useServersStore } from '@/store/servers';
import { ModsTable } from '@/components/Tables/ModsTable';
import { RulesTable } from '@/components/Tables/RulesTable';
import { ConflictsList } from '@/components/ModsRules/ConflictsList';
import { LiveRuleLab } from '@/components/ModsRules/LiveRuleLab';

interface ModsRulesPageProps {
  className?: string;
}

export const ModsRules: React.FC<ModsRulesPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [mods, setMods] = useState<any[]>([]);
  const [rules, setRules] = useState<any[]>([]);
  const [conflicts, setConflicts] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState('all');
  const [liveRuleLabOpen, setLiveRuleLabOpen] = useState(false);

  // Fetch mods data
  const fetchMods = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const response = await fetch(`/api/v1/servers/${serverId}/mods`);
      if (response.ok) {
        const data = await response.json();
        setMods(data);
      } else {
        // Use mock data for demo
        setMods(generateMockMods());
      }
    } catch (error) {
      console.error('Error fetching mods:', error);
      // Use mock data for demo
      setMods(generateMockMods());
    } finally {
      setIsLoading(false);
    }
  };

  // Fetch rules data
  const fetchRules = async () => {
    if (!serverId) return;
    
    try {
      const response = await fetch(`/api/v1/servers/${serverId}/rules`);
      if (response.ok) {
        const data = await response.json();
        setRules(data);
      } else {
        // Use mock data for demo
        setRules(generateMockRules());
      }
    } catch (error) {
      console.error('Error fetching rules:', error);
      // Use mock data for demo
      setRules(generateMockRules());
    }
  };

  // Fetch conflicts data
  const fetchConflicts = async () => {
    if (!serverId) return;
    
    try {
      const response = await fetch(`/api/v1/servers/${serverId}/mods/conflicts`);
      if (response.ok) {
        const data = await response.json();
        setConflicts(data);
      } else {
        // Use mock data for demo
        setConflicts(generateMockConflicts());
      }
    } catch (error) {
      console.error('Error fetching conflicts:', error);
      // Use mock data for demo
      setConflicts(generateMockConflicts());
    }
  };

  // Generate mock mods for demo
  const generateMockMods = () => {
    return [
      {
        id: '1',
        name: 'JEI',
        version: '11.6.0.1015',
        author: 'mezz',
        description: 'Just Enough Items - View items and recipes',
        status: 'enabled',
        category: 'utility',
        dependencies: ['forge'],
        conflicts: [],
        loadTime: 45,
        memoryUsage: 12.5,
        lastUpdated: new Date(Date.now() - 86400000)
      },
      {
        id: '2',
        name: 'OptiFine',
        version: 'HD_U_I1',
        author: 'sp614x',
        description: 'OptiFine - Minecraft optimization mod',
        status: 'enabled',
        category: 'optimization',
        dependencies: ['forge'],
        conflicts: ['Sodium'],
        loadTime: 120,
        memoryUsage: 8.2,
        lastUpdated: new Date(Date.now() - 172800000)
      },
      {
        id: '3',
        name: 'Sodium',
        version: '0.4.10',
        author: 'jellysquid3',
        description: 'Sodium - Modern rendering engine',
        status: 'disabled',
        category: 'optimization',
        dependencies: ['fabric'],
        conflicts: ['OptiFine'],
        loadTime: 0,
        memoryUsage: 0,
        lastUpdated: new Date(Date.now() - 259200000)
      },
      {
        id: '4',
        name: 'Create',
        version: '0.5.1f',
        author: 'simibubi',
        description: 'Create - Building and automation mod',
        status: 'enabled',
        category: 'content',
        dependencies: ['forge', 'flywheel'],
        conflicts: [],
        loadTime: 280,
        memoryUsage: 45.8,
        lastUpdated: new Date(Date.now() - 432000000)
      },
      {
        id: '5',
        name: 'Applied Energistics 2',
        version: '12.0.0',
        author: 'AlgorithmX2',
        description: 'AE2 - Advanced storage and automation',
        status: 'enabled',
        category: 'content',
        dependencies: ['forge'],
        conflicts: [],
        loadTime: 195,
        memoryUsage: 32.1,
        lastUpdated: new Date(Date.now() - 604800000)
      }
    ];
  };

  // Generate mock rules for demo
  const generateMockRules = () => {
    return [
      {
        id: '1',
        name: 'TPS Protection',
        description: 'Automatically freeze chunks when TPS drops below 15',
        enabled: true,
        priority: 1,
        conditions: [
          { type: 'tps', operator: '<', value: 15 },
          { type: 'chunk_entities', operator: '>', value: 50 }
        ],
        actions: [
          { type: 'freeze_chunk', duration: 300 }
        ],
        lastTriggered: new Date(Date.now() - 1800000),
        triggerCount: 12
      },
      {
        id: '2',
        name: 'Entity Limit',
        description: 'Limit entities per chunk to prevent lag',
        enabled: true,
        priority: 2,
        conditions: [
          { type: 'chunk_entities', operator: '>', value: 100 }
        ],
        actions: [
          { type: 'despawn_entities', count: 20 }
        ],
        lastTriggered: new Date(Date.now() - 3600000),
        triggerCount: 8
      },
      {
        id: '3',
        name: 'Redstone Throttle',
        description: 'Throttle redstone updates in high-load areas',
        enabled: false,
        priority: 3,
        conditions: [
          { type: 'redstone_updates', operator: '>', value: 1000 }
        ],
        actions: [
          { type: 'throttle_redstone', factor: 0.5 }
        ],
        lastTriggered: null,
        triggerCount: 0
      }
    ];
  };

  // Generate mock conflicts for demo
  const generateMockConflicts = () => {
    return [
      {
        id: '1',
        type: 'mod_conflict',
        severity: 'high',
        mods: ['OptiFine', 'Sodium'],
        description: 'OptiFine and Sodium are incompatible rendering mods',
        resolution: 'disable_one',
        suggestedAction: 'Disable Sodium (OptiFine is already enabled)',
        impact: 'Server crashes on startup'
      },
      {
        id: '2',
        type: 'dependency_missing',
        severity: 'medium',
        mods: ['Create'],
        description: 'Create requires Flywheel but it is not installed',
        resolution: 'install_dependency',
        suggestedAction: 'Install Flywheel mod',
        impact: 'Create features may not work properly'
      },
      {
        id: '3',
        type: 'version_mismatch',
        severity: 'low',
        mods: ['JEI'],
        description: 'JEI version may not be compatible with current Forge version',
        resolution: 'update_mod',
        suggestedAction: 'Update JEI to latest version',
        impact: 'Minor compatibility issues'
      }
    ];
  };

  useEffect(() => {
    fetchMods();
    fetchRules();
    fetchConflicts();
    
    // Refresh data every 60 seconds
    const interval = setInterval(() => {
      fetchMods();
      fetchRules();
      fetchConflicts();
    }, 60000);
    return () => clearInterval(interval);
  }, [serverId]);

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view mods & rules</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Mods & Rules</h2>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <Package className="h-3 w-3" />
              {mods.filter(m => m.status === 'enabled').length} Enabled
            </Badge>
            {conflicts.length > 0 && (
              <Badge variant="destructive" className="flex items-center gap-1">
                <AlertTriangle className="h-3 w-3" />
                {conflicts.length} Conflicts
              </Badge>
            )}
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setLiveRuleLabOpen(true)}
          >
            <TestTube className="h-4 w-4 mr-2" />
            Live Rule Lab
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              fetchMods();
              fetchRules();
              fetchConflicts();
            }}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search mods and rules..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Items</SelectItem>
            <SelectItem value="enabled">Enabled Only</SelectItem>
            <SelectItem value="disabled">Disabled Only</SelectItem>
            <SelectItem value="conflicts">Conflicts Only</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="mods" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="mods" className="flex items-center gap-2">
            <Package className="h-4 w-4" />
            Mods
          </TabsTrigger>
          <TabsTrigger value="rules" className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            Rules
          </TabsTrigger>
          <TabsTrigger value="conflicts" className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4" />
            Conflicts
            {conflicts.length > 0 && (
              <Badge variant="destructive" className="ml-1 text-xs">
                {conflicts.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="analytics" className="flex items-center gap-2">
            <Zap className="h-4 w-4" />
            Analytics
          </TabsTrigger>
        </TabsList>

        <TabsContent value="mods" className="space-y-4">
          <ModsTable
            mods={mods}
            searchQuery={searchQuery}
            filterType={filterType}
            onModToggle={(modId) => {
              setMods(prev => prev.map(mod => 
                mod.id === modId 
                  ? { ...mod, status: mod.status === 'enabled' ? 'disabled' : 'enabled' }
                  : mod
              ));
            }}
            onModConfigure={(modId) => {
              console.log('Configure mod:', modId);
            }}
          />
        </TabsContent>

        <TabsContent value="rules" className="space-y-4">
          <RulesTable
            rules={rules}
            searchQuery={searchQuery}
            onRuleToggle={(ruleId) => {
              setRules(prev => prev.map(rule => 
                rule.id === ruleId 
                  ? { ...rule, enabled: !rule.enabled }
                  : rule
              ));
            }}
            onRuleEdit={(ruleId) => {
              console.log('Edit rule:', ruleId);
            }}
            onRuleDelete={(ruleId) => {
              setRules(prev => prev.filter(rule => rule.id !== ruleId));
            }}
          />
        </TabsContent>

        <TabsContent value="conflicts" className="space-y-4">
          <ConflictsList
            conflicts={conflicts}
            onResolve={(conflictId) => {
              setConflicts(prev => prev.filter(conflict => conflict.id !== conflictId));
            }}
            onIgnore={(conflictId) => {
              setConflicts(prev => prev.map(conflict => 
                conflict.id === conflictId 
                  ? { ...conflict, ignored: true }
                  : conflict
              ));
            }}
          />
        </TabsContent>

        <TabsContent value="analytics" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Total Load Time</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {mods.reduce((sum, mod) => sum + mod.loadTime, 0)}ms
                </div>
                <p className="text-xs text-muted-foreground">
                  Combined mod initialization time
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Memory Usage</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {mods.reduce((sum, mod) => sum + mod.memoryUsage, 0).toFixed(1)}MB
                </div>
                <p className="text-xs text-muted-foreground">
                  Total mod memory footprint
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Active Rules</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {rules.filter(rule => rule.enabled).length}
                </div>
                <p className="text-xs text-muted-foreground">
                  Rules currently monitoring
                </p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Rule Trigger History</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {rules.filter(rule => rule.triggerCount > 0).map(rule => (
                  <div key={rule.id} className="flex items-center justify-between p-2 border rounded">
                    <div>
                      <span className="font-medium">{rule.name}</span>
                      <p className="text-sm text-muted-foreground">{rule.description}</p>
                    </div>
                    <div className="text-right">
                      <div className="font-medium">{rule.triggerCount} triggers</div>
                      <div className="text-sm text-muted-foreground">
                        Last: {rule.lastTriggered ? new Date(rule.lastTriggered).toLocaleString() : 'Never'}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Live Rule Lab Modal */}
      <LiveRuleLab
        isOpen={liveRuleLabOpen}
        onClose={() => setLiveRuleLabOpen(false)}
        onSave={(rule) => {
          setRules(prev => [...prev, { ...rule, id: Date.now().toString() }]);
        }}
      />
    </div>
  );
};

export default ModsRules;
