import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { 
  Globe, 
  Settings, 
  Zap, 
  Shield, 
  AlertTriangle,
  Info,
  Cpu,
  MemoryStick
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { type ServerFormData } from '@/lib/validation/server-schema';

interface StepWorldProps {
  formData: ServerFormData;
  updateFormData: (updates: Partial<ServerFormData>) => void;
  errors: Record<string, string>;
  versions: string[];
  isLoadingVersions: boolean;
  onValidate: () => boolean;
}

const worldTypes = [
  { value: 'new', label: 'New World', description: 'Create a new world' },
  { value: 'existing', label: 'Existing World', description: 'Use an existing world' },
  { value: 'import', label: 'Import World', description: 'Import a world from file' },
  { value: 'default', label: 'Default', description: 'Standard world generation' }
];

const quarantineBehaviors = [
  { value: 'pause_entity', label: 'Pause Entity', description: 'Pause problematic entities' },
  { value: 'restart_region', label: 'Restart Region', description: 'Restart the affected region' }
];

export const StepWorld: React.FC<StepWorldProps> = ({
  formData,
  updateFormData,
  errors,
  versions,
  isLoadingVersions,
  onValidate
}) => {
  const [gpuEnabled, setGpuEnabled] = useState(typeof formData.gpuPregeneration === 'object' && formData.gpuPregeneration?.enabled || false);

  const handleGpuToggle = (enabled: boolean) => {
    setGpuEnabled(enabled);
    updateFormData({
      gpuPregeneration: { 
        enabled,
        radius: typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.radius || 1000 : 1000,
        concurrency: typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.concurrency || 4 : 4,
        deferUntilStart: typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.deferUntilStart || false : false
      }
    });
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-semibold mb-2">World & Performance</h3>
        <p className="text-muted-foreground">
          Configure world generation and performance settings
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* World Configuration */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Globe className="h-5 w-5" />
              World Settings
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="worldSeed">World Seed (Optional)</Label>
              <Input
                id="levelSeed"
                placeholder="Enter world seed or leave empty for random"
                value={formData.levelSeed || ''}
                onChange={(e) => updateFormData({ levelSeed: e.target.value })}
              />
              <p className="text-xs text-muted-foreground mt-1">
                Leave empty for a random seed
              </p>
            </div>

            <div>
              <Label htmlFor="worldType">World Type</Label>
              <Select
                value={formData.worldType}
                onValueChange={(value: 'new' | 'existing' | 'import' | 'default') => updateFormData({ worldType: value })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {worldTypes.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      <div>
                        <div className="font-medium">{type.label}</div>
                        <div className="text-xs text-muted-foreground">{type.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="renderDistance" className="text-sm font-medium text-foreground">Render Distance</Label>
              <div className="space-y-3 mt-2">
                <div className="bg-muted/50 rounded-lg p-4">
                  <Slider
                    value={[formData.renderDistance || 10]}
                    onValueChange={([value]) => updateFormData({ renderDistance: value })}
                    min={2}
                    max={32}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-sm mt-3">
                    <span className="text-muted-foreground">2 chunks</span>
                    <span className="font-semibold text-primary bg-primary/10 px-3 py-1 rounded-full">
                      {formData.renderDistance} chunks
                    </span>
                    <span className="text-muted-foreground">32 chunks</span>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  Higher values require more memory and processing power
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* GPU Acceleration */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5" />
              GPU Acceleration
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-muted/30 rounded-lg border">
              <div>
                <Label htmlFor="gpuEnabled" className="text-sm font-medium text-foreground">Enable GPU Pregeneration</Label>
                <p className="text-xs text-muted-foreground mt-1">
                  Use GPU to accelerate world generation
                </p>
              </div>
              <Switch
                id="gpuEnabled"
                checked={gpuEnabled}
                onCheckedChange={handleGpuToggle}
                className="data-[state=checked]:bg-primary"
              />
            </div>

            {gpuEnabled && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.2 }}
                className="space-y-4"
              >
                <div>
                  <Label htmlFor="gpuRadius" className="text-sm font-medium text-foreground">Pregeneration Radius</Label>
                  <div className="space-y-3 mt-2">
                    <div className="bg-muted/50 rounded-lg p-4">
                      <Slider
                        value={[typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.radius || 1000 : 1000]}
                        onValueChange={([value]) => updateFormData({
                          gpuPregeneration: typeof formData.gpuPregeneration === 'object' ? { ...formData.gpuPregeneration, radius: value } : { enabled: false, radius: value, concurrency: 4, deferUntilStart: false }
                        })}
                        min={100}
                        max={10000}
                        step={100}
                        className="w-full"
                      />
                      <div className="flex justify-between text-sm mt-3">
                        <span className="text-muted-foreground">100 blocks</span>
                        <span className="font-semibold text-primary bg-primary/10 px-3 py-1 rounded-full">
                          {typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.radius || 1000 : 1000} blocks
                        </span>
                        <span className="text-muted-foreground">10,000 blocks</span>
                      </div>
                    </div>
                    {errors.gpuRadius && (
                      <p className="text-sm text-red-500">{errors.gpuRadius}</p>
                    )}
                  </div>
                </div>

                <div>
                  <Label htmlFor="gpuConcurrency" className="text-sm font-medium text-foreground">GPU Concurrency</Label>
                  <div className="space-y-3 mt-2">
                    <div className="bg-muted/50 rounded-lg p-4">
                      <Slider
                        value={[typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.concurrency || 4 : 4]}
                        onValueChange={([value]) => updateFormData({
                          gpuPregeneration: typeof formData.gpuPregeneration === 'object' ? { ...formData.gpuPregeneration, concurrency: value } : { enabled: false, radius: 1000, concurrency: value, deferUntilStart: false }
                        })}
                        min={1}
                        max={16}
                        step={1}
                        className="w-full"
                      />
                      <div className="flex justify-between text-sm mt-3">
                        <span className="text-muted-foreground">1 thread</span>
                        <span className="font-semibold text-primary bg-primary/10 px-3 py-1 rounded-full">
                          {typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.concurrency || 4 : 4} threads
                        </span>
                        <span className="text-muted-foreground">16 threads</span>
                      </div>
                    </div>
                    {errors.gpuConcurrency && (
                      <p className="text-sm text-red-500">{errors.gpuConcurrency}</p>
                    )}
                  </div>
                </div>

                <div className="flex items-center space-x-3 p-3 bg-muted/20 rounded-lg border">
                  <input
                    type="checkbox"
                    id="deferUntilStart"
                    checked={typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.deferUntilStart || false : false}
                    onChange={(e) => updateFormData({
                      gpuPregeneration: typeof formData.gpuPregeneration === 'object' ? { ...formData.gpuPregeneration, deferUntilStart: e.target.checked } : { enabled: false, radius: 1000, concurrency: 4, deferUntilStart: e.target.checked }
                    })}
                    className="w-4 h-4 text-primary bg-background border-2 border-muted-foreground rounded focus:ring-2 focus:ring-primary focus:ring-offset-2"
                  />
                  <Label htmlFor="deferUntilStart" className="text-sm font-medium text-foreground cursor-pointer">
                    Defer pregeneration until after first start
                  </Label>
                </div>

                <Alert>
                  <Info className="h-4 w-4" />
                  <AlertDescription>
                    GPU pregeneration can significantly speed up world generation but requires a compatible GPU.
                    The process will run in the background after server startup.
                  </AlertDescription>
                </Alert>
              </motion.div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Crash Isolation */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Crash Isolation Settings
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="tickTimeout">Tick Timeout (ms)</Label>
              <Input
                id="tickTimeout"
                type="number"
                min="1000"
                max="300000"
                value={typeof formData.crashIsolation === 'object' ? formData.crashIsolation?.tickTimeout || 60000 : 60000}
                onChange={(e) => updateFormData({
                  crashIsolation: typeof formData.crashIsolation === 'object' ? { ...formData.crashIsolation, tickTimeout: parseInt(e.target.value) || 60000 } : { enabled: false, tickTimeout: parseInt(e.target.value) || 60000, quarantineBehavior: 'pause_entity' }
                })}
              />
              <p className="text-xs text-muted-foreground mt-1">
                Maximum time a single tick can take before triggering isolation
              </p>
            </div>

            <div>
              <Label htmlFor="quarantineBehavior">Quarantine Behavior</Label>
              <Select
                value={typeof formData.crashIsolation === 'object' ? formData.crashIsolation?.quarantineBehavior || 'pause_entity' : 'pause_entity'}
                onValueChange={(value: 'pause_entity' | 'restart_region') => updateFormData({
                  crashIsolation: typeof formData.crashIsolation === 'object' ? { ...formData.crashIsolation, quarantineBehavior: value } : { enabled: false, tickTimeout: 60000, quarantineBehavior: value }
                })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {quarantineBehaviors.map((behavior) => (
                    <SelectItem key={behavior.value} value={behavior.value}>
                      <div>
                        <div className="font-medium">{behavior.label}</div>
                        <div className="text-xs text-muted-foreground">{behavior.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <Alert>
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Crash isolation helps prevent server crashes by detecting and isolating problematic entities or regions.
              These settings determine how the server responds to potential crash conditions.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Performance Summary */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Performance Summary
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="flex items-center gap-2">
              <Globe className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-sm font-medium">World Type</p>
                <p className="text-xs text-muted-foreground">
                  {worldTypes.find(t => t.value === formData.worldType)?.label}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Cpu className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-sm font-medium">Render Distance</p>
                <p className="text-xs text-muted-foreground">
                  {formData.renderDistance} chunks
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Zap className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-sm font-medium">GPU Acceleration</p>
                <p className="text-xs text-muted-foreground">
                  {gpuEnabled ? 'Enabled' : 'Disabled'}
                </p>
              </div>
            </div>
          </div>
          
          {gpuEnabled && (
            <div className="mt-4 pt-4 border-t">
              <div className="flex items-center gap-2 mb-2">
                <Badge variant="outline">GPU Settings</Badge>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Radius:</span> {typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.radius || 1000 : 1000} blocks
                </div>
                <div>
                  <span className="text-muted-foreground">Concurrency:</span> {typeof formData.gpuPregeneration === 'object' ? formData.gpuPregeneration?.concurrency || 4 : 4} threads
                </div>
                <div>
                  <span className="text-muted-foreground">Defer until start:</span> {typeof formData.gpuPregeneration === 'object' ? (formData.gpuPregeneration?.deferUntilStart ? 'Yes' : 'No') : 'No'}
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};
