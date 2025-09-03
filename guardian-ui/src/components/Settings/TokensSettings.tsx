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
  FolderDown,
  Key,
  Lock,
  Unlock,
  Eye,
  EyeOff,
  Copy,
  Trash2,
  Plus,
  Edit,
  Save,
  X
} from 'lucide-react';

interface TokenData {
  id: string;
  name: string;
  description: string;
  type: 'api' | 'webhook' | 'auth' | 'service';
  permissions: string[];
  expiresAt: string | null;
  lastUsedAt: string | null;
  createdAt: string;
  isActive: boolean;
  token: string;
}

interface TokensSettingsData {
  // Token Management
  enableTokens: boolean;
  tokenExpiration: number;
  tokenRotation: boolean;
  tokenRotationInterval: number;
  
  // Token Security
  tokenEncryption: boolean;
  tokenHashing: boolean;
  tokenSalt: string;
  tokenAlgorithm: 'sha256' | 'sha512' | 'bcrypt' | 'argon2';
  
  // Token Permissions
  defaultPermissions: string[];
  permissionGroups: string[];
  permissionRoles: string[];
  
  // Token Monitoring
  enableTokenMonitoring: boolean;
  tokenMonitoringInterval: number;
  tokenAuditLog: boolean;
  tokenUsageTracking: boolean;
  
  // Token Cleanup
  enableTokenCleanup: boolean;
  tokenCleanupInterval: number;
  tokenCleanupAge: number;
  tokenCleanupInactive: boolean;
  
  // Advanced
  enableTokenProfiling: boolean;
  tokenProfilingInterval: number;
  enableTokenDebug: boolean;
  tokenDebugLevel: 'none' | 'basic' | 'detailed' | 'verbose';
}

