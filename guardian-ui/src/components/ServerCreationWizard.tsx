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
import { Textarea } from '@/components/ui/textarea';
import { Separator } from '@/components/ui/separator';
import { 
  Server, 
  Cpu, 
  Network, 
  FolderOpen, 
  Settings, 
  CheckCircle, 
  AlertTriangle,
  Loader2,
  Info,
  Zap,
  Shield,
  HardDrive,
  Globe,
  Clock,
  RefreshCw,
  Download,
  Search,
  X,
  ChevronLeft,
  ChevronRight,
  Sparkles
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
  { id: 1, title: 'Server Type', description: 'Choose your server type and version', icon: Server },
  { id: 2, title: 'Basic Settings', description: 'Name, Java, and memory configuration', icon: Cpu },
  { id: 3, title: 'Network', description: 'Ports, RCON, and player settings', icon: Network },
  { id: 4, title: 'Advanced', description: 'Auto-start, backups, and optimization', icon: Settings }
];

const serverPresets = [
  {
    id: 'vanilla-survival',
    name: 'Vanilla Survival',
    description: 'Classic Minecraft survival experience',
    type: 'vanilla',
    version: '1.21.1',
    memory: 2048,
    javaArgs: '-Xmx2G -Xms1G -XX:+UseG1GC',
    features: ['Survival', 'Multiplayer', 'Vanilla']
  },
  {
    id: 'modded-forge',
    name: 'Modded (Forge)',
    description: 'Forge modded server with mod support',
    type: 'forge',
    version: '1.20.1',
    memory: 4096,
    javaArgs: '-Xmx4G -Xms2G -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M',
    features: ['Mods', 'Forge', 'Custom Content']
  },
  {
    id: 'modded-fabric',
    name: 'Modded (Fabric)',
    description: 'Fabric modded server with performance mods',
    type: 'fabric',
    version: '1.20.1',
    memory: 4096,
    javaArgs: '-Xmx4G -Xms2G -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M',
    features: ['Mods', 'Fabric', 'Performance']
  },
  {
    id: 'paper-optimized',
    name: 'Paper (Optimized)',
    description: 'High-performance Paper server with optimizations',
    type: 'paper',
    version: '1.21.1',
    memory: 4096,
    javaArgs: '-Xmx4G -Xms2G -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M',
    features: ['Performance', 'Plugins', 'Optimized']
  },
  {
    id: 'creative-building',
    name: 'Creative Building',
    description: 'Creative mode server for building projects',
    type: 'vanilla',
    version: '1.21.1',
    memory: 2048,
    javaArgs: '-Xmx2G -Xms1G -XX:+UseG1GC',
    features: ['Creative', 'Building', 'WorldEdit']
  }
];

