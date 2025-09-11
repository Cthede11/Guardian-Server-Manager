import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useServers } from '@/store/servers-new';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  AlertTriangle,
  CheckCircle,
  Info,
  Activity,
  Database,
  Server,
  Users,
  RefreshCw,
  Network,
  Shield
} from 'lucide-react';

interface HASettingsData {
  // High Availability Settings
  enableHA: boolean;
  haMode: 'active-passive' | 'active-active' | 'load-balanced';
  haNodes: number;
  haReplicationFactor: number;
  
  // Node Configuration
  nodeId: string;
  nodeRole: 'primary' | 'secondary' | 'standby';
  nodePriority: number;
  nodeWeight: number;
  
  // Failover Settings
  failoverTimeout: number;
  failoverThreshold: number;
  failoverStrategy: 'automatic' | 'manual' | 'semi-automatic';
  failoverCooldown: number;
  
  // Health Monitoring
  healthCheckInterval: number;
  healthCheckTimeout: number;
  healthCheckRetries: number;
  healthCheckEndpoint: string;
  
  // Load Balancing
  enableLoadBalancing: boolean;
  loadBalancingAlgorithm: 'round-robin' | 'least-connections' | 'weighted' | 'ip-hash';
  loadBalancingWeight: number;
  loadBalancingThreshold: number;
  
  // Data Replication
  enableReplication: boolean;
  replicationMode: 'synchronous' | 'asynchronous' | 'semi-synchronous';
  replicationLag: number;
  replicationTimeout: number;
  
  // Backup and Recovery
  enableBackup: boolean;
  backupInterval: number;
  backupRetention: number;
  backupCompression: boolean;
  backupEncryption: boolean;
  
  // Network Settings
  haNetworkInterface: string;
  haNetworkPort: number;
  haNetworkProtocol: 'tcp' | 'udp' | 'http' | 'https';
  haNetworkTimeout: number;
  
  // Security Settings
  enableHASecurity: boolean;
  haAuthentication: boolean;
  haEncryption: boolean;
  haAccessControl: boolean;
  
  // Monitoring and Logging
  enableHAMonitoring: boolean;
  haMonitoringInterval: number;
  haLogLevel: 'none' | 'basic' | 'detailed' | 'verbose';
  haLogRetention: number;
  
  // Advanced Settings
  enableHAProfiling: boolean;
  haProfilingInterval: number;
  enableHADebug: boolean;
  haDebugLevel: 'none' | 'basic' | 'detailed' | 'verbose';
}

