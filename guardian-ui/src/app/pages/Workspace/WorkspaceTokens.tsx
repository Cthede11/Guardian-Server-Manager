import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { 
  Key, 
  Eye, 
  EyeOff, 
  Copy, 
  ExternalLink,
  CheckCircle,
  AlertCircle,
  Info,
  RefreshCw,
  Save,
  TestTube,
  Shield,
  Settings,
  Database,
  Globe
} from 'lucide-react';

interface APIKeyData {
  id: string;
  name: string;
  key: string;
  type: 'curseforge' | 'modrinth';
  status: 'active' | 'inactive' | 'invalid';
  lastTested: string | null;
  rateLimit: {
    requests: number;
    period: string;
  };
  description: string;
  isRequired: boolean;
  createdAt: string;
  updatedAt: string;
}

export const WorkspaceTokens: React.FC = () => {
  const [apiKeys, setApiKeys] = useState<APIKeyData[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState<string | null>(null);
  const [showKeys, setShowKeys] = useState<{ [key: string]: boolean }>({});
  const [message, setMessage] = useState('');
  const [messageType, setMessageType] = useState<'success' | 'error' | 'info'>('info');

  // Initialize with default API keys
  useEffect(() => {
    const defaultKeys: APIKeyData[] = [
      {
        id: 'curseforge',
        name: 'CurseForge API Key',
        key: '',
        type: 'curseforge',
        status: 'inactive',
        lastTested: null,
        rateLimit: { requests: 100, period: 'minute' },
        description: 'Optional: Add your own key for higher rate limits. App works with default key.',
        isRequired: false,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      },
      {
        id: 'modrinth',
        name: 'Modrinth API Key',
        key: '',
        type: 'modrinth',
        status: 'inactive',
        lastTested: null,
        rateLimit: { requests: 300, period: 'minute' },
        description: 'Optional: Add your token for higher rate limits (300 req/min). App works without it.',
        isRequired: false,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      }
    ];

    // Load saved keys from localStorage
    const savedKeys = defaultKeys.map(key => {
      const saved = localStorage.getItem(`${key.type}_api_key`);
      return saved ? { ...key, key: saved, status: 'active' as const } : key;
    });

    setApiKeys(savedKeys);
  }, []);

  const showMessage = (text: string, type: 'success' | 'error' | 'info') => {
    setMessage(text);
    setMessageType(type);
    setTimeout(() => setMessage(''), 5000);
  };

  const updateApiKey = (id: string, key: string) => {
    setApiKeys(prev => prev.map(apiKey => 
      apiKey.id === id 
        ? { ...apiKey, key, updatedAt: new Date().toISOString() }
        : apiKey
    ));
  };

  const toggleKeyVisibility = (id: string) => {
    setShowKeys(prev => ({ ...prev, [id]: !prev[id] }));
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    showMessage('Copied to clipboard!', 'success');
  };

  const testApiKey = async (apiKey: APIKeyData) => {
    if (!apiKey.key) {
      showMessage(`${apiKey.name} is empty`, 'error');
      return;
    }

    setIsTesting(apiKey.id);
    
    try {
      const testUrl = apiKey.type === 'curseforge' 
        ? 'https://api.curseforge.com/v1/games/432/versions'
        : 'https://staging-api.modrinth.com/v2/tag/game_version';
      
      const headers: Record<string, string> = {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'User-Agent': 'Guardian-ModManager/1.0.0 (modded-manager.com)',
      };
      
      if (apiKey.type === 'curseforge') {
        headers['x-api-key'] = apiKey.key;
      } else {
        headers['Authorization'] = apiKey.key;
      }

      const response = await fetch(testUrl, { headers });
      
      if (response.ok) {
        setApiKeys(prev => prev.map(key => 
          key.id === apiKey.id 
            ? { 
                ...key, 
                status: 'active' as const,
                lastTested: new Date().toISOString()
              }
            : key
        ));
        showMessage(`${apiKey.name} is valid!`, 'success');
      } else {
        setApiKeys(prev => prev.map(key => 
          key.id === apiKey.id 
            ? { 
                ...key, 
                status: 'invalid' as const,
                lastTested: new Date().toISOString()
              }
            : key
        ));
        showMessage(`${apiKey.name} is invalid or expired`, 'error');
      }
    } catch (error) {
      setApiKeys(prev => prev.map(key => 
        key.id === apiKey.id 
          ? { 
              ...key, 
              status: 'invalid' as const,
              lastTested: new Date().toISOString()
            }
          : key
      ));
      showMessage(`Failed to test ${apiKey.name}`, 'error');
    } finally {
      setIsTesting(null);
    }
  };

  const saveApiKeys = async () => {
    setIsSaving(true);
    
    try {
      // Save to localStorage
      apiKeys.forEach(apiKey => {
        if (apiKey.key) {
          localStorage.setItem(`${apiKey.type}_api_key`, apiKey.key);
        } else {
          localStorage.removeItem(`${apiKey.type}_api_key`);
        }
      });

      // Update environment variables for the current session
      apiKeys.forEach(apiKey => {
        if (apiKey.key) {
          (window as any).env = (window as any).env || {};
          (window as any).env[`VITE_${apiKey.type.toUpperCase()}_API_KEY`] = apiKey.key;
        }
      });

      showMessage('API keys saved successfully!', 'success');
    } catch (error) {
      showMessage('Failed to save API keys', 'error');
    } finally {
      setIsSaving(false);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'invalid':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      default:
        return <Key className="w-4 h-4 text-gray-400" />;
    }
  };

  const getStatusBadge = (apiKey: APIKeyData) => {
    if (!apiKey.key) {
      return <Badge variant="secondary">Not Set</Badge>;
    }
    
    switch (apiKey.status) {
      case 'active':
        return <Badge variant="default" className="bg-green-500">Valid</Badge>;
      case 'invalid':
        return <Badge variant="destructive">Invalid</Badge>;
      default:
        return <Badge variant="outline">Set</Badge>;
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'curseforge':
        return <img src="https://www.curseforge.com/favicon.ico" alt="CurseForge" className="w-5 h-5" />;
      case 'modrinth':
        return <img src="https://modrinth.com/favicon.ico" alt="Modrinth" className="w-5 h-5" />;
      default:
        return <Key className="w-5 h-5" />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'curseforge':
        return 'border-orange-200 bg-orange-50';
      case 'modrinth':
        return 'border-green-200 bg-green-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">API Keys</h2>
        <p className="text-muted-foreground">
          Configure API keys for external services to access real mod data
        </p>
      </div>

      <Alert className="border-blue-200 bg-blue-50">
        <Info className="h-4 w-4" />
        <AlertDescription>
          <strong>Good news!</strong> The app works out of the box with default API keys. 
          You only need to add your own keys if you want higher rate limits or personal usage tracking.
        </AlertDescription>
      </Alert>

      {message && (
        <Alert className={messageType === 'error' ? 'border-red-200 bg-red-50' : messageType === 'success' ? 'border-green-200 bg-green-50' : ''}>
          {messageType === 'error' ? <AlertCircle className="h-4 w-4" /> : messageType === 'success' ? <CheckCircle className="h-4 w-4" /> : <Info className="h-4 w-4" />}
          <AlertDescription>{message}</AlertDescription>
        </Alert>
      )}

      <div className="grid gap-6">
        {apiKeys.map((apiKey) => (
          <Card key={apiKey.id} className={`${getTypeColor(apiKey.type)}`}>
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  {getTypeIcon(apiKey.type)}
                  <div>
                    <div className="flex items-center gap-2">
                      <span>{apiKey.name}</span>
                      {apiKey.isRequired && (
                        <Badge variant="destructive" className="text-xs">Required</Badge>
                      )}
                    </div>
                    <CardDescription className="mt-1">
                      {apiKey.description}
                    </CardDescription>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {getStatusIcon(apiKey.status)}
                  {getStatusBadge(apiKey)}
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor={`${apiKey.id}-key`}>API Key</Label>
                <div className="relative">
                  <Input
                    id={`${apiKey.id}-key`}
                    type={showKeys[apiKey.id] ? 'text' : 'password'}
                    value={apiKey.key}
                    onChange={(e) => updateApiKey(apiKey.id, e.target.value)}
                    placeholder={`Enter your ${apiKey.name}`}
                    className="pr-20"
                  />
                  <div className="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => toggleKeyVisibility(apiKey.id)}
                    >
                      {showKeys[apiKey.id] ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard(apiKey.key)}
                      disabled={!apiKey.key}
                    >
                      <Copy className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </div>

              <div className="flex gap-2">
                <Button
                  onClick={() => testApiKey(apiKey)}
                  disabled={!apiKey.key || isTesting === apiKey.id}
                  variant="outline"
                  className="flex-1"
                >
                  {isTesting === apiKey.id ? (
                    <>
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                      Testing...
                    </>
                  ) : (
                    <>
                      <TestTube className="w-4 h-4 mr-2" />
                      Test API Key
                    </>
                  )}
                </Button>
                
                <Button
                  variant="outline"
                  onClick={() => window.open(
                    apiKey.type === 'curseforge' 
                      ? 'https://docs.curseforge.com/#authentication'
                      : 'https://modrinth.com/settings/tokens',
                    '_blank'
                  )}
                  className="flex-1"
                >
                  <ExternalLink className="w-4 h-4 mr-2" />
                  Get API Key
                </Button>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm text-muted-foreground">
                <div>
                  <div className="font-medium">Rate Limit</div>
                  <div>{apiKey.rateLimit.requests} requests/{apiKey.rateLimit.period}</div>
                  {apiKey.type === 'modrinth' && (
                    <div className="text-xs text-muted-foreground mt-1">
                      Same limit with or without token
                    </div>
                  )}
                </div>
                <div>
                  <div className="font-medium">Last Tested</div>
                  <div>
                    {apiKey.lastTested 
                      ? new Date(apiKey.lastTested).toLocaleString()
                      : 'Never'
                    }
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Separator />

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Security Information
          </CardTitle>
          <CardDescription>
            How your API keys are stored and used
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <h4 className="font-medium mb-2">Data Sources</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>• CurseForge: Mod search and details</li>
                <li>• Modrinth: Modern mod loaders (Fabric, Quilt)</li>
                <li>• Combined results for comprehensive coverage</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium mb-2">Security</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>• Keys stored locally in browser</li>
                <li>• Never transmitted to our servers</li>
                <li>• Used only for API requests</li>
              </ul>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-end">
        <Button 
          onClick={saveApiKeys}
          disabled={isSaving}
          className="min-w-32"
        >
          {isSaving ? (
            <>
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              Saving...
            </>
          ) : (
            <>
              <Save className="w-4 h-4 mr-2" />
              Save API Keys
            </>
          )}
        </Button>
      </div>
    </div>
  );
};