import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useServers } from '@/store/servers-new';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  Server, 
  Shield, 
  HardDrive,
  Network,
  Activity,
  AlertTriangle,
  CheckCircle,
  Info
} from 'lucide-react';

interface GeneralSettingsData {
  serverName: string;
  serverDescription: string;
  maxPlayers: number;
  motd: string;
  serverIcon: string;
  difficulty: 'peaceful' | 'easy' | 'normal' | 'hard';
  gamemode: 'survival' | 'creative' | 'adventure' | 'spectator';
  hardcore: boolean;
  pvp: boolean;
  allowFlight: boolean;
  allowNether: boolean;
  allowEnd: boolean;
  onlineMode: boolean;
  whitelist: boolean;
  enforceWhitelist: boolean;
  enableCommandBlock: boolean;
  enableQuery: boolean;
  enableRcon: boolean;
  rconPort: number;
  rconPassword: string;
  queryPort: number;
  serverPort: number;
  viewDistance: number;
  simulationDistance: number;
  chunkLoading: 'eager' | 'lazy';
  maxWorldSize: number;
  maxBuildHeight: number;
  spawnProtection: number;
  functionPermissionLevel: number;
  rateLimit: number;
  networkCompressionThreshold: number;
  maxTickTime: number;
  maxChainedNeighborUpdates: number;
  maxThreads: number;
  useNativeTransport: boolean;
  enableJmxMonitoring: boolean;
  syncChunkWrites: boolean;
  enableStatus: boolean;
  hideOnlinePlayers: boolean;
  logIps: boolean;
  preventProxyConnections: boolean;
  enforceSecureProfile: boolean;
  requireResourcePack: boolean;
  resourcePackPrompt: string;
  resourcePackSha1: string;
  resourcePackUrl: string;
}

