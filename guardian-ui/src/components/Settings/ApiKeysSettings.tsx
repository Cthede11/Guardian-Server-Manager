import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useToast } from '@/hooks/use-toast';
import { apiClient as api } from '@/lib/api';
import { 
  Key, 
  CheckCircle, 
  XCircle, 
  AlertTriangle, 
  Info, 
  ExternalLink,
  Eye,
  EyeOff,
  Copy,
  RefreshCw,
  Shield,
  Globe,
  Package
} from 'lucide-react';

interface ApiKeyData {
  id: string;
  name: string;
  provider: 'curseforge' | 'modrinth' | 'github' | 'custom';
  key: string;
  isValid: boolean;
  lastTested: string | null;
  lastUsed: string | null;
  createdAt: string;
  isActive: boolean;
  description?: string;
  rateLimit?: {
    limit: number;
    remaining: number;
    reset: string;
  };
}

interface ApiKeysSettingsData {
  // API Key Management
  enableApiKeys: boolean;
  autoTestKeys: boolean;
  keyValidationInterval: number;
  
  // CurseForge Settings
  curseForgeApiKey: string;
  curseForgeEnabled: boolean;
  curseForgeRateLimit: number;
  
  // Modrinth Settings
  modrinthApiKey: string;
  modrinthEnabled: boolean;
  modrinthRateLimit: number;
  
  // GitHub Settings
  githubApiKey: string;
  githubEnabled: boolean;
  githubRateLimit: number;
  
  // Custom API Settings
  customApiKeys: Array<{
    name: string;
    url: string;
    key: string;
    enabled: boolean;
  }>;
  
  // Security Settings
  encryptApiKeys: boolean;
  keyRotation: boolean;
  keyRotationInterval: number;
  
  // Monitoring
  enableKeyMonitoring: boolean;
  keyMonitoringInterval: number;
  keyUsageTracking: boolean;
}

const API_PROVIDERS = [
  {
    id: 'curseforge',
    name: 'CurseForge',
    description: 'Mod and modpack browsing from CurseForge',
    icon: <Package className="h-4 w-4" />,
    color: 'bg-orange-500',
    url: 'https://console.curseforge.com/',
    rateLimit: '100 requests/minute'
  },
  {
    id: 'modrinth',
    name: 'Modrinth',
    description: 'Mod and modpack browsing from Modrinth',
    icon: <Globe className="h-4 w-4" />,
    color: 'bg-green-500',
    url: 'https://modrinth.com/settings/tokens',
    rateLimit: '1000 requests/hour'
  },
  {
    id: 'github',
    name: 'GitHub',
    description: 'GitHub integration for mod repositories',
    icon: <ExternalLink className="h-4 w-4" />,
    color: 'bg-gray-500',
    url: 'https://github.com/settings/tokens',
    rateLimit: '5000 requests/hour'
  }
];

