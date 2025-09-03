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
  Folder,
  FolderOpen,
  File,
  FolderPlus,
  FolderMinus,
  FolderX,
  FolderCheck,
  FolderClock,
  FolderUp,
  FolderDown
} from 'lucide-react';

interface PathsSettingsData {
  // Server Paths
  serverPath: string;
  worldPath: string;
  modsPath: string;
  configPath: string;
  logsPath: string;
  backupsPath: string;
  tempPath: string;
  cachePath: string;
  
  // Data Paths
  dataPath: string;
  databasePath: string;
  metricsPath: string;
  snapshotsPath: string;
  archivesPath: string;
  exportsPath: string;
  importsPath: string;
  
  // System Paths
  systemPath: string;
  binPath: string;
  libPath: string;
  includePath: string;
  sharePath: string;
  varPath: string;
  etcPath: string;
  homePath: string;
  
  // Network Paths
  networkPath: string;
  mountPath: string;
  remotePath: string;
  localPath: string;
  sharedPath: string;
  privatePath: string;
  publicPath: string;
  securePath: string;
  
  // Path Settings
  pathSeparator: '/' | '\\';
  pathEncoding: 'utf-8' | 'utf-16' | 'ascii';
  pathCaseSensitive: boolean;
  pathNormalize: boolean;
  pathResolve: boolean;
  pathValidate: boolean;
  
  // Path Permissions
  pathPermissions: string;
  pathOwner: string;
  pathGroup: string;
  pathMode: string;
  pathUmask: string;
  pathChmod: string;
  pathChown: string;
  pathChgrp: string;
  
  // Path Monitoring
  enablePathMonitoring: boolean;
  pathMonitoringInterval: number;
  pathMonitoringDepth: number;
  pathMonitoringExclude: string;
  pathMonitoringInclude: string;
  
  // Path Security
  enablePathSecurity: boolean;
  pathSecurityLevel: 'none' | 'basic' | 'strict' | 'paranoid';
  pathSecurityScan: boolean;
  pathSecurityQuarantine: boolean;
  pathSecurityBlock: boolean;
  
  // Path Backup
  enablePathBackup: boolean;
  pathBackupInterval: number;
  pathBackupRetention: number;
  pathBackupCompression: boolean;
  pathBackupEncryption: boolean;
  
  // Path Cleanup
  enablePathCleanup: boolean;
  pathCleanupInterval: number;
  pathCleanupAge: number;
  pathCleanupSize: string;
  pathCleanupPattern: string;
}