export const ServerCreationWizard: React.FC<ServerCreationWizardProps> = ({ 
  onClose, 
  onServerCreated 
}) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedPreset, setSelectedPreset] = useState<string | null>(null);
  const [javaInstallations, setJavaInstallations] = useState<any[]>([]);
  const [availablePorts, setAvailablePorts] = useState<number[]>([]);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [formData, setFormData] = useState({
    // Step 1: Server Type
    name: '',
    type: 'vanilla' as 'vanilla' | 'forge' | 'fabric' | 'paper' | 'purpur' | 'spigot' | 'bukkit',
    version: '1.21.1',
    description: '',
    
    // Step 2: Basic Settings
    javaPath: '',
    javaArgs: ['-Xmx4G', '-Xms2G', '-XX:+UseG1GC'],
    memory: 4096,
    maxPlayers: 20,
    difficulty: 'normal' as 'peaceful' | 'easy' | 'normal' | 'hard',
    gamemode: 'survival' as 'survival' | 'creative' | 'adventure' | 'spectator',
    pvp: true,
    allowFlight: false,
    allowNether: true,
    allowEnd: true,
    
    // Step 3: Network Configuration
    serverPort: 25565,
    rconPort: 25575,
    rconPassword: '',
    queryPort: 25565,
    enableQuery: true,
    enableRcon: true,
    
    // File Paths
    paths: {
      world: './world',
      mods: './mods',
      config: './config',
      logs: './logs',
      backups: './backups'
    },
    
    // Step 4: Advanced Settings
    settings: {
      autoStart: false,
      autoRestart: true,
      maxRestarts: 3,
      backupInterval: 24,
      backupRetention: 7,
      worldBorder: 29999984,
      viewDistance: 10,
      simulationDistance: 10,
      chunkLoading: 'lazy' as 'lazy' | 'eager',
      optimizeChunks: true,
      enableJmx: false
    },
    
    // Additional properties needed for server creation
    modpack: '',
    motd: 'A Minecraft Server',
    onlineMode: true,
    whitelist: false,
    enableCommandBlock: false,
    gpuEnabled: false,
    gpuQueueSize: 1000,
    gpuMaxWorkers: 4,
    haEnabled: false,
    blueGreen: false,
    healthCheckInterval: 5000,
    composerProfile: 'development',
    composerTimeout: 30000
  });

  useEffect(() => {
    loadJavaInstallations();
    loadDefaultSettings();
    findAvailablePorts();
  }, []);

  const validateStep = (step: number): boolean => {
    const newErrors: Record<string, string> = {};
    
    switch (step) {
      case 1:
        if (!formData.name.trim()) {
          newErrors.name = 'Server name is required';
        } else if (formData.name.length < 3) {
          newErrors.name = 'Server name must be at least 3 characters';
        } else if (formData.name.length > 32) {
          newErrors.name = 'Server name must be less than 32 characters';
        }
        if (!formData.type) {
          newErrors.type = 'Server type is required';
        }
        if (!formData.version) {
          newErrors.version = 'Minecraft version is required';
        }
        break;
        
      case 2:
        if (!formData.javaPath) {
          newErrors.javaPath = 'Java installation is required';
        }
        if (formData.memory < 512) {
          newErrors.memory = 'Memory must be at least 512MB';
        } else if (formData.memory > 32768) {
          newErrors.memory = 'Memory cannot exceed 32GB';
        }
        if (formData.maxPlayers < 1) {
          newErrors.maxPlayers = 'Max players must be at least 1';
        } else if (formData.maxPlayers > 1000) {
          newErrors.maxPlayers = 'Max players cannot exceed 1000';
        }
        break;
        
      case 3:
        if (formData.serverPort < 1 || formData.serverPort > 65535) {
          newErrors.serverPort = 'Server port must be between 1 and 65535';
        }
        if (formData.rconPort < 1 || formData.rconPort > 65535) {
          newErrors.rconPort = 'RCON port must be between 1 and 65535';
        }
        if (formData.queryPort < 1 || formData.queryPort > 65535) {
          newErrors.queryPort = 'Query port must be between 1 and 65535';
        }
        if (formData.enableRcon && !formData.rconPassword.trim()) {
          newErrors.rconPassword = 'RCON password is required when RCON is enabled';
        }
        break;
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const findAvailablePorts = async () => {
    // Find available ports starting from 25565
    const ports: number[] = [];
    for (let port = 25565; port <= 25575; port++) {
      ports.push(port);
    }
    setAvailablePorts(ports);
  };

  const applyPreset = (presetId: string) => {
    const preset = serverPresets.find(p => p.id === presetId);
    if (preset) {
      setSelectedPreset(presetId);
      setFormData(prev => ({
        ...prev,
        type: preset.type as any,
        version: preset.version,
        memory: preset.memory,
        javaArgs: Array.isArray(preset.javaArgs) ? preset.javaArgs : preset.javaArgs.split(' ')
      }));
    }
  };

  const loadJavaInstallations = async () => {
    try {
      // TODO: Load Java installations from settings or API
      setJavaInstallations([]);
    } catch (error) {
      errorHandler.handleError(error as Error, 'Load Java Installations');
    }
  };

  const loadDefaultSettings = async () => {
    try {
      // Use default values for now
      setFormData(prev => ({
        ...prev,
        javaArgs: ['-Xmx2G', '-XX:+UseG1GC'],
        memory: 2048,
        javaPath: ''
      }));
    } catch (error) {
      errorHandler.handleError(error as Error, 'Load Default Settings');
    }
  };

  const nextStep = () => {
    if (validateStep(currentStep) && currentStep < steps.length) {
      setCurrentStep(currentStep + 1);
    }
  };

  const prevStep = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1);
    }
  };

  const goToStep = (step: number) => {
    if (step >= 1 && step <= steps.length) {
      setCurrentStep(step);
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
        version: formData.version,
        modpack: formData.modpack,
        path: formData.paths.world,
        port: formData.serverPort,
        maxPlayers: formData.maxPlayers,
        memory: formData.memory,
        jvmArgs: formData.javaArgs,
        properties: {
          rconPort: formData.rconPort.toString(),
          rconPassword: formData.rconPassword,
          queryPort: formData.queryPort.toString(),
          motd: formData.motd,
          difficulty: formData.difficulty,
          gamemode: formData.gamemode,
          pvp: formData.pvp.toString(),
          onlineMode: formData.onlineMode.toString(),
          whitelist: formData.whitelist.toString(),
          enableCommandBlock: formData.enableCommandBlock.toString(),
          viewDistance: formData.settings.viewDistance.toString(),
          simulationDistance: formData.settings.simulationDistance.toString()
        },
        createdAt: now,
        updatedAt: now
      };

      // Create server directory and files
      await fileManager.createDirectory(`servers/${serverId}`);
      
      // Create server settings
      await settingsManager.createServerSettings(serverId, {
        general: {
          name: formData.name,
          description: formData.description,
          version: formData.version,
          modpack: formData.modpack,
          maxPlayers: formData.maxPlayers,
          motd: formData.motd,
          difficulty: formData.difficulty,
          gamemode: formData.gamemode,
          pvp: formData.pvp,
          onlineMode: formData.onlineMode,
          whitelist: formData.whitelist,
          enableCommandBlock: formData.enableCommandBlock,
          viewDistance: formData.settings.viewDistance,
          simulationDistance: formData.settings.simulationDistance
        },
        jvm: {
          memory: formData.memory,
          flags: formData.javaArgs,
          gcType: 'G1GC'
        },
        gpu: {
          enabled: formData.gpuEnabled,
          queueSize: formData.gpuQueueSize,
          maxWorkers: formData.gpuMaxWorkers
        },
        ha: {
          enabled: formData.haEnabled,
          blueGreen: formData.blueGreen,
          healthCheckInterval: formData.healthCheckInterval
        },
        paths: {
          world: formData.paths.world,
          mods: formData.paths.mods,
          config: formData.paths.config,
          logs: formData.paths.logs
        },
        composer: {
          profile: formData.composerProfile,
          dockerfile: 'Dockerfile'
        },
        tokens: {
          rcon: formData.rconPassword,
          query: formData.queryPort.toString()
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
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-6xl max-h-[95vh] overflow-hidden">
        <CardHeader className="border-b bg-gradient-to-r from-blue-50 to-indigo-50">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-blue-100 rounded-lg">
                <Server className="h-6 w-6 text-blue-600" />
              </div>
              <div>
                <CardTitle className="text-2xl">Create New Server</CardTitle>
                <p className="text-sm text-muted-foreground">
                  Set up your Minecraft server in just a few steps
                </p>
              </div>
            </div>
            <Button variant="ghost" size="sm" onClick={onClose} className="h-8 w-8 p-0">
              <X className="h-4 w-4" />
            </Button>
          </div>
          
          {/* Progress Bar */}
          <div className="mt-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Step {currentStep} of {steps.length}</span>
              <span className="text-sm text-muted-foreground">{Math.round(progress)}% complete</span>
            </div>
            <Progress value={progress} className="h-2" />
          </div>
          
          {/* Step Navigation */}
          <div className="flex items-center justify-center mt-4 gap-2">
            {steps.map((step, index) => (
              <button
                key={step.id}
                onClick={() => goToStep(step.id)}
                className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                  currentStep === step.id
                    ? 'bg-blue-100 text-blue-700'
                    : currentStep > step.id
                    ? 'bg-green-100 text-green-700'
                    : 'bg-gray-100 text-gray-500 hover:bg-gray-200'
                }`}
              >
                <step.icon className="h-4 w-4" />
                {step.title}
              </button>
            ))}
          </div>
        </CardHeader>
        
        <CardContent className="p-0">
          <form onSubmit={handleSubmit} className="h-full">
            {/* Step 1: Server Type */}
            {currentStep === 1 && (
              <div className="p-8 space-y-8">
                <div className="text-center">
                  <h3 className="text-2xl font-bold mb-2">Choose Your Server Type</h3>
                  <p className="text-muted-foreground">Select a preset or customize your server configuration</p>
                </div>

                {/* Server Presets */}
              <div className="space-y-4">
                  <h4 className="text-lg font-semibold flex items-center gap-2">
                    <Sparkles className="h-5 w-5 text-yellow-500" />
                    Quick Start Presets
                  </h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {serverPresets.map((preset) => (
                      <Card 
                        key={preset.id}
                        className={`cursor-pointer transition-all hover:shadow-md ${
                          selectedPreset === preset.id 
                            ? 'ring-2 ring-blue-500 bg-blue-50' 
                            : 'hover:bg-gray-50'
                        }`}
                        onClick={() => applyPreset(preset.id)}
                      >
                        <CardContent className="p-4">
                          <div className="flex items-start justify-between mb-3">
                <div>
                              <h5 className="font-semibold">{preset.name}</h5>
                              <p className="text-sm text-muted-foreground">{preset.description}</p>
                            </div>
                            {selectedPreset === preset.id && (
                              <CheckCircle className="h-5 w-5 text-blue-500" />
                            )}
                          </div>
                          <div className="space-y-2">
                            <div className="flex items-center gap-2 text-sm">
                              <Badge variant="secondary">{preset.type}</Badge>
                              <Badge variant="outline">{preset.version}</Badge>
                            </div>
                            <div className="flex flex-wrap gap-1">
                              {preset.features.map((feature) => (
                                <Badge key={feature} variant="outline" className="text-xs">
                                  {feature}
                                </Badge>
                              ))}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              Memory: {preset.memory}MB
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                </div>

                <Separator />

                {/* Custom Configuration */}
                <div className="space-y-6">
                  <h4 className="text-lg font-semibold flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Custom Configuration
                  </h4>
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="space-y-2">
                      <Label htmlFor="serverName" className="text-sm font-medium">
                        Server Name <span className="text-red-500">*</span>
                      </Label>
                  <Input
                    id="serverName"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    placeholder="My Awesome Server"
                        className={errors.name ? 'border-red-500' : ''}
                  />
                      {errors.name && (
                        <p className="text-sm text-red-500">{errors.name}</p>
                      )}
                </div>
                
                    <div className="space-y-2">
                      <Label htmlFor="serverType" className="text-sm font-medium">
                        Server Type <span className="text-red-500">*</span>
                      </Label>
                  <Select
                    value={formData.type}
                    onValueChange={(value) => setFormData({ ...formData, type: value as any })}
                  >
                        <SelectTrigger className={errors.type ? 'border-red-500' : ''}>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                          <SelectItem value="vanilla">
                            <div className="flex items-center gap-2">
                              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                              Vanilla
                            </div>
                          </SelectItem>
                          <SelectItem value="forge">
                            <div className="flex items-center gap-2">
                              <div className="w-2 h-2 bg-orange-500 rounded-full"></div>
                              Forge
                            </div>
                          </SelectItem>
                          <SelectItem value="fabric">
                            <div className="flex items-center gap-2">
                              <div className="w-2 h-2 bg-purple-500 rounded-full"></div>
                              Fabric
                            </div>
                          </SelectItem>
                          <SelectItem value="paper">
                            <div className="flex items-center gap-2">
                              <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                              Paper
                            </div>
                          </SelectItem>
                          <SelectItem value="purpur">
                            <div className="flex items-center gap-2">
                              <div className="w-2 h-2 bg-pink-500 rounded-full"></div>
                              Purpur
                            </div>
                          </SelectItem>
                    </SelectContent>
                  </Select>
                      {errors.type && (
                        <p className="text-sm text-red-500">{errors.type}</p>
                      )}
                    </div>
                </div>

                  <div className="space-y-2">
                    <Label htmlFor="version" className="text-sm font-medium">
                      Minecraft Version <span className="text-red-500">*</span>
                    </Label>
                  <Select
                    value={formData.version}
                    onValueChange={(value) => setFormData({ ...formData, version: value })}
                  >
                      <SelectTrigger className={errors.version ? 'border-red-500' : ''}>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {getVersionsForModpack('forge').map((version: any) => (
                        <SelectItem key={version.id} value={version.id}>
                            <div className="flex items-center gap-2">
                              {version.id === '1.21.7' && <Badge variant="default" className="text-xs">Latest</Badge>}
                              {version.name}
                            </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                    {errors.version && (
                      <p className="text-sm text-red-500">{errors.version}</p>
                    )}
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="description" className="text-sm font-medium">
                      Description (Optional)
                    </Label>
                    <Textarea
                      id="description"
                      value={formData.description}
                      onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                      placeholder="Describe your server..."
                      rows={3}
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Step 2: Basic Settings */}
            {currentStep === 2 && (
              <div className="p-8 space-y-8">
                <div className="text-center">
                  <h3 className="text-2xl font-bold mb-2">Basic Server Settings</h3>
                  <p className="text-muted-foreground">Configure Java, memory, and game settings</p>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  {/* Java Configuration */}
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Cpu className="h-5 w-5" />
                        Java Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="javaPath" className="text-sm font-medium">
                          Java Installation <span className="text-red-500">*</span>
                        </Label>
                  <Select
                    value={formData.javaPath}
                    onValueChange={(value) => setFormData({ ...formData, javaPath: value })}
                  >
                          <SelectTrigger className={errors.javaPath ? 'border-red-500' : ''}>
                      <SelectValue placeholder="Select Java installation" />
                    </SelectTrigger>
                    <SelectContent>
                      {javaInstallations.map((java) => (
                        <SelectItem key={java.id} value={java.path}>
                                <div className="flex items-center gap-2">
                                  <Badge variant="outline">Java {java.version}</Badge>
                                  <span className="text-sm text-muted-foreground truncate">
                                    {java.path}
                                  </span>
                                </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                        {errors.javaPath && (
                          <p className="text-sm text-red-500">{errors.javaPath}</p>
                        )}
                </div>

                      <div className="space-y-2">
                        <Label htmlFor="memory" className="text-sm font-medium">
                          Memory Allocation <span className="text-red-500">*</span>
                        </Label>
                        <div className="space-y-2">
                  <Input
                    id="memory"
                    type="number"
                    value={formData.memory}
                    onChange={(e) => setFormData({ ...formData, memory: parseInt(e.target.value) })}
                    min="512"
                    max="32768"
                    step="512"
                            className={errors.memory ? 'border-red-500' : ''}
                          />
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Info className="h-3 w-3" />
                            Recommended: {formData.type === 'vanilla' ? '2-4GB' : '4-8GB'} for {formData.type} servers
                          </div>
                        </div>
                        {errors.memory && (
                          <p className="text-sm text-red-500">{errors.memory}</p>
                        )}
                </div>

                      <div className="space-y-2">
                        <Label htmlFor="javaArgs" className="text-sm font-medium">
                          Java Arguments
                        </Label>
                        <Textarea
                    id="javaArgs"
                    value={Array.isArray(formData.javaArgs) ? formData.javaArgs.join(' ') : formData.javaArgs}
                    onChange={(e) => setFormData({ ...formData, javaArgs: e.target.value.split(' ') })}
                    placeholder="-Xmx4G -Xms2G -XX:+UseG1GC"
                          rows={3}
                          className="font-mono text-sm"
                  />
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Info className="h-3 w-3" />
                          Advanced JVM arguments for performance tuning
                </div>
              </div>
                    </CardContent>
                  </Card>

                  {/* Game Settings */}
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Globe className="h-5 w-5" />
                        Game Settings
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="maxPlayers" className="text-sm font-medium">
                          Max Players <span className="text-red-500">*</span>
                        </Label>
                        <Input
                          id="maxPlayers"
                          type="number"
                          value={formData.maxPlayers}
                          onChange={(e) => setFormData({ ...formData, maxPlayers: parseInt(e.target.value) })}
                          min="1"
                          max="1000"
                          className={errors.maxPlayers ? 'border-red-500' : ''}
                        />
                        {errors.maxPlayers && (
                          <p className="text-sm text-red-500">{errors.maxPlayers}</p>
                        )}
                      </div>

                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label htmlFor="difficulty" className="text-sm font-medium">
                            Difficulty
                          </Label>
                          <Select
                            value={formData.difficulty}
                            onValueChange={(value) => setFormData({ ...formData, difficulty: value as any })}
                          >
                            <SelectTrigger>
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="peaceful">Peaceful</SelectItem>
                              <SelectItem value="easy">Easy</SelectItem>
                              <SelectItem value="normal">Normal</SelectItem>
                              <SelectItem value="hard">Hard</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>

                        <div className="space-y-2">
                          <Label htmlFor="gamemode" className="text-sm font-medium">
                            Gamemode
                          </Label>
                          <Select
                            value={formData.gamemode}
                            onValueChange={(value) => setFormData({ ...formData, gamemode: value as any })}
                          >
                            <SelectTrigger>
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="survival">Survival</SelectItem>
                              <SelectItem value="creative">Creative</SelectItem>
                              <SelectItem value="adventure">Adventure</SelectItem>
                              <SelectItem value="spectator">Spectator</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                      </div>

                      <div className="space-y-3">
                        <div className="flex items-center justify-between">
                          <Label htmlFor="pvp" className="text-sm font-medium">
                            Enable PvP
                          </Label>
                          <Switch
                            id="pvp"
                            checked={formData.pvp}
                            onCheckedChange={(checked) => setFormData({ ...formData, pvp: checked })}
                          />
                        </div>

                        <div className="flex items-center justify-between">
                          <Label htmlFor="allowFlight" className="text-sm font-medium">
                            Allow Flight
                          </Label>
                          <Switch
                            id="allowFlight"
                            checked={formData.allowFlight}
                            onCheckedChange={(checked) => setFormData({ ...formData, allowFlight: checked })}
                          />
                        </div>

                        <div className="flex items-center justify-between">
                          <Label htmlFor="allowNether" className="text-sm font-medium">
                            Allow Nether
                          </Label>
                          <Switch
                            id="allowNether"
                            checked={formData.allowNether}
                            onCheckedChange={(checked) => setFormData({ ...formData, allowNether: checked })}
                          />
                        </div>

                        <div className="flex items-center justify-between">
                          <Label htmlFor="allowEnd" className="text-sm font-medium">
                            Allow End
                          </Label>
                          <Switch
                            id="allowEnd"
                            checked={formData.allowEnd}
                            onCheckedChange={(checked) => setFormData({ ...formData, allowEnd: checked })}
                          />
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </div>
            )}

            {/* Step 3: Network Configuration */}
            {currentStep === 3 && (
              <div className="p-8 space-y-8">
                <div className="text-center">
                  <h3 className="text-2xl font-bold mb-2">Network Configuration</h3>
                  <p className="text-muted-foreground">Configure ports, RCON, and network settings</p>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  {/* Port Configuration */}
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Network className="h-5 w-5" />
                        Port Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="serverPort" className="text-sm font-medium">
                          Server Port <span className="text-red-500">*</span>
                        </Label>
                        <div className="space-y-2">
                    <Input
                      id="serverPort"
                      type="number"
                      value={formData.serverPort}
                      onChange={(e) => setFormData({ ...formData, serverPort: parseInt(e.target.value) })}
                      min="1"
                      max="65535"
                            className={errors.serverPort ? 'border-red-500' : ''}
                    />
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Info className="h-3 w-3" />
                            Default: 25565. Make sure this port is open in your firewall.
                  </div>
                        </div>
                        {errors.serverPort && (
                          <p className="text-sm text-red-500">{errors.serverPort}</p>
                        )}
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="queryPort" className="text-sm font-medium">
                          Query Port
                        </Label>
                        <div className="space-y-2">
                    <Input
                      id="queryPort"
                      type="number"
                      value={formData.queryPort}
                      onChange={(e) => setFormData({ ...formData, queryPort: parseInt(e.target.value) })}
                      min="1"
                      max="65535"
                            className={errors.queryPort ? 'border-red-500' : ''}
                    />
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Info className="h-3 w-3" />
                            Used for server list queries. Usually same as server port.
                  </div>
                        </div>
                        {errors.queryPort && (
                          <p className="text-sm text-red-500">{errors.queryPort}</p>
                        )}
                </div>

                      <div className="space-y-2">
                        <Label htmlFor="rconPort" className="text-sm font-medium">
                          RCON Port
                        </Label>
                        <div className="space-y-2">
                    <Input
                      id="rconPort"
                      type="number"
                      value={formData.rconPort}
                      onChange={(e) => setFormData({ ...formData, rconPort: parseInt(e.target.value) })}
                      min="1"
                      max="65535"
                            className={errors.rconPort ? 'border-red-500' : ''}
                    />
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Info className="h-3 w-3" />
                            Remote console port for server management.
                  </div>
                        </div>
                        {errors.rconPort && (
                          <p className="text-sm text-red-500">{errors.rconPort}</p>
                        )}
                      </div>
                    </CardContent>
                  </Card>

                  {/* RCON Configuration */}
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Shield className="h-5 w-5" />
                        RCON Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="flex items-center justify-between">
                        <Label htmlFor="enableRcon" className="text-sm font-medium">
                          Enable RCON
                        </Label>
                        <Switch
                          id="enableRcon"
                          checked={formData.enableRcon}
                          onCheckedChange={(checked) => setFormData({ ...formData, enableRcon: checked })}
                        />
                      </div>

                      {formData.enableRcon && (
                        <div className="space-y-2">
                          <Label htmlFor="rconPassword" className="text-sm font-medium">
                            RCON Password <span className="text-red-500">*</span>
                          </Label>
                          <div className="space-y-2">
                    <Input
                      id="rconPassword"
                      type="password"
                      value={formData.rconPassword}
                      onChange={(e) => setFormData({ ...formData, rconPassword: e.target.value })}
                      placeholder="Enter RCON password"
                              className={errors.rconPassword ? 'border-red-500' : ''}
                    />
                            <div className="flex items-center gap-2 text-xs text-muted-foreground">
                              <Info className="h-3 w-3" />
                              Use a strong password for security.
                  </div>
                </div>
                          {errors.rconPassword && (
                            <p className="text-sm text-red-500">{errors.rconPassword}</p>
                          )}
              </div>
            )}

                      <div className="flex items-center justify-between">
                        <Label htmlFor="enableQuery" className="text-sm font-medium">
                          Enable Query
                        </Label>
                        <Switch
                          id="enableQuery"
                          checked={formData.enableQuery}
                          onCheckedChange={(checked) => setFormData({ ...formData, enableQuery: checked })}
                  />
                </div>

                      <Alert>
                        <Info className="h-4 w-4" />
                        <AlertDescription>
                          RCON allows remote server management. Query enables server list information.
                          Both require proper firewall configuration.
                        </AlertDescription>
                      </Alert>
                    </CardContent>
                  </Card>
                </div>
              </div>
            )}

            {/* Step 4: Advanced Settings */}
            {currentStep === 4 && (
              <div className="p-8 space-y-8">
                <div className="text-center">
                  <h3 className="text-2xl font-bold mb-2">Advanced Settings</h3>
                  <p className="text-muted-foreground">Configure auto-start, backups, and performance optimization</p>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  {/* Server Management */}
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Settings className="h-5 w-5" />
                        Server Management
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                        <div className="space-y-1">
                          <Label htmlFor="autoStart" className="text-sm font-medium">
                            Auto-start on app launch
                          </Label>
                          <p className="text-xs text-muted-foreground">
                            Automatically start this server when Guardian launches
                          </p>
                        </div>
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
                        <div className="space-y-1">
                          <Label htmlFor="autoRestart" className="text-sm font-medium">
                            Auto-restart on crash
                          </Label>
                          <p className="text-xs text-muted-foreground">
                            Automatically restart the server if it crashes
                          </p>
                        </div>
                  <Switch
                    id="autoRestart"
                    checked={formData.settings.autoRestart}
                    onCheckedChange={(checked) => setFormData({
                      ...formData,
                      settings: { ...formData.settings, autoRestart: checked }
                    })}
                  />
                </div>

                      {formData.settings.autoRestart && (
                        <div className="space-y-2">
                          <Label htmlFor="maxRestarts" className="text-sm font-medium">
                            Max Restarts
                          </Label>
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
                      )}
                    </CardContent>
                  </Card>

                  {/* Backup Configuration */}
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <HardDrive className="h-5 w-5" />
                        Backup Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="backupInterval" className="text-sm font-medium">
                          Backup Interval (hours)
                        </Label>
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
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Info className="h-3 w-3" />
                          How often to create automatic backups
                  </div>
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="backupRetention" className="text-sm font-medium">
                          Backup Retention (days)
                        </Label>
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
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Info className="h-3 w-3" />
                          How long to keep backup files
                  </div>
                </div>
                    </CardContent>
                  </Card>
                </div>

                {/* Performance Settings */}
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Zap className="h-5 w-5" />
                      Performance Settings
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="viewDistance" className="text-sm font-medium">
                          View Distance
                        </Label>
                        <Input
                          id="viewDistance"
                          type="number"
                          value={formData.settings.viewDistance}
                          onChange={(e) => setFormData({
                            ...formData,
                            settings: { ...formData.settings, viewDistance: parseInt(e.target.value) }
                          })}
                          min="3"
                          max="32"
                        />
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="simulationDistance" className="text-sm font-medium">
                          Simulation Distance
                        </Label>
                        <Input
                          id="simulationDistance"
                          type="number"
                          value={formData.settings.simulationDistance}
                          onChange={(e) => setFormData({
                            ...formData,
                            settings: { ...formData.settings, simulationDistance: parseInt(e.target.value) }
                          })}
                          min="3"
                          max="32"
                        />
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="worldBorder" className="text-sm font-medium">
                          World Border
                        </Label>
                        <Input
                          id="worldBorder"
                          type="number"
                          value={formData.settings.worldBorder}
                          onChange={(e) => setFormData({
                            ...formData,
                            settings: { ...formData.settings, worldBorder: parseInt(e.target.value) }
                          })}
                          min="1"
                          max="29999984"
                        />
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="chunkLoading" className="text-sm font-medium">
                          Chunk Loading
                        </Label>
                        <Select
                          value={formData.settings.chunkLoading}
                          onValueChange={(value) => setFormData({
                            ...formData,
                            settings: { ...formData.settings, chunkLoading: value as any }
                          })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="lazy">Lazy</SelectItem>
                            <SelectItem value="eager">Eager</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="space-y-1">
                        <Label htmlFor="optimizeChunks" className="text-sm font-medium">
                          Optimize Chunks
                        </Label>
                        <p className="text-xs text-muted-foreground">
                          Enable chunk optimization for better performance
                        </p>
                      </div>
                      <Switch
                        id="optimizeChunks"
                        checked={formData.settings.optimizeChunks}
                        onCheckedChange={(checked) => setFormData({
                          ...formData,
                          settings: { ...formData.settings, optimizeChunks: checked }
                        })}
                      />
                    </div>
                  </CardContent>
                </Card>
              </div>
            )}

            {/* Navigation Buttons */}
            <div className="border-t bg-gray-50 px-8 py-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                {currentStep > 1 && (
                    <Button 
                      type="button" 
                      variant="outline" 
                      onClick={prevStep}
                      className="flex items-center gap-2"
                    >
                      <ChevronLeft className="h-4 w-4" />
                    Previous
                  </Button>
                )}
              </div>
              
                <div className="flex items-center gap-3">
                  <Button 
                    type="button" 
                    variant="ghost" 
                    onClick={onClose}
                    className="text-muted-foreground hover:text-foreground"
                  >
                    Cancel
                  </Button>
                  
                {currentStep < steps.length ? (
                    <Button 
                      type="button" 
                      onClick={nextStep}
                      className="flex items-center gap-2"
                    >
                      Next
                      <ChevronRight className="h-4 w-4" />
                  </Button>
                ) : (
                    <Button 
                      type="submit" 
                      disabled={isSubmitting}
                      className="flex items-center gap-2 bg-green-600 hover:bg-green-700"
                    >
                    {isSubmitting ? (
                      <>
                          <Loader2 className="h-4 w-4 animate-spin" />
                          Creating Server...
                      </>
                    ) : (
                        <>
                          <CheckCircle className="h-4 w-4" />
                          Create Server
                        </>
                    )}
                  </Button>
                )}
                </div>
              </div>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};