export const ApiKeysSettings: React.FC = () => {
  const [settings, setSettings] = useState<ApiKeysSettingsData>({
    // API Key Management
    enableApiKeys: true,
    autoTestKeys: true,
    keyValidationInterval: 3600000, // 1 hour
    
    // CurseForge Settings
    curseForgeApiKey: '',
    curseForgeEnabled: true,
    curseForgeRateLimit: 100,
    
    // Modrinth Settings
    modrinthApiKey: '',
    modrinthEnabled: true,
    modrinthRateLimit: 1000,
    
    // GitHub Settings
    githubApiKey: '',
    githubEnabled: false,
    githubRateLimit: 5000,
    
    // Custom API Settings
    customApiKeys: [],
    
    // Security Settings
    encryptApiKeys: true,
    keyRotation: false,
    keyRotationInterval: 86400000, // 24 hours
    
    // Monitoring
    enableKeyMonitoring: true,
    keyMonitoringInterval: 300000, // 5 minutes
    keyUsageTracking: true
  });
  
  const [apiKeys, setApiKeys] = useState<ApiKeyData[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isTesting, setIsTesting] = useState<string | null>(null);
  const [showKeys, setShowKeys] = useState<Record<string, boolean>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const { toast } = useToast();

  useEffect(() => {
    loadSettings();
    loadApiKeys();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await api.getAppSettings();
      if (response.ok && response.data) {
        const data = response.data;
        setSettings(prev => ({
          ...prev,
          curseForgeApiKey: data.curseForgeApiKey || '',
          modrinthApiKey: data.modrinthApiKey || '',
          githubApiKey: data.githubApiKey || '',
          enableApiKeys: data.enableApiKeys !== false,
          autoTestKeys: data.autoTestKeys !== false,
          encryptApiKeys: data.encryptApiKeys !== false,
        }));
      }
    } catch (error) {
      console.error('Failed to load API key settings:', error);
    }
  };

  const loadApiKeys = async () => {
    try {
      const response = await api.getApiKeys();
      if (response.ok && response.data) {
        setApiKeys(response.data);
      }
    } catch (error) {
      console.error('Failed to load API keys:', error);
    }
  };

  const saveSettings = async () => {
    try {
      setIsLoading(true);
      const response = await api.updateAppSettings({
        curseForgeApiKey: settings.curseForgeApiKey,
        modrinthApiKey: settings.modrinthApiKey,
        githubApiKey: settings.githubApiKey,
        enableApiKeys: settings.enableApiKeys,
        autoTestKeys: settings.autoTestKeys,
        encryptApiKeys: settings.encryptApiKeys,
      });
      
      if (response.ok) {
        toast({
          title: 'Settings Saved',
          description: 'API key settings have been saved successfully.',
          variant: 'success'
        });
      } else {
        throw new Error(response.error || 'Failed to save settings');
      }
    } catch (error) {
      console.error('Failed to save settings:', error);
      toast({
        title: 'Save Failed',
        description: 'Failed to save API key settings. Please try again.',
        variant: 'destructive'
      });
    } finally {
      setIsLoading(false);
    }
  };

  const testApiKey = async (provider: string) => {
    if (!settings[`${provider}ApiKey` as keyof ApiKeysSettingsData]) {
      toast({
        title: 'No API Key',
        description: `Please enter a ${provider} API key first.`,
        variant: 'warning'
      });
      return;
    }

    setIsTesting(provider);
    try {
      const response = await api.testApiKey(provider, settings[`${provider}ApiKey` as keyof ApiKeysSettingsData] as string);
      
      if (response.ok && response.data) {
        const isValid = response.data.valid;
        setErrors(prev => ({
          ...prev,
          [`${provider}ApiKey`]: isValid ? '' : 'Invalid API key'
        }));
        
        toast({
          title: isValid ? 'API Key Valid' : 'API Key Invalid',
          description: isValid 
            ? `${provider} API key is working correctly.`
            : `${provider} API key is invalid or expired.`,
          variant: isValid ? 'success' : 'destructive'
        });

        // Update API key data
        if (isValid) {
          setApiKeys(prev => prev.map(key => 
            key.provider === provider 
              ? { ...key, isValid: true, lastTested: new Date().toISOString() }
              : key
          ));
        }
      } else {
        throw new Error(response.error || 'Failed to test API key');
      }
    } catch (error) {
      console.error(`Failed to test ${provider} API key:`, error);
      setErrors(prev => ({
        ...prev,
        [`${provider}ApiKey`]: 'Failed to test API key'
      }));
      toast({
        title: 'Test Failed',
        description: `Failed to test ${provider} API key. Please check your connection.`,
        variant: 'destructive'
      });
    } finally {
      setIsTesting(null);
    }
  };

  const handleSettingChange = (key: keyof ApiKeysSettingsData, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    
    // Clear error when user starts typing
    if (key.includes('ApiKey') && errors[key]) {
      setErrors(prev => ({ ...prev, [key]: '' }));
    }
  };

  const handleApiKeyChange = (provider: string, value: string) => {
    handleSettingChange(`${provider}ApiKey` as keyof ApiKeysSettingsData, value);
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast({
      title: 'Copied',
      description: 'API key copied to clipboard.',
      variant: 'success'
    });
  };

  const getProviderInfo = (provider: string) => {
    return API_PROVIDERS.find(p => p.id === provider) || API_PROVIDERS[0];
  };

  const getValidationStatus = (provider: string) => {
    const key = settings[`${provider}ApiKey` as keyof ApiKeysSettingsData] as string;
    const error = errors[`${provider}ApiKey`];
    
    if (error) return 'error';
    if (key && key.length > 0) return 'success';
    return 'neutral';
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'error': return <XCircle className="h-4 w-4 text-red-500" />;
      case 'success': return <CheckCircle className="h-4 w-4 text-green-500" />;
      default: return <Info className="h-4 w-4 text-blue-500" />;
    }
  };

  const isSaveDisabled = () => {
    return Object.values(errors).some(error => error.length > 0) || 
           (!settings.curseForgeApiKey && !settings.modrinthApiKey && !settings.githubApiKey);
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* API Key Management */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Key className="h-5 w-5" />
            <span>API Key Management</span>
          </CardTitle>
          <CardDescription>
            Configure API keys for external services to enable mod browsing and installation
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            {/* Enable API Keys */}
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="enableApiKeys">Enable API Keys</Label>
                <p className="text-sm text-muted-foreground">
                  Enable API key functionality for external services
                </p>
              </div>
              <Switch
                id="enableApiKeys"
                checked={settings.enableApiKeys}
                onCheckedChange={(checked) => handleSettingChange('enableApiKeys', checked)}
              />
            </div>

            {settings.enableApiKeys && (
              <>
                {/* Auto Test Keys */}
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="autoTestKeys">Auto Test Keys</Label>
                    <p className="text-sm text-muted-foreground">
                      Automatically test API keys when they are entered
                    </p>
                  </div>
                  <Switch
                    id="autoTestKeys"
                    checked={settings.autoTestKeys}
                    onCheckedChange={(checked) => handleSettingChange('autoTestKeys', checked)}
                  />
                </div>

                {/* Security Settings */}
                <div className="flex items-center justify-between">
                  <div>
                    <Label htmlFor="encryptApiKeys">Encrypt API Keys</Label>
                    <p className="text-sm text-muted-foreground">
                      Encrypt API keys when storing them locally
                    </p>
                  </div>
                  <Switch
                    id="encryptApiKeys"
                    checked={settings.encryptApiKeys}
                    onCheckedChange={(checked) => handleSettingChange('encryptApiKeys', checked)}
                  />
                </div>
              </>
            )}
          </div>
        </CardContent>
      </Card>

      {/* API Provider Keys */}
      {settings.enableApiKeys && (
        <div className="space-y-6">
          {API_PROVIDERS.map((provider) => {
            const providerKey = `${provider.id}ApiKey` as keyof ApiKeysSettingsData;
            const providerEnabled = `${provider.id}Enabled` as keyof ApiKeysSettingsData;
            const currentKey = settings[providerKey] as string;
            const isEnabled = settings[providerEnabled] as boolean;
            const validationStatus = getValidationStatus(provider.id);
            const isTestingKey = isTesting === provider.id;
            const showKey = showKeys[provider.id];

            return (
              <Card key={provider.id}>
                <CardHeader>
                  <CardTitle className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <div className={`w-3 h-3 rounded-full ${provider.color}`} />
                      {provider.icon}
                      <span>{provider.name}</span>
                      <Badge variant={isEnabled ? 'default' : 'secondary'}>
                        {isEnabled ? 'Enabled' : 'Disabled'}
                      </Badge>
                    </div>
                    <Switch
                      checked={isEnabled}
                      onCheckedChange={(checked) => handleSettingChange(providerEnabled, checked)}
                    />
                  </CardTitle>
                  <CardDescription>
                    {provider.description} • Rate Limit: {provider.rateLimit}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {isEnabled && (
                    <div className="space-y-4">
                      <div>
                        <Label htmlFor={providerKey}>{provider.name} API Key</Label>
                        <div className="flex gap-2">
                          <div className="flex-1 relative">
                            <Input
                              id={providerKey}
                              type={showKey ? 'text' : 'password'}
                              placeholder={`Enter your ${provider.name} API key`}
                              value={currentKey}
                              onChange={(e) => handleApiKeyChange(provider.id, e.target.value)}
                              className={errors[providerKey] ? 'border-red-500' : ''}
                            />
                            <Button
                              variant="ghost"
                              size="sm"
                              className="absolute right-1 top-1 h-8 w-8 p-0"
                              onClick={() => setShowKeys(prev => ({ ...prev, [provider.id]: !showKey }))}
                            >
                              {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                            </Button>
                          </div>
                          <Button
                            variant="outline"
                            onClick={() => testApiKey(provider.id)}
                            disabled={!currentKey.trim() || isTestingKey}
                          >
                            {isTestingKey ? (
                              <RefreshCw className="h-4 w-4 animate-spin" />
                            ) : (
                              <CheckCircle className="h-4 w-4" />
                            )}
                            {isTestingKey ? 'Testing...' : 'Test'}
                          </Button>
                          {currentKey && (
                            <Button
                              variant="outline"
                              onClick={() => copyToClipboard(currentKey)}
                            >
                              <Copy className="h-4 w-4" />
                            </Button>
                          )}
                        </div>
                        {errors[providerKey] && (
                          <p className="text-sm text-red-500 mt-1">{errors[providerKey]}</p>
                        )}
                        <div className="flex items-center justify-between mt-2">
                          <div className="flex items-center space-x-2">
                            {getStatusIcon(validationStatus)}
                            <span className="text-sm text-muted-foreground">
                              {validationStatus === 'success' ? 'Valid' : 
                               validationStatus === 'error' ? 'Invalid' : 'Not tested'}
                            </span>
                          </div>
                          <a
                            href={provider.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-sm text-blue-500 hover:underline flex items-center space-x-1"
                          >
                            <span>Get API Key</span>
                            <ExternalLink className="h-3 w-3" />
                          </a>
                        </div>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Security Notice */}
      <Alert>
        <Shield className="h-4 w-4" />
        <AlertDescription>
          <strong>Security Notice:</strong> API keys are stored locally and encrypted when possible. 
          Never share your API keys with others. If you suspect a key has been compromised, 
          revoke it immediately from the provider's console and generate a new one.
        </AlertDescription>
      </Alert>

      {/* Save Button */}
      <div className="flex justify-end">
        <Button
          onClick={saveSettings}
          disabled={isSaveDisabled() || isLoading}
          className="min-w-32"
        >
          {isLoading ? (
            <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <CheckCircle className="h-4 w-4 mr-2" />
          )}
          {isLoading ? 'Saving...' : 'Save Settings'}
        </Button>
      </div>
    </div>
  );
};