export const HASettings: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { 
    fetchSettings, 
    updateSettings,
    settings 
  } = useServers();
  
  const [settings, setSettings] = useState<HASettingsData>({
    // High Availability Settings
    enableHA: false,
    haMode: 'active-passive',
    haNodes: 2,
    haReplicationFactor: 2,
    
    // Node Configuration
    nodeId: 'node-1',
    nodeRole: 'primary',
    nodePriority: 100,
    nodeWeight: 50,
    
    // Failover Settings
    failoverTimeout: 30000,
    failoverThreshold: 3,
    failoverStrategy: 'automatic',
    failoverCooldown: 60000,
    
    // Health Monitoring
    healthCheckInterval: 5000,
    healthCheckTimeout: 10000,
    healthCheckRetries: 3,
    healthCheckEndpoint: '/health',
    
    // Load Balancing
    enableLoadBalancing: false,
    loadBalancingAlgorithm: 'round-robin',
    loadBalancingWeight: 50,
    loadBalancingThreshold: 80,
    
    // Data Replication
    enableReplication: true,
    replicationMode: 'asynchronous',
    replicationLag: 1000,
    replicationTimeout: 30000,
    
    // Backup and Recovery
    enableBackup: true,
    backupInterval: 3600000,
    backupRetention: 7,
    backupCompression: true,
    backupEncryption: false,
    
    // Network Settings
    haNetworkInterface: 'eth0',
    haNetworkPort: 8080,
    haNetworkProtocol: 'tcp',
    haNetworkTimeout: 30000,
    
    // Security Settings
    enableHASecurity: true,
    haAuthentication: true,
    haEncryption: false,
    haAccessControl: true,
    
    // Monitoring and Logging
    enableHAMonitoring: true,
    haMonitoringInterval: 10000,
    haLogLevel: 'basic',
    haLogRetention: 30,
    
    // Advanced Settings
    enableHAProfiling: false,
    haProfilingInterval: 60000,
    enableHADebug: false,
    haDebugLevel: 'none'
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
      console.error('Failed to fetch HA settings:', error);
    }
  };

  useEffect(() => {
    loadSettings();
  }, []);

  // Sync settings with server store data
  useEffect(() => {
    if (serverId && settings[serverId]) {
      const serverData = settings[serverId];
      if (serverData.ha) {
        setSettings(prev => ({
          ...prev,
          ...serverData.ha,
        }));
      }
    }
  }, [serverId, serverSettings]);

  const handleSettingChange = async (key: keyof HASettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    
    if (!serverId) return;
    
    try {
      // Update server configuration
      await updateSettings(serverId, {
        ha: {
          ...settings,
          [key]: value
        }
      });
    } catch (error) {
      console.error('Failed to update HA settings:', error);
    }
  };

  const getValidationStatus = (key: keyof HASettingsData) => {
    const value = settings[key];
    
    // Ensure value is a number for comparison
    const numValue = typeof value === 'number' ? value : 0;
    
    switch (key) {
      case 'haNodes':
        return numValue < 2 || numValue > 10 ? 'error' : 'success';
      case 'haReplicationFactor':
        return numValue < 1 || numValue > 5 ? 'error' : 'success';
      case 'nodePriority':
        return numValue < 1 || numValue > 1000 ? 'error' : 'success';
      case 'nodeWeight':
        return numValue < 1 || numValue > 100 ? 'error' : 'success';
      case 'failoverTimeout':
        return numValue < 1000 || numValue > 300000 ? 'error' : 'success';
      case 'failoverThreshold':
        return numValue < 1 || numValue > 10 ? 'error' : 'success';
      case 'failoverCooldown':
        return numValue < 1000 || numValue > 600000 ? 'error' : 'success';
      case 'healthCheckInterval':
        return numValue < 1000 || numValue > 60000 ? 'error' : 'success';
      case 'healthCheckTimeout':
        return numValue < 1000 || numValue > 60000 ? 'error' : 'success';
      case 'healthCheckRetries':
        return numValue < 1 || numValue > 10 ? 'error' : 'success';
      case 'loadBalancingWeight':
        return numValue < 1 || numValue > 100 ? 'error' : 'success';
      case 'loadBalancingThreshold':
        return numValue < 1 || numValue > 100 ? 'error' : 'success';
      case 'replicationLag':
        return numValue < 0 || numValue > 10000 ? 'error' : 'success';
      case 'replicationTimeout':
        return numValue < 1000 || numValue > 300000 ? 'error' : 'success';
      case 'backupInterval':
        return numValue < 60000 || numValue > 86400000 ? 'error' : 'success';
      case 'backupRetention':
        return numValue < 1 || numValue > 365 ? 'error' : 'success';
      case 'haNetworkPort':
        return numValue < 1 || numValue > 65535 ? 'error' : 'success';
      case 'haNetworkTimeout':
        return numValue < 1000 || numValue > 300000 ? 'error' : 'success';
      case 'haMonitoringInterval':
        return numValue < 1000 || numValue > 60000 ? 'error' : 'success';
      case 'haLogRetention':
        return numValue < 1 || numValue > 365 ? 'error' : 'success';
      case 'haProfilingInterval':
        return numValue < 1000 || numValue > 600000 ? 'error' : 'success';
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
      case 'active-passive': return 'bg-blue-500';
      case 'active-active': return 'bg-green-500';
      case 'load-balanced': return 'bg-purple-500';
      default: return 'bg-gray-500';
    }
  };

  const getModeLabel = (mode: string) => {
    switch (mode) {
      case 'active-passive': return 'Active-Passive';
      case 'active-active': return 'Active-Active';
      case 'load-balanced': return 'Load Balanced';
      default: return 'Unknown';
    }
  };

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'primary': return 'bg-green-500';
      case 'secondary': return 'bg-yellow-500';
      case 'standby': return 'bg-gray-500';
      default: return 'bg-gray-500';
    }
  };

  const getRoleLabel = (role: string) => {
    switch (role) {
      case 'primary': return 'Primary';
      case 'secondary': return 'Secondary';
      case 'standby': return 'Standby';
      default: return 'Unknown';
    }
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* High Availability Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Server className="h-5 w-5" />
            <span>High Availability Settings</span>
          </CardTitle>
          <CardDescription>
            Configure high availability and failover settings
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableHA">Enable High Availability</Label>
                  <p className="text-sm text-muted-foreground">Enable HA clustering and failover</p>
                </div>
                <Switch
                  id="enableHA"
                  checked={settings.enableHA}
                  onCheckedChange={(checked) => handleSettingChange('enableHA', checked)}
                />
              </div>
              
              {settings.enableHA && (
                <>
                  <div>
                    <Label htmlFor="haMode">HA Mode</Label>
                    <Select value={settings.haMode} onValueChange={(value) => handleSettingChange('haMode', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="active-passive">Active-Passive</SelectItem>
                        <SelectItem value="active-active">Active-Active</SelectItem>
                        <SelectItem value="load-balanced">Load Balanced</SelectItem>
                      </SelectContent>
                    </Select>
                    <div className="flex items-center space-x-2 mt-2">
                      <div className={`w-3 h-3 rounded-full ${getModeColor(settings.haMode)}`} />
                      <span className="text-sm text-muted-foreground">{getModeLabel(settings.haMode)}</span>
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="haNodes">Number of Nodes</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="haNodes"
                        type="number"
                        value={settings.haNodes}
                        onChange={(e) => handleSettingChange('haNodes', parseInt(e.target.value))}
                        min="2"
                        max="10"
                      />
                      {getStatusIcon(getValidationStatus('haNodes'))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableHA && (
                <>
                  <div>
                    <Label htmlFor="haReplicationFactor">Replication Factor</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="haReplicationFactor"
                        type="number"
                        value={settings.haReplicationFactor}
                        onChange={(e) => handleSettingChange('haReplicationFactor', parseInt(e.target.value))}
                        min="1"
                        max="5"
                      />
                      {getStatusIcon(getValidationStatus('haReplicationFactor'))}
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="nodeId">Node ID</Label>
                    <Input
                      id="nodeId"
                      value={settings.nodeId}
                      onChange={(e) => handleSettingChange('nodeId', e.target.value)}
                      placeholder="node-1"
                    />
                  </div>
                  
                  <div>
                    <Label htmlFor="nodeRole">Node Role</Label>
                    <Select value={settings.nodeRole} onValueChange={(value) => handleSettingChange('nodeRole', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="primary">Primary</SelectItem>
                        <SelectItem value="secondary">Secondary</SelectItem>
                        <SelectItem value="standby">Standby</SelectItem>
                      </SelectContent>
                    </Select>
                    <div className="flex items-center space-x-2 mt-2">
                      <div className={`w-3 h-3 rounded-full ${getRoleColor(settings.nodeRole)}`} />
                      <span className="text-sm text-muted-foreground">{getRoleLabel(settings.nodeRole)}</span>
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Failover Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <RefreshCw className="h-5 w-5" />
            <span>Failover Settings</span>
          </CardTitle>
          <CardDescription>
            Configure failover behavior and thresholds
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="failoverStrategy">Failover Strategy</Label>
                <Select value={settings.failoverStrategy} onValueChange={(value) => handleSettingChange('failoverStrategy', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="automatic">Automatic</SelectItem>
                    <SelectItem value="manual">Manual</SelectItem>
                    <SelectItem value="semi-automatic">Semi-Automatic</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="failoverTimeout">Failover Timeout (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="failoverTimeout"
                    type="number"
                    value={settings.failoverTimeout}
                    onChange={(e) => handleSettingChange('failoverTimeout', parseInt(e.target.value))}
                    min="1000"
                    max="300000"
                  />
                  {getStatusIcon(getValidationStatus('failoverTimeout'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="failoverThreshold">Failover Threshold</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="failoverThreshold"
                    type="number"
                    value={settings.failoverThreshold}
                    onChange={(e) => handleSettingChange('failoverThreshold', parseInt(e.target.value))}
                    min="1"
                    max="10"
                  />
                  {getStatusIcon(getValidationStatus('failoverThreshold'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="failoverCooldown">Failover Cooldown (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="failoverCooldown"
                    type="number"
                    value={settings.failoverCooldown}
                    onChange={(e) => handleSettingChange('failoverCooldown', parseInt(e.target.value))}
                    min="1000"
                    max="600000"
                  />
                  {getStatusIcon(getValidationStatus('failoverCooldown'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="nodePriority">Node Priority</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="nodePriority"
                    type="number"
                    value={settings.nodePriority}
                    onChange={(e) => handleSettingChange('nodePriority', parseInt(e.target.value))}
                    min="1"
                    max="1000"
                  />
                  {getStatusIcon(getValidationStatus('nodePriority'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="nodeWeight">Node Weight</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="nodeWeight"
                    type="number"
                    value={settings.nodeWeight}
                    onChange={(e) => handleSettingChange('nodeWeight', parseInt(e.target.value))}
                    min="1"
                    max="100"
                  />
                  {getStatusIcon(getValidationStatus('nodeWeight'))}
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Health Monitoring */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>Health Monitoring</span>
          </CardTitle>
          <CardDescription>
            Configure health checks and monitoring
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="healthCheckInterval">Health Check Interval (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="healthCheckInterval"
                    type="number"
                    value={settings.healthCheckInterval}
                    onChange={(e) => handleSettingChange('healthCheckInterval', parseInt(e.target.value))}
                    min="1000"
                    max="60000"
                  />
                  {getStatusIcon(getValidationStatus('healthCheckInterval'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="healthCheckTimeout">Health Check Timeout (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="healthCheckTimeout"
                    type="number"
                    value={settings.healthCheckTimeout}
                    onChange={(e) => handleSettingChange('healthCheckTimeout', parseInt(e.target.value))}
                    min="1000"
                    max="60000"
                  />
                  {getStatusIcon(getValidationStatus('healthCheckTimeout'))}
                </div>
              </div>
              
              <div>
                <Label htmlFor="healthCheckRetries">Health Check Retries</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="healthCheckRetries"
                    type="number"
                    value={settings.healthCheckRetries}
                    onChange={(e) => handleSettingChange('healthCheckRetries', parseInt(e.target.value))}
                    min="1"
                    max="10"
                  />
                  {getStatusIcon(getValidationStatus('healthCheckRetries'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="healthCheckEndpoint">Health Check Endpoint</Label>
                <Input
                  id="healthCheckEndpoint"
                  value={settings.healthCheckEndpoint}
                  onChange={(e) => handleSettingChange('healthCheckEndpoint', e.target.value)}
                  placeholder="/health"
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableHAMonitoring">Enable HA Monitoring</Label>
                  <p className="text-sm text-muted-foreground">Enable high availability monitoring</p>
                </div>
                <Switch
                  id="enableHAMonitoring"
                  checked={settings.enableHAMonitoring}
                  onCheckedChange={(checked) => handleSettingChange('enableHAMonitoring', checked)}
                />
              </div>
              
              {settings.enableHAMonitoring && (
                <div>
                  <Label htmlFor="haMonitoringInterval">Monitoring Interval (ms)</Label>
                  <div className="flex items-center space-x-2">
                    <Input
                      id="haMonitoringInterval"
                      type="number"
                      value={settings.haMonitoringInterval}
                      onChange={(e) => handleSettingChange('haMonitoringInterval', parseInt(e.target.value))}
                      min="1000"
                      max="60000"
                    />
                    {getStatusIcon(getValidationStatus('haMonitoringInterval'))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Load Balancing */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Users className="h-5 w-5" />
            <span>Load Balancing</span>
          </CardTitle>
          <CardDescription>
            Configure load balancing for active-active mode
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableLoadBalancing">Enable Load Balancing</Label>
                  <p className="text-sm text-muted-foreground">Enable load balancing across nodes</p>
                </div>
                <Switch
                  id="enableLoadBalancing"
                  checked={settings.enableLoadBalancing}
                  onCheckedChange={(checked) => handleSettingChange('enableLoadBalancing', checked)}
                />
              </div>
              
              {settings.enableLoadBalancing && (
                <>
                  <div>
                    <Label htmlFor="loadBalancingAlgorithm">Load Balancing Algorithm</Label>
                    <Select value={settings.loadBalancingAlgorithm} onValueChange={(value) => handleSettingChange('loadBalancingAlgorithm', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="round-robin">Round Robin</SelectItem>
                        <SelectItem value="least-connections">Least Connections</SelectItem>
                        <SelectItem value="weighted">Weighted</SelectItem>
                        <SelectItem value="ip-hash">IP Hash</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div>
                    <Label htmlFor="loadBalancingWeight">Load Balancing Weight</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="loadBalancingWeight"
                        type="number"
                        value={settings.loadBalancingWeight}
                        onChange={(e) => handleSettingChange('loadBalancingWeight', parseInt(e.target.value))}
                        min="1"
                        max="100"
                      />
                      {getStatusIcon(getValidationStatus('loadBalancingWeight'))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableLoadBalancing && (
                <>
                  <div>
                    <Label htmlFor="loadBalancingThreshold">Load Balancing Threshold (%)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="loadBalancingThreshold"
                        type="number"
                        value={settings.loadBalancingThreshold}
                        onChange={(e) => handleSettingChange('loadBalancingThreshold', parseInt(e.target.value))}
                        min="1"
                        max="100"
                      />
                      {getStatusIcon(getValidationStatus('loadBalancingThreshold'))}
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Data Replication */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Database className="h-5 w-5" />
            <span>Data Replication</span>
          </CardTitle>
          <CardDescription>
            Configure data replication between nodes
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableReplication">Enable Replication</Label>
                  <p className="text-sm text-muted-foreground">Enable data replication</p>
                </div>
                <Switch
                  id="enableReplication"
                  checked={settings.enableReplication}
                  onCheckedChange={(checked) => handleSettingChange('enableReplication', checked)}
                />
              </div>
              
              {settings.enableReplication && (
                <>
                  <div>
                    <Label htmlFor="replicationMode">Replication Mode</Label>
                    <Select value={settings.replicationMode} onValueChange={(value) => handleSettingChange('replicationMode', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="synchronous">Synchronous</SelectItem>
                        <SelectItem value="asynchronous">Asynchronous</SelectItem>
                        <SelectItem value="semi-synchronous">Semi-Synchronous</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div>
                    <Label htmlFor="replicationLag">Replication Lag (ms)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="replicationLag"
                        type="number"
                        value={settings.replicationLag}
                        onChange={(e) => handleSettingChange('replicationLag', parseInt(e.target.value))}
                        min="0"
                        max="10000"
                      />
                      {getStatusIcon(getValidationStatus('replicationLag'))}
                    </div>
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableReplication && (
                <>
                  <div>
                    <Label htmlFor="replicationTimeout">Replication Timeout (ms)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="replicationTimeout"
                        type="number"
                        value={settings.replicationTimeout}
                        onChange={(e) => handleSettingChange('replicationTimeout', parseInt(e.target.value))}
                        min="1000"
                        max="300000"
                      />
                      {getStatusIcon(getValidationStatus('replicationTimeout'))}
                    </div>
                  </div>
                </>
              )}
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
            Configure HA network communication
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <Label htmlFor="haNetworkInterface">Network Interface</Label>
                <Input
                  id="haNetworkInterface"
                  value={settings.haNetworkInterface}
                  onChange={(e) => handleSettingChange('haNetworkInterface', e.target.value)}
                  placeholder="eth0"
                />
              </div>
              
              <div>
                <Label htmlFor="haNetworkPort">Network Port</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="haNetworkPort"
                    type="number"
                    value={settings.haNetworkPort}
                    onChange={(e) => handleSettingChange('haNetworkPort', parseInt(e.target.value))}
                    min="1"
                    max="65535"
                  />
                  {getStatusIcon(getValidationStatus('haNetworkPort'))}
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="haNetworkProtocol">Network Protocol</Label>
                <Select value={settings.haNetworkProtocol} onValueChange={(value) => handleSettingChange('haNetworkProtocol', value)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="tcp">TCP</SelectItem>
                    <SelectItem value="udp">UDP</SelectItem>
                    <SelectItem value="http">HTTP</SelectItem>
                    <SelectItem value="https">HTTPS</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="haNetworkTimeout">Network Timeout (ms)</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="haNetworkTimeout"
                    type="number"
                    value={settings.haNetworkTimeout}
                    onChange={(e) => handleSettingChange('haNetworkTimeout', parseInt(e.target.value))}
                    min="1000"
                    max="300000"
                  />
                  {getStatusIcon(getValidationStatus('haNetworkTimeout'))}
                </div>
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
            Configure HA security and access control
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableHASecurity">Enable HA Security</Label>
                  <p className="text-sm text-muted-foreground">Enable HA security features</p>
                </div>
                <Switch
                  id="enableHASecurity"
                  checked={settings.enableHASecurity}
                  onCheckedChange={(checked) => handleSettingChange('enableHASecurity', checked)}
                />
              </div>
              
              {settings.enableHASecurity && (
                <>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="haAuthentication">HA Authentication</Label>
                      <p className="text-sm text-muted-foreground">Enable HA authentication</p>
                    </div>
                    <Switch
                      id="haAuthentication"
                      checked={settings.haAuthentication}
                      onCheckedChange={(checked) => handleSettingChange('haAuthentication', checked)}
                    />
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="haEncryption">HA Encryption</Label>
                      <p className="text-sm text-muted-foreground">Enable HA encryption</p>
                    </div>
                    <Switch
                      id="haEncryption"
                      checked={settings.haEncryption}
                      onCheckedChange={(checked) => handleSettingChange('haEncryption', checked)}
                    />
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableHASecurity && (
                <>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="haAccessControl">HA Access Control</Label>
                      <p className="text-sm text-muted-foreground">Enable HA access control</p>
                    </div>
                    <Switch
                      id="haAccessControl"
                      checked={settings.haAccessControl}
                      onCheckedChange={(checked) => handleSettingChange('haAccessControl', checked)}
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
