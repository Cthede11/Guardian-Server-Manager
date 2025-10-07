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
import { apiClient as api } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useServers } from '@/store/servers-new';
import { ModsTable } from '@/components/Tables/ModsTable';
import { RulesTable } from '@/components/Tables/RulesTable';
import { ConflictsList } from '@/components/ModsRules/ConflictsList';
import { CompatibilityPage } from '@/components/Compatibility/CompatibilityPage';
import { AnalyticsPage } from '@/components/Analytics/AnalyticsPage';
import { ErrorEmptyState, NoModsEmptyState, NoRulesEmptyState, NoConflictsEmptyState } from '@/components/ui/EmptyState';
import { ModsTableLoading, TableLoading } from '@/components/ui/LoadingStates';
import { LiveRuleLab } from '@/components/ModsRules/LiveRuleLab';

interface ModsRulesPageProps {
  className?: string;
}

export const ModsRules: React.FC<ModsRulesPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServers();
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
      const response = await api.getMods(serverId);
      if (response.ok && response.data) {
        setMods(response.data as any[]);
      } else {
        console.error('Failed to fetch mods:', response.error);
        setMods([]);
      }
    } catch (error) {
      console.error('Error fetching mods:', error);
      setMods([]);
    } finally {
      setIsLoading(false);
    }
  };

  // Fetch rules data
  const fetchRules = async () => {
    if (!serverId) return;
    
    try {
      const response = await api.getRules(serverId);
      if (response.ok && response.data) {
        setRules(response.data as any[]);
      } else {
        console.error('Failed to fetch rules:', response.error);
        setRules([]);
      }
    } catch (error) {
      console.error('Error fetching rules:', error);
      setRules([]);
    }
  };

  // Fetch conflicts data
  const fetchConflicts = async () => {
    if (!serverId) return;
    
    try {
      const response = await api.getConflicts(serverId);
      if (response.ok && response.data) {
        setConflicts(response.data as any[]);
      } else {
        console.error('Failed to fetch conflicts:', response.error);
        setConflicts([]);
      }
    } catch (error) {
      console.error('Error fetching conflicts:', error);
      setConflicts([]);
    }
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
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its mods and rules."
        />
      </div>
    );
  }

  if (isLoading && mods.length === 0) {
    return (
      <div className={`p-6 space-y-6 ${className}`}>
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h2 className="text-2xl font-bold">Mods & Rules</h2>
            <div className="flex items-center gap-2">
              <Badge variant="outline" className="flex items-center gap-1">
                <Package className="h-3 w-3" />
                Loading...
              </Badge>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="outline"
              disabled={true}
            >
              <TestTube className="h-4 w-4 mr-2" />
              Live Rule Lab
            </Button>
            <Button
              size="sm"
              variant="outline"
              disabled={true}
            >
              <RefreshCw className="h-4 w-4 animate-spin" />
              Loading...
            </Button>
          </div>
        </div>

        {/* Loading Content */}
        <ModsTableLoading count={5} />
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
          <CompatibilityPage serverId={serverId || ''} />
        </TabsContent>

        <TabsContent value="analytics" className="space-y-4">
          <AnalyticsPage serverId={serverId || ''} />
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
