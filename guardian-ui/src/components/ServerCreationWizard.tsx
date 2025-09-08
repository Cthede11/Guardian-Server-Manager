import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Server, 
  Cpu, 
  Network, 
  FolderOpen, 
  Settings, 
  CheckCircle, 
  AlertTriangle,
  Loader2
} from 'lucide-react';
import { fileManager, type ServerConfig } from '@/lib/file-manager';
import { settingsManager } from '@/lib/settings-manager';
import { errorHandler } from '@/lib/error-handler';
import { getVersionsForModpack } from '@/lib/constants/minecraft-versions';

interface ServerCreationWizardProps {
  onClose: () => void;
  onServerCreated: (server: ServerConfig) => void;
}

const steps = [
  { id: 1, title: 'Basic Info', description: 'Server name and type', icon: Server },
  { id: 2, title: 'Java Config', description: 'Java version and memory', icon: Cpu },
  { id: 3, title: 'Network', description: 'Ports and RCON', icon: Network },
  { id: 4, title: 'File Paths', description: 'Directory structure', icon: FolderOpen },
  { id: 5, title: 'Advanced', description: 'Auto-start and backups', icon: Settings }
];

export const ServerCreationWizard: React.FC<ServerCreationWizardProps> = ({ 
  onClose, 
  onServerCreated 
}) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [javaInstallations, setJavaInstallations] = useState<any[]>([]);
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

  useEffect(() => {
    loadJavaInstallations();
    loadDefaultSettings();
  }, []);

  const loadJavaInstallations = async () => {
    try {
      const config = await fileManager.getConfig();
      if (config) {
        setJavaInstallations(config.javaInstallations);
      }
    } catch (error) {
      errorHandler.handleError(error as Error, 'Load Java Installations');
    }
  };

  const loadDefaultSettings = async () => {
    try {
      const settings = await settingsManager.getAppSettings();
      setFormData(prev => ({
        ...prev,
        javaArgs: settings.servers.defaultJavaArgs,
        memory: settings.servers.defaultMemory,
        javaPath: settings.servers.defaultJavaPath
      }));
    } catch (error) {
      errorHandler.handleError(error as Error, 'Load Default Settings');
    }
  };

  const nextStep = () => {
    if (currentStep < steps.length) {
      setCurrentStep(currentStep + 1);
    }
  };

  const prevStep = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    
    try {
      const serverId = generateId();
      const now = new Date().toISOString();
      
      const serverConfig: ServerConfig = {
        id: serverId,
        name: formData.name,
        type: formData.type,
        version: formData.version,
        loader: formData.type,
        java: {
          path: formData.javaPath,
          args: formData.javaArgs,
          version: '21' // Default Java version
        },
        network: {
          serverPort: formData.serverPort,
          rconPort: formData.rconPort,
          rconPassword: formData.rconPassword,
          queryPort: formData.queryPort
        },
        paths: {
          world: formData.paths.world,
          mods: formData.paths.mods,
          config: formData.paths.config,
          logs: formData.paths.logs,
          backups: formData.paths.backups
        },
        settings: {
          autoStart: false,
          autoRestart: true,
          maxRestarts: 3,
          backupInterval: 24,
          backupRetention: 7,
          memory: formData.memory,
          javaArgs: formData.javaArgs
        },
        created: now,
        lastModified: now
      };

      // Create server directory and files
      await fileManager.createServerDirectory(serverId, serverConfig);
      
      // Create server settings
      await settingsManager.createServerSettings(serverId, {
        id: serverId,
        name: formData.name,
        type: formData.type,
        version: formData.version,
        java: {
          path: formData.javaPath,
          args: formData.javaArgs,
          memory: formData.memory
        },
        network: {
          serverPort: formData.serverPort,
          rconPort: formData.rconPort,
          rconPassword: formData.rconPassword,
          queryPort: formData.queryPort
        },
        paths: formData.paths,
        settings: formData.settings,
        advanced: {
          jvmArgs: formData.javaArgs.split(' '),
          environment: {},
          workingDirectory: '',
          priority: 'normal'
        }
      });

      onServerCreated(serverConfig);
      onClose();
    } catch (error) {
      errorHandler.handleError(error as Error, 'Server Creation');
    } finally {
      setIsSubmitting(false);
    }
  };

  const generateId = (): string => {
    return Math.random().toString(36).substr(2, 9);
  };

  const progress = (currentStep / steps.length) * 100;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-4xl max-h-[90vh] overflow-y-auto">
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Create New Server</CardTitle>
            <Button variant="ghost" size="sm" onClick={onClose}>
              ×
            </Button>
          </div>
          <Progress value={progress} className="w-full" />
        </CardHeader>
        
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Step 1: Basic Info */}
            {currentStep === 1 && (
              <div className="space-y-4">
                <div>
                  <Label htmlFor="serverName">Server Name</Label>
                  <Input
                    id="serverName"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    placeholder="My Awesome Server"
                    required
                  />
                </div>
                
                <div>
                  <Label htmlFor="serverType">Server Type</Label>
                  <Select
                    value={formData.type}
                    onValueChange={(value) => setFormData({ ...formData, type: value as any })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="vanilla">Vanilla</SelectItem>
                      <SelectItem value="forge">Forge</SelectItem>
                      <SelectItem value="fabric">Fabric</SelectItem>
                      <SelectItem value="paper">Paper</SelectItem>
                      <SelectItem value="purpur">Purpur</SelectItem>
                      <SelectItem value="spigot">Spigot</SelectItem>
                      <SelectItem value="bukkit">Bukkit</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label htmlFor="version">Minecraft Version</Label>
                  <Select
                    value={formData.version}
                    onValueChange={(value) => setFormData({ ...formData, version: value })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {getVersionsForModpack().map((version: any) => (
                        <SelectItem key={version.version} value={version.version}>
                          {version.version} {version.is_latest ? '(Latest)' : ''}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            )}

            {/* Step 2: Java Configuration */}
            {currentStep === 2 && (
              <div className="space-y-4">
                <div>
                  <Label htmlFor="javaPath">Java Installation</Label>
                  <Select
                    value={formData.javaPath}
                    onValueChange={(value) => setFormData({ ...formData, javaPath: value })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select Java installation" />
                    </SelectTrigger>
                    <SelectContent>
                      {javaInstallations.map((java) => (
                        <SelectItem key={java.id} value={java.path}>
                          Java {java.version} - {java.path}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label htmlFor="memory">Memory Allocation (MB)</Label>
                  <Input
                    id="memory"
                    type="number"
                    value={formData.memory}
                    onChange={(e) => setFormData({ ...formData, memory: parseInt(e.target.value) })}
                    min="512"
                    max="32768"
                    step="512"
                  />
                </div>

                <div>
                  <Label htmlFor="javaArgs">Java Arguments</Label>
                  <Input
                    id="javaArgs"
                    value={formData.javaArgs}
                    onChange={(e) => setFormData({ ...formData, javaArgs: e.target.value })}
                    placeholder="-Xmx4G -Xms2G -XX:+UseG1GC"
                  />
                </div>
              </div>
            )}

            {/* Step 3: Network Configuration */}
            {currentStep === 3 && (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="serverPort">Server Port</Label>
                    <Input
                      id="serverPort"
                      type="number"
                      value={formData.serverPort}
                      onChange={(e) => setFormData({ ...formData, serverPort: parseInt(e.target.value) })}
                      min="1"
                      max="65535"
                    />
                  </div>
                  <div>
                    <Label htmlFor="queryPort">Query Port</Label>
                    <Input
                      id="queryPort"
                      type="number"
                      value={formData.queryPort}
                      onChange={(e) => setFormData({ ...formData, queryPort: parseInt(e.target.value) })}
                      min="1"
                      max="65535"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="rconPort">RCON Port</Label>
                    <Input
                      id="rconPort"
                      type="number"
                      value={formData.rconPort}
                      onChange={(e) => setFormData({ ...formData, rconPort: parseInt(e.target.value) })}
                      min="1"
                      max="65535"
                    />
                  </div>
                  <div>
                    <Label htmlFor="rconPassword">RCON Password</Label>
                    <Input
                      id="rconPassword"
                      type="password"
                      value={formData.rconPassword}
                      onChange={(e) => setFormData({ ...formData, rconPassword: e.target.value })}
                      placeholder="Enter RCON password"
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Step 4: File Paths */}
            {currentStep === 4 && (
              <div className="space-y-4">
                <div>
                  <Label htmlFor="worldPath">World Directory</Label>
                  <Input
                    id="worldPath"
                    value={formData.paths.world}
                    onChange={(e) => setFormData({ 
                      ...formData, 
                      paths: { ...formData.paths, world: e.target.value }
                    })}
                  />
                </div>

                <div>
                  <Label htmlFor="modsPath">Mods Directory</Label>
                  <Input
                    id="modsPath"
                    value={formData.paths.mods}
                    onChange={(e) => setFormData({ 
                      ...formData, 
                      paths: { ...formData.paths, mods: e.target.value }
                    })}
                  />
                </div>

                <div>
                  <Label htmlFor="configPath">Config Directory</Label>
                  <Input
                    id="configPath"
                    value={formData.paths.config}
                    onChange={(e) => setFormData({ 
                      ...formData, 
                      paths: { ...formData.paths, config: e.target.value }
                    })}
                  />
                </div>
              </div>
            )}

            {/* Step 5: Advanced Settings */}
            {currentStep === 5 && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="autoStart">Auto-start on app launch</Label>
                  <Switch
                    id="autoStart"
                    checked={formData.settings.autoStart}
                    onCheckedChange={(checked) => setFormData({
                      ...formData,
                      settings: { ...formData.settings, autoStart: checked }
                    })}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label htmlFor="autoRestart">Auto-restart on crash</Label>
                  <Switch
                    id="autoRestart"
                    checked={formData.settings.autoRestart}
                    onCheckedChange={(checked) => setFormData({
                      ...formData,
                      settings: { ...formData.settings, autoRestart: checked }
                    })}
                  />
                </div>

                <div>
                  <Label htmlFor="maxRestarts">Max Restarts</Label>
                  <Input
                    id="maxRestarts"
                    type="number"
                    value={formData.settings.maxRestarts}
                    onChange={(e) => setFormData({
                      ...formData,
                      settings: { ...formData.settings, maxRestarts: parseInt(e.target.value) }
                    })}
                    min="1"
                    max="10"
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="backupInterval">Backup Interval (hours)</Label>
                    <Input
                      id="backupInterval"
                      type="number"
                      value={formData.settings.backupInterval}
                      onChange={(e) => setFormData({
                        ...formData,
                        settings: { ...formData.settings, backupInterval: parseInt(e.target.value) }
                      })}
                      min="1"
                      max="168"
                    />
                  </div>
                  <div>
                    <Label htmlFor="backupRetention">Backup Retention (days)</Label>
                    <Input
                      id="backupRetention"
                      type="number"
                      value={formData.settings.backupRetention}
                      onChange={(e) => setFormData({
                        ...formData,
                        settings: { ...formData.settings, backupRetention: parseInt(e.target.value) }
                      })}
                      min="1"
                      max="365"
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Navigation Buttons */}
            <div className="flex justify-between pt-4">
              <div>
                {currentStep > 1 && (
                  <Button type="button" variant="outline" onClick={prevStep}>
                    Previous
                  </Button>
                )}
              </div>
              
              <div className="flex gap-2">
                {currentStep < steps.length ? (
                  <Button type="button" onClick={nextStep}>
                    Next
                  </Button>
                ) : (
                  <Button type="submit" disabled={isSubmitting}>
                    {isSubmitting ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Creating...
                      </>
                    ) : (
                      'Create Server'
                    )}
                  </Button>
                )}
                
                <Button type="button" variant="outline" onClick={onClose}>
                  Cancel
                </Button>
              </div>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};
