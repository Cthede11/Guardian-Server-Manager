import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useServers } from '@/store/servers-new';
import { Button } from '@/components/ui/button';
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
  Activity,
  Network
} from 'lucide-react';

interface JVMSettingsData {
  // Memory Settings
  initialHeapSize: string;
  maxHeapSize: string;
  metaspaceSize: string;
  maxMetaspaceSize: string;
  directMemorySize: string;
  
  // GC Settings
  gcType: 'G1GC' | 'ParallelGC' | 'SerialGC' | 'ZGC' | 'ShenandoahGC';
  gcTuning: string;
  gcLogging: boolean;
  gcLogFile: string;
  
  // Performance Settings
  useG1GC: boolean;
  useStringDeduplication: boolean;
  useCompressedOops: boolean;
  useBiasedLocking: boolean;
  useTieredCompilation: boolean;
  useServer: boolean;
  useLargePages: boolean;
  useTransparentHugePages: boolean;
  
  // JVM Arguments
  customJvmArgs: string;
  additionalArgs: string;
  
  // Monitoring
  enableJmx: boolean;
  jmxPort: number;
  jmxHost: string;
  enableFlightRecorder: boolean;
  flightRecorderOptions: string;
  
  // Network Settings
  networkBufferSize: string;
  maxNetworkThreads: number;
  useEpoll: boolean;
  useKQueue: boolean;
  
  // Security
  enableSecurityManager: boolean;
  securityPolicy: string;
  trustedCodebase: string;
  
  // Logging
  enableGC: boolean;
  enableClassUnloading: boolean;
  enableVerboseGC: boolean;
  logLevel: 'INFO' | 'DEBUG' | 'WARN' | 'ERROR';
  
  // Advanced
  useAikarFlags: boolean;
  useOptimizedFlags: boolean;
  customFlags: string;
}

