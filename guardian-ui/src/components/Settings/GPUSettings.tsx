import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useServers } from '@/store/servers-new';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { GPUMetrics } from './GPUMetrics';
import { 
  Cpu, 
  MemoryStick, 
  AlertTriangle,
  CheckCircle,
  Info,
  Zap,
  Activity,
  Monitor,
  Layers,
  Shield,
  Beaker
} from 'lucide-react';

interface GPUSettingsData {
  // GPU Worker Settings
  enableGpuWorker: boolean;
  gpuWorkerPort: number;
  gpuWorkerHost: string;
  gpuWorkerTimeout: number;
  gpuWorkerRetries: number;
  
  // GPU Selection
  gpuDevice: string;
  gpuMemoryLimit: string;
  gpuComputeCapability: string;
  gpuDriverVersion: string;
  
  // Performance Settings
  gpuThreads: number;
  gpuBatchSize: number;
  gpuQueueSize: number;
  gpuMaxConcurrent: number;
  
  // Memory Management
  gpuMemoryPool: string;
  gpuMemoryThreshold: number;
  gpuMemoryCleanup: boolean;
  gpuMemoryCompression: boolean;
  
  // Workload Settings
  enableChunkGeneration: boolean;
  enableWorldgen: boolean;
  enablePregen: boolean;
  enableSharding: boolean;
  enableBackups: boolean;
  
  // Quality Settings
  chunkQuality: 'low' | 'medium' | 'high' | 'ultra';
  worldgenQuality: 'low' | 'medium' | 'high' | 'ultra';
  pregenQuality: 'low' | 'medium' | 'high' | 'ultra';
  
  // Advanced Settings
  enableGpuProfiling: boolean;
  gpuProfilingInterval: number;
  enableGpuDebug: boolean;
  gpuDebugLevel: 'none' | 'basic' | 'detailed' | 'verbose';
  
  // Fallback Settings
  enableCpuFallback: boolean;
  cpuFallbackThreshold: number;
  cpuFallbackTimeout: number;
  
  // Monitoring
  enableGpuMetrics: boolean;
  gpuMetricsInterval: number;
  gpuMetricsRetention: number;
  
  // Security
  enableGpuSandbox: boolean;
  gpuSandboxLevel: 'none' | 'basic' | 'strict' | 'paranoid';
  gpuResourceLimits: boolean;
}

