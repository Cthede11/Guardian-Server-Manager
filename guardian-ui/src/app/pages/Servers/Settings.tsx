import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Settings as SettingsIcon, 
  Save, 
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Server,
  Cpu,
  Zap,
  HardDrive,
  FolderOpen,
  Code,
  Key,
  Monitor,
  MemoryStick,
  Network,
  Shield,
  Clock,
  Activity
} from 'lucide-react';
import { GeneralSettings } from '@/components/Settings/GeneralSettings';
import { JVMSettings } from '@/components/Settings/JVMSettings';
import { GPUSettings } from '@/components/Settings/GPUSettings';
import { HASettings } from '@/components/Settings/HASettings';
import { PathsSettings } from '@/components/Settings/PathsSettings';
import { ComposerSettings } from '@/components/Settings/ComposerSettings';
import { TokensSettings } from '@/components/Settings/TokensSettings';

interface SettingsStats {
  totalSettings: number;
  modifiedSettings: number;
  criticalSettings: number;
  lastModified: string;
  hasUnsavedChanges: boolean;
  validationErrors: number;
}

export const Settings: React.FC = () => {
  const [activeTab, setActiveTab] = useState('general');
  const [stats, setStats] = useState<SettingsStats>({
    totalSettings: 0,
    modifiedSettings: 0,
    criticalSettings: 0,
    lastModified: 'Never',
    hasUnsavedChanges: false,
    validationErrors: 0
  });
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date());

  const fetchStats = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setStats({
        totalSettings: 47,
        modifiedSettings: 3,
        criticalSettings: 2,
        lastModified: new Date(Date.now() - 1800000).toISOString(),
        hasUnsavedChanges: true,
        validationErrors: 1
      });
      
      setLastRefresh(new Date());
    } catch (error) {
      console.error('Failed to fetch settings stats:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
    
    // Auto-refresh every 60 seconds
    const interval = setInterval(fetchStats, 60000);
    return () => clearInterval(interval);
  }, []);

  const handleRefresh = () => {
    fetchStats();
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      setStats(prev => ({
        ...prev,
        hasUnsavedChanges: false,
        modifiedSettings: 0,
        validationErrors: 0,
        lastModified: new Date().toISOString()
      }));
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      setIsSaving(false);
    }
  };

  const getStatusColor = () => {
    if (stats.validationErrors > 0) return 'destructive';
    if (stats.hasUnsavedChanges) return 'default';
    return 'secondary';
  };

  const getStatusIcon = () => {
    if (stats.validationErrors > 0) return <XCircle className="h-4 w-4 text-red-500" />;
    if (stats.hasUnsavedChanges) return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
    return <CheckCircle className="h-4 w-4 text-green-500" />;
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Server Settings</h1>
          <p className="text-muted-foreground">
            Configure server parameters, JVM options, and system settings
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button
            onClick={handleSave}
            disabled={isSaving || !stats.hasUnsavedChanges}
            size="sm"
          >
            {isSaving ? (
              <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <Save className="h-4 w-4 mr-2" />
            )}
            Save Changes
          </Button>
        </div>
      </div>

      {/* Status Alert */}
      {stats.hasUnsavedChanges && (
        <Alert variant={getStatusColor()}>
          {getStatusIcon()}
          <AlertDescription>
            {stats.validationErrors > 0 
              ? `You have ${stats.validationErrors} validation error(s) that must be fixed before saving.`
              : `You have ${stats.modifiedSettings} unsaved change(s). Don't forget to save your changes.`
            }
          </AlertDescription>
        </Alert>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Settings</CardTitle>
            <SettingsIcon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalSettings}</div>
            <p className="text-xs text-muted-foreground">
              {stats.criticalSettings} critical
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Modified</CardTitle>
            {getStatusIcon()}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.modifiedSettings}</div>
            <p className="text-xs text-muted-foreground">
              Unsaved changes
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Status</CardTitle>
            {getStatusIcon()}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.validationErrors > 0 ? 'Error' : stats.hasUnsavedChanges ? 'Modified' : 'Saved'}
            </div>
            <p className="text-xs text-muted-foreground">
              {stats.validationErrors > 0 ? 'Validation errors' : 'Last saved'}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Last Modified</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.lastModified === 'Never' 
                ? 'Never' 
                : new Date(stats.lastModified).toLocaleTimeString()
              }
            </div>
            <p className="text-xs text-muted-foreground">
              {stats.lastModified === 'Never' 
                ? 'No changes' 
                : new Date(stats.lastModified).toLocaleDateString()
              }
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1">
        <TabsList className="grid w-full grid-cols-7">
          <TabsTrigger value="general" className="flex items-center space-x-2">
            <Server className="h-4 w-4" />
            <span>General</span>
          </TabsTrigger>
          <TabsTrigger value="jvm" className="flex items-center space-x-2">
            <Cpu className="h-4 w-4" />
            <span>JVM</span>
          </TabsTrigger>
          <TabsTrigger value="gpu" className="flex items-center space-x-2">
            <Monitor className="h-4 w-4" />
            <span>GPU</span>
          </TabsTrigger>
          <TabsTrigger value="ha" className="flex items-center space-x-2">
            <Shield className="h-4 w-4" />
            <span>HA</span>
          </TabsTrigger>
          <TabsTrigger value="paths" className="flex items-center space-x-2">
            <FolderOpen className="h-4 w-4" />
            <span>Paths</span>
          </TabsTrigger>
          <TabsTrigger value="composer" className="flex items-center space-x-2">
            <Code className="h-4 w-4" />
            <span>Composer</span>
          </TabsTrigger>
          <TabsTrigger value="tokens" className="flex items-center space-x-2">
            <Key className="h-4 w-4" />
            <span>Tokens</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="flex-1">
          <GeneralSettings />
        </TabsContent>

        <TabsContent value="jvm" className="flex-1">
          <JVMSettings />
        </TabsContent>

        <TabsContent value="gpu" className="flex-1">
          <GPUSettings />
        </TabsContent>

        <TabsContent value="ha" className="flex-1">
          <HASettings />
        </TabsContent>

        <TabsContent value="paths" className="flex-1">
          <PathsSettings />
        </TabsContent>

        <TabsContent value="composer" className="flex-1">
          <ComposerSettings />
        </TabsContent>

        <TabsContent value="tokens" className="flex-1">
          <TokensSettings />
        </TabsContent>
      </Tabs>
    </div>
  );
};