export const GeneralSettings: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { 
    fetchSettings, 
    updateSettings,
    settings: serverSettings 
  } = useServers();
  
  const [settings, setSettings] = useState<GeneralSettingsData>({
    serverName: 'My Minecraft Server',
    serverDescription: 'A fun Minecraft server for everyone!',
    maxPlayers: 20,
    motd: 'Welcome to our server!',
    serverIcon: '',
    difficulty: 'normal',
    gamemode: 'survival',
    hardcore: false,
    pvp: true,
    allowFlight: false,
    allowNether: true,
    allowEnd: true,
    onlineMode: true,
    whitelist: false,
    enforceWhitelist: false,
    enableCommandBlock: false,
    enableQuery: true,
    enableRcon: true,
    rconPort: 25575,
    rconPassword: '',
    queryPort: 25565,
    serverPort: 25565,
    viewDistance: 10,
    simulationDistance: 10,
    chunkLoading: 'eager',
    maxWorldSize: 29999984,
    maxBuildHeight: 320,
    spawnProtection: 16,
    functionPermissionLevel: 2,
    rateLimit: 0,
    networkCompressionThreshold: 256,
    maxTickTime: 60000,
    maxChainedNeighborUpdates: 1000000,
    maxThreads: 8,
    useNativeTransport: true,
    enableJmxMonitoring: false,
    syncChunkWrites: true,
    enableStatus: true,
    hideOnlinePlayers: false,
    logIps: true,
    preventProxyConnections: true,
    enforceSecureProfile: true,
    requireResourcePack: false,
    resourcePackPrompt: '',
    resourcePackSha1: '',
    resourcePackUrl: ''
  });
  // Loading and changes tracking removed for now

  const loadSettings = async () => {
    if (!serverId) return;
    
    try {
      // Load server configuration files
      await Promise.all([
        fetchSettings(serverId)
      ]);
    } catch (error) {
      console.error('Failed to fetch general settings:', error);
    }
  };

  useEffect(() => {
    loadSettings();
  }, []);

  // Sync settings with server store data
  useEffect(() => {
    if (serverId && serverSettings[serverId]) {
      const serverData = serverSettings[serverId];
      if (serverData.general) {
        setSettings(prev => ({
          ...prev,
          serverName: serverData.general?.name || prev.serverName,
          serverDescription: serverData.general?.description || prev.serverDescription,
          maxPlayers: serverData.general?.maxPlayers || prev.maxPlayers,
          motd: serverData.general?.motd || prev.motd,
          difficulty: (serverData.general?.difficulty as "normal" | "peaceful" | "easy" | "hard") || prev.difficulty,
          gamemode: (serverData.general?.gamemode as "survival" | "creative" | "adventure" | "spectator") || prev.gamemode,
          pvp: serverData.general?.pvp ?? prev.pvp,
          onlineMode: serverData.general?.onlineMode ?? prev.onlineMode,
          whitelist: serverData.general?.whitelist ?? prev.whitelist,
          enableCommandBlock: serverData.general?.enableCommandBlock ?? prev.enableCommandBlock,
          viewDistance: serverData.general?.viewDistance || prev.viewDistance,
          simulationDistance: serverData.general?.simulationDistance || prev.simulationDistance,
        }));
      }
    }
  }, [serverId, serverSettings]);

  const handleSettingChange = async (key: keyof GeneralSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    
    if (!serverId) return;
    
    try {
      // Map settings to server.properties format
      const serverProperties: Record<string, string> = {};
      
      switch (key) {
        case 'maxPlayers':
          serverProperties['max-players'] = value.toString();
          break;
        case 'motd':
          serverProperties['motd'] = value;
          break;
        case 'difficulty':
          serverProperties['difficulty'] = value;
          break;
        case 'gamemode':
          serverProperties['gamemode'] = value;
          break;
        case 'pvp':
          serverProperties['pvp'] = value.toString();
          break;
        case 'onlineMode':
          serverProperties['online-mode'] = value.toString();
          break;
        case 'whitelist':
          serverProperties['white-list'] = value.toString();
          break;
        case 'enableCommandBlock':
          serverProperties['enable-command-block'] = value.toString();
          break;
        case 'viewDistance':
          serverProperties['view-distance'] = value.toString();
          break;
        case 'simulationDistance':
          serverProperties['simulation-distance'] = value.toString();
          break;
        case 'serverPort':
          serverProperties['server-port'] = value.toString();
          break;
        case 'queryPort':
          serverProperties['query.port'] = value.toString();
          break;
        case 'rconPort':
          serverProperties['rcon.port'] = value.toString();
          break;
        case 'rconPassword':
          serverProperties['rcon.password'] = value;
          break;
        case 'enableQuery':
          serverProperties['enable-query'] = value.toString();
          break;
        case 'enableRcon':
          serverProperties['enable-rcon'] = value.toString();
          break;
        case 'hardcore':
          serverProperties['hardcore'] = value.toString();
          break;
        case 'allowFlight':
          serverProperties['allow-flight'] = value.toString();
          break;
        case 'allowNether':
          serverProperties['allow-nether'] = value.toString();
          break;
        case 'allowEnd':
          serverProperties['allow-end'] = value.toString();
          break;
        case 'enforceWhitelist':
          serverProperties['enforce-whitelist'] = value.toString();
          break;
        case 'spawnProtection':
          serverProperties['spawn-protection'] = value.toString();
          break;
        case 'functionPermissionLevel':
          serverProperties['function-permission-level'] = value.toString();
          break;
        case 'rateLimit':
          serverProperties['rate-limit'] = value.toString();
          break;
        case 'networkCompressionThreshold':
          serverProperties['network-compression-threshold'] = value.toString();
          break;
        case 'maxTickTime':
          serverProperties['max-tick-time'] = value.toString();
          break;
        case 'maxChainedNeighborUpdates':
          serverProperties['max-chained-neighbor-updates'] = value.toString();
          break;
        case 'maxThreads':
          serverProperties['max-threads'] = value.toString();
          break;
        case 'useNativeTransport':
          serverProperties['use-native-transport'] = value.toString();
          break;
        case 'enableJmxMonitoring':
          serverProperties['enable-jmx-monitoring'] = value.toString();
          break;
        case 'syncChunkWrites':
          serverProperties['sync-chunk-writes'] = value.toString();
          break;
        case 'enableStatus':
          serverProperties['enable-status'] = value.toString();
          break;
        case 'hideOnlinePlayers':
          serverProperties['hide-online-players'] = value.toString();
          break;
        case 'logIps':
          serverProperties['log-ips'] = value.toString();
          break;
        case 'preventProxyConnections':
          serverProperties['prevent-proxy-connections'] = value.toString();
          break;
        case 'enforceSecureProfile':
          serverProperties['enforce-secure-profile'] = value.toString();
          break;
        case 'requireResourcePack':
          serverProperties['require-resource-pack'] = value.toString();
          break;
        case 'resourcePackPrompt':
          serverProperties['resource-pack-prompt'] = value;
          break;
        case 'resourcePackSha1':
          serverProperties['resource-pack-sha1'] = value;
          break;
        case 'resourcePackUrl':
          serverProperties['resource-pack-url'] = value;
          break;
      }
      
      // Update server settings
      const currentServerSettings = serverSettings[serverId] || {};
      await updateSettings(serverId, {
        ...currentServerSettings,
        general: {
          ...currentServerSettings.general,
          name: settings.serverName,
          description: settings.serverDescription,
          version: currentServerSettings.general?.version || '1.20.1',
          modpack: currentServerSettings.general?.modpack,
          maxPlayers: settings.maxPlayers,
          motd: settings.motd,
          difficulty: settings.difficulty,
          gamemode: settings.gamemode,
          pvp: settings.pvp,
          onlineMode: settings.onlineMode,
          whitelist: settings.whitelist,
          enableCommandBlock: settings.enableCommandBlock,
          viewDistance: settings.viewDistance,
          simulationDistance: settings.simulationDistance
        }
      });
    } catch (error) {
      console.error('Failed to update server properties:', error);
    }
  };

  const getValidationStatus = (key: keyof GeneralSettingsData) => {
    const value = settings[key];
    
    // Ensure value is a number for comparison
    const numValue = typeof value === 'number' ? value : 0;
    
    switch (key) {
      case 'maxPlayers':
        return numValue < 1 || numValue > 1000 ? 'error' : 'success';
      case 'serverPort':
      case 'queryPort':
      case 'rconPort':
        return numValue < 1 || numValue > 65535 ? 'error' : 'success';
      case 'viewDistance':
      case 'simulationDistance':
        return numValue < 3 || numValue > 32 ? 'error' : 'success';
      case 'maxTickTime':
        return numValue < 1000 ? 'error' : 'success';
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

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Basic Server Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Server className="h-5 w-5" />
            <span>Basic Server Settings</span>
          </CardTitle>
          <CardDescription>
            Configure basic server information and behavior
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="serverName">Server Name</Label>
                <Input
                  id="serverName"
                  value={settings.serverName}
                  onChange={(e) => handleSettingChange('serverName', e.target.value)}
                  placeholder="Enter server name"
                />
              </div>
              
              <div>
                <Label htmlFor="serverDescription">Description</Label>
                <Textarea
                  id="serverDescription"
                  value={settings.serverDescription}
                  onChange={(e) => handleSettingChange('serverDescription', e.target.value)}
                  placeholder="Enter server description"
                  rows={3}
                />
              </div>
              
              <div>
                <Label htmlFor="motd">Message of the Day</Label>
                <Textarea
                  id="motd"
                  value={settings.motd}
                  onChange={(e) => handleSettingChange('motd', e.target.value)}
                  placeholder="Enter MOTD"
                  rows={2}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="maxPlayers">Max Players</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="maxPlayers"
                    type="number"
                    value={settings.maxPlayers}
                    onChange={(e) => handleSettingChange('maxPlayers', parseInt(e.target.value))}
                    min="1"
                    max="1000"
                  />
                  {getStatusIcon(getValidationStatus('maxPlayers'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="difficulty">Difficulty</Label>
                <Select value={settings.difficulty} onValueChange={(value) => handleSettingChange('difficulty', value)}>
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
              
              <div>
                <Label htmlFor="gamemode">Default Gamemode</Label>
                <Select value={settings.gamemode} onValueChange={(value) => handleSettingChange('gamemode', value)}>
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
          </div>
        </CardContent>
      </Card>

      {/* Gameplay Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>Gameplay Settings</span>
          </CardTitle>
          <CardDescription>
            Configure gameplay features and restrictions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="hardcore">Hardcore Mode</Label>
                  <p className="text-sm text-muted-foreground">Players are banned on death</p>
                </div>
                <Switch
                  id="hardcore"
                  checked={settings.hardcore}
                  onCheckedChange={(checked) => handleSettingChange('hardcore', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="pvp">PvP Enabled</Label>
                  <p className="text-sm text-muted-foreground">Allow player vs player combat</p>
                </div>
                <Switch
                  id="pvp"
                  checked={settings.pvp}
                  onCheckedChange={(checked) => handleSettingChange('pvp', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="allowFlight">Allow Flight</Label>
                  <p className="text-sm text-muted-foreground">Allow players to fly</p>
                </div>
                <Switch
                  id="allowFlight"
                  checked={settings.allowFlight}
                  onCheckedChange={(checked) => handleSettingChange('allowFlight', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="allowNether">Allow Nether</Label>
                  <p className="text-sm text-muted-foreground">Enable Nether dimension</p>
                </div>
                <Switch
                  id="allowNether"
                  checked={settings.allowNether}
                  onCheckedChange={(checked) => handleSettingChange('allowNether', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="allowEnd">Allow End</Label>
                  <p className="text-sm text-muted-foreground">Enable End dimension</p>
                </div>
                <Switch
                  id="allowEnd"
                  checked={settings.allowEnd}
                  onCheckedChange={(checked) => handleSettingChange('allowEnd', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableCommandBlock">Command Blocks</Label>
                  <p className="text-sm text-muted-foreground">Enable command blocks</p>
                </div>
                <Switch
                  id="enableCommandBlock"
                  checked={settings.enableCommandBlock}
                  onCheckedChange={(checked) => handleSettingChange('enableCommandBlock', checked)}
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
            Configure network ports and connectivity options
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="serverPort">Server Port</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="serverPort"
                    type="number"
                    value={settings.serverPort}
                    onChange={(e) => handleSettingChange('serverPort', parseInt(e.target.value))}
                    min="1"
                    max="65535"
                  />
                  {getStatusIcon(getValidationStatus('serverPort'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="queryPort">Query Port</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="queryPort"
                    type="number"
                    value={settings.queryPort}
                    onChange={(e) => handleSettingChange('queryPort', parseInt(e.target.value))}
                    min="1"
                    max="65535"
                  />
                  {getStatusIcon(getValidationStatus('queryPort'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="rconPort">RCON Port</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="rconPort"
                    type="number"
                    value={settings.rconPort}
                    onChange={(e) => handleSettingChange('rconPort', parseInt(e.target.value))}
                    min="1"
                    max="65535"
                  />
                  {getStatusIcon(getValidationStatus('rconPort'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="onlineMode">Online Mode</Label>
                  <p className="text-sm text-muted-foreground">Verify players with Mojang</p>
                </div>
                <Switch
                  id="onlineMode"
                  checked={settings.onlineMode}
                  onCheckedChange={(checked) => handleSettingChange('onlineMode', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableQuery">Enable Query</Label>
                  <p className="text-sm text-muted-foreground">Enable server query protocol</p>
                </div>
                <Switch
                  id="enableQuery"
                  checked={settings.enableQuery}
                  onCheckedChange={(checked) => handleSettingChange('enableQuery', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableRcon">Enable RCON</Label>
                  <p className="text-sm text-muted-foreground">Enable remote console</p>
                </div>
                <Switch
                  id="enableRcon"
                  checked={settings.enableRcon}
                  onCheckedChange={(checked) => handleSettingChange('enableRcon', checked)}
                />
              </div>
            </div>
          </div>
          
          {settings.enableRcon && (
            <div className="mt-6">
              <Label htmlFor="rconPassword">RCON Password</Label>
              <Input
                id="rconPassword"
                type="password"
                value={settings.rconPassword}
                onChange={(e) => handleSettingChange('rconPassword', e.target.value)}
                placeholder="Enter RCON password"
              />
            </div>
          )}
        </CardContent>
      </Card>

      {/* Performance Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <HardDrive className="h-5 w-5" />
            <span>Performance Settings</span>
          </CardTitle>
          <CardDescription>
            Configure server performance and optimization settings
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="viewDistance">View Distance</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="viewDistance"
                    type="number"
                    value={settings.viewDistance}
                    onChange={(e) => handleSettingChange('viewDistance', parseInt(e.target.value))}
                    min="3"
                    max="32"
                  />
                  {getStatusIcon(getValidationStatus('viewDistance'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="simulationDistance">Simulation Distance</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="simulationDistance"
                    type="number"
                    value={settings.simulationDistance}
                    onChange={(e) => handleSettingChange('simulationDistance', parseInt(e.target.value))}
                    min="3"
                    max="32"
                  />
                  {getStatusIcon(getValidationStatus('simulationDistance'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="maxTickTime">Max Tick Time (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="maxTickTime"
                    type="number"
                    value={settings.maxTickTime}
                    onChange={(e) => handleSettingChange('maxTickTime', parseInt(e.target.value))}
                    min="1000"
                  />
                  {getStatusIcon(getValidationStatus('maxTickTime'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="chunkLoading">Chunk Loading</Label>
                <Select value={settings.chunkLoading} onValueChange={(value) => handleSettingChange('chunkLoading', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="eager">Eager</SelectItem>
                    <SelectItem value="lazy">Lazy</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="maxThreads">Max Threads</Label>
                <Input
                  id="maxThreads"
                  type="number"
                  value={settings.maxThreads}
                  onChange={(e) => handleSettingChange('maxThreads', parseInt(e.target.value))}
                  min="1"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="useNativeTransport">Native Transport</Label>
                  <p className="text-sm text-muted-foreground">Use native transport for better performance</p>
                </div>
                <Switch
                  id="useNativeTransport"
                  checked={settings.useNativeTransport}
                  onCheckedChange={(checked) => handleSettingChange('useNativeTransport', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Security Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Security Settings</span>
          </CardTitle>
          <CardDescription>
            Configure security and access control settings
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="whitelist">Whitelist</Label>
                  <p className="text-sm text-muted-foreground">Only allow whitelisted players</p>
                </div>
                <Switch
                  id="whitelist"
                  checked={settings.whitelist}
                  onCheckedChange={(checked) => handleSettingChange('whitelist', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enforceWhitelist">Enforce Whitelist</Label>
                  <p className="text-sm text-muted-foreground">Kick non-whitelisted players</p>
                </div>
                <Switch
                  id="enforceWhitelist"
                  checked={settings.enforceWhitelist}
                  onCheckedChange={(checked) => handleSettingChange('enforceWhitelist', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="preventProxyConnections">Prevent Proxy</Label>
                  <p className="text-sm text-muted-foreground">Prevent proxy connections</p>
                </div>
                <Switch
                  id="preventProxyConnections"
                  checked={settings.preventProxyConnections}
                  onCheckedChange={(checked) => handleSettingChange('preventProxyConnections', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enforceSecureProfile">Secure Profile</Label>
                  <p className="text-sm text-muted-foreground">Enforce secure profile</p>
                </div>
                <Switch
                  id="enforceSecureProfile"
                  checked={settings.enforceSecureProfile}
                  onCheckedChange={(checked) => handleSettingChange('enforceSecureProfile', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="logIps">Log IPs</Label>
                  <p className="text-sm text-muted-foreground">Log player IP addresses</p>
                </div>
                <Switch
                  id="logIps"
                  checked={settings.logIps}
                  onCheckedChange={(checked) => handleSettingChange('logIps', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="hideOnlinePlayers">Hide Online Players</Label>
                  <p className="text-sm text-muted-foreground">Hide online player count</p>
                </div>
                <Switch
                  id="hideOnlinePlayers"
                  checked={settings.hideOnlinePlayers}
                  onCheckedChange={(checked) => handleSettingChange('hideOnlinePlayers', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

// export { GeneralSettings };
