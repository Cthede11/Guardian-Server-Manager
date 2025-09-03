import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  Cpu, 
  MemoryStick, 
  Settings as SettingsIcon,
  AlertTriangle,
  CheckCircle,
  Info,
  Zap,
  HardDrive,
  Activity,
  Shield,
  Network,
  Monitor,
  Layers,
  Database,
  FileText,
  Server,
  Users,
  Clock,
  RefreshCw,
  Package,
  Globe,
  Folder,
  FolderOpen,
  File,
  FolderPlus,
  FolderMinus,
  FolderX,
  FolderCheck,
  FolderAlert,
  FolderUp,
  FolderDown
} from 'lucide-react';

interface ComposerSettingsData {
  // Composer Settings
  enableComposer: boolean;
  composerMode: 'development' | 'production' | 'testing';
  composerTimeout: number;
  composerRetries: number;
  
  // Package Management
  packageManager: 'npm' | 'yarn' | 'pnpm' | 'bun';
  packageRegistry: string;
  packageCache: boolean;
  packageOffline: boolean;
  
  // Build Settings
  buildMode: 'debug' | 'release' | 'profile';
  buildOptimization: boolean;
  buildMinification: boolean;
  buildSourcemaps: boolean;
  
  // Dependencies
  dependencyResolution: 'strict' | 'loose' | 'auto';
  dependencyLocking: boolean;
  dependencyUpdates: boolean;
  dependencyAudit: boolean;
  
  // Scripts
  preBuildScript: string;
  postBuildScript: string;
  preStartScript: string;
  postStartScript: string;
  
  // Environment
  environment: 'development' | 'staging' | 'production';
  environmentVariables: string;
  environmentFiles: string;
  
  // Advanced
  enableComposerProfiling: boolean;
  composerProfilingInterval: number;
  enableComposerDebug: boolean;
  composerDebugLevel: 'none' | 'basic' | 'detailed' | 'verbose';
}