export const TokensSettings: React.FC = () => {
  const [settings, setSettings] = useState<TokensSettingsData>({
    // Token Management
    enableTokens: true,
    tokenExpiration: 86400,
    tokenRotation: true,
    tokenRotationInterval: 3600,
    
    // Token Security
    tokenEncryption: true,
    tokenHashing: true,
    tokenSalt: 'guardian-salt-2024',
    tokenAlgorithm: 'sha256',
    
    // Token Permissions
    defaultPermissions: ['read', 'write', 'execute'],
    permissionGroups: ['admin', 'user', 'guest'],
    permissionRoles: ['owner', 'moderator', 'member'],
    
    // Token Monitoring
    enableTokenMonitoring: true,
    tokenMonitoringInterval: 60000,
    tokenAuditLog: true,
    tokenUsageTracking: true,
    
    // Token Cleanup
    enableTokenCleanup: true,
    tokenCleanupInterval: 86400000,
    tokenCleanupAge: 30,
    tokenCleanupInactive: true,
    
    // Advanced
    enableTokenProfiling: false,
    tokenProfilingInterval: 60000,
    enableTokenDebug: false,
    tokenDebugLevel: 'none'
  });
  
  const [tokens, setTokens] = useState<TokenData[]>([
    {
      id: '1',
      name: 'API Token',
      description: 'Main API access token',
      type: 'api',
      permissions: ['read', 'write'],
      expiresAt: '2024-12-31T23:59:59Z',
      lastUsedAt: '2024-01-15T10:30:00Z',
      createdAt: '2024-01-01T00:00:00Z',
      isActive: true,
      token: 'gt_1234567890abcdef'
    },
    {
      id: '2',
      name: 'Webhook Token',
      description: 'Webhook integration token',
      type: 'webhook',
      permissions: ['read'],
      expiresAt: null,
      lastUsedAt: '2024-01-14T15:45:00Z',
      createdAt: '2024-01-01T00:00:00Z',
      isActive: true,
      token: 'gt_abcdef1234567890'
    }
  ]);
  
  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [showToken, setShowToken] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);

  const fetchSettings = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch tokens settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const handleSettingChange = (key: keyof TokensSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const getValidationStatus = (key: keyof TokensSettingsData) => {
    const value = settings[key];
    
    switch (key) {
      case 'tokenExpiration':
        return value < 60 || value > 31536000 ? 'error' : 'success';
      case 'tokenRotationInterval':
        return value < 60 || value > 86400 ? 'error' : 'success';
      case 'tokenMonitoringInterval':
        return value < 1000 || value > 3600000 ? 'error' : 'success';
      case 'tokenCleanupInterval':
        return value < 3600000 || value > 604800000 ? 'error' : 'success';
      case 'tokenCleanupAge':
        return value < 1 || value > 365 ? 'error' : 'success';
      case 'tokenProfilingInterval':
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

  const getTokenTypeColor = (type: string) => {
    switch (type) {
      case 'api': return 'bg-blue-500';
      case 'webhook': return 'bg-green-500';
      case 'auth': return 'bg-purple-500';
      case 'service': return 'bg-orange-500';
      default: return 'bg-gray-500';
    }
  };

  const getTokenTypeLabel = (type: string) => {
    switch (type) {
      case 'api': return 'API';
      case 'webhook': return 'Webhook';
      case 'auth': return 'Auth';
      case 'service': return 'Service';
      default: return 'Unknown';
    }
  };

  const getTokenStatus = (token: TokenData) => {
    if (!token.isActive) return 'inactive';
    if (token.expiresAt && new Date(token.expiresAt) < new Date()) return 'expired';
    return 'active';
  };

  const getTokenStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-500';
      case 'inactive': return 'text-gray-500';
      case 'expired': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  const getTokenStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return <CheckCircle className="h-4 w-4" />;
      case 'inactive': return <X className="h-4 w-4" />;
      case 'expired': return <AlertTriangle className="h-4 w-4" />;
      default: return <Info className="h-4 w-4" />;
    }
  };

  const handleCreateToken = () => {
    setIsCreating(true);
    // Mock token creation
    setTimeout(() => {
      const newToken: TokenData = {
        id: Date.now().toString(),
        name: 'New Token',
        description: 'Newly created token',
        type: 'api',
        permissions: ['read'],
        expiresAt: null,
        lastUsedAt: null,
        createdAt: new Date().toISOString(),
        isActive: true,
        token: `gt_${Math.random().toString(36).substr(2, 16)}`
      };
      setTokens(prev => [...prev, newToken]);
      setIsCreating(false);
    }, 1000);
  };

  const handleDeleteToken = (id: string) => {
    setTokens(prev => prev.filter(token => token.id !== id));
  };

  const handleToggleToken = (id: string) => {
    setTokens(prev => prev.map(token => 
      token.id === id ? { ...token, isActive: !token.isActive } : token
    ));
  };

  const handleCopyToken = (token: string) => {
    navigator.clipboard.writeText(token);
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Token Management */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Key className="h-5 w-5" />
            <span>Token Management</span>
          </CardTitle>
          <CardDescription>
            Configure token generation and management
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableTokens">Enable Tokens</Label>
                  <p className="text-sm text-muted-foreground">Enable token-based authentication</p>
                </div>
                <Switch
                  id="enableTokens"
                  checked={settings.enableTokens}
                  onCheckedChange={(checked) => handleSettingChange('enableTokens', checked)}
                />
              </div>
              
              {settings.enableTokens && (
                <>
                  <div>
                    <Label htmlFor="tokenExpiration">Token Expiration (seconds)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="tokenExpiration"
                        type="number"
                        value={settings.tokenExpiration}
                        onChange={(e) => handleSettingChange('tokenExpiration', parseInt(e.target.value))}
                        min="60"
                        max="31536000"
                      />
                      {getStatusIcon(getValidationStatus('tokenExpiration'))}
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="tokenRotation">Token Rotation</Label>
                      <p className="text-sm text-muted-foreground">Enable automatic token rotation</p>
                    </div>
                    <Switch
                      id="tokenRotation"
                      checked={settings.tokenRotation}
                      onCheckedChange={(checked) => handleSettingChange('tokenRotation', checked)}
                    />
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableTokens && (
                <>
                  {settings.tokenRotation && (
                    <div>
                      <Label htmlFor="tokenRotationInterval">Rotation Interval (seconds)</Label>
                      <div className="flex items-center space-x-2">
                        <Input
                          id="tokenRotationInterval"
                          type="number"
                          value={settings.tokenRotationInterval}
                          onChange={(e) => handleSettingChange('tokenRotationInterval', parseInt(e.target.value))}
                          min="60"
                          max="86400"
                        />
                        {getStatusIcon(getValidationStatus('tokenRotationInterval'))}
                      </div>
                    </div>
                  )}
                  
                  <div>
                    <Label htmlFor="tokenAlgorithm">Token Algorithm</Label>
                    <Select value={settings.tokenAlgorithm} onValueChange={(value) => handleSettingChange('tokenAlgorithm', value)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="sha256">SHA-256</SelectItem>
                        <SelectItem value="sha512">SHA-512</SelectItem>
                        <SelectItem value="bcrypt">BCrypt</SelectItem>
                        <SelectItem value="argon2">Argon2</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Token Security */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Token Security</span>
          </CardTitle>
          <CardDescription>
            Configure token security and encryption
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="tokenEncryption">Token Encryption</Label>
                  <p className="text-sm text-muted-foreground">Encrypt tokens at rest</p>
                </div>
                <Switch
                  id="tokenEncryption"
                  checked={settings.tokenEncryption}
                  onCheckedChange={(checked) => handleSettingChange('tokenEncryption', checked)}
                />
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="tokenHashing">Token Hashing</Label>
                  <p className="text-sm text-muted-foreground">Hash tokens for storage</p>
                </div>
                <Switch
                  id="tokenHashing"
                  checked={settings.tokenHashing}
                  onCheckedChange={(checked) => handleSettingChange('tokenHashing', checked)}
                />
              </div>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="tokenSalt">Token Salt</Label>
                <Input
                  id="tokenSalt"
                  value={settings.tokenSalt}
                  onChange={(e) => handleSettingChange('tokenSalt', e.target.value)}
                  placeholder="guardian-salt-2024"
                />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Token List */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Lock className="h-5 w-5" />
              <span>Tokens</span>
            </div>
            <Button onClick={handleCreateToken} disabled={isCreating}>
              <Plus className="h-4 w-4 mr-2" />
              {isCreating ? 'Creating...' : 'Create Token'}
            </Button>
          </CardTitle>
          <CardDescription>
            Manage API tokens and access keys
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {tokens.map((token) => {
              const status = getTokenStatus(token);
              return (
                <div key={token.id} className="flex items-center justify-between p-4 border rounded-lg">
                  <div className="flex items-center space-x-4">
                    <div className={`w-3 h-3 rounded-full ${getTokenTypeColor(token.type)}`} />
                    <div>
                      <div className="font-medium">{token.name}</div>
                      <div className="text-sm text-muted-foreground">{token.description}</div>
                      <div className="flex items-center space-x-2 mt-1">
                        <Badge variant="outline">{getTokenTypeLabel(token.type)}</Badge>
                        <div className={`flex items-center space-x-1 ${getTokenStatusColor(status)}`}>
                          {getTokenStatusIcon(status)}
                          <span className="text-sm capitalize">{status}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowToken(showToken === token.id ? null : token.id)}
                    >
                      {showToken === token.id ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                    </Button>
                    
                    {showToken === token.id && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleCopyToken(token.token)}
                      >
                        <Copy className="h-4 w-4" />
                      </Button>
                    )}
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleToggleToken(token.id)}
                    >
                      {token.isActive ? <Unlock className="h-4 w-4" /> : <Lock className="h-4 w-4" />}
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDeleteToken(token.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Token Monitoring */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Monitor className="h-5 w-5" />
            <span>Token Monitoring</span>
          </CardTitle>
          <CardDescription>
            Configure token monitoring and auditing
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableTokenMonitoring">Enable Token Monitoring</Label>
                  <p className="text-sm text-muted-foreground">Monitor token usage and activity</p>
                </div>
                <Switch
                  id="enableTokenMonitoring"
                  checked={settings.enableTokenMonitoring}
                  onCheckedChange={(checked) => handleSettingChange('enableTokenMonitoring', checked)}
                />
              </div>
              
              {settings.enableTokenMonitoring && (
                <>
                  <div>
                    <Label htmlFor="tokenMonitoringInterval">Monitoring Interval (ms)</Label>
                    <div className="flex items-center space-x-2">
                      <Input
                        id="tokenMonitoringInterval"
                        type="number"
                        value={settings.tokenMonitoringInterval}
                        onChange={(e) => handleSettingChange('tokenMonitoringInterval', parseInt(e.target.value))}
                        min="1000"
                        max="3600000"
                      />
                      {getStatusIcon(getValidationStatus('tokenMonitoringInterval'))}
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="tokenAuditLog">Token Audit Log</Label>
                      <p className="text-sm text-muted-foreground">Log token access and changes</p>
                    </div>
                    <Switch
                      id="tokenAuditLog"
                      checked={settings.tokenAuditLog}
                      onCheckedChange={(checked) => handleSettingChange('tokenAuditLog', checked)}
                    />
                  </div>
                </>
              )}
            </div>
            
            <div className="space-y-4">
              {settings.enableTokenMonitoring && (
                <>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="tokenUsageTracking">Usage Tracking</Label>
                      <p className="text-sm text-muted-foreground">Track token usage statistics</p>
                    </div>
                    <Switch
                      id="tokenUsageTracking"
                      checked={settings.tokenUsageTracking}
                      onCheckedChange={(checked) => handleSettingChange('tokenUsageTracking', checked)}
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
