import React from 'react';
import { motion } from 'framer-motion';
import { 
  CheckCircle, 
  Server, 
  Package, 
  Globe, 
  Zap, 
  Settings,
  MemoryStick,
  FolderOpen,
  AlertTriangle,
  Info
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { type ServerFormData } from '@/lib/validation/server-schema';

interface StepReviewProps {
  formData: ServerFormData;
  updateFormData: (updates: Partial<ServerFormData>) => void;
  errors: Record<string, string>;
  versions: string[];
  isLoadingVersions: boolean;
  onValidate: () => boolean;
}

export const StepReview: React.FC<StepReviewProps> = ({
  formData,
  updateFormData,
  errors,
  versions,
  isLoadingVersions,
  onValidate
}) => {
  const formatMemory = (min: number, max: number) => `${min}GB - ${max}GB`;

  const getEditionColor = (edition: string) => {
    switch (edition) {
      case 'Vanilla': return 'bg-green-100 text-green-800';
      case 'Fabric': return 'bg-blue-100 text-blue-800';
      case 'Forge': return 'bg-orange-100 text-orange-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getWorldTypeLabel = (type: string) => {
    switch (type) {
      case 'default': return 'Default';
      case 'flat': return 'Flat';
      case 'custom': return 'Custom';
      default: return type;
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-semibold mb-2">Review & Create</h3>
        <p className="text-muted-foreground">
          Review your server configuration before creating
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Server Basics */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Server className="h-5 w-5" />
              Server Basics
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Name</span>
                <span className="text-sm">{formData.name}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Edition</span>
                <Badge className={getEditionColor(formData.edition)}>
                  {formData.edition}
                </Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Version</span>
                <span className="text-sm">{formData.version}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Memory</span>
                <span className="text-sm">{formatMemory(formData.memory.min, formData.memory.max)}</span>
              </div>
            </div>
            <Separator />
            <div className="space-y-2">
              <div className="flex items-start gap-2">
                <FolderOpen className="h-4 w-4 text-muted-foreground mt-0.5" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium">Install Path</p>
                  <p className="text-xs text-muted-foreground break-all">{formData.installPath}</p>
                  <p className="text-xs text-blue-600 mt-1">
                    Server will be created in: {formData.installPath || 'data/servers'}
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-2">
                <Zap className="h-4 w-4 text-muted-foreground mt-0.5" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium">Java Path</p>
                  <p className="text-xs text-muted-foreground break-all">{formData.javaPath}</p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Mods & Modpacks */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Package className="h-5 w-5" />
              Mods & Modpacks
            </CardTitle>
          </CardHeader>
          <CardContent>
            {formData.modpack ? (
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Modpack</span>
                  <Badge variant="outline">{formData.modpack.source}</Badge>
                </div>
                <div className="space-y-2">
                  <p className="text-sm font-medium">Selected Modpack</p>
                  <p className="text-xs text-muted-foreground">
                    Pack ID: {formData.modpack.packId}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Version: {formData.modpack.packVersionId}
                  </p>
                  {formData.modpack.serverOnly && (
                    <Badge variant="secondary" className="text-xs">
                      Server subset only
                    </Badge>
                  )}
                </div>
              </div>
            ) : formData.individualMods.length > 0 ? (
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Individual Mods</span>
                  <Badge variant="outline">{formData.individualMods.length} mods</Badge>
                </div>
                <div className="space-y-1">
                  {formData.individualMods.map((mod: any, index: number) => (
                    <div key={index} className="flex items-center justify-between text-xs">
                      <span className="truncate flex-1">{mod.modId}</span>
                      <Badge variant="outline" className="text-xs ml-2">
                        {mod.provider}
                      </Badge>
                    </div>
                  ))}
                </div>
              </div>
            ) : (
              <div className="text-center py-4">
                <Package className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                <p className="text-sm text-muted-foreground">No mods or modpacks selected</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* World Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Globe className="h-5 w-5" />
              World Settings
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">World Type</span>
                <span className="text-sm">{getWorldTypeLabel(formData.worldType)}</span>
              </div>
              {formData.levelSeed && (
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Seed</span>
                  <span className="text-sm font-mono text-xs">{formData.levelSeed}</span>
                </div>
              )}
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Render Distance</span>
                <span className="text-sm">{formData.renderDistance} chunks</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Performance Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Settings className="h-5 w-5" />
              Performance Settings
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">GPU Pregeneration</span>
                <Badge variant={formData.gpuPregeneration.enabled ? "default" : "outline"}>
                  {formData.gpuPregeneration.enabled ? 'Enabled' : 'Disabled'}
                </Badge>
              </div>
              
              {formData.gpuPregeneration.enabled && (
                <div className="space-y-2 pl-4 border-l-2 border-primary/20">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">Radius</span>
                    <span className="text-xs">{formData.gpuPregeneration.radius} blocks</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">Concurrency</span>
                    <span className="text-xs">{formData.gpuPregeneration.concurrency} threads</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">Defer until start</span>
                    <span className="text-xs">{formData.gpuPregeneration.deferUntilStart ? 'Yes' : 'No'}</span>
                  </div>
                </div>
              )}

              <Separator />
              
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Crash Isolation</span>
                  <Badge variant="outline">Enabled</Badge>
                </div>
                <div className="space-y-1 pl-4 border-l-2 border-primary/20">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">Tick Timeout</span>
                    <span className="text-xs">{formData.crashIsolation.tickTimeout}ms</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">Quarantine</span>
                    <span className="text-xs capitalize">
                      {formData.crashIsolation.quarantineBehavior.replace('_', ' ')}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Summary & Warnings */}
      <div className="space-y-4">
        {/* Resource Requirements */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <MemoryStick className="h-5 w-5" />
              Resource Requirements
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-primary">{formData.memory.max}GB</div>
                <div className="text-sm text-muted-foreground">RAM Required</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-primary">
                  {formData.gpuPregeneration.enabled ? 'GPU' : 'CPU'}
                </div>
                <div className="text-sm text-muted-foreground">Processing</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-primary">
                  {formData.renderDistance * 2}²
                </div>
                <div className="text-sm text-muted-foreground">Chunk Area</div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Warnings */}
        {formData.memory.max < 4 && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <strong>Low Memory Warning:</strong> {formData.memory.max}GB may not be sufficient for a stable server.
              Consider increasing to at least 4GB for better performance.
            </AlertDescription>
          </Alert>
        )}

        {formData.gpuPregeneration.enabled && formData.gpuPregeneration.radius > 5000 && (
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              <strong>Large Pregeneration Area:</strong> A radius of {formData.gpuPregeneration.radius} blocks will generate a very large world area.
              This may take significant time and storage space.
            </AlertDescription>
          </Alert>
        )}

        {formData.edition !== 'Vanilla' && formData.individualMods.length === 0 && !formData.modpack && (
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              <strong>No Mods Selected:</strong> You've chosen {formData.edition} but haven't selected any mods or modpacks.
              The server will run with just the loader and no additional content.
            </AlertDescription>
          </Alert>
        )}

        {/* Final Validation */}
        {Object.keys(errors).length > 0 && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <strong>Configuration Issues:</strong> Please fix the following errors before creating the server:
              <ul className="mt-2 list-disc list-inside">
                {Object.entries(errors).map(([key, error]) => (
                  <li key={key} className="text-sm">{error}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}
      </div>
    </div>
  );
};
