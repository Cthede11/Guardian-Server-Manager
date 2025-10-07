import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { 
  Server, 
  FolderOpen, 
  Zap, 
  MemoryStick, 
  AlertTriangle,
  CheckCircle,
  Loader2
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { apiClient as api } from '@/lib/api';
import { useToast } from '@/hooks/use-toast';

interface ServerFormData {
  name: string;
  edition: 'Vanilla' | 'Fabric' | 'Forge';
  version: string;
  installPath: string;
  memory: { min: number; max: number };
  maxPlayers: number;
  port: number;
  motd: string;
  difficulty: 'easy' | 'normal' | 'hard' | 'peaceful';
  gamemode: 'survival' | 'creative' | 'adventure' | 'spectator';
  pvp: boolean;
  allowFlight: boolean;
  allowNether: boolean;
  allowEnd: boolean;
  spawnProtection: number;
  viewDistance: number;
  simulationDistance: number;
  hardcore: boolean;
  onlineMode: boolean;
  whiteList: boolean;
  enableCommandBlock: boolean;
  spawnAnimals: boolean;
  spawnMonsters: boolean;
  spawnNpcs: boolean;
  generateStructures: boolean;
  allowCheats: boolean;
  levelType: string;
  levelSeed: string;
  generatorSettings: string;
  levelName: string;
  javaPath?: string;
  serverProperties: Record<string, string>;
}

interface StepBasicsProps {
  formData: ServerFormData;
  updateFormData: (updates: Partial<ServerFormData>) => void;
  errors: Record<string, string>;
  versions: string[];
  isLoadingVersions: boolean;
  onValidate: () => boolean;
}

const memoryPresets = [
  { label: '2GB', value: { min: 1, max: 2 } },
  { label: '4GB', value: { min: 2, max: 4 } },
  { label: '6GB', value: { min: 3, max: 6 } },
  { label: '8GB', value: { min: 4, max: 8 } },
  { label: '12GB', value: { min: 6, max: 12 } },
  { label: '16GB', value: { min: 8, max: 16 } }
];

export const StepBasics: React.FC<StepBasicsProps> = ({
  formData,
  updateFormData,
  errors,
  versions,
  isLoadingVersions,
  onValidate
}) => {
  const [isValidating, setIsValidating] = useState(false);
  const [validationResults, setValidationResults] = useState<Record<string, boolean>>({});
  const { toast } = useToast();

  // Auto-detect Java path on mount and set default install path
  useEffect(() => {
    if (!formData.javaPath) {
      detectJavaPath();
    }
    if (!formData.installPath) {
      // Set a default path based on common Minecraft server locations
      const defaultPath = 'C:\\Minecraft\\Servers\\MyServer';
      updateFormData({ installPath: defaultPath });
    }
  }, []);

  const detectJavaPath = async () => {
    try {
      setIsValidating(true);
      const response = await api.call<{success: boolean, data: {java_path?: string, version?: string}, error: string}>('/api/server/detect-java');
      console.log('Java detection response:', response);
      
      // Handle API response structure
      let javaPath = null;
      let version = null;
      
      if (response.success && response.data) {
        javaPath = response.data.java_path;
        version = response.data.version;
      }
      
      if (javaPath) {
        updateFormData({ javaPath });
        setValidationResults(prev => ({ ...prev, javaPath: true }));
        toast({
          title: "Java Detected",
          description: `Found Java ${version || 'unknown version'} at ${javaPath}`,
          variant: "default"
        });
      } else {
        toast({
          title: "Java Not Found",
          description: "Could not automatically detect Java. Please specify the path manually.",
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error('Failed to detect Java:', error);
      toast({
        title: "Java Detection Failed",
        description: "Failed to detect Java installation. Please specify the path manually.",
        variant: "destructive"
      });
    } finally {
      setIsValidating(false);
    }
  };

  const validateField = async (field: string, value: string) => {
    if (!value.trim()) return;

    setIsValidating(true);
    try {
      const response = await api.call<{valid: boolean, error?: string}>('/api/server/validate', {
        method: 'POST',
        body: JSON.stringify({
          [field]: value,
          ...(field === 'name' && { serverId: null }) // For uniqueness check
        })
      });

      setValidationResults(prev => ({ 
        ...prev, 
        [field]: response.valid || false 
      }));

      if (!response.valid && response.error) {
        toast({
          title: "Validation Error",
          description: response.error,
          variant: "destructive"
        });
      }
    } catch (error) {
      console.error(`Failed to validate ${field}:`, error);
      setValidationResults(prev => ({ ...prev, [field]: false }));
    } finally {
      setIsValidating(false);
    }
  };

  const handleBrowseDirectory = async () => {
    try {
      console.log('Opening directory dialog...');
      const selected = await openDialog({
        directory: true,
        title: 'Select Server Installation Directory'
      });
      
      console.log('Dialog result:', selected);
      
      if (selected) {
        updateFormData({ installPath: selected as string });
        validateField('installPath', selected as string);
        toast({
          title: "Directory Selected",
          description: `Selected: ${selected}`,
          variant: "default"
        });
      }
    } catch (error) {
      console.error('Failed to open directory dialog:', error);
      
      // Fallback: Show a prompt for manual input
      const manualPath = prompt(
        'Please enter the full path to your server installation directory:\n\n' +
        'Example: C:\\Minecraft\\Servers\\MyServer\n\n' +
        'Note: The directory dialog is not available. Please enter the path manually.'
      );
      
      if (manualPath && manualPath.trim()) {
        updateFormData({ installPath: manualPath.trim() });
        validateField('installPath', manualPath.trim());
        toast({
          title: "Directory Set",
          description: `Set to: ${manualPath.trim()}`,
          variant: "default"
        });
      } else {
        toast({
          title: "Dialog Error",
          description: "Directory dialog is not available. Please enter the path manually in the input field.",
          variant: "destructive"
        });
      }
    }
  };

  const handleMemoryPreset = (preset: typeof memoryPresets[0]) => {
    updateFormData({ memory: preset.value });
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-semibold mb-2">Server Configuration</h3>
        <p className="text-muted-foreground">
          Configure the basic settings for your Minecraft server
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Server Name */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Server className="h-5 w-5" />
              Server Name
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="serverName">Server Name *</Label>
              <div className="flex gap-2">
                <Input
                  id="serverName"
                  placeholder="My Minecraft Server"
                  value={formData.name}
                  onChange={(e) => updateFormData({ name: e.target.value })}
                  onBlur={() => validateField('name', formData.name)}
                  className={errors.name ? 'border-red-500' : ''}
                />
                {validationResults.name && (
                  <CheckCircle className="h-5 w-5 text-green-500 mt-2" />
                )}
              </div>
              {errors.name && (
                <p className="text-sm text-red-500">{errors.name}</p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Edition & Version */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5" />
              Edition & Version
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="edition">Edition *</Label>
              <Select
                value={formData.edition}
                onValueChange={(value) => updateFormData({ 
                  edition: value as 'Vanilla' | 'Fabric' | 'Forge',
                  version: '' // Reset version when edition changes
                })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Vanilla">Vanilla</SelectItem>
                  <SelectItem value="Fabric">Fabric</SelectItem>
                  <SelectItem value="Forge">Forge</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="version">Version *</Label>
              <Select
                value={formData.version}
                onValueChange={(value) => updateFormData({ version: value })}
                disabled={isLoadingVersions || versions.length === 0}
              >
                <SelectTrigger>
                  <SelectValue placeholder={isLoadingVersions ? "Loading versions..." : "Select version"} />
                </SelectTrigger>
                <SelectContent>
                  {versions.map((version) => (
                    <SelectItem key={version} value={version}>
                      {version}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {isLoadingVersions && (
                <div className="flex items-center gap-2 mt-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span className="text-sm text-muted-foreground">Loading versions...</span>
                </div>
              )}
              {errors.version && (
                <p className="text-sm text-red-500">{errors.version}</p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Installation Path */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <FolderOpen className="h-5 w-5" />
              Installation Path
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="installPath">Server Directory *</Label>
              <div className="flex gap-2">
                <Input
                  id="installPath"
                  placeholder="C:\Minecraft\Servers\MyServer"
                  value={formData.installPath}
                  onChange={(e) => updateFormData({ installPath: e.target.value })}
                  onBlur={() => validateField('installPath', formData.installPath)}
                  className={errors.installPath ? 'border-red-500' : ''}
                />
                <Button variant="outline" onClick={handleBrowseDirectory} className="bg-primary/10 hover:bg-primary/20 border-primary/30 hover:border-primary/50">
                  Browse
                </Button>
                {validationResults.installPath && (
                  <CheckCircle className="h-5 w-5 text-green-500 mt-2" />
                )}
              </div>
              {errors.installPath && (
                <p className="text-sm text-red-500">{errors.installPath}</p>
              )}
              <p className="text-xs text-muted-foreground mt-1">
                Directory where server files will be installed
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Java Path */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5" />
              Java Configuration
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="javaPath">Java Path *</Label>
              <div className="flex gap-2">
                <Input
                  id="javaPath"
                  placeholder="C:\Program Files\Java\jdk-17\bin\java.exe"
                  value={formData.javaPath || ''}
                  onChange={(e) => updateFormData({ javaPath: e.target.value })}
                  onBlur={() => validateField('javaPath', formData.javaPath || '')}
                  className={errors.javaPath ? 'border-red-500' : ''}
                />
                <Button 
                  variant="outline" 
                  onClick={detectJavaPath}
                  disabled={isValidating}
                  className="bg-primary/10 hover:bg-primary/20 border-primary/30 hover:border-primary/50 disabled:opacity-50"
                >
                  {isValidating ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    'Detect'
                  )}
                </Button>
                {validationResults.javaPath && (
                  <CheckCircle className="h-5 w-5 text-green-500 mt-2" />
                )}
              </div>
              {errors.javaPath && (
                <p className="text-sm text-red-500">{errors.javaPath}</p>
              )}
              <p className="text-xs text-muted-foreground mt-1">
                Path to Java executable (Java 17+ recommended)
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Memory Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <MemoryStick className="h-5 w-5" />
            Memory Allocation
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="memoryMin">Minimum Memory (GB)</Label>
              <Input
                id="memoryMin"
                type="number"
                min="1"
                max="32"
                value={formData.memory.min}
                onChange={(e) => updateFormData({ 
                  memory: { ...formData.memory, min: parseInt(e.target.value) || 1 }
                })}
              />
            </div>
            <div>
              <Label htmlFor="memoryMax">Maximum Memory (GB)</Label>
              <Input
                id="memoryMax"
                type="number"
                min="1"
                max="32"
                value={formData.memory.max}
                onChange={(e) => updateFormData({ 
                  memory: { ...formData.memory, max: parseInt(e.target.value) || 4 }
                })}
              />
            </div>
          </div>
          
          {errors.memory && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{errors.memory}</AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <Label>Quick Presets</Label>
            <div className="flex flex-wrap gap-2">
              {memoryPresets.map((preset) => (
                <Badge
                  key={preset.label}
                  variant="outline"
                  className="cursor-pointer hover:bg-primary hover:text-primary-foreground border-2 border-primary/30 hover:border-primary/50 px-3 py-1 transition-all duration-200"
                  onClick={() => handleMemoryPreset(preset)}
                >
                  {preset.label}
                </Badge>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Validation Status */}
      {Object.keys(validationResults).length > 0 && (
        <Alert>
          <CheckCircle className="h-4 w-4" />
          <AlertDescription>
            {Object.values(validationResults).every(Boolean) 
              ? "All fields validated successfully" 
              : "Some fields need validation"
            }
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
};
