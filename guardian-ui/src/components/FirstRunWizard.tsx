import React, { useState, useEffect } from 'react';
import { 
  CheckCircle, 
  AlertTriangle, 
  Key, 
  FolderOpen, 
  Settings, 
  Play, 
  ArrowRight, 
  ArrowLeft,
  Download,
  Server,
  Database,
  Palette,
  Shield,
  Zap
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { apiClient as api } from '@/lib/api';
import { settingsManager } from '@/lib/settings-manager';

interface FirstRunWizardProps {
  isOpen: boolean;
  onComplete: () => void;
  onSkip: () => void;
}

interface WizardStep {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  required: boolean;
}

interface WizardSettings {
  curseForgeApiKey: string;
  modrinthApiKey: string;
  javaPath: string;
  javaVersion: string;
  serversDirectory: string;
  backupsDirectory: string;
  enableGpuAcceleration: boolean;
  gpuProvider: string;
  theme: 'dark' | 'light' | 'system';
  language: string;
  autoStart: boolean;
  notifications: boolean;
}

const wizardSteps: WizardStep[] = [
  {
    id: 'welcome',
    title: 'Welcome to Guardian',
    description: 'Let\'s set up your Minecraft server manager',
    icon: <Server className="h-6 w-6" />,
    required: false
  },
  {
    id: 'api-keys',
    title: 'API Keys',
    description: 'Configure CurseForge and Modrinth API keys',
    icon: <Key className="h-6 w-6" />,
    required: true
  },
  {
    id: 'java-path',
    title: 'Java Installation',
    description: 'Set up Java path and version',
    icon: <Zap className="h-6 w-6" />,
    required: true
  },
  {
    id: 'directories',
    title: 'Directories',
    description: 'Configure default server and backup directories',
    icon: <FolderOpen className="h-6 w-6" />,
    required: true
  },
  {
    id: 'gpu-settings',
    title: 'GPU Acceleration',
    description: 'Configure GPU acceleration settings',
    icon: <Shield className="h-6 w-6" />,
    required: false
  },
  {
    id: 'theme',
    title: 'Theme & Appearance',
    description: 'Choose your preferred theme and settings',
    icon: <Palette className="h-6 w-6" />,
    required: false
  },
  {
    id: 'complete',
    title: 'Setup Complete',
    description: 'You\'re ready to start managing servers!',
    icon: <CheckCircle className="h-6 w-6" />,
    required: false
  }
];

export const FirstRunWizard: React.FC<FirstRunWizardProps> = ({
  isOpen,
  onComplete,
  onSkip
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [settings, setSettings] = useState<WizardSettings>({
    curseForgeApiKey: '',
    modrinthApiKey: '',
    javaPath: '',
    javaVersion: '17',
    serversDirectory: '',
    backupsDirectory: '',
    enableGpuAcceleration: false,
    gpuProvider: 'auto',
    theme: 'dark',
    language: 'en',
    autoStart: false,
    notifications: true
  });
  const [isLoading, setIsLoading] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const { updateAppSettings, getAppSettings } = settingsManager;

  useEffect(() => {
    if (isOpen) {
      loadCurrentSettings();
    }
  }, [isOpen]);

  const loadCurrentSettings = async () => {
    try {
      const currentSettings = getAppSettings();
      setSettings(prev => ({ 
        ...prev, 
        theme: currentSettings.theme,
        language: currentSettings.language,
        autoStart: currentSettings.autoStart,
        notifications: currentSettings.notifications.enabled
      }));
    } catch (error) {
      console.error('Failed to load current settings:', error);
    }
  };

  const validateStep = (stepId: string): boolean => {
    const newErrors: Record<string, string> = {};

    switch (stepId) {
      case 'api-keys':
        if (!settings.curseForgeApiKey.trim()) {
          newErrors.curseForgeApiKey = 'CurseForge API key is required';
        }
        if (!settings.modrinthApiKey.trim()) {
          newErrors.modrinthApiKey = 'Modrinth API key is required';
        }
        break;
      case 'java-path':
        if (!settings.javaPath.trim()) {
          newErrors.javaPath = 'Java path is required';
        }
        break;
      case 'directories':
        if (!settings.serversDirectory.trim()) {
          newErrors.serversDirectory = 'Servers directory is required';
        }
        if (!settings.backupsDirectory.trim()) {
          newErrors.backupsDirectory = 'Backups directory is required';
        }
        break;
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleNext = async () => {
    const currentStepData = wizardSteps[currentStep];
    
    if (currentStepData.required && !validateStep(currentStepData.id)) {
      return;
    }

    if (currentStep === wizardSteps.length - 1) {
      // Complete setup
      await handleComplete();
    } else {
      setCurrentStep(prev => prev + 1);
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(prev => prev - 1);
    }
  };

  const handleComplete = async () => {
    setIsLoading(true);
    try {
      await updateAppSettings({
        theme: settings.theme,
        language: settings.language,
        autoStart: settings.autoStart,
        notifications: {
          enabled: settings.notifications,
          serverStatus: true,
          errors: true,
          updates: true
        }
      });
      onComplete();
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleBrowseDirectory = async (field: 'serversDirectory' | 'backupsDirectory') => {
    try {
      const selected = await openDialog({
        directory: true,
        title: `Select ${field === 'serversDirectory' ? 'Servers' : 'Backups'} Directory`
      });
      
      if (selected) {
        setSettings(prev => ({ ...prev, [field]: selected as string }));
      }
    } catch (error) {
      console.error('Failed to browse directory:', error);
    }
  };

  const handleBrowseJava = async () => {
    try {
      const selected = await openDialog({
        filters: [
          { name: 'Java Executable', extensions: ['exe'] },
          { name: 'All Files', extensions: ['*'] }
        ],
        title: 'Select Java Executable'
      });
      
      if (selected) {
        setSettings(prev => ({ ...prev, javaPath: selected as string }));
      }
    } catch (error) {
      console.error('Failed to browse Java executable:', error);
    }
  };

  const testApiKey = async (provider: 'curseforge' | 'modrinth') => {
    const apiKey = provider === 'curseforge' ? settings.curseForgeApiKey : settings.modrinthApiKey;
    if (!apiKey.trim()) return;

    try {
      // Test API key by making a simple request
      const response = await fetch(`/api/test-${provider}-key`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ apiKey })
      });
      
      if (response.ok) {
        // Show success feedback
        console.log(`${provider} API key is valid`);
      } else {
        throw new Error('Invalid API key');
      }
    } catch (error) {
      console.error(`Failed to test ${provider} API key:`, error);
    }
  };

  const renderStepContent = () => {
    const step = wizardSteps[currentStep];
    
    switch (step.id) {
      case 'welcome':
        return (
          <div className="text-center space-y-6">
            <div className="mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center">
              <Server className="h-8 w-8 text-blue-600" />
            </div>
            <div>
              <h3 className="text-2xl font-bold mb-2">Welcome to Guardian Server Manager</h3>
              <p className="text-muted-foreground">
                Your all-in-one solution for managing Minecraft servers with GPU acceleration,
                mod management, and advanced monitoring capabilities.
              </p>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-left">
              <div className="flex items-start gap-3">
                <Zap className="h-5 w-5 text-yellow-500 mt-1" />
                <div>
                  <h4 className="font-medium">GPU Acceleration</h4>
                  <p className="text-sm text-muted-foreground">5x faster world generation</p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <Database className="h-5 w-5 text-green-500 mt-1" />
                <div>
                  <h4 className="font-medium">Mod Management</h4>
                  <p className="text-sm text-muted-foreground">CurseForge & Modrinth integration</p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <Shield className="h-5 w-5 text-purple-500 mt-1" />
                <div>
                  <h4 className="font-medium">Advanced Monitoring</h4>
                  <p className="text-sm text-muted-foreground">Real-time performance analytics</p>
                </div>
              </div>
            </div>
          </div>
        );

      case 'api-keys':
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-xl font-semibold mb-2">API Keys Configuration</h3>
              <p className="text-muted-foreground">
                Configure API keys to enable mod browsing and installation from CurseForge and Modrinth.
              </p>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="curseForgeApiKey">CurseForge API Key</Label>
                <div className="flex gap-2">
                  <Input
                    id="curseForgeApiKey"
                    type="password"
                    placeholder="Enter your CurseForge API key"
                    value={settings.curseForgeApiKey}
                    onChange={(e) => setSettings(prev => ({ ...prev, curseForgeApiKey: e.target.value }))}
                    className={errors.curseForgeApiKey ? 'border-red-500' : ''}
                  />
                  <Button
                    variant="outline"
                    onClick={() => testApiKey('curseforge')}
                    disabled={!settings.curseForgeApiKey.trim()}
                  >
                    Test
                  </Button>
                </div>
                {errors.curseForgeApiKey && (
                  <p className="text-sm text-red-500 mt-1">{errors.curseForgeApiKey}</p>
                )}
                <p className="text-xs text-muted-foreground mt-1">
                  Get your API key from{' '}
                  <a href="https://console.curseforge.com/" target="_blank" rel="noopener noreferrer" className="text-blue-500 hover:underline">
                    CurseForge Console
                  </a>
                </p>
              </div>

              <div>
                <Label htmlFor="modrinthApiKey">Modrinth API Key</Label>
                <div className="flex gap-2">
                  <Input
                    id="modrinthApiKey"
                    type="password"
                    placeholder="Enter your Modrinth API key"
                    value={settings.modrinthApiKey}
                    onChange={(e) => setSettings(prev => ({ ...prev, modrinthApiKey: e.target.value }))}
                    className={errors.modrinthApiKey ? 'border-red-500' : ''}
                  />
                  <Button
                    variant="outline"
                    onClick={() => testApiKey('modrinth')}
                    disabled={!settings.modrinthApiKey.trim()}
                  >
                    Test
                  </Button>
                </div>
                {errors.modrinthApiKey && (
                  <p className="text-sm text-red-500 mt-1">{errors.modrinthApiKey}</p>
                )}
                <p className="text-xs text-muted-foreground mt-1">
                  Get your API key from{' '}
                  <a href="https://modrinth.com/settings/api" target="_blank" rel="noopener noreferrer" className="text-blue-500 hover:underline">
                    Modrinth Settings
                  </a>
                </p>
              </div>
            </div>
          </div>
        );

      case 'java-path':
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-xl font-semibold mb-2">Java Installation</h3>
              <p className="text-muted-foreground">
                Configure the Java installation path and version for running Minecraft servers.
              </p>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="javaPath">Java Executable Path</Label>
                <div className="flex gap-2">
                  <Input
                    id="javaPath"
                    placeholder="C:\Program Files\Java\jdk-17\bin\java.exe"
                    value={settings.javaPath}
                    onChange={(e) => setSettings(prev => ({ ...prev, javaPath: e.target.value }))}
                    className={errors.javaPath ? 'border-red-500' : ''}
                  />
                  <Button variant="outline" onClick={handleBrowseJava}>
                    Browse
                  </Button>
                </div>
                {errors.javaPath && (
                  <p className="text-sm text-red-500 mt-1">{errors.javaPath}</p>
                )}
              </div>

              <div>
                <Label htmlFor="javaVersion">Java Version</Label>
                <Select value={settings.javaVersion} onValueChange={(value) => setSettings(prev => ({ ...prev, javaVersion: value }))}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="8">Java 8</SelectItem>
                    <SelectItem value="11">Java 11</SelectItem>
                    <SelectItem value="17">Java 17 (Recommended)</SelectItem>
                    <SelectItem value="21">Java 21</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground mt-1">
                  Java 17 is recommended for most Minecraft versions
                </p>
              </div>
            </div>
          </div>
        );

      case 'directories':
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-xl font-semibold mb-2">Directory Configuration</h3>
              <p className="text-muted-foreground">
                Set up default directories for servers and backups.
              </p>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="serversDirectory">Servers Directory</Label>
                <div className="flex gap-2">
                  <Input
                    id="serversDirectory"
                    placeholder="C:\Minecraft\Servers"
                    value={settings.serversDirectory}
                    onChange={(e) => setSettings(prev => ({ ...prev, serversDirectory: e.target.value }))}
                    className={errors.serversDirectory ? 'border-red-500' : ''}
                  />
                  <Button variant="outline" onClick={() => handleBrowseDirectory('serversDirectory')}>
                    Browse
                  </Button>
                </div>
                {errors.serversDirectory && (
                  <p className="text-sm text-red-500 mt-1">{errors.serversDirectory}</p>
                )}
                <p className="text-xs text-muted-foreground mt-1">
                  Directory where all server instances will be stored
                </p>
              </div>

              <div>
                <Label htmlFor="backupsDirectory">Backups Directory</Label>
                <div className="flex gap-2">
                  <Input
                    id="backupsDirectory"
                    placeholder="C:\Minecraft\Backups"
                    value={settings.backupsDirectory}
                    onChange={(e) => setSettings(prev => ({ ...prev, backupsDirectory: e.target.value }))}
                    className={errors.backupsDirectory ? 'border-red-500' : ''}
                  />
                  <Button variant="outline" onClick={() => handleBrowseDirectory('backupsDirectory')}>
                    Browse
                  </Button>
                </div>
                {errors.backupsDirectory && (
                  <p className="text-sm text-red-500 mt-1">{errors.backupsDirectory}</p>
                )}
                <p className="text-xs text-muted-foreground mt-1">
                  Directory where server backups will be stored
                </p>
              </div>
            </div>
          </div>
        );

      case 'gpu-settings':
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-xl font-semibold mb-2">GPU Acceleration</h3>
              <p className="text-muted-foreground">
                Configure GPU acceleration for faster world generation and processing.
              </p>
            </div>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="enableGpuAcceleration">Enable GPU Acceleration</Label>
                  <p className="text-sm text-muted-foreground">
                    Use GPU for world generation and chunk processing
                  </p>
                </div>
                <Switch
                  id="enableGpuAcceleration"
                  checked={settings.enableGpuAcceleration}
                  onCheckedChange={(checked) => setSettings(prev => ({ ...prev, enableGpuAcceleration: checked }))}
                />
              </div>

              {settings.enableGpuAcceleration && (
                <div>
                  <Label htmlFor="gpuProvider">GPU Provider</Label>
                  <Select value={settings.gpuProvider} onValueChange={(value) => setSettings(prev => ({ ...prev, gpuProvider: value }))}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="auto">Auto-detect</SelectItem>
                      <SelectItem value="cuda">CUDA (NVIDIA)</SelectItem>
                      <SelectItem value="webgpu">WebGPU (Cross-platform)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground mt-1">
                    WebGPU provides better cross-platform compatibility
                  </p>
                </div>
              )}
            </div>
          </div>
        );

      case 'theme':
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-xl font-semibold mb-2">Theme & Appearance</h3>
              <p className="text-muted-foreground">
                Customize the appearance and behavior of Guardian.
              </p>
            </div>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="theme">Theme</Label>
                <Select value={settings.theme} onValueChange={(value) => setSettings(prev => ({ ...prev, theme: value as 'dark' | 'light' | 'system' }))}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="dark">Dark</SelectItem>
                    <SelectItem value="light">Light</SelectItem>
                    <SelectItem value="system">System</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="language">Language</Label>
                <Select value={settings.language} onValueChange={(value) => setSettings(prev => ({ ...prev, language: value }))}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="en">English</SelectItem>
                    <SelectItem value="es">Español</SelectItem>
                    <SelectItem value="fr">Français</SelectItem>
                    <SelectItem value="de">Deutsch</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="autoStart">Auto-start with system</Label>
                  <p className="text-sm text-muted-foreground">
                    Start Guardian automatically when you log in
                  </p>
                </div>
                <Switch
                  id="autoStart"
                  checked={settings.autoStart}
                  onCheckedChange={(checked) => setSettings(prev => ({ ...prev, autoStart: checked }))}
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <Label htmlFor="notifications">Enable notifications</Label>
                  <p className="text-sm text-muted-foreground">
                    Show desktop notifications for server events
                  </p>
                </div>
                <Switch
                  id="notifications"
                  checked={settings.notifications}
                  onCheckedChange={(checked) => setSettings(prev => ({ ...prev, notifications: checked }))}
                />
              </div>
            </div>
          </div>
        );

      case 'complete':
        return (
          <div className="text-center space-y-6">
            <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center">
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
            <div>
              <h3 className="text-2xl font-bold mb-2">Setup Complete!</h3>
              <p className="text-muted-foreground">
                Guardian is now configured and ready to use. You can start creating and managing your Minecraft servers.
              </p>
            </div>
            <div className="bg-muted p-4 rounded-lg text-left">
              <h4 className="font-medium mb-2">What's next?</h4>
              <ul className="text-sm text-muted-foreground space-y-1">
                <li>• Create your first server using the + button</li>
                <li>• Browse and install mods from CurseForge and Modrinth</li>
                <li>• Configure GPU acceleration for better performance</li>
                <li>• Set up automated backups and monitoring</li>
              </ul>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  if (!isOpen) return null;

  const currentStepData = wizardSteps[currentStep];
  const progress = ((currentStep + 1) / wizardSteps.length) * 100;

  return (
    <Dialog open={isOpen} onOpenChange={() => {}}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-3">
            {currentStepData.icon}
            {currentStepData.title}
            {currentStepData.required && (
              <Badge variant="destructive" className="text-xs">Required</Badge>
            )}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-6">
          {/* Progress Bar */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm text-muted-foreground">
              <span>Step {currentStep + 1} of {wizardSteps.length}</span>
              <span>{Math.round(progress)}% complete</span>
            </div>
            <Progress value={progress} className="h-2" />
          </div>

          {/* Step Content */}
          <div className="min-h-[400px]">
            {renderStepContent()}
          </div>

          {/* Navigation */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="flex gap-2">
              {currentStep > 0 && (
                <Button variant="outline" onClick={handlePrevious}>
                  <ArrowLeft className="h-4 w-4 mr-2" />
                  Previous
                </Button>
              )}
              <Button variant="ghost" onClick={onSkip}>
                Skip Setup
              </Button>
            </div>
            
            <div className="flex gap-2">
              {currentStep === wizardSteps.length - 1 ? (
                <Button onClick={handleComplete} disabled={isLoading}>
                  {isLoading ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                      Completing...
                    </>
                  ) : (
                    <>
                      <CheckCircle className="h-4 w-4 mr-2" />
                      Complete Setup
                    </>
                  )}
                </Button>
              ) : (
                <Button onClick={handleNext}>
                  Next
                  <ArrowRight className="h-4 w-4 ml-2" />
                </Button>
              )}
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default FirstRunWizard;