export const PathsSettings: React.FC = () => {
  const [settings, setSettings] = useState<PathsSettingsData>({
    // Server Paths
    serverPath: '/opt/guardian/servers',
    worldPath: '/opt/guardian/worlds',
    modsPath: '/opt/guardian/mods',
    configPath: '/opt/guardian/configs',
    logsPath: '/opt/guardian/logs',
    backupsPath: '/opt/guardian/backups',
    tempPath: '/tmp/guardian',
    cachePath: '/var/cache/guardian',
    
    // Data Paths
    dataPath: '/opt/guardian/data',
    databasePath: '/opt/guardian/database',
    metricsPath: '/opt/guardian/metrics',
    snapshotsPath: '/opt/guardian/snapshots',
    archivesPath: '/opt/guardian/archives',
    exportsPath: '/opt/guardian/exports',
    importsPath: '/opt/guardian/imports',
    
    // System Paths
    systemPath: '/opt/guardian/system',
    binPath: '/opt/guardian/bin',
    libPath: '/opt/guardian/lib',
    includePath: '/opt/guardian/include',
    sharePath: '/opt/guardian/share',
    varPath: '/var/guardian',
    etcPath: '/etc/guardian',
    homePath: '/home/guardian',
    
    // Network Paths
    networkPath: '/opt/guardian/network',
    mountPath: '/mnt/guardian',
    remotePath: '/opt/guardian/remote',
    localPath: '/opt/guardian/local',
    sharedPath: '/opt/guardian/shared',
    privatePath: '/opt/guardian/private',
    publicPath: '/opt/guardian/public',
    securePath: '/opt/guardian/secure',
    
    // Path Settings
    pathSeparator: '/',
    pathEncoding: 'utf-8',
    pathCaseSensitive: true,
    pathNormalize: true,
    pathResolve: true,
    pathValidate: true,
    
    // Path Permissions
    pathPermissions: '755',
    pathOwner: 'guardian',
    pathGroup: 'guardian',
    pathMode: '755',
    pathUmask: '022',
    pathChmod: '755',
    pathChown: 'guardian:guardian',
    pathChgrp: 'guardian',
    
    // Path Monitoring
    enablePathMonitoring: true,
    pathMonitoringInterval: 60000,
    pathMonitoringDepth: 10,
    pathMonitoringExclude: '*.tmp,*.log,*.cache',
    pathMonitoringInclude: '*.jar,*.json,*.yaml,*.yml',
    
    // Path Security
    enablePathSecurity: true,
    pathSecurityLevel: 'basic',
    pathSecurityScan: true,
    pathSecurityQuarantine: false,
    pathSecurityBlock: false,
    
    // Path Backup
    enablePathBackup: true,
    pathBackupInterval: 3600000,
    pathBackupRetention: 7,
    pathBackupCompression: true,
    pathBackupEncryption: false,
    
    // Path Cleanup
    enablePathCleanup: true,
    pathCleanupInterval: 86400000,
    pathCleanupAge: 7,
    pathCleanupSize: '1G',
    pathCleanupPattern: '*.tmp,*.log,*.cache'
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
      console.error('Failed to fetch paths settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const handleSettingChange = (key: keyof PathsSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const getValidationStatus = (key: keyof PathsSettingsData) => {
    const value = settings[key];
    
    switch (key) {
      case 'pathMonitoringInterval':
        return value < 1000 || value > 3600000 ? 'error' : 'success';
      case 'pathMonitoringDepth':
        return value < 1 || value > 100 ? 'error' : 'success';
      case 'pathBackupInterval':
        return value < 60000 || value > 86400000 ? 'error' : 'success';
      case 'pathBackupRetention':
        return value < 1 || value > 365 ? 'error' : 'success';
      case 'pathCleanupInterval':
        return value < 3600000 || value > 604800000 ? 'error' : 'success';
      case 'pathCleanupAge':
        return value < 1 || value > 365 ? 'error' : 'success';
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

  const getPathIcon = (path: string) => {
    if (path.includes('server')) return <Server className="h-4 w-4" />;
    if (path.includes('world')) return <Globe className="h-4 w-4" />;
    if (path.includes('mod')) return <Package className="h-4 w-4" />;
    if (path.includes('config')) return <SettingsIcon className="h-4 w-4" />;
    if (path.includes('log')) return <FileText className="h-4 w-4" />;
    if (path.includes('backup')) return <Database className="h-4 w-4" />;
    if (path.includes('temp')) return <Clock className="h-4 w-4" />;
    if (path.includes('cache')) return <HardDrive className="h-4 w-4" />;
    if (path.includes('data')) return <Database className="h-4 w-4" />;
    if (path.includes('system')) return <Cpu className="h-4 w-4" />;
    if (path.includes('network')) return <Network className="h-4 w-4" />;
    return <Folder className="h-4 w-4" />;
  };

  const getPathStatus = (path: string) => {
    // Mock path validation
    if (path.startsWith('/opt/guardian')) return 'success';
    if (path.startsWith('/var/guardian')) return 'success';
    if (path.startsWith('/etc/guardian')) return 'success';
    if (path.startsWith('/home/guardian')) return 'success';
    if (path.startsWith('/mnt/guardian')) return 'success';
    if (path.startsWith('/tmp/guardian')) return 'warning';
    return 'error';
  };

  const getPathStatusColor = (status: string) => {
    switch (status) {
      case 'success': return 'text-green-500';
      case 'warning': return 'text-yellow-500';
      case 'error': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  const getPathStatusIcon = (status: string) => {
    switch (status) {
      case 'success': return <FolderCheck className="h-4 w-4" />;
      case 'warning': return <AlertTriangle className="h-4 w-4" />;
      case 'error': return <FolderX className="h-4 w-4" />;
      default: return <Folder className="h-4 w-4" />;
    }
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Server Paths */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Server className="h-5 w-5" />
            <span>Server Paths</span>
          </CardTitle>
          <CardDescription>
            Configure server-related directory paths
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="serverPath">Server Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="serverPath"
                    value={settings.serverPath}
                    onChange={(e) => handleSettingChange('serverPath', e.target.value)}
                    placeholder="/opt/guardian/servers"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.serverPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.serverPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="worldPath">World Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="worldPath"
                    value={settings.worldPath}
                    onChange={(e) => handleSettingChange('worldPath', e.target.value)}
                    placeholder="/opt/guardian/worlds"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.worldPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.worldPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="modsPath">Mods Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="modsPath"
                    value={settings.modsPath}
                    onChange={(e) => handleSettingChange('modsPath', e.target.value)}
                    placeholder="/opt/guardian/mods"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.modsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.modsPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="configPath">Config Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="configPath"
                    value={settings.configPath}
                    onChange={(e) => handleSettingChange('configPath', e.target.value)}
                    placeholder="/opt/guardian/configs"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.configPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.configPath))}
                  </div>
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="logsPath">Logs Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="logsPath"
                    value={settings.logsPath}
                    onChange={(e) => handleSettingChange('logsPath', e.target.value)}
                    placeholder="/opt/guardian/logs"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.logsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.logsPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="backupsPath">Backups Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="backupsPath"
                    value={settings.backupsPath}
                    onChange={(e) => handleSettingChange('backupsPath', e.target.value)}
                    placeholder="/opt/guardian/backups"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.backupsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.backupsPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="tempPath">Temp Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="tempPath"
                    value={settings.tempPath}
                    onChange={(e) => handleSettingChange('tempPath', e.target.value)}
                    placeholder="/tmp/guardian"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.tempPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.tempPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="cachePath">Cache Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="cachePath"
                    value={settings.cachePath}
                    onChange={(e) => handleSettingChange('cachePath', e.target.value)}
                    placeholder="/var/cache/guardian"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.cachePath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.cachePath))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Data Paths */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Database className="h-5 w-5" />
            <span>Data Paths</span>
          </CardTitle>
          <CardDescription>
            Configure data storage directory paths
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="dataPath">Data Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="dataPath"
                    value={settings.dataPath}
                    onChange={(e) => handleSettingChange('dataPath', e.target.value)}
                    placeholder="/opt/guardian/data"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.dataPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.dataPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="databasePath">Database Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="databasePath"
                    value={settings.databasePath}
                    onChange={(e) => handleSettingChange('databasePath', e.target.value)}
                    placeholder="/opt/guardian/database"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.databasePath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.databasePath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="metricsPath">Metrics Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="metricsPath"
                    value={settings.metricsPath}
                    onChange={(e) => handleSettingChange('metricsPath', e.target.value)}
                    placeholder="/opt/guardian/metrics"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.metricsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.metricsPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="snapshotsPath">Snapshots Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="snapshotsPath"
                    value={settings.snapshotsPath}
                    onChange={(e) => handleSettingChange('snapshotsPath', e.target.value)}
                    placeholder="/opt/guardian/snapshots"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.snapshotsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.snapshotsPath))}
                  </div>
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="archivesPath">Archives Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="archivesPath"
                    value={settings.archivesPath}
                    onChange={(e) => handleSettingChange('archivesPath', e.target.value)}
                    placeholder="/opt/guardian/archives"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.archivesPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.archivesPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="exportsPath">Exports Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="exportsPath"
                    value={settings.exportsPath}
                    onChange={(e) => handleSettingChange('exportsPath', e.target.value)}
                    placeholder="/opt/guardian/exports"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.exportsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.exportsPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="importsPath">Imports Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="importsPath"
                    value={settings.importsPath}
                    onChange={(e) => handleSettingChange('importsPath', e.target.value)}
                    placeholder="/opt/guardian/imports"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.importsPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.importsPath))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* System Paths */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Cpu className="h-5 w-5" />
            <span>System Paths</span>
          </CardTitle>
          <CardDescription>
            Configure system-related directory paths
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="systemPath">System Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="systemPath"
                    value={settings.systemPath}
                    onChange={(e) => handleSettingChange('systemPath', e.target.value)}
                    placeholder="/opt/guardian/system"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.systemPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.systemPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="binPath">Bin Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="binPath"
                    value={settings.binPath}
                    onChange={(e) => handleSettingChange('binPath', e.target.value)}
                    placeholder="/opt/guardian/bin"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.binPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.binPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="libPath">Lib Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="libPath"
                    value={settings.libPath}
                    onChange={(e) => handleSettingChange('libPath', e.target.value)}
                    placeholder="/opt/guardian/lib"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.libPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.libPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="includePath">Include Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="includePath"
                    value={settings.includePath}
                    onChange={(e) => handleSettingChange('includePath', e.target.value)}
                    placeholder="/opt/guardian/include"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.includePath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.includePath))}
                  </div>
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="sharePath">Share Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="sharePath"
                    value={settings.sharePath}
                    onChange={(e) => handleSettingChange('sharePath', e.target.value)}
                    placeholder="/opt/guardian/share"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.sharePath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.sharePath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="varPath">Var Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="varPath"
                    value={settings.varPath}
                    onChange={(e) => handleSettingChange('varPath', e.target.value)}
                    placeholder="/var/guardian"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.varPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.varPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="etcPath">Etc Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="etcPath"
                    value={settings.etcPath}
                    onChange={(e) => handleSettingChange('etcPath', e.target.value)}
                    placeholder="/etc/guardian"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.etcPath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.etcPath))}
                  </div>
                </div>
              </div>
              
              <div>
                <Label htmlFor="homePath">Home Path</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="homePath"
                    value={settings.homePath}
                    onChange={(e) => handleSettingChange('homePath', e.target.value)}
                    placeholder="/home/guardian"
                  />
                  <div className={`${getPathStatusColor(getPathStatus(settings.homePath))}`}>
                    {getPathStatusIcon(getPathStatus(settings.homePath))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Path Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <SettingsIcon className="h-5 w-5" />
            <span>Path Settings</span>
          </CardTitle>
          <CardDescription>
            Configure path behavior and validation
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="pathSeparator">Path Separator</Label>
                <Select value={settings.pathSeparator} onValueChange={(value) => handleSettingChange('pathSeparator', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="/">Forward Slash (/)</SelectItem>
                    <SelectItem value="\\">Backslash (\)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="pathEncoding">Path Encoding</Label>
                <Select value={settings.pathEncoding} onValueChange={(value) => handleSettingChange('pathEncoding', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="utf-8">UTF-8</SelectItem>
                    <SelectItem value="utf-16">UTF-16</SelectItem>
                    <SelectItem value="ascii">ASCII</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="pathCaseSensitive">Case Sensitive</Label>
                  <p className="text-sm text-muted-foreground">Enable case-sensitive path matching</p>
                </div>
                <Switch
                  id="pathCaseSensitive"
                  checked={settings.pathCaseSensitive}
                  onCheckedChange={(checked) => handleSettingChange('pathCaseSensitive', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="pathNormalize">Normalize Paths</Label>
                  <p className="text-sm text-muted-foreground">Normalize path separators</p>
                </div>
                <Switch
                  id="pathNormalize"
                  checked={settings.pathNormalize}
                  onCheckedChange={(checked) => handleSettingChange('pathNormalize', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="pathResolve">Resolve Paths</Label>
                  <p className="text-sm text-muted-foreground">Resolve relative paths to absolute</p>
                </div>
                <Switch
                  id="pathResolve"
                  checked={settings.pathResolve}
                  onCheckedChange={(checked) => handleSettingChange('pathResolve', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="pathValidate">Validate Paths</Label>
                  <p className="text-sm text-muted-foreground">Validate path existence and permissions</p>
                </div>
                <Switch
                  id="pathValidate"
                  checked={settings.pathValidate}
                  onCheckedChange={(checked) => handleSettingChange('pathValidate', checked)}
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Path Permissions */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Path Permissions</span>
          </CardTitle>
          <CardDescription>
            Configure path permissions and ownership
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="pathPermissions">Path Permissions</Label>
                <Input
                  id="pathPermissions"
                  value={settings.pathPermissions}
                  onChange={(e) => handleSettingChange('pathPermissions', e.target.value)}
                  placeholder="755"
                />
              </div>
              
              <div>
                <Label htmlFor="pathOwner">Path Owner</Label>
                <Input
                  id="pathOwner"
                  value={settings.pathOwner}
                  onChange={(e) => handleSettingChange('pathOwner', e.target.value)}
                  placeholder="guardian"
                />
              </div>
              
              <div>
                <Label htmlFor="pathGroup">Path Group</Label>
                <Input
                  id="pathGroup"
                  value={settings.pathGroup}
                  onChange={(e) => handleSettingChange('pathGroup', e.target.value)}
                  placeholder="guardian"
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="pathMode">Path Mode</Label>
                <Input
                  id="pathMode"
                  value={settings.pathMode}
                  onChange={(e) => handleSettingChange('pathMode', e.target.value)}
                  placeholder="755"
                />
              </div>
              
              <div>
                <Label htmlFor="pathUmask">Path Umask</Label>
                <Input
                  id="pathUmask"
                  value={settings.pathUmask}
                  onChange={(e) => handleSettingChange('pathUmask', e.target.value)}
                  placeholder="022"
                />
              </div>
              
              <div>
                <Label htmlFor="pathChown">Path Chown</Label>
                <Input
                  id="pathChown"
                  value={settings.pathChown}
                  onChange={(e) => handleSettingChange('pathChown', e.target.value)}
                  placeholder="guardian:guardian"
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Path Monitoring */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Monitor className="h-5 w-5" />
            <span>Path Monitoring</span>
          </CardTitle>
          <CardDescription>
            Configure path monitoring and file system watching
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enablePathMonitoring">Enable Path Monitoring</Label>
                  <p className="text-sm text-muted-foreground">Enable file system monitoring</p>
                </div>
                <Switch
                  id="enablePathMonitoring"
                  checked={settings.enablePathMonitoring}
                  onCheckedChange={(checked) => handleSettingChange('enablePathMonitoring', checked)}
                />
              </div>
              
              {settings.enablePathMonitoring && (
                <>
                  <div>
                    <Label htmlFor="pathMonitoringInterval">Monitoring Interval (ms)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="pathMonitoringInterval"
                        type="number"
                        value={settings.pathMonitoringInterval}
                        onChange={(e) => handleSettingChange('pathMonitoringInterval', parseInt(e.target.value))}
                        min="1000"
                        max="3600000"
                      />
                      {getStatusIcon(getValidationStatus('pathMonitoringInterval'))}
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="pathMonitoringDepth">Monitoring Depth</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="pathMonitoringDepth"
                        type="number"
                        value={settings.pathMonitoringDepth}
                        onChange={(e) => handleSettingChange('pathMonitoringDepth', parseInt(e.target.value))}
                        min="1"
                        max="100"
                      />
                      {getStatusIcon(getValidationStatus('pathMonitoringDepth'))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enablePathMonitoring && (
                <>
                  <div>
                    <Label htmlFor="pathMonitoringExclude">Exclude Patterns</Label>
                    <Input
                      id="pathMonitoringExclude"
                      value={settings.pathMonitoringExclude}
                      onChange={(e) => handleSettingChange('pathMonitoringExclude', e.target.value)}
                      placeholder="*.tmp,*.log,*.cache"
                    />
                  </div>
                  
                  <div>
                    <Label htmlFor="pathMonitoringInclude">Include Patterns</Label>
                    <Input
                      id="pathMonitoringInclude"
                      value={settings.pathMonitoringInclude}
                      onChange={(e) => handleSettingChange('pathMonitoringInclude', e.target.value)}
                      placeholder="*.jar,*.json,*.yaml,*.yml"
                    />
                  </div>
                </>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