export const GPUSettings: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { 
    fetchSettings, 
    updateSettings,
    settings: serverSettings 
  } = useServers();
  
  const [settings, setSettings] = useState<GPUSettingsData>({
    // GPU Worker Settings
    enableGpuWorker: true,
    gpuWorkerPort: 8080,
    gpuWorkerHost: 'localhost',
    gpuWorkerTimeout: 30000,
    gpuWorkerRetries: 3,
    
    // GPU Selection
    gpuDevice: 'auto',
    gpuMemoryLimit: '2G',
    gpuComputeCapability: '7.5',
    gpuDriverVersion: 'auto',
    
    // Performance Settings
    gpuThreads: 4,
    gpuBatchSize: 1024,
    gpuQueueSize: 1000,
    gpuMaxConcurrent: 8,
    
    // Memory Management
    gpuMemoryPool: 'unified',
    gpuMemoryThreshold: 80,
    gpuMemoryCleanup: true,
    gpuMemoryCompression: false,
    
    // Workload Settings
    enableChunkGeneration: true,
    enableWorldgen: true,
    enablePregen: true,
    enableSharding: false,
    enableBackups: false,
    
    // Quality Settings
    chunkQuality: 'high',
    worldgenQuality: 'high',
    pregenQuality: 'medium',
    
    // Advanced Settings
    enableGpuProfiling: false,
    gpuProfilingInterval: 1000,
    enableGpuDebug: false,
    gpuDebugLevel: 'none',
    
    // Fallback Settings
    enableCpuFallback: true,
    cpuFallbackThreshold: 5000,
    cpuFallbackTimeout: 10000,
    
    // Monitoring
    enableGpuMetrics: true,
    gpuMetricsInterval: 5000,
    gpuMetricsRetention: 24,
    
    // Security
    enableGpuSandbox: true,
    gpuSandboxLevel: 'basic',
    gpuResourceLimits: true
  });
  // Loading state removed for now
  // const [isLoading, setIsLoading] = useState(false);
  // Changes tracking removed for now
  // const [hasChanges, setHasChanges] = useState(false);

  const loadSettings = async () => {
    if (!serverId) return;
    
    try {
      // Load server configuration
      await fetchSettings(serverId);
    } catch (error) {
      console.error('Failed to fetch GPU settings:', error);
    }
  };

  useEffect(() => {
    loadSettings();
  }, []);

  // Sync settings with server store data
  useEffect(() => {
    if (serverId && serverSettings[serverId]) {
      const serverData = serverSettings[serverId];
      if (serverData.gpu) {
        setSettings(prev => ({
          ...prev,
          ...serverData.gpu,
        }));
      }
    }
  }, [serverId, serverSettings]);

  const handleSettingChange = async (key: keyof GPUSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    
    if (!serverId) return;
    
    try {
      // Update server configuration
      const currentServerSettings = serverSettings[serverId] || {};
      await updateSettings(serverId, {
        ...currentServerSettings,
        gpu: {
          ...currentServerSettings.gpu,
          enabled: settings.enableGpuWorker,
          queueSize: settings.gpuQueueSize || 100,
          [key]: value
        }
      });
    } catch (error) {
      console.error('Failed to update GPU settings:', error);
    }
  };

  const getValidationStatus = (key: keyof GPUSettingsData) => {
    const value = settings[key];
    
    // Ensure value is a number for comparison
    const numValue = typeof value === 'number' ? value : 0;
    
    switch (key) {
      case 'gpuWorkerPort':
        return numValue < 1 || numValue > 65535 ? 'error' : 'success';
      case 'gpuThreads':
        return numValue < 1 || numValue > 32 ? 'error' : 'success';
      case 'gpuBatchSize':
        return numValue < 1 || numValue > 10000 ? 'error' : 'success';
      case 'gpuQueueSize':
        return numValue < 1 || numValue > 10000 ? 'error' : 'success';
      case 'gpuMaxConcurrent':
        return numValue < 1 || numValue > 16 ? 'error' : 'success';
      case 'gpuMemoryThreshold':
        return numValue < 1 || numValue > 100 ? 'error' : 'success';
      case 'gpuProfilingInterval':
        return numValue < 100 || numValue > 10000 ? 'error' : 'success';
      case 'cpuFallbackThreshold':
        return numValue < 1000 || numValue > 30000 ? 'error' : 'success';
      case 'cpuFallbackTimeout':
        return numValue < 1000 || numValue > 60000 ? 'error' : 'success';
      case 'gpuMetricsInterval':
        return numValue < 1000 || numValue > 60000 ? 'error' : 'success';
      case 'gpuMetricsRetention':
        return numValue < 1 || numValue > 168 ? 'error' : 'success';
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

  const getQualityColor = (quality: string) => {
    switch (quality) {
      case 'low': return 'bg-red-500';
      case 'medium': return 'bg-yellow-500';
      case 'high': return 'bg-green-500';
      case 'ultra': return 'bg-purple-500';
      default: return 'bg-gray-500';
    }
  };

  const getQualityLabel = (quality: string) => {
    switch (quality) {
      case 'low': return 'Low';
      case 'medium': return 'Medium';
      case 'high': return 'High';
      case 'ultra': return 'Ultra';
      default: return 'Unknown';
    }
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Experimental Warning */}
      <Card className="border-orange-200 bg-orange-50">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-orange-800">
            <Beaker className="h-5 w-5" />
            Experimental Feature
          </CardTitle>
          <CardDescription className="text-orange-700">
            GPU acceleration is an experimental feature. It may cause instability or crashes. 
            Use with caution and ensure you have proper GPU drivers installed.
          </CardDescription>
        </CardHeader>
      </Card>

      {/* GPU Metrics */}
      <GPUMetrics />

      {/* GPU Worker Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Monitor className="h-5 w-5" />
            <span>GPU Worker Settings</span>
          </CardTitle>
          <CardDescription>
            Configure GPU worker connection and communication
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableGpuWorker">Enable GPU Worker</Label>
                  <p className="text-sm text-muted-foreground">Enable GPU-accelerated processing</p>
                </div>
                <Switch
                  id="enableGpuWorker"
                  checked={settings.enableGpuWorker}
                  onCheckedChange={(checked) => handleSettingChange('enableGpuWorker', checked)}
                />
              </div>
              
              {settings.enableGpuWorker && (
                <>
                  <div>
                    <Label htmlFor="gpuWorkerHost">GPU Worker Host</Label>
                    <Input
                      id="gpuWorkerHost"
                      value={settings.gpuWorkerHost}
                      onChange={(e) => handleSettingChange('gpuWorkerHost', e.target.value)}
                      placeholder="localhost"
                    />
                  </div>
                  
                  <div>
                    <Label htmlFor="gpuWorkerPort">GPU Worker Port</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="gpuWorkerPort"
                        type="number"
                        value={settings.gpuWorkerPort}
                        onChange={(e) => handleSettingChange('gpuWorkerPort', parseInt(e.target.value))}
                        min="1"
                        max="65535"
                      />
                      {getStatusIcon(getValidationStatus('gpuWorkerPort'))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="gpuWorkerTimeout">Worker Timeout (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuWorkerTimeout"
                    type="number"
                    value={settings.gpuWorkerTimeout}
                    onChange={(e) => handleSettingChange('gpuWorkerTimeout', parseInt(e.target.value))}
                    min="1000"
                    max="300000"
                  />
                  {getStatusIcon(getValidationStatus('gpuWorkerTimeout'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="gpuWorkerRetries">Worker Retries</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuWorkerRetries"
                    type="number"
                    value={settings.gpuWorkerRetries}
                    onChange={(e) => handleSettingChange('gpuWorkerRetries', parseInt(e.target.value))}
                    min="0"
                    max="10"
                  />
                  {getStatusIcon(getValidationStatus('gpuWorkerRetries'))}
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* GPU Selection */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Cpu className="h-5 w-5" />
            <span>GPU Selection</span>
          </CardTitle>
          <CardDescription>
            Configure GPU device and memory allocation
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="gpuDevice">GPU Device</Label>
                <Select value={settings.gpuDevice} onValueChange={(value) => handleSettingChange('gpuDevice', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto">Auto-detect</SelectItem>
                    <SelectItem value="cuda:0">CUDA Device 0</SelectItem>
                    <SelectItem value="cuda:1">CUDA Device 1</SelectItem>
                    <SelectItem value="opencl:0">OpenCL Device 0</SelectItem>
                    <SelectItem value="opencl:1">OpenCL Device 1</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="gpuMemoryLimit">GPU Memory Limit</Label>
                <Input
                  id="gpuMemoryLimit"
                  value={settings.gpuMemoryLimit}
                  onChange={(e) => handleSettingChange('gpuMemoryLimit', e.target.value)}
                  placeholder="e.g., 2G, 2048M"
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="gpuComputeCapability">Compute Capability</Label>
                <Input
                  id="gpuComputeCapability"
                  value={settings.gpuComputeCapability}
                  onChange={(e) => handleSettingChange('gpuComputeCapability', e.target.value)}
                  placeholder="e.g., 7.5, 8.6"
                />
              </div>
              
              <div>
                <Label htmlFor="gpuDriverVersion">Driver Version</Label>
                <Input
                  id="gpuDriverVersion"
                  value={settings.gpuDriverVersion}
                  onChange={(e) => handleSettingChange('gpuDriverVersion', e.target.value)}
                  placeholder="auto"
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Performance Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>Performance Settings</span>
          </CardTitle>
          <CardDescription>
            Configure GPU performance and concurrency
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="gpuThreads">GPU Threads</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuThreads"
                    type="number"
                    value={settings.gpuThreads}
                    onChange={(e) => handleSettingChange('gpuThreads', parseInt(e.target.value))}
                    min="1"
                    max="32"
                  />
                  {getStatusIcon(getValidationStatus('gpuThreads'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="gpuBatchSize">Batch Size</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuBatchSize"
                    type="number"
                    value={settings.gpuBatchSize}
                    onChange={(e) => handleSettingChange('gpuBatchSize', parseInt(e.target.value))}
                    min="1"
                    max="10000"
                  />
                  {getStatusIcon(getValidationStatus('gpuBatchSize'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="gpuQueueSize">Queue Size</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuQueueSize"
                    type="number"
                    value={settings.gpuQueueSize}
                    onChange={(e) => handleSettingChange('gpuQueueSize', parseInt(e.target.value))}
                    min="1"
                    max="10000"
                  />
                  {getStatusIcon(getValidationStatus('gpuQueueSize'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="gpuMaxConcurrent">Max Concurrent</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuMaxConcurrent"
                    type="number"
                    value={settings.gpuMaxConcurrent}
                    onChange={(e) => handleSettingChange('gpuMaxConcurrent', parseInt(e.target.value))}
                    min="1"
                    max="16"
                  />
                  {getStatusIcon(getValidationStatus('gpuMaxConcurrent'))}
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Memory Management */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <MemoryStick className="h-5 w-5" />
            <span>Memory Management</span>
          </CardTitle>
          <CardDescription>
            Configure GPU memory allocation and management
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="gpuMemoryPool">Memory Pool</Label>
                <Select value={settings.gpuMemoryPool} onValueChange={(value) => handleSettingChange('gpuMemoryPool', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="unified">Unified Memory</SelectItem>
                    <SelectItem value="dedicated">Dedicated Memory</SelectItem>
                    <SelectItem value="shared">Shared Memory</SelectItem>
                    <SelectItem value="hybrid">Hybrid Memory</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="gpuMemoryThreshold">Memory Threshold (%)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="gpuMemoryThreshold"
                    type="number"
                    value={settings.gpuMemoryThreshold}
                    onChange={(e) => handleSettingChange('gpuMemoryThreshold', parseInt(e.target.value))}
                    min="1"
                    max="100"
                  />
                  {getStatusIcon(getValidationStatus('gpuMemoryThreshold'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="gpuMemoryCleanup">Memory Cleanup</Label>
                  <p className="text-sm text-muted-foreground">Enable automatic memory cleanup</p>
                </div>
                <Switch
                  id="gpuMemoryCleanup"
                  checked={settings.gpuMemoryCleanup}
                  onCheckedChange={(checked) => handleSettingChange('gpuMemoryCleanup', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="gpuMemoryCompression">Memory Compression</Label>
                  <p className="text-sm text-muted-foreground">Enable memory compression</p>
                </div>
                <Switch
                  id="gpuMemoryCompression"
                  checked={settings.gpuMemoryCompression}
                  onCheckedChange={(checked) => handleSettingChange('gpuMemoryCompression', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Workload Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Layers className="h-5 w-5" />
            <span>Workload Settings</span>
          </CardTitle>
          <CardDescription>
            Configure which workloads use GPU acceleration
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableChunkGeneration">Chunk Generation</Label>
                  <p className="text-sm text-muted-foreground">Enable GPU-accelerated chunk generation</p>
                </div>
                <Switch
                  id="enableChunkGeneration"
                  checked={settings.enableChunkGeneration}
                  onCheckedChange={(checked) => handleSettingChange('enableChunkGeneration', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableWorldgen">World Generation</Label>
                  <p className="text-sm text-muted-foreground">Enable GPU-accelerated world generation</p>
                </div>
                <Switch
                  id="enableWorldgen"
                  checked={settings.enableWorldgen}
                  onCheckedChange={(checked) => handleSettingChange('enableWorldgen', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enablePregen">Pre-generation</Label>
                  <p className="text-sm text-muted-foreground">Enable GPU-accelerated pre-generation</p>
                </div>
                <Switch
                  id="enablePregen"
                  checked={settings.enablePregen}
                  onCheckedChange={(checked) => handleSettingChange('enablePregen', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableSharding">Sharding</Label>
                  <p className="text-sm text-muted-foreground">Enable GPU-accelerated sharding</p>
                </div>
                <Switch
                  id="enableSharding"
                  checked={settings.enableSharding}
                  onCheckedChange={(checked) => handleSettingChange('enableSharding', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableBackups">Backups</Label>
                  <p className="text-sm text-muted-foreground">Enable GPU-accelerated backups</p>
                </div>
                <Switch
                  id="enableBackups"
                  checked={settings.enableBackups}
                  onCheckedChange={(checked) => handleSettingChange('enableBackups', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Quality Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Zap className="h-5 w-5" />
            <span>Quality Settings</span>
          </CardTitle>
          <CardDescription>
            Configure quality levels for different workloads
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="chunkQuality">Chunk Quality</Label>
                <Select value={settings.chunkQuality} onValueChange={(value) => handleSettingChange('chunkQuality', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">Low</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="ultra">Ultra</SelectItem>
                  </SelectContent>
                </Select>
                <div className="flex items-center space-x-2 mt-2">
                  <div className={`w-3 h-3 rounded-full ${getQualityColor(settings.chunkQuality)}`} />
                  <span className="text-sm text-muted-foreground">{getQualityLabel(settings.chunkQuality)}</span>
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="worldgenQuality">Worldgen Quality</Label>
                <Select value={settings.worldgenQuality} onValueChange={(value) => handleSettingChange('worldgenQuality', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">Low</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="ultra">Ultra</SelectItem>
                  </SelectContent>
                </Select>
                <div className="flex items-center space-x-2 mt-2">
                  <div className={`w-3 h-3 rounded-full ${getQualityColor(settings.worldgenQuality)}`} />
                  <span className="text-sm text-muted-foreground">{getQualityLabel(settings.worldgenQuality)}</span>
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="pregenQuality">Pregen Quality</Label>
                <Select value={settings.pregenQuality} onValueChange={(value) => handleSettingChange('pregenQuality', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">Low</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="ultra">Ultra</SelectItem>
                  </SelectContent>
                </Select>
                <div className="flex items-center space-x-2 mt-2">
                  <div className={`w-3 h-3 rounded-full ${getQualityColor(settings.pregenQuality)}`} />
                  <span className="text-sm text-muted-foreground">{getQualityLabel(settings.pregenQuality)}</span>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Fallback Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Fallback Settings</span>
          </CardTitle>
          <CardDescription>
            Configure CPU fallback when GPU is unavailable
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableCpuFallback">Enable CPU Fallback</Label>
                  <p className="text-sm text-muted-foreground">Fallback to CPU when GPU fails</p>
                </div>
                <Switch
                  id="enableCpuFallback"
                  checked={settings.enableCpuFallback}
                  onCheckedChange={(checked) => handleSettingChange('enableCpuFallback', checked)}
                />
              </div>
              
              {settings.enableCpuFallback && (
                <div>
                  <Label htmlFor="cpuFallbackThreshold">Fallback Threshold (ms)</Label>
                  <div className="flex items-center space-x-2">
                    <Input
                      id="cpuFallbackThreshold"
                      type="number"
                      value={settings.cpuFallbackThreshold}
                      onChange={(e) => handleSettingChange('cpuFallbackThreshold', parseInt(e.target.value))}
                      min="1000"
                      max="30000"
                    />
                    {getStatusIcon(getValidationStatus('cpuFallbackThreshold'))}
                  </div>
                </div>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableCpuFallback && (
                <div>
                  <Label htmlFor="cpuFallbackTimeout">Fallback Timeout (ms)</Label>
                  <div className="flex items-center space-x-2">
                    <Input
                      id="cpuFallbackTimeout"
                      type="number"
                      value={settings.cpuFallbackTimeout}
                      onChange={(e) => handleSettingChange('cpuFallbackTimeout', parseInt(e.target.value))}
                      min="1000"
                      max="60000"
                    />
                    {getStatusIcon(getValidationStatus('cpuFallbackTimeout'))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
