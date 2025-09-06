import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { 
  Key, 
  ExternalLink, 
  CheckCircle, 
  AlertCircle, 
  Info,
  Eye,
  EyeOff,
  Copy,
  RefreshCw
} from 'lucide-react';
import { useThemeStore } from '@/store/theme';

interface APISettings {
  curseforgeApiKey: string;
  modrinthApiKey: string;
}

export const APISettings: React.FC = () => {
  const { colorScheme } = useThemeStore();
  const [settings, setSettings] = useState<APISettings>({
    curseforgeApiKey: '',
    modrinthApiKey: '',
  });
  const [showCurseForgeKey, setShowCurseForgeKey] = useState(false);
  const [showModrinthKey, setShowModrinthKey] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState<{
    curseforge: 'idle' | 'testing' | 'success' | 'error';
    modrinth: 'idle' | 'testing' | 'success' | 'error';
  }>({
    curseforge: 'idle',
    modrinth: 'idle',
  });
  const [message, setMessage] = useState('');

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = () => {
    // Load from localStorage or environment
    const curseforgeKey = localStorage.getItem('curseforge_api_key') || '';
    const modrinthKey = localStorage.getItem('modrinth_api_key') || '';
    
    setSettings({
      curseforgeApiKey: curseforgeKey,
      modrinthApiKey: modrinthKey,
    });
  };

  const saveSettings = () => {
    setIsLoading(true);
    try {
      localStorage.setItem('curseforge_api_key', settings.curseforgeApiKey);
      localStorage.setItem('modrinth_api_key', settings.modrinthApiKey);
      
      // Update environment variables for the current session
      if (settings.curseforgeApiKey) {
        (window as any).env = (window as any).env || {};
        (window as any).env.VITE_CURSEFORGE_API_KEY = settings.curseforgeApiKey;
      }
      if (settings.modrinthApiKey) {
        (window as any).env = (window as any).env || {};
        (window as any).env.VITE_MODRINTH_API_KEY = settings.modrinthApiKey;
      }
      
      setMessage('API keys saved successfully!');
      setTimeout(() => setMessage(''), 3000);
    } catch (error) {
      setMessage('Failed to save API keys');
      setTimeout(() => setMessage(''), 3000);
    } finally {
      setIsLoading(false);
    }
  };

  const testAPI = async (api: 'curseforge' | 'modrinth') => {
    setStatus(prev => ({ ...prev, [api]: 'testing' }));
    
    try {
      const apiKey = api === 'curseforge' ? settings.curseforgeApiKey : settings.modrinthApiKey;
      
      if (!apiKey) {
        setStatus(prev => ({ ...prev, [api]: 'error' }));
        setMessage(`${api} API key is required`);
        return;
      }

      // Test the API key by making a simple request
      const testUrl = api === 'curseforge' 
        ? 'https://api.curseforge.com/v1/games/432/versions'
        : 'https://api.modrinth.com/v2/tag/game_version';
      
      const headers: Record<string, string> = {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      };
      
      if (api === 'curseforge') {
        headers['x-api-key'] = apiKey;
      } else {
        headers['Authorization'] = apiKey;
      }

      const response = await fetch(testUrl, { headers });
      
      if (response.ok) {
        setStatus(prev => ({ ...prev, [api]: 'success' }));
        setMessage(`${api} API key is valid!`);
      } else {
        setStatus(prev => ({ ...prev, [api]: 'error' }));
        setMessage(`${api} API key is invalid or expired`);
      }
    } catch (error) {
      setStatus(prev => ({ ...prev, [api]: 'error' }));
      setMessage(`Failed to test ${api} API key`);
    }
    
    setTimeout(() => setMessage(''), 5000);
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setMessage('Copied to clipboard!');
    setTimeout(() => setMessage(''), 2000);
  };

  const getStatusIcon = (api: 'curseforge' | 'modrinth') => {
    const statusValue = status[api];
    switch (statusValue) {
      case 'testing':
        return <RefreshCw className="w-4 h-4 animate-spin text-blue-500" />;
      case 'success':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      default:
        return <Key className="w-4 h-4 text-gray-400" />;
    }
  };

  const getStatusBadge = (api: 'curseforge' | 'modrinth') => {
    const statusValue = status[api];
    const key = settings[`${api}ApiKey` as keyof APISettings];
    
    if (!key) {
      return <Badge variant="secondary">Not Set</Badge>;
    }
    
    switch (statusValue) {
      case 'testing':
        return <Badge variant="outline">Testing...</Badge>;
      case 'success':
        return <Badge variant="default" className="bg-green-500">Valid</Badge>;
      case 'error':
        return <Badge variant="destructive">Invalid</Badge>;
      default:
        return <Badge variant="outline">Set</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">API Settings</h2>
        <p className="text-muted-foreground">
          Configure API keys for CurseForge and Modrinth to access real mod data
        </p>
      </div>

      {message && (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>{message}</AlertDescription>
        </Alert>
      )}

      <div className="grid gap-6 md:grid-cols-2">
        {/* CurseForge API */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <img 
                src="https://www.curseforge.com/favicon.ico" 
                alt="CurseForge" 
                className="w-5 h-5"
              />
              CurseForge API
              {getStatusIcon('curseforge')}
            </CardTitle>
            <CardDescription>
              Required for accessing CurseForge mod data
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <Label htmlFor="curseforge-key">API Key</Label>
              {getStatusBadge('curseforge')}
            </div>
            
            <div className="relative">
              <Input
                id="curseforge-key"
                type={showCurseForgeKey ? 'text' : 'password'}
                value={settings.curseforgeApiKey}
                onChange={(e) => setSettings(prev => ({ ...prev, curseforgeApiKey: e.target.value }))}
                placeholder="Enter your CurseForge API key"
                className="pr-20"
              />
              <div className="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowCurseForgeKey(!showCurseForgeKey)}
                >
                  {showCurseForgeKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => copyToClipboard(settings.curseforgeApiKey)}
                  disabled={!settings.curseforgeApiKey}
                >
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <Button
                onClick={() => testAPI('curseforge')}
                disabled={!settings.curseforgeApiKey || status.curseforge === 'testing'}
                className="w-full"
              >
                {status.curseforge === 'testing' ? (
                  <>
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    Testing...
                  </>
                ) : (
                  'Test API Key'
                )}
              </Button>
              
              <Button
                variant="outline"
                onClick={() => window.open('https://docs.curseforge.com/#authentication', '_blank')}
                className="w-full"
              >
                <ExternalLink className="w-4 h-4 mr-2" />
                Get API Key
              </Button>
            </div>

            <div className="text-sm text-muted-foreground">
              <p>• Rate limit: 100 requests/minute</p>
              <p>• Required for mod search and details</p>
              <p>• Free to obtain</p>
            </div>
          </CardContent>
        </Card>

        {/* Modrinth API */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <img 
                src="https://modrinth.com/favicon.ico" 
                alt="Modrinth" 
                className="w-5 h-5"
              />
              Modrinth API
              {getStatusIcon('modrinth')}
            </CardTitle>
            <CardDescription>
              Optional but recommended for higher rate limits
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <Label htmlFor="modrinth-key">API Key</Label>
              {getStatusBadge('modrinth')}
            </div>
            
            <div className="relative">
              <Input
                id="modrinth-key"
                type={showModrinthKey ? 'text' : 'password'}
                value={settings.modrinthApiKey}
                onChange={(e) => setSettings(prev => ({ ...prev, modrinthApiKey: e.target.value }))}
                placeholder="Enter your Modrinth API key (optional)"
                className="pr-20"
              />
              <div className="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowModrinthKey(!showModrinthKey)}
                >
                  {showModrinthKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => copyToClipboard(settings.modrinthApiKey)}
                  disabled={!settings.modrinthApiKey}
                >
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <Button
                onClick={() => testAPI('modrinth')}
                disabled={!settings.modrinthApiKey || status.modrinth === 'testing'}
                className="w-full"
              >
                {status.modrinth === 'testing' ? (
                  <>
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    Testing...
                  </>
                ) : (
                  'Test API Key'
                )}
              </Button>
              
              <Button
                variant="outline"
                onClick={() => window.open('https://modrinth.com/api', '_blank')}
                className="w-full"
              >
                <ExternalLink className="w-4 h-4 mr-2" />
                Get API Key
              </Button>
            </div>

            <div className="text-sm text-muted-foreground">
              <p>• Rate limit: 300 requests/minute (with key)</p>
              <p>• 60 requests/minute (without key)</p>
              <p>• Optional but recommended</p>
            </div>
          </CardContent>
        </Card>
      </div>

      <Separator />

      <Card>
        <CardHeader>
          <CardTitle>API Usage Information</CardTitle>
          <CardDescription>
            How your API keys are used and stored
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
          onClick={saveSettings}
          disabled={isLoading}
          className="min-w-32"
        >
          {isLoading ? (
            <>
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              Saving...
            </>
          ) : (
            'Save Settings'
          )}
        </Button>
      </div>
    </div>
  );
};