export const JVMSettings: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { 
    fetchSettings, 
    updateSettings,
    settings 
  } = useServers();
  
  const [settings, setSettings] = useState<JVMSettingsData>({
    // Memory Settings
    initialHeapSize: '2G',
    maxHeapSize: '4G',
    metaspaceSize: '256M',
    maxMetaspaceSize: '512M',
    directMemorySize: '1G',
    
    // GC Settings
    gcType: 'G1GC',
    gcTuning: '-XX:+UseG1GC -XX:MaxGCPauseMillis=200 -XX:G1HeapRegionSize=32M',
    gcLogging: true,
    gcLogFile: 'gc.log',
    
    // Performance Settings
    useG1GC: true,
    useStringDeduplication: true,
    useCompressedOops: true,
    useBiasedLocking: false,
    useTieredCompilation: true,
    useServer: true,
    useLargePages: false,
    useTransparentHugePages: false,
    
    // JVM Arguments
    customJvmArgs: '',
    additionalArgs: '',
    
    // Monitoring
    enableJmx: true,
    jmxPort: 9999,
    jmxHost: 'localhost',
    enableFlightRecorder: false,
    flightRecorderOptions: 'defaultrecording=true,duration=60s,filename=flight.jfr',
    
    // Network Settings
    networkBufferSize: '64M',
    maxNetworkThreads: 4,
    useEpoll: true,
    useKQueue: false,
    
    // Security
    enableSecurityManager: false,
    securityPolicy: '',
    trustedCodebase: '',
    
    // Logging
    enableGC: true,
    enableClassUnloading: true,
    enableVerboseGC: false,
    logLevel: 'INFO',
    
    // Advanced
    useAikarFlags: true,
    useOptimizedFlags: true,
    customFlags: ''
  });
  // Loading and changes tracking removed for now

  const loadSettings = async () => {
    if (!serverId) return;
    
    try {
      // Load server JVM configuration
      await fetchSettings(serverId);
    } catch (error) {
      console.error('Failed to fetch JVM settings:', error);
    }
  };

  useEffect(() => {
    loadSettings();
  }, []);

  // Sync settings with server store data
  useEffect(() => {
    if (serverId && settings[serverId]) {
      const serverData = settings[serverId];
      if (serverData.jvm) {
        setSettings(prev => ({
          ...prev,
          ...serverData.jvm,
        }));
      }
    }
  }, [serverId, serverSettings]);

  const handleSettingChange = async (key: keyof JVMSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    
    if (!serverId) return;
    
    try {
      // Update JVM arguments
      if (key === 'customJvmArgs' || key === 'additionalArgs' || key === 'gcTuning' || key === 'flightRecorderOptions' || key === 'customFlags') {
        // These are JVM arguments that need to be updated
        const currentArgs = settings.customJvmArgs ? settings.customJvmArgs.split(' ') : [];
        const additionalArgs = settings.additionalArgs ? settings.additionalArgs.split(' ') : [];
        const gcTuning = settings.gcTuning ? settings.gcTuning.split(' ') : [];
        const flightRecorder = settings.flightRecorderOptions ? settings.flightRecorderOptions.split(' ') : [];
        const customFlags = settings.customFlags ? settings.customFlags.split(' ') : [];
        
        const allArgs = [...currentArgs, ...additionalArgs, ...gcTuning, ...flightRecorder, ...customFlags].filter(Boolean);
        
        await updateSettings(serverId, { jvmArgs: allArgs });
      }
    } catch (error) {
      console.error('Failed to update JVM settings:', error);
    }
  };

  const getValidationStatus = (key: keyof JVMSettingsData) => {
    const value = settings[key];
    
    // Ensure value is a number for comparison
    const numValue = typeof value === 'number' ? value : 0;
    
    switch (key) {
      case 'jmxPort':
        return numValue < 1 || numValue > 65535 ? 'error' : 'success';
      case 'maxNetworkThreads':
        return numValue < 1 || numValue > 32 ? 'error' : 'success';
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

  const generateAikarFlags = () => {
    const aikarFlags = [
      '-XX:+UseG1GC',
      '-XX:+ParallelRefProcEnabled',
      '-XX:MaxGCPauseMillis=200',
      '-XX:+UnlockExperimentalVMOptions',
      '-XX:+DisableExplicitGC',
      '-XX:+AlwaysPreTouch',
      '-XX:G1NewSizePercent=30',
      '-XX:G1MaxNewSizePercent=40',
      '-XX:G1HeapRegionSize=8M',
      '-XX:G1ReservePercent=20',
      '-XX:G1HeapWastePercent=5',
      '-XX:G1MixedGCCountTarget=4',
      '-XX:InitiatingHeapOccupancyPercent=15',
      '-XX:G1MixedGCLiveThresholdPercent=90',
      '-XX:G1RSetUpdatingPauseTimePercent=5',
      '-XX:SurvivorRatio=32',
      '-XX:+PerfDisableSharedMem',
      '-XX:MaxTenuringThreshold=1'
    ];
    
    handleSettingChange('customJvmArgs', aikarFlags.join(' '));
  };

  const generateOptimizedFlags = () => {
    const optimizedFlags = [
      '-XX:+UseG1GC',
      '-XX:MaxGCPauseMillis=200',
      '-XX:+UseStringDeduplication',
      '-XX:+UseCompressedOops',
      '-XX:+UseBiasedLocking',
      '-XX:+TieredCompilation',
      '-server',
      '-XX:+AggressiveOpts',
      '-XX:+UseFastAccessorMethods',
      '-XX:+OptimizeStringConcat'
    ];
    
    handleSettingChange('customJvmArgs', optimizedFlags.join(' '));
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Memory Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <MemoryStick className="h-5 w-5" />
            <span>Memory Settings</span>
          </CardTitle>
          <CardDescription>
            Configure JVM memory allocation and management
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="initialHeapSize">Initial Heap Size</Label>
                <Input
                  id="initialHeapSize"
                  value={settings.initialHeapSize}
                  onChange={(e) => handleSettingChange('initialHeapSize', e.target.value)}
                  placeholder="e.g., 2G, 2048M"
                />
              </div>
              
              <div>
                <Label htmlFor="maxHeapSize">Max Heap Size</Label>
                <Input
                  id="maxHeapSize"
                  value={settings.maxHeapSize}
                  onChange={(e) => handleSettingChange('maxHeapSize', e.target.value)}
                  placeholder="e.g., 4G, 4096M"
                />
              </div>
              
              <div>
                <Label htmlFor="metaspaceSize">Metaspace Size</Label>
                <Input
                  id="metaspaceSize"
                  value={settings.metaspaceSize}
                  onChange={(e) => handleSettingChange('metaspaceSize', e.target.value)}
                  placeholder="e.g., 256M"
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="maxMetaspaceSize">Max Metaspace Size</Label>
                <Input
                  id="maxMetaspaceSize"
                  value={settings.maxMetaspaceSize}
                  onChange={(e) => handleSettingChange('maxMetaspaceSize', e.target.value)}
                  placeholder="e.g., 512M"
                />
              </div>
              
              <div>
                <Label htmlFor="directMemorySize">Direct Memory Size</Label>
                <Input
                  id="directMemorySize"
                  value={settings.directMemorySize}
                  onChange={(e) => handleSettingChange('directMemorySize', e.target.value)}
                  placeholder="e.g., 1G"
                />
              </div>
              
              <div>
                <Label htmlFor="networkBufferSize">Network Buffer Size</Label>
                <Input
                  id="networkBufferSize"
                  value={settings.networkBufferSize}
                  onChange={(e) => handleSettingChange('networkBufferSize', e.target.value)}
                  placeholder="e.g., 64M"
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Garbage Collection Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Zap className="h-5 w-5" />
            <span>Garbage Collection</span>
          </CardTitle>
          <CardDescription>
            Configure garbage collection algorithm and tuning
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="gcType">GC Type</Label>
                <Select value={settings.gcType} onValueChange={(value) => handleSettingChange('gcType', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="G1GC">G1GC (Recommended)</SelectItem>
                    <SelectItem value="ParallelGC">Parallel GC</SelectItem>
                    <SelectItem value="SerialGC">Serial GC</SelectItem>
                    <SelectItem value="ZGC">ZGC (Experimental)</SelectItem>
                    <SelectItem value="ShenandoahGC">Shenandoah GC</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="gcTuning">GC Tuning Parameters</Label>
                <Textarea
                  id="gcTuning"
                  value={settings.gcTuning}
                  onChange={(e) => handleSettingChange('gcTuning', e.target.value)}
                  placeholder="GC tuning parameters"
                  rows={3}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="gcLogging">GC Logging</Label>
                  <p className="text-sm text-muted-foreground">Enable garbage collection logging</p>
                </div>
                <Switch
                  id="gcLogging"
                  checked={settings.gcLogging}
                  onCheckedChange={(checked) => handleSettingChange('gcLogging', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="gcLogFile">GC Log File</Label>
                <Input
                  id="gcLogFile"
                  value={settings.gcLogFile}
                  onChange={(e) => handleSettingChange('gcLogFile', e.target.value)}
                  placeholder="gc.log"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableGC">Enable GC</Label>
                  <p className="text-sm text-muted-foreground">Enable garbage collection</p>
                </div>
                <Switch
                  id="enableGC"
                  checked={settings.enableGC}
                  onCheckedChange={(checked) => handleSettingChange('enableGC', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableVerboseGC">Verbose GC</Label>
                  <p className="text-sm text-muted-foreground">Enable verbose garbage collection</p>
                </div>
                <Switch
                  id="enableVerboseGC"
                  checked={settings.enableVerboseGC}
                  onCheckedChange={(checked) => handleSettingChange('enableVerboseGC', checked)}
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
            Configure JVM performance optimizations
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useG1GC">Use G1GC</Label>
                  <p className="text-sm text-muted-foreground">Enable G1 garbage collector</p>
                </div>
                <Switch
                  id="useG1GC"
                  checked={settings.useG1GC}
                  onCheckedChange={(checked) => handleSettingChange('useG1GC', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useStringDeduplication">String Deduplication</Label>
                  <p className="text-sm text-muted-foreground">Enable string deduplication</p>
                </div>
                <Switch
                  id="useStringDeduplication"
                  checked={settings.useStringDeduplication}
                  onCheckedChange={(checked) => handleSettingChange('useStringDeduplication', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useCompressedOops">Compressed OOPs</Label>
                  <p className="text-sm text-muted-foreground">Use compressed ordinary object pointers</p>
                </div>
                <Switch
                  id="useCompressedOops"
                  checked={settings.useCompressedOops}
                  onCheckedChange={(checked) => handleSettingChange('useCompressedOops', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useTieredCompilation">Tiered Compilation</Label>
                  <p className="text-sm text-muted-foreground">Enable tiered compilation</p>
                </div>
                <Switch
                  id="useTieredCompilation"
                  checked={settings.useTieredCompilation}
                  onCheckedChange={(checked) => handleSettingChange('useTieredCompilation', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useServer">Server Mode</Label>
                  <p className="text-sm text-muted-foreground">Use server JVM optimizations</p>
                </div>
                <Switch
                  id="useServer"
                  checked={settings.useServer}
                  onCheckedChange={(checked) => handleSettingChange('useServer', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useLargePages">Large Pages</Label>
                  <p className="text-sm text-muted-foreground">Use large memory pages</p>
                </div>
                <Switch
                  id="useLargePages"
                  checked={settings.useLargePages}
                  onCheckedChange={(checked) => handleSettingChange('useLargePages', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useBiasedLocking">Biased Locking</Label>
                  <p className="text-sm text-muted-foreground">Enable biased locking</p>
                </div>
                <Switch
                  id="useBiasedLocking"
                  checked={settings.useBiasedLocking}
                  onCheckedChange={(checked) => handleSettingChange('useBiasedLocking', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableClassUnloading">Class Unloading</Label>
                  <p className="text-sm text-muted-foreground">Enable class unloading</p>
                </div>
                <Switch
                  id="enableClassUnloading"
                  checked={settings.enableClassUnloading}
                  onCheckedChange={(checked) => handleSettingChange('enableClassUnloading', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Network Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Network className="h-5 w-5" />
            <span>Network Settings</span>
          </CardTitle>
          <CardDescription>
            Configure network-related JVM settings
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="maxNetworkThreads">Max Network Threads</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="maxNetworkThreads"
                    type="number"
                    value={settings.maxNetworkThreads}
                    onChange={(e) => handleSettingChange('maxNetworkThreads', parseInt(e.target.value))}
                    min="1"
                    max="32"
                  />
                  {getStatusIcon(getValidationStatus('maxNetworkThreads'))}
                </div>
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useEpoll">Use Epoll</Label>
                  <p className="text-sm text-muted-foreground">Use epoll for Linux</p>
                </div>
                <Switch
                  id="useEpoll"
                  checked={settings.useEpoll}
                  onCheckedChange={(checked) => handleSettingChange('useEpoll', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useKQueue">Use KQueue</Label>
                  <p className="text-sm text-muted-foreground">Use kqueue for macOS/BSD</p>
                </div>
                <Switch
                  id="useKQueue"
                  checked={settings.useKQueue}
                  onCheckedChange={(checked) => handleSettingChange('useKQueue', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Monitoring Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <SettingsIcon className="h-5 w-5" />
            <span>Monitoring Settings</span>
          </CardTitle>
          <CardDescription>
            Configure JVM monitoring and profiling
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableJmx">Enable JMX</Label>
                  <p className="text-sm text-muted-foreground">Enable Java Management Extensions</p>
                </div>
                <Switch
                  id="enableJmx"
                  checked={settings.enableJmx}
                  onCheckedChange={(checked) => handleSettingChange('enableJmx', checked)}
                />
              </div>
              
              {settings.enableJmx && (
                <>
                  <div>
                    <Label htmlFor="jmxPort">JMX Port</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="jmxPort"
                        type="number"
                        value={settings.jmxPort}
                        onChange={(e) => handleSettingChange('jmxPort', parseInt(e.target.value))}
                        min="1"
                        max="65535"
                      />
                      {getStatusIcon(getValidationStatus('jmxPort'))}
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="jmxHost">JMX Host</Label>
                    <Input
                      id="jmxHost"
                      value={settings.jmxHost}
                      onChange={(e) => handleSettingChange('jmxHost', e.target.value)}
                      placeholder="localhost"
                    />
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableFlightRecorder">Flight Recorder</Label>
                  <p className="text-sm text-muted-foreground">Enable Java Flight Recorder</p>
                </div>
                <Switch
                  id="enableFlightRecorder"
                  checked={settings.enableFlightRecorder}
                  onCheckedChange={(checked) => handleSettingChange('enableFlightRecorder', checked)}
                />
              </div>
              
              {settings.enableFlightRecorder && (
                <div>
                  <Label htmlFor="flightRecorderOptions">Flight Recorder Options</Label>
                  <Textarea
                    id="flightRecorderOptions"
                    value={settings.flightRecorderOptions}
                    onChange={(e) => handleSettingChange('flightRecorderOptions', e.target.value)}
                    placeholder="Flight recorder options"
                    rows={2}
                  />
                </div>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* JVM Arguments */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Cpu className="h-5 w-5" />
            <span>JVM Arguments</span>
          </CardTitle>
          <CardDescription>
            Configure custom JVM arguments and flags
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="flex items-center space-x-4">
              <Button onClick={generateAikarFlags} variant="outline" size="sm">
                <Zap className="h-4 w-4 mr-2" />
                Aikar's Flags
              </Button>
              <Button onClick={generateOptimizedFlags} variant="outline" size="sm">
                <Activity className="h-4 w-4 mr-2" />
                Optimized Flags
              </Button>
            </div>
            
            <div>
              <Label htmlFor="customJvmArgs">Custom JVM Arguments</Label>
              <Textarea
                id="customJvmArgs"
                value={settings.customJvmArgs}
                onChange={(e) => handleSettingChange('customJvmArgs', e.target.value)}
                placeholder="Enter custom JVM arguments"
                rows={4}
              />
            </div>
            
            <div>
              <Label htmlFor="additionalArgs">Additional Arguments</Label>
              <Textarea
                id="additionalArgs"
                value={settings.additionalArgs}
                onChange={(e) => handleSettingChange('additionalArgs', e.target.value)}
                placeholder="Enter additional arguments"
                rows={3}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
