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
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';

interface ServerCreationWizardProps {
  onClose: () => void;
  onServerCreated: (server: any) => void;
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
    id: 'quilt-modded',
    name: 'Quilt (Modded)',
    description: 'Quilt modded server with performance optimizations',
    type: 'quilt',
    version: '1.21.1',
    memory: 4096,
    javaArgs: '-Xmx4G -Xms2G -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M',
    features: ['Mods', 'Performance', 'Quilt']
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
  const [submitProgress, setSubmitProgress] = useState(0);
  const [selectedPreset, setSelectedPreset] = useState<string | null>(null);
  const [javaInstallations, setJavaInstallations] = useState<any[]>([]);
  const [availablePorts, setAvailablePorts] = useState<number[]>([]);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isDragOverJar, setIsDragOverJar] = useState(false);
  const [isLoadingJava, setIsLoadingJava] = useState(false);
  const [isValidatingPorts, setIsValidatingPorts] = useState(false);
  const [portValidationResults, setPortValidationResults] = useState<Record<string, boolean>>({});
  const [presetApplied, setPresetApplied] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    // Step 1: Server Type
    name: '',
    type: 'vanilla' as 'vanilla' | 'forge' | 'fabric' | 'quilt',
    version: '1.21.1',
    description: '',
    serverJarPath: '',
    
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

  // Tauri global file-drop listener; only accept when dragging over our dropzone
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<string[] | { paths: string[] }>('tauri://file-drop', (event: any) => {
      if (!isDragOverJar) return;
      const payload = event?.payload;
      const paths: string[] = Array.isArray(payload)
        ? payload as string[]
        : Array.isArray(payload?.paths)
          ? payload.paths
          : [];
      if (paths.length === 0) return;
      const picked = paths.find(p => p.toLowerCase().endsWith('.jar')) || paths[0];
      if (picked) {
        setFormData(prev => ({ ...prev, serverJarPath: picked }));
        const lower = picked.toLowerCase();
        if (lower.includes('1.21')) {
          setFormData(prev => ({ ...prev, version: '1.21.1' }));
        } else if (lower.includes('1.20.6')) {
          setFormData(prev => ({ ...prev, version: '1.20.6' }));
        }
        setIsDragOverJar(false);
      }
    }).then(fn => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, [isDragOverJar]);

  const validateStep = (step: number): boolean => {
    const newErrors: Record<string, string> = {};
    
    switch (step) {
      case 1:
        if (!formData.name.trim()) {
          newErrors.name = 'Server name is required';
        } else if (formData.name.length < 1) {
          newErrors.name = 'Server name must be at least 1 character';
        } else if (formData.name.length > 50) {
          newErrors.name = 'Server name must be less than 50 characters';
        } else if (!/^[a-zA-Z0-9\s\-_]+$/.test(formData.name)) {
          newErrors.name = 'Server name can only contain letters, numbers, spaces, hyphens, and underscores';
        }
        if (!formData.type) {
          newErrors.type = 'Server type is required';
        }
        if (!formData.version) {
          newErrors.version = 'Minecraft version is required';
        }
        break;
        
      case 2:
        if (!formData.javaPath && formData.type !== 'vanilla') {
          newErrors.javaPath = 'Java installation is required for non-vanilla servers';
        }
        if (formData.memory < 512) {
          newErrors.memory = 'Memory must be at least 512MB';
        } else if (formData.memory > 32768) {
          newErrors.memory = 'Memory cannot exceed 32GB';
        } else if (formData.memory % 512 !== 0) {
          newErrors.memory = 'Memory should be a multiple of 512MB for optimal performance';
        }
        if (formData.maxPlayers < 1) {
          newErrors.maxPlayers = 'Max players must be at least 1';
        } else if (formData.maxPlayers > 1000) {
          newErrors.maxPlayers = 'Max players cannot exceed 1000';
        }
        break;
        
      case 3:
        if (formData.serverPort < 1024 || formData.serverPort > 65535) {
          newErrors.serverPort = 'Server port must be between 1024 and 65535 (privileged ports not allowed)';
        }
        if (formData.rconPort < 1024 || formData.rconPort > 65535) {
          newErrors.rconPort = 'RCON port must be between 1024 and 65535 (privileged ports not allowed)';
        }
        if (formData.queryPort < 1024 || formData.queryPort > 65535) {
          newErrors.queryPort = 'Query port must be between 1024 and 65535 (privileged ports not allowed)';
        }
        if (formData.serverPort === formData.rconPort) {
          newErrors.rconPort = 'RCON port must be different from server port';
        }
        if (formData.serverPort === formData.queryPort) {
          newErrors.queryPort = 'Query port must be different from server port';
        }
        if (formData.rconPort === formData.queryPort) {
          newErrors.queryPort = 'Query port must be different from RCON port';
        }
        if (formData.enableRcon && !formData.rconPassword.trim()) {
          newErrors.rconPassword = 'RCON password is required when RCON is enabled';
        } else if (formData.enableRcon && formData.rconPassword.length < 8) {
          newErrors.rconPassword = 'RCON password must be at least 8 characters for security';
        }
        break;
        
      case 4:
        // Advanced settings validation
        if (formData.settings.viewDistance < 3 || formData.settings.viewDistance > 32) {
          newErrors.viewDistance = 'View distance must be between 3 and 32';
        }
        if (formData.settings.simulationDistance < 3 || formData.settings.simulationDistance > 32) {
          newErrors.simulationDistance = 'Simulation distance must be between 3 and 32';
        }
        if (formData.settings.worldBorder < 1 || formData.settings.worldBorder > 29999984) {
          newErrors.worldBorder = 'World border must be between 1 and 29999984';
        }
        if (formData.settings.maxRestarts < 1 || formData.settings.maxRestarts > 10) {
          newErrors.maxRestarts = 'Max restarts must be between 1 and 10';
        }
        if (formData.settings.backupInterval < 1 || formData.settings.backupInterval > 168) {
          newErrors.backupInterval = 'Backup interval must be between 1 and 168 hours';
        }
        if (formData.settings.backupRetention < 1 || formData.settings.backupRetention > 365) {
          newErrors.backupRetention = 'Backup retention must be between 1 and 365 days';
        }
        break;
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const findAvailablePorts = async () => {
    // Find available ports starting from 25565 (valid non-privileged ports)
    const ports: number[] = [];
    for (let port = 25565; port <= 25575; port++) {
      ports.push(port);
    }
    setAvailablePorts(ports);
    
    // Auto-assign available ports (ensure they're all >= 1024)
    if (ports.length > 0) {
      setFormData(prev => ({
        ...prev,
        serverPort: ports[0],
        rconPort: ports[1] || ports[0] + 1,
        queryPort: ports[2] || ports[0] + 2
      }));
    }
  };

  const validatePort = async (port: number): Promise<boolean> => {
    try {
      // In a real implementation, this would check if the port is actually available
      // For now, we'll just check if it's in a valid range
      return port >= 1 && port <= 65535;
    } catch (error) {
      console.error('Port validation error:', error);
      return false;
    }
  };

  const applyPreset = (presetId: string) => {
    const preset = serverPresets.find(p => p.id === presetId);
    if (preset) {
      setSelectedPreset(presetId);
      setPresetApplied(presetId);
      setFormData(prev => ({
        ...prev,
        type: preset.type as any,
        version: preset.version,
        memory: preset.memory,
        javaArgs: Array.isArray(preset.javaArgs) ? preset.javaArgs : preset.javaArgs.split(' '),
        name: prev.name || `${preset.name} Server`
      }));
      
      // Clear any existing errors when applying preset
      setErrors({});
      
      // Show success feedback
      console.log(`✅ Applied preset: ${preset.name}`);
      
      // Clear the applied indicator after 2 seconds
      setTimeout(() => setPresetApplied(null), 2000);
    }
  };

  const loadJavaInstallations = async () => {
    setIsLoadingJava(true);
    try {
      // Try to use Tauri command to find Java installations
      let javaInstallations = [];
      
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const javaPaths = await invoke<string[]>('find_java_installations');
        
        javaInstallations = javaPaths.map((path, index) => ({
          id: `java-${index}`,
          version: 'Unknown',
          path: path,
          vendor: 'Unknown'
        }));
      } catch (tauriError) {
        console.log('Tauri command not available, using fallback Java detection');
        
        // Fallback: Common Java installation paths
        const commonPaths = [
          'C:\\Program Files\\Java\\jdk-21\\bin\\java.exe',
          'C:\\Program Files\\Java\\jdk-17\\bin\\java.exe',
          'C:\\Program Files\\Java\\jdk-11\\bin\\java.exe',
          'C:\\Program Files\\Java\\jdk-8\\bin\\java.exe',
          'C:\\Program Files\\Java\\jre1.8.0_301\\bin\\java.exe',
          'C:\\Program Files\\Eclipse Adoptium\\jdk-21.0.1.12-hotspot\\bin\\java.exe',
          'C:\\Program Files\\Eclipse Adoptium\\jdk-17.0.9.9-hotspot\\bin\\java.exe',
          'C:\\Program Files\\Eclipse Adoptium\\jdk-11.0.21.9-hotspot\\bin\\java.exe',
          'C:\\Program Files\\Eclipse Adoptium\\jdk-8.0.392.8-hotspot\\bin\\java.exe',
          'C:\\Program Files\\Microsoft\\jdk-21.0.1.12.1\\bin\\java.exe',
          'C:\\Program Files\\Microsoft\\jdk-17.0.9.9\\bin\\java.exe',
          'C:\\Program Files\\Microsoft\\jdk-11.0.21.9\\bin\\java.exe',
          'C:\\Program Files\\Microsoft\\jdk-8.0.392.8\\bin\\java.exe'
        ];
        
        javaInstallations = commonPaths.map((path, index) => ({
          id: `java-${index}`,
          version: path.includes('21') ? '21' : path.includes('17') ? '17' : path.includes('11') ? '11' : path.includes('8') ? '8' : 'Unknown',
          path: path,
          vendor: path.includes('Microsoft') ? 'Microsoft' : path.includes('Eclipse') ? 'Eclipse Adoptium' : 'Oracle'
        }));
      }
      
      // Simulate async loading
      await new Promise(resolve => setTimeout(resolve, 500));
      setJavaInstallations(javaInstallations);
      
      // Auto-select the best Java version (prefer Java 21, then 17, then 11, then 8)
      const bestJava = javaInstallations.find(j => j.version === '21') || 
                      javaInstallations.find(j => j.version === '17') || 
                      javaInstallations.find(j => j.version === '11') || 
                      javaInstallations.find(j => j.version === '8') || 
                      javaInstallations[0];
      
      if (bestJava) {
        setFormData(prev => ({ ...prev, javaPath: bestJava.path }));
      }
    } catch (error) {
      errorHandler.handleError(error as Error, 'Load Java Installations');
      setJavaInstallations([]);
    } finally {
      setIsLoadingJava(false);
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

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'Enter' && e.ctrlKey) {
        if (currentStep < steps.length) {
          nextStep();
        } else {
          handleSubmit(e as any);
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [currentStep, onClose]);

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
    
    // Validate all steps before submission
    let isValid = true;
    for (let step = 1; step <= steps.length; step++) {
      if (!validateStep(step)) {
        isValid = false;
        break;
      }
    }
    
    if (!isValid) {
      // Focus on the first step with errors
      const firstErrorStep = steps.find((_, index) => {
        setCurrentStep(index + 1);
        return !validateStep(index + 1);
      });
      if (firstErrorStep) {
        setCurrentStep(firstErrorStep.id);
      }
      return;
    }
    
    setIsSubmitting(true);
    setSubmitProgress(0);
    
    try {
      // Create server data for API - match backend schema
      setSubmitProgress(10);
      const serverData = {
        name: formData.name,
        minecraft_version: formData.version,
        loader: formData.type,
        loader_version: 'latest',
        port: formData.serverPort,
        rcon_port: formData.rconPort,
        query_port: formData.queryPort,
        max_players: formData.maxPlayers,
        memory: formData.memory,
        java_args: Array.isArray(formData.javaArgs) ? formData.javaArgs : (formData.javaArgs as string).split(' '),
        server_args: [],
        auto_start: formData.settings.autoStart,
        auto_restart: formData.settings.autoRestart,
        world_name: formData.paths.world.replace('./', ''),
        difficulty: formData.difficulty,
        gamemode: formData.gamemode,
        pvp: formData.pvp,
        online_mode: formData.onlineMode,
        whitelist: formData.whitelist,
        enable_command_block: formData.enableCommandBlock,
        view_distance: formData.settings.viewDistance,
        simulation_distance: formData.settings.simulationDistance,
        motd: formData.motd,
        host: 'localhost',
        java_path: formData.javaPath || 'java',
        jvm_args: Array.isArray(formData.javaArgs) ? formData.javaArgs.join(' ') : (formData.javaArgs as string),
        server_jar: formData.serverJarPath || 'server.jar',
        rcon_password: formData.rconPassword,
        // Additional fields for advanced features
        description: formData.description,
        allow_flight: formData.allowFlight,
        allow_nether: formData.allowNether,
        allow_end: formData.allowEnd,
        enable_rcon: formData.enableRcon,
        enable_query: formData.enableQuery,
        paths: formData.paths,
        settings: formData.settings,
        gpu_enabled: formData.gpuEnabled,
        gpu_queue_size: formData.gpuQueueSize,
        gpu_max_workers: formData.gpuMaxWorkers,
        ha_enabled: formData.haEnabled,
        blue_green: formData.blueGreen,
        health_check_interval: formData.healthCheckInterval,
        composer_profile: formData.composerProfile,
        composer_timeout: formData.composerTimeout
      };

      setSubmitProgress(30);
      
      // Call the API to create the server
      const { api } = await import('@/lib/client');
      const createdServer = await api.createServer(serverData);
      
      setSubmitProgress(70);
      
      // Update the servers store to refresh the list
      const { useServers } = await import('@/store/servers-new');
      const { fetchServers } = useServers.getState();
      await fetchServers();

      setSubmitProgress(90);

      // Call the callback with the created server
      onServerCreated(createdServer);
      
      setSubmitProgress(100);
      
      // Show success message
      console.log('✅ Server created successfully:', createdServer);
      
      // Close the wizard after a brief delay to show success state
      setTimeout(() => {
        onClose();
      }, 1000);
    } catch (error) {
      console.error('❌ Failed to create server:', error);
      errorHandler.handleError(error as Error, 'Server Creation');
      
      // Show error message to user
      alert(`Failed to create server: ${error instanceof Error ? error.message : 'Unknown error occurred'}`);
    } finally {
      setIsSubmitting(false);
      setSubmitProgress(0);
    }
  };

  const generateId = (): string => {
    return Math.random().toString(36).substr(2, 9);
  };

  const progress = (currentStep / steps.length) * 100;

  return (
    <div 
      className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-2 sm:p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="server-creation-title"
      aria-describedby="server-creation-description"
    >
      <div className="w-full max-w-6xl max-h-[95vh] sm:max-h-[90vh] overflow-hidden shadow-2xl bg-gray-900 border border-gray-700 rounded-3xl">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-600 via-purple-600 to-indigo-600 px-8 py-6 text-white relative overflow-hidden">
          <div className="absolute inset-0 bg-black/10"></div>
          <div className="relative flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-14 h-14 bg-white/20 backdrop-blur-sm rounded-2xl flex items-center justify-center border border-white/30">
                <Server className="w-7 h-7" />
              </div>
              <div>
                <h2 
                  id="server-creation-title"
                  className="text-3xl font-bold tracking-tight"
                >
                  Create New Server
                </h2>
                <p 
                  id="server-creation-description"
                  className="text-blue-100 text-lg"
                >
                  Set up your Minecraft server with our guided wizard
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="w-10 h-10 bg-white/20 hover:bg-white/30 backdrop-blur-sm rounded-xl flex items-center justify-center transition-all duration-200 border border-white/30 hover:scale-105"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
          
          {/* Progress Bar */}
          <div className="mt-6">
            <div className="flex items-center justify-between mb-4">
              <span className="text-sm font-semibold text-gray-300">Step {currentStep} of {steps.length}</span>
              <span className="text-sm text-gray-400">{Math.round(progress)}% complete</span>
            </div>
            <div className="w-full bg-gray-700 rounded-full h-3 mb-6">
              <div 
                className="bg-gradient-to-r from-blue-500 via-purple-500 to-indigo-500 h-3 rounded-full transition-all duration-500 shadow-lg"
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="flex items-center justify-between">
              {steps.map((step, index) => {
                const Icon = step.icon;
                return (
                  <div key={step.id} className="flex items-center gap-3">
                    <div className={`w-10 h-10 rounded-2xl flex items-center justify-center text-sm font-bold transition-all duration-300 ${
                      currentStep >= step.id 
                        ? 'bg-gradient-to-r from-blue-500 to-purple-500 text-white shadow-lg scale-110' 
                        : 'bg-gray-700 text-gray-400 border border-gray-600'
                    }`}>
                      {currentStep > step.id ? <CheckCircle className="w-5 h-5" /> : <Icon className="w-5 h-5" />}
                    </div>
                    <div className="text-left">
                      <div className={`text-sm font-semibold ${
                        currentStep >= step.id 
                          ? 'text-blue-400' 
                          : 'text-gray-400'
                      }`}>
                        {step.title}
                      </div>
                      <div className={`text-xs ${
                        currentStep >= step.id 
                          ? 'text-blue-300' 
                          : 'text-gray-500'
                      }`}>
                        {step.description}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
        
        {/* Content Area */}
        <div className="bg-gray-800 p-0">
          <form onSubmit={handleSubmit} className="h-full">
            {/* Step 1: Server Type */}
            {currentStep === 1 && (
              <div className="p-8 space-y-8">
                <div className="text-center">
                  <h3 className="text-3xl font-bold text-white mb-3">Choose Your Server Type</h3>
                  <p className="text-gray-400 text-lg">Select a preset or customize your server configuration</p>
                </div>

                {/* Server Presets */}
                <div className="space-y-6">
                  <div className="text-center">
                    <h4 className="text-2xl font-bold flex items-center justify-center gap-3 mb-2 text-white">
                      <Sparkles className="h-6 w-6 text-yellow-400" />
                      Quick Start Presets
                    </h4>
                    <p className="text-gray-400">Choose a preset to get started quickly, or customize your own configuration below</p>
                    {presetApplied && (
                      <div className="mt-4 p-3 bg-green-900/30 border border-green-500/50 rounded-lg">
                        <p className="text-green-400 text-sm font-medium">
                          ✅ Preset applied successfully! Your server configuration has been updated.
                        </p>
                      </div>
                    )}
                  </div>
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
                    {serverPresets.map((preset) => (
                      <div 
                        key={preset.id}
                        className={`group cursor-pointer transition-all duration-300 hover:shadow-2xl hover:scale-105 bg-gray-800 border rounded-2xl p-6 ${
                          selectedPreset === preset.id 
                            ? 'ring-2 ring-blue-500 bg-gradient-to-br from-blue-900/50 to-indigo-900/50 shadow-xl border-blue-500' 
                            : 'hover:bg-gradient-to-br hover:from-gray-700 hover:to-gray-800 border-gray-600 hover:border-gray-500'
                        }`}
                        onClick={() => applyPreset(preset.id)}
                      >
                        <div className="flex items-start justify-between mb-4">
                          <div className="flex-1">
                            <h5 className="font-bold text-lg mb-2 text-white">{preset.name}</h5>
                            <p className="text-sm text-gray-400 leading-relaxed">{preset.description}</p>
                          </div>
                          {selectedPreset === preset.id && (
                            <div className={`p-2 rounded-full transition-all duration-300 ${
                              presetApplied === preset.id 
                                ? 'bg-green-500 animate-pulse' 
                                : 'bg-blue-500'
                            }`}>
                              <CheckCircle className="h-5 w-5 text-white" />
                            </div>
                          )}
                        </div>
                        <div className="space-y-3">
                          <div className="flex items-center gap-2">
                            <span className="px-3 py-1 bg-blue-600 text-white text-xs font-medium rounded-full">{preset.type}</span>
                            <span className="px-3 py-1 bg-gray-600 text-gray-200 text-xs font-medium rounded-full border border-gray-500">{preset.version}</span>
                          </div>
                          <div className="flex flex-wrap gap-2">
                            {preset.features.map((feature) => (
                              <span key={feature} className="px-2 py-1 bg-gray-700 text-gray-300 text-xs font-medium rounded-lg border border-gray-600">
                                {feature}
                              </span>
                            ))}
                          </div>
                          <div className="flex items-center justify-between pt-2 border-t border-gray-600">
                            <span className="text-xs text-gray-400 font-medium">Memory:</span>
                            <span className="text-sm font-bold text-white">{preset.memory}MB</span>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="border-t border-gray-600 my-8"></div>

                {/* Custom Configuration */}
                <div className="space-y-8">
                  <div className="text-center">
                    <h4 className="text-2xl font-bold flex items-center justify-center gap-3 mb-2 text-white">
                      <Settings className="h-6 w-6 text-blue-400" />
                      Custom Configuration
                    </h4>
                    <p className="text-gray-400">Or create your own custom server configuration</p>
                  </div>
                  
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 sm:gap-8">
                    <div className="space-y-3">
                      <Label htmlFor="serverName" className="text-sm font-semibold text-gray-300 flex items-center gap-2">
                        Server Name <span className="text-red-400">*</span>
                      </Label>
                      <Input
                        id="serverName"
                        value={formData.name}
                        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        placeholder="My Awesome Server"
                        className={`h-12 text-base bg-gray-700 border-gray-600 text-white placeholder-gray-400 ${errors.name ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : 'focus:border-blue-500 focus:ring-blue-500'}`}
                      />
                      {errors.name && (
                        <div className="flex items-center gap-2 text-sm text-red-400">
                          <AlertTriangle className="h-4 w-4" />
                          {errors.name}
                        </div>
                      )}
                    </div>
                
                    <div className="space-y-3">
                      <Label htmlFor="serverType" className="text-sm font-semibold text-gray-300 flex items-center gap-2">
                        Server Type <span className="text-red-400">*</span>
                      </Label>
                      <Select
                        value={formData.type}
                        onValueChange={(value) => setFormData({ ...formData, type: value as any })}
                      >
                        <SelectTrigger className={`h-12 text-base bg-gray-700 border-gray-600 text-white ${errors.type ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : 'focus:border-blue-500 focus:ring-blue-500'}`}>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent className="bg-gray-800 border-gray-600">
                          <SelectItem value="vanilla" className="text-white hover:bg-gray-700">
                            <div className="flex items-center gap-3">
                              <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                              <span className="font-medium">Vanilla</span>
                              <span className="px-2 py-1 bg-gray-600 text-gray-200 text-xs rounded-full border border-gray-500">Official</span>
                            </div>
                          </SelectItem>
                          <SelectItem value="forge" className="text-white hover:bg-gray-700">
                            <div className="flex items-center gap-3">
                              <div className="w-3 h-3 bg-orange-500 rounded-full"></div>
                              <span className="font-medium">Forge</span>
                              <span className="px-2 py-1 bg-gray-600 text-gray-200 text-xs rounded-full border border-gray-500">Mods</span>
                            </div>
                          </SelectItem>
                          <SelectItem value="fabric" className="text-white hover:bg-gray-700">
                            <div className="flex items-center gap-3">
                              <div className="w-3 h-3 bg-purple-500 rounded-full"></div>
                              <span className="font-medium">Fabric</span>
                              <span className="px-2 py-1 bg-gray-600 text-gray-200 text-xs rounded-full border border-gray-500">Performance</span>
                            </div>
                          </SelectItem>
                          <SelectItem value="quilt" className="text-white hover:bg-gray-700">
                            <div className="flex items-center gap-3">
                              <div className="w-3 h-3 bg-pink-500 rounded-full"></div>
                              <span className="font-medium">Quilt</span>
                              <span className="px-2 py-1 bg-gray-600 text-gray-200 text-xs rounded-full border border-gray-500">Fork</span>
                            </div>
                          </SelectItem>
                        </SelectContent>
                      </Select>
                      {errors.type && (
                        <div className="flex items-center gap-2 text-sm text-red-400">
                          <AlertTriangle className="h-4 w-4" />
                          {errors.type}
                        </div>
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

                  <div className="space-y-4">
                    <Label htmlFor="serverJarPath" className="text-sm font-semibold text-gray-700 flex items-center gap-2">
                      <FolderOpen className="h-4 w-4" />
                      Server JAR File (Optional)
                    </Label>
                    <div
                      className={`relative flex gap-3 rounded-xl border-2 transition-all duration-200 ${
                        isDragOverJar 
                          ? 'border-blue-400 border-dashed bg-blue-50 scale-105 shadow-lg' 
                          : 'border-gray-200 hover:border-gray-300'
                      } p-4`}
                      onDragEnter={(e) => { e.preventDefault(); setIsDragOverJar(true); }}
                      onDragOver={(e) => { e.preventDefault(); setIsDragOverJar(true); }}
                      onDragLeave={() => setIsDragOverJar(false)}
                    >
                      <div className="flex-1">
                        <Input
                          id="serverJarPath"
                          value={formData.serverJarPath}
                          onChange={(e) => setFormData({ ...formData, serverJarPath: e.target.value })}
                          placeholder="Drag a server.jar here or click Browse..."
                          className="h-12 text-base border-0 bg-transparent focus:ring-0"
                        />
                      </div>
                      <Button
                        type="button"
                        variant="outline"
                        className="h-12 px-6 font-medium"
                        onClick={async () => {
                          try {
                            const selected = await openDialog({
                              multiple: false,
                              filters: [
                                { name: 'Java Archives', extensions: ['jar'] },
                              ],
                            });
                            const path = Array.isArray(selected) ? selected[0] : selected;
                            if (typeof path === 'string' && path.length > 0) {
                              setFormData(prev => ({ ...prev, serverJarPath: path }));
                              // Optional: naive version hint from filename
                              const lower = path.toLowerCase();
                              if (lower.includes('1.21')) {
                                setFormData(prev => ({ ...prev, version: '1.21.1' }));
                              } else if (lower.includes('1.20.6')) {
                                setFormData(prev => ({ ...prev, version: '1.20.6' }));
                              }
                            }
                          } catch (e) {
                            const fallback = prompt('Enter path to server.jar file:');
                            if (fallback) {
                              setFormData(prev => ({ ...prev, serverJarPath: fallback }));
                            }
                          }
                        }}
                      >
                        <FolderOpen className="h-4 w-4 mr-2" />
                        Browse
                      </Button>
                    </div>
                    {isDragOverJar && (
                      <div className="flex items-center gap-2 text-sm text-blue-600 font-medium">
                        <Download className="h-4 w-4 animate-bounce" />
                        Drop the .jar file to select it
                      </div>
                    )}
                    <div className="bg-gray-50 rounded-lg p-4">
                      <div className="flex items-start gap-3">
                        <Info className="h-5 w-5 text-blue-500 mt-0.5" />
                        <div className="text-sm text-gray-600">
                          {formData.type === 'vanilla'
                            ? 'Optional for Vanilla: If left empty, Guardian will automatically download the official Mojang server.jar for the selected version.'
                            : 'Provide the server.jar for this loader, or the installer-generated jar.'}
                        </div>
                      </div>
                    </div>
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
                  <Card className="border-0 shadow-lg bg-gradient-to-br from-white to-gray-50">
                    <CardHeader className="bg-gradient-to-r from-blue-50 to-indigo-50">
                      <CardTitle className="flex items-center gap-3 text-xl">
                        <div className="p-2 bg-blue-100 rounded-lg">
                          <Cpu className="h-6 w-6 text-blue-600" />
                        </div>
                        Java Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-6 p-6">
                      <div className="space-y-3">
                        <Label htmlFor="javaPath" className="text-sm font-semibold text-gray-700 flex items-center gap-2">
                          Java Installation {formData.type !== 'vanilla' && <span className="text-red-500">*</span>}
                        </Label>
                        {isLoadingJava ? (
                          <div className="flex items-center gap-3 p-4 bg-gray-50 rounded-lg">
                            <Loader2 className="h-5 w-5 animate-spin text-blue-500" />
                            <span className="text-sm text-gray-600">Scanning for Java installations...</span>
                          </div>
                        ) : (
                          <Select
                            value={formData.javaPath}
                            onValueChange={(value) => setFormData({ ...formData, javaPath: value })}
                          >
                            <SelectTrigger className={`h-12 text-base ${errors.javaPath ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : 'focus:border-blue-500 focus:ring-blue-500'}`}>
                              <SelectValue placeholder="Select Java installation" />
                            </SelectTrigger>
                            <SelectContent>
                              {javaInstallations.length === 0 ? (
                                <div className="p-4 text-center text-gray-500">
                                  <AlertTriangle className="h-8 w-8 mx-auto mb-2 text-yellow-500" />
                                  <p className="text-sm">No Java installations found</p>
                                  <p className="text-xs text-gray-400 mt-1">Please install Java to continue</p>
                                </div>
                              ) : (
                                javaInstallations.map((java) => (
                                  <SelectItem key={java.id} value={java.path}>
                                    <div className="flex items-center gap-3">
                                      <Badge variant="outline" className="font-medium">Java {java.version}</Badge>
                                      <span className="text-sm text-gray-600 truncate max-w-xs">
                                        {java.path}
                                      </span>
                                    </div>
                                  </SelectItem>
                                ))
                              )}
                            </SelectContent>
                          </Select>
                        )}
                        {errors.javaPath && (
                          <div className="flex items-center gap-2 text-sm text-red-600">
                            <AlertTriangle className="h-4 w-4" />
                            {errors.javaPath}
                          </div>
                        )}
                      </div>

                      <div className="space-y-3">
                        <Label htmlFor="memory" className="text-sm font-semibold text-gray-700 flex items-center gap-2">
                          Memory Allocation <span className="text-red-500">*</span>
                        </Label>
                        <div className="space-y-3">
                          <div className="relative">
                            <Input
                              id="memory"
                              type="number"
                              value={formData.memory}
                              onChange={(e) => setFormData({ ...formData, memory: parseInt(e.target.value) })}
                              min="512"
                              max="32768"
                              step="512"
                              className={`h-12 text-base pr-16 ${errors.memory ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : 'focus:border-blue-500 focus:ring-blue-500'}`}
                            />
                            <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-sm text-gray-500 font-medium">
                              MB
                            </div>
                          </div>
                          <div className="bg-blue-50 rounded-lg p-4">
                            <div className="flex items-start gap-3">
                              <Info className="h-5 w-5 text-blue-500 mt-0.5" />
                              <div className="text-sm text-blue-700">
                                <p className="font-medium">Recommended: {formData.type === 'vanilla' ? '2-4GB' : '4-8GB'} for {formData.type} servers</p>
                                <p className="text-xs text-blue-600 mt-1">Use multiples of 512MB for optimal performance</p>
                              </div>
                            </div>
                          </div>
                        </div>
                        {errors.memory && (
                          <div className="flex items-center gap-2 text-sm text-red-600">
                            <AlertTriangle className="h-4 w-4" />
                            {errors.memory}
                          </div>
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
                      min="1024"
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
                      min="1024"
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
                      min="1024"
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
                    className={errors.maxRestarts ? 'border-red-500' : ''}
                  />
                  {errors.maxRestarts && (
                    <p className="text-sm text-red-500">{errors.maxRestarts}</p>
                  )}
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
                      className={errors.backupInterval ? 'border-red-500' : ''}
                    />
                        {errors.backupInterval && (
                          <p className="text-sm text-red-500">{errors.backupInterval}</p>
                        )}
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
                      className={errors.backupRetention ? 'border-red-500' : ''}
                    />
                        {errors.backupRetention && (
                          <p className="text-sm text-red-500">{errors.backupRetention}</p>
                        )}
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
                          className={errors.viewDistance ? 'border-red-500' : ''}
                        />
                        {errors.viewDistance && (
                          <p className="text-sm text-red-500">{errors.viewDistance}</p>
                        )}
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
                          className={errors.simulationDistance ? 'border-red-500' : ''}
                        />
                        {errors.simulationDistance && (
                          <p className="text-sm text-red-500">{errors.simulationDistance}</p>
                        )}
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
                          className={errors.worldBorder ? 'border-red-500' : ''}
                        />
                        {errors.worldBorder && (
                          <p className="text-sm text-red-500">{errors.worldBorder}</p>
                        )}
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

            {/* Progress Bar for Server Creation */}
            {isSubmitting && (
              <div className="border-t bg-gradient-to-r from-blue-50 to-indigo-50 px-8 py-4">
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-blue-700">Creating Server...</span>
                    <span className="text-sm text-blue-600">{submitProgress}%</span>
                  </div>
                  <Progress 
                    value={submitProgress} 
                    className="h-2 bg-blue-100" 
                    aria-label={`Server creation progress: ${submitProgress}%`}
                  />
                </div>
              </div>
            )}

            {/* Navigation Buttons */}
            <div className="border-t border-gray-600 bg-gray-800 px-8 py-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  {currentStep > 1 && (
                    <Button 
                      type="button" 
                      variant="outline" 
                      onClick={prevStep}
                      className="flex items-center gap-2 h-12 px-6 font-medium bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:text-white"
                    >
                      <ChevronLeft className="h-4 w-4" />
                      Previous
                    </Button>
                  )}
                </div>
                
                <div className="flex items-center gap-4">
                  <Button 
                    type="button" 
                    variant="ghost" 
                    onClick={onClose}
                    className="h-12 px-6 text-gray-400 hover:text-white hover:bg-gray-700 font-medium"
                  >
                    Cancel
                  </Button>
                  
                  {currentStep < steps.length ? (
                    <Button 
                      type="button" 
                      onClick={nextStep}
                      className="flex items-center gap-2 h-12 px-8 bg-gradient-to-r from-blue-500 to-indigo-600 hover:from-blue-600 hover:to-indigo-700 text-white font-semibold shadow-lg hover:shadow-xl transition-all duration-200"
                    >
                      Next
                      <ChevronRight className="h-4 w-4" />
                    </Button>
                  ) : (
                    <Button 
                      type="submit" 
                      disabled={isSubmitting}
                      className="flex items-center gap-2 h-12 px-8 bg-gradient-to-r from-green-500 to-emerald-600 hover:from-green-600 hover:to-emerald-700 text-white font-semibold shadow-lg hover:shadow-xl transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isSubmitting ? (
                        <>
                          <Loader2 className="h-5 w-5 animate-spin" />
                          Creating Server... {submitProgress}%
                        </>
                      ) : (
                        <>
                          <CheckCircle className="h-5 w-5" />
                          Create Server
                        </>
                      )}
                    </Button>
                  )}
                </div>
              </div>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};
