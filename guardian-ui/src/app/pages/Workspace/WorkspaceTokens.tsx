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
  X, 
  Settings, 
  AlertTriangle,
  CheckCircle,
  Info,
  Shield,
  Database,
  Server,
  Users,
  Globe,
  Network,
  Activity,
  Clock,
  RefreshCw,
  TestTube,
  Download,
  Upload,
  FileText,
  Monitor,
  Zap,
  Layers,
  Package,
  HardDrive,
  Cloud,
  Folder,
  File
} from 'lucide-react';

interface WorkspaceTokenData {
  id: string;
  name: string;
  description: string;
  type: 'api' | 'webhook' | 'auth' | 'service' | 'integration' | 'monitoring';
  permissions: string[];
  scopes: string[];
  status: 'active' | 'inactive' | 'expired' | 'revoked';
  token: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
  usageCount: number;
  rateLimit: {
    requests: number;
    period: 'minute' | 'hour' | 'day';
  };
  ipWhitelist: string[];
  userAgentWhitelist: string[];
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  tags: string[];
  metadata: {
    environment: 'development' | 'staging' | 'production';
    version: string;
    clientId?: string;
    redirectUri?: string;
  };
}

export const WorkspaceTokens: React.FC = () => {
  const [tokens, setTokens] = useState<WorkspaceTokenData[]>([
    {
      id: '1',
      name: 'API Access Token',
      description: 'Main API access token for workspace operations',
      type: 'api',
      permissions: ['read', 'write', 'admin'],
      scopes: ['servers', 'users', 'backups', 'monitoring'],
      status: 'active',
      token: 'gt_ws_1234567890abcdef',
      expiresAt: '2024-12-31T23:59:59Z',
      lastUsedAt: '2024-01-15T10:30:00Z',
      usageCount: 15420,
      rateLimit: {
        requests: 1000,
        period: 'hour'
      },
      ipWhitelist: [],
      userAgentWhitelist: [],
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-15T10:30:00Z',
      createdBy: 'admin',
      tags: ['production', 'api'],
      metadata: {
        environment: 'production',
        version: '1.0.0'
      }
    },
    {
      id: '2',
      name: 'Webhook Integration',
      description: 'Webhook token for external integrations',
      type: 'webhook',
      permissions: ['read'],
      scopes: ['events', 'notifications'],
      status: 'active',
      token: 'gt_ws_abcdef1234567890',
      expiresAt: null,
      lastUsedAt: '2024-01-14T15:45:00Z',
      usageCount: 892,
      rateLimit: {
        requests: 100,
        period: 'minute'
      },
      ipWhitelist: ['192.168.1.0/24', '10.0.0.0/8'],
      userAgentWhitelist: ['Guardian-Webhook/1.0'],
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-14T15:45:00Z',
      createdBy: 'admin',
      tags: ['webhook', 'integration'],
      metadata: {
        environment: 'production',
        version: '1.0.0',
        clientId: 'webhook-client-123',
        redirectUri: 'https://example.com/webhook'
      }
    }
  ]);

  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [selectedToken, setSelectedToken] = useState<WorkspaceTokenData | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [showToken, setShowToken] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');

  const fetchData = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch workspace tokens:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'api': return <Key className="h-4 w-4" />;
      case 'webhook': return <Network className="h-4 w-4" />;
      case 'auth': return <Shield className="h-4 w-4" />;
      case 'service': return <Server className="h-4 w-4" />;
      case 'integration': return <Layers className="h-4 w-4" />;
      case 'monitoring': return <Monitor className="h-4 w-4" />;
      default: return <Key className="h-4 w-4" />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'api': return 'bg-blue-500';
      case 'webhook': return 'bg-green-500';
      case 'auth': return 'bg-purple-500';
      case 'service': return 'bg-orange-500';
      case 'integration': return 'bg-pink-500';
      case 'monitoring': return 'bg-cyan-500';
      default: return 'bg-gray-500';
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'api': return 'API';
      case 'webhook': return 'Webhook';
      case 'auth': return 'Auth';
      case 'service': return 'Service';
      case 'integration': return 'Integration';
      case 'monitoring': return 'Monitoring';
      default: return 'Unknown';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-500';
      case 'inactive': return 'text-gray-500';
      case 'expired': return 'text-red-500';
      case 'revoked': return 'text-red-600';
      default: return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return <CheckCircle className="h-4 w-4" />;
      case 'inactive': return <X className="h-4 w-4" />;
      case 'expired': return <AlertTriangle className="h-4 w-4" />;
      case 'revoked': return <X className="h-4 w-4" />;
      default: return <Info className="h-4 w-4" />;
    }
  };

  const getEnvironmentColor = (environment: string) => {
    switch (environment) {
      case 'development': return 'bg-yellow-500';
      case 'staging': return 'bg-blue-500';
      case 'production': return 'bg-green-500';
      default: return 'bg-gray-500';
    }
  };

  const filteredTokens = tokens.filter(token => {
    const matchesSearch = token.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         token.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         token.token.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesType = filterType === 'all' || token.type === filterType;
    const matchesStatus = filterStatus === 'all' || token.status === filterStatus;
    return matchesSearch && matchesType && matchesStatus;
  });

  const handleCreateToken = () => {
    setIsCreating(true);
    // Mock token creation
    setTimeout(() => {
      const newToken: WorkspaceTokenData = {
        id: Date.now().toString(),
        name: 'New Token',
        description: 'A new workspace token',
        type: 'api',
        permissions: ['read'],
        scopes: ['servers'],
        status: 'active',
        token: `gt_ws_${Math.random().toString(36).substr(2, 16)}`,
        expiresAt: null,
        lastUsedAt: null,
        usageCount: 0,
        rateLimit: {
          requests: 100,
          period: 'hour'
        },
        ipWhitelist: [],
        userAgentWhitelist: [],
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        createdBy: 'admin',
        tags: [],
        metadata: {
          environment: 'development',
          version: '1.0.0'
        }
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
      token.id === id ? { 
        ...token, 
        status: token.status === 'active' ? 'inactive' : 'active' 
      } : token
    ));
  };

  const handleCopyToken = (token: string) => {
    navigator.clipboard.writeText(token);
  };

  const handleRevokeToken = (id: string) => {
    setTokens(prev => prev.map(token => 
      token.id === id ? { 
        ...token, 
        status: 'revoked' as const,
        updatedAt: new Date().toISOString()
      } : token
    ));
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Workspace Tokens */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Key className="h-5 w-5" />
              <span>Workspace Tokens</span>
            </div>
            <Button onClick={handleCreateToken} disabled={isCreating}>
              <Plus className="h-4 w-4 mr-2" />
              {isCreating ? 'Creating...' : 'Create Token'}
            </Button>
          </CardTitle>
          <CardDescription>
            Manage workspace-level API tokens and access keys
          </CardDescription>
        </CardHeader>
        <CardContent>
          {/* Search and Filters */}
          <div className="flex items-center space-x-4 mb-6">
            <div className="flex-1">
              <div className="relative">
                <Key className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search tokens..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <Select value={filterType} onValueChange={setFilterType}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="Filter by type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Types</SelectItem>
                <SelectItem value="api">API</SelectItem>
                <SelectItem value="webhook">Webhook</SelectItem>
                <SelectItem value="auth">Auth</SelectItem>
                <SelectItem value="service">Service</SelectItem>
                <SelectItem value="integration">Integration</SelectItem>
                <SelectItem value="monitoring">Monitoring</SelectItem>
              </SelectContent>
            </Select>
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="Filter by status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="active">Active</SelectItem>
                <SelectItem value="inactive">Inactive</SelectItem>
                <SelectItem value="expired">Expired</SelectItem>
                <SelectItem value="revoked">Revoked</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Tokens List */}
          <div className="space-y-4">
            {filteredTokens.map((token) => (
              <div key={token.id} className="p-4 border rounded-lg">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center space-x-3">
                    <div className={`w-8 h-8 rounded-lg ${getTypeColor(token.type)} flex items-center justify-center text-white`}>
                      {getTypeIcon(token.type)}
                    </div>
                    <div>
                      <div className="font-medium">{token.name}</div>
                      <div className="text-sm text-muted-foreground">{token.description}</div>
                      <div className="flex items-center space-x-2 mt-1">
                        <Badge variant="outline">{getTypeLabel(token.type)}</Badge>
                        <div className={`flex items-center space-x-1 ${getStatusColor(token.status)}`}>
                          {getStatusIcon(token.status)}
                          <span className="text-sm capitalize">{token.status}</span>
                        </div>
                        <Badge className={`${getEnvironmentColor(token.metadata.environment)} text-white`}>
                          {token.metadata.environment}
                        </Badge>
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
                      onClick={() => setSelectedToken(token)}
                    >
                      <Edit className="h-4 w-4" />
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleToggleToken(token.id)}
                    >
                      {token.status === 'active' ? <Unlock className="h-4 w-4" /> : <Lock className="h-4 w-4" />}
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleRevokeToken(token.id)}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
                
                {showToken === token.id && (
                  <div className="mb-4 p-3 bg-muted rounded-lg">
                    <div className="flex items-center justify-between">
                      <div className="font-mono text-sm">{token.token}</div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleCopyToken(token.token)}
                      >
                        <Copy className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                )}
                
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Permissions</div>
                    <div className="flex flex-wrap gap-1">
                      {token.permissions.slice(0, 3).map((permission) => (
                        <Badge key={permission} variant="secondary" className="text-xs">
                          {permission}
                        </Badge>
                      ))}
                      {token.permissions.length > 3 && (
                        <Badge variant="secondary" className="text-xs">
                          +{token.permissions.length - 3}
                        </Badge>
                      )}
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Scopes</div>
                    <div className="flex flex-wrap gap-1">
                      {token.scopes.slice(0, 3).map((scope) => (
                        <Badge key={scope} variant="outline" className="text-xs">
                          {scope}
                        </Badge>
                      ))}
                      {token.scopes.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{token.scopes.length - 3}
                        </Badge>
                      )}
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Usage</div>
                    <div className="text-sm text-muted-foreground">
                      {token.usageCount.toLocaleString()} requests
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {token.rateLimit.requests}/{token.rateLimit.period}
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Last Used</div>
                    <div className="text-sm text-muted-foreground">
                      {token.lastUsedAt ? (
                        new Date(token.lastUsedAt).toLocaleDateString()
                      ) : (
                        'Never'
                      )}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      Expires: {token.expiresAt ? new Date(token.expiresAt).toLocaleDateString() : 'Never'}
                    </div>
                  </div>
                </div>
                
                {token.tags.length > 0 && (
                  <div className="mt-4 pt-4 border-t">
                    <div className="flex items-center space-x-2">
                      <span className="text-sm font-medium">Tags:</span>
                      <div className="flex flex-wrap gap-1">
                        {token.tags.map((tag) => (
                          <Badge key={tag} variant="outline" className="text-xs">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Zap className="h-5 w-5" />
            <span>Quick Actions</span>
          </CardTitle>
          <CardDescription>
            Common token management operations
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <TestTube className="h-6 w-6" />
              <span>Test All Tokens</span>
            </Button>
            
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <RefreshCw className="h-6 w-6" />
              <span>Refresh Status</span>
            </Button>
            
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <Download className="h-6 w-6" />
              <span>Export Tokens</span>
            </Button>
            
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <Shield className="h-6 w-6" />
              <span>Security Audit</span>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