export const ComposerSettings: React.FC = () => {
  const [settings, setSettings] = useState<ComposerSettingsData>({
    // Composer Settings
    enableComposer: true,
    composerMode: 'development',
    composerTimeout: 300000,
    composerRetries: 3,
    
    // Package Management
    packageManager: 'npm',
    packageRegistry: 'https://registry.npmjs.org/',
    packageCache: true,
    packageOffline: false,
    
    // Build Settings
    buildMode: 'debug',
    buildOptimization: false,
    buildMinification: false,
    buildSourcemaps: true,
    
    // Dependencies
    dependencyResolution: 'strict',
    dependencyLocking: true,
    dependencyUpdates: false,
    dependencyAudit: true,
    
    // Scripts
    preBuildScript: '',
    postBuildScript: '',
    preStartScript: '',
    postStartScript: '',
    
    // Environment
    environment: 'development',
    environmentVariables: '',
    environmentFiles: '.env,.env.local,.env.development',
    
    // Advanced
    enableComposerProfiling: false,
    composerProfilingInterval: 60000,
    enableComposerDebug: false,
    composerDebugLevel: 'none'
  });
  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  const fetchSettings = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch composer settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const handleSettingChange = (key: keyof ComposerSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const getValidationStatus = (key: keyof ComposerSettingsData) => {
    const value = settings[key];
    
    switch (key) {
      case 'composerTimeout':
        return value < 1000 || value > 1800000 ? 'error' : 'success';
      case 'composerRetries':
        return value < 0 || value > 10 ? 'error' : 'success';
      case 'composerProfilingInterval':
        return value < 1000 || value > 600000 ? 'error' : 'success';
      default:
        return 'success';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'error': return <AlertTriangle className="h-4 w-4 text-red-500" />;
      case 'success': return <CheckCircle className="h-4 w-4 text-green-500" />;
      default: return <Info className="h-4 w-4 text-blue-500" />;
    }
  };

  const getModeColor = (mode: string) => {
    switch (mode) {
      case 'development': return 'bg-blue-500';
      case 'production': return 'bg-green-500';
      case 'testing': return 'bg-yellow-500';
      default: return 'bg-gray-500';
    }
  };

  const getModeLabel = (mode: string) => {
    switch (mode) {
      case 'development': return 'Development';
      case 'production': return 'Production';
      case 'testing': return 'Testing';
      default: return 'Unknown';
    }
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Composer Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Package className="h-5 w-5" />
            <span>Composer Settings</span>
          </CardTitle>
          <CardDescription>
            Configure package management and build settings
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableComposer">Enable Composer</Label>
                  <p className="text-sm text-muted-foreground">Enable package management</p>
                </div>
                <Switch
                  id="enableComposer"
                  checked={settings.enableComposer}
                  onCheckedChange={(checked) => handleSettingChange('enableComposer', checked)}
                />
              </div>
              
              {settings.enableComposer && (
                <>
                  <div>
                    <Label htmlFor="composerMode">Composer Mode</Label>
                    <Select value={settings.composerMode} onValueChange={(value) => handleSettingChange('composerMode', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="development">Development</SelectItem>
                        <SelectItem value="production">Production</SelectItem>
                        <SelectItem value="testing">Testing</SelectItem>
                      </SelectContent>
                    </Select>
                    <div className="flex items-center space-x-2 mt-2">
                      <div className={`w-3 h-3 rounded-full ${getModeColor(settings.composerMode)}`} />
                      <span className="text-sm text-muted-foreground">{getModeLabel(settings.composerMode)}</span>
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="composerTimeout">Composer Timeout (ms)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="composerTimeout"
                        type="number"
                        value={settings.composerTimeout}
                        onChange={(e) => handleSettingChange('composerTimeout', parseInt(e.target.value))}
                        min="1000"
                        max="1800000"
                      />
                      {getStatusIcon(getValidationStatus('composerTimeout'))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableComposer && (
                <>
                  <div>
                    <Label htmlFor="composerRetries">Composer Retries</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="composerRetries"
                        type="number"
                        value={settings.composerRetries}
                        onChange={(e) => handleSettingChange('composerRetries', parseInt(e.target.value))}
                        min="0"
                        max="10"
                      />
                      {getStatusIcon(getValidationStatus('composerRetries'))}
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="packageManager">Package Manager</Label>
                    <Select value={settings.packageManager} onValueChange={(value) => handleSettingChange('packageManager', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="npm">NPM</SelectItem>
                        <SelectItem value="yarn">Yarn</SelectItem>
                        <SelectItem value="pnpm">PNPM</SelectItem>
                        <SelectItem value="bun">Bun</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div>
                    <Label htmlFor="packageRegistry">Package Registry</Label>
                    <Input
                      id="packageRegistry"
                      value={settings.packageRegistry}
                      onChange={(e) => handleSettingChange('packageRegistry', e.target.value)}
                      placeholder="https://registry.npmjs.org/"
                    />
                  </div>
                </>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Build Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Zap className="h-5 w-5" />
            <span>Build Settings</span>
          </CardTitle>
          <CardDescription>
            Configure build process and optimization
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="buildMode">Build Mode</Label>
                <Select value={settings.buildMode} onValueChange={(value) => handleSettingChange('buildMode', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="debug">Debug</SelectItem>
                    <SelectItem value="release">Release</SelectItem>
                    <SelectItem value="profile">Profile</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="buildOptimization">Build Optimization</Label>
                  <p className="text-sm text-muted-foreground">Enable build optimizations</p>
                </div>
                <Switch
                  id="buildOptimization"
                  checked={settings.buildOptimization}
                  onCheckedChange={(checked) => handleSettingChange('buildOptimization', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="buildMinification">Build Minification</Label>
                  <p className="text-sm text-muted-foreground">Enable code minification</p>
                </div>
                <Switch
                  id="buildMinification"
                  checked={settings.buildMinification}
                  onCheckedChange={(checked) => handleSettingChange('buildMinification', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="buildSourcemaps">Build Sourcemaps</Label>
                  <p className="text-sm text-muted-foreground">Generate source maps</p>
                </div>
                <Switch
                  id="buildSourcemaps"
                  checked={settings.buildSourcemaps}
                  onCheckedChange={(checked) => handleSettingChange('buildSourcemaps', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Dependencies */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Layers className="h-5 w-5" />
            <span>Dependencies</span>
          </CardTitle>
          <CardDescription>
            Configure dependency management and resolution
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="dependencyResolution">Dependency Resolution</Label>
                <Select value={settings.dependencyResolution} onValueChange={(value) => handleSettingChange('dependencyResolution', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="strict">Strict</SelectItem>
                    <SelectItem value="loose">Loose</SelectItem>
                    <SelectItem value="auto">Auto</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="dependencyLocking">Dependency Locking</Label>
                  <p className="text-sm text-muted-foreground">Lock dependency versions</p>
                </div>
                <Switch
                  id="dependencyLocking"
                  checked={settings.dependencyLocking}
                  onCheckedChange={(checked) => handleSettingChange('dependencyLocking', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="dependencyUpdates">Dependency Updates</Label>
                  <p className="text-sm text-muted-foreground">Enable automatic updates</p>
                </div>
                <Switch
                  id="dependencyUpdates"
                  checked={settings.dependencyUpdates}
                  onCheckedChange={(checked) => handleSettingChange('dependencyUpdates', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="dependencyAudit">Dependency Audit</Label>
                  <p className="text-sm text-muted-foreground">Enable security auditing</p>
                </div>
                <Switch
                  id="dependencyAudit"
                  checked={settings.dependencyAudit}
                  onCheckedChange={(checked) => handleSettingChange('dependencyAudit', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Scripts */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <FileText className="h-5 w-5" />
            <span>Scripts</span>
          </CardTitle>
          <CardDescription>
            Configure build and lifecycle scripts
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="preBuildScript">Pre-build Script</Label>
                <Input
                  id="preBuildScript"
                  value={settings.preBuildScript}
                  onChange={(e) => handleSettingChange('preBuildScript', e.target.value)}
                  placeholder="npm run lint"
                />
              </div>
              
              <div>
                <Label htmlFor="postBuildScript">Post-build Script</Label>
                <Input
                  id="postBuildScript"
                  value={settings.postBuildScript}
                  onChange={(e) => handleSettingChange('postBuildScript', e.target.value)}
                  placeholder="npm run test"
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="preStartScript">Pre-start Script</Label>
                <Input
                  id="preStartScript"
                  value={settings.preStartScript}
                  onChange={(e) => handleSettingChange('preStartScript', e.target.value)}
                  placeholder="npm run migrate"
                />
              </div>
              
              <div>
                <Label htmlFor="postStartScript">Post-start Script</Label>
                <Input
                  id="postStartScript"
                  value={settings.postStartScript}
                  onChange={(e) => handleSettingChange('postStartScript', e.target.value)}
                  placeholder="npm run seed"
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Environment */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Globe className="h-5 w-5" />
            <span>Environment</span>
          </CardTitle>
          <CardDescription>
            Configure environment variables and files
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="environment">Environment</Label>
                <Select value={settings.environment} onValueChange={(value) => handleSettingChange('environment', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="development">Development</SelectItem>
                    <SelectItem value="staging">Staging</SelectItem>
                    <SelectItem value="production">Production</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="environmentFiles">Environment Files</Label>
                <Input
                  id="environmentFiles"
                  value={settings.environmentFiles}
                  onChange={(e) => handleSettingChange('environmentFiles', e.target.value)}
                  placeholder=".env,.env.local,.env.development"
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="environmentVariables">Environment Variables</Label>
                <Textarea
                  id="environmentVariables"
                  value={settings.environmentVariables}
                  onChange={(e) => handleSettingChange('environmentVariables', e.target.value)}
                  placeholder="NODE_ENV=development&#10;PORT=3000&#10;DEBUG=true"
                  rows={4}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
