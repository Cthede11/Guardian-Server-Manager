// Comprehensive Server Creation Wizard Component
import React, { useState, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, CheckCircle, AlertTriangle } from 'lucide-react';
import { apiClient as api } from '@/lib/api';
import { useToast } from '@/hooks/use-toast';
import { 
  serverFormSchema, 
  validateStep, 
  formatValidationErrors,
  type ServerFormData 
} from '@/lib/validation/server-schema';

// Import wizard steps
import { StepBasics } from './wizard/StepBasics';
import { StepMods } from './wizard/StepMods';
import { StepWorld } from './wizard/StepWorld';
import { StepReview } from './wizard/StepReview';
import { ProgressPane } from './wizard/ProgressPane';

interface ServerInfo {
  id: string;
  name: string;
  status: string;
  tps: number;
  tick_p95: number;
  heap_mb: number;
  players_online: number;
  gpu_queue_ms: number;
  last_snapshot_at?: string;
  blue_green: {
    active: string;
    candidate_healthy: boolean;
  };
  version?: string;
  max_players?: number;
  uptime?: number;
  memory_usage?: number;
  cpu_usage?: number;
  world_size?: number;
  last_backup?: string;
  auto_start?: boolean;
  auto_restart?: boolean;
  created_at?: string;
  updated_at?: string;
}

interface ServerCreationWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onServerCreated?: (server: ServerInfo) => void;
  onClose?: () => void;
}

// ServerFormData is now imported from the validation schema

const initialFormData: ServerFormData = {
  name: '',
  edition: 'Vanilla',
  version: '',
  installPath: '',
  javaPath: '',
  memory: { min: 2, max: 4 },
  maxPlayers: 20,
  port: 25565,
  motd: 'A Minecraft Server',
  difficulty: 'normal',
  gamemode: 'survival',
  levelType: 'default',
  levelSeed: '',
  levelName: 'world',
  worldType: 'default',
  pvp: true,
  allowFlight: false,
  allowNether: true,
  allowEnd: true,
  spawnProtection: 16,
  viewDistance: 10,
  simulationDistance: 10,
  renderDistance: 10,
  hardcore: false,
  onlineMode: true,
  whiteList: false,
  enableCommandBlock: false,
  spawnAnimals: true,
  spawnMonsters: true,
  spawnNpcs: true,
  generateStructures: true,
  allowCheats: false,
  modpack: undefined,
  individualMods: [],
  gpuPregeneration: {
    enabled: false,
    radius: 1000,
    concurrency: 4,
    deferUntilStart: false
  },
  crashIsolation: {
    tickTimeout: 60000,
    quarantineBehavior: 'pause_entity'
  },
  serverProperties: {},
  generatorSettings: ''
};

export function ServerCreationWizard({ open, onOpenChange, onServerCreated, onClose }: ServerCreationWizardProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState<ServerFormData>(initialFormData);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [versions, setVersions] = useState<string[]>(['1.21.1', '1.21', '1.20.6', '1.20.4', '1.20.2']); // Start with fallback versions
  const [isLoadingVersions, setIsLoadingVersions] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [creationProgress, setCreationProgress] = useState(0);
  const [creationStage, setCreationStage] = useState('');
  const { toast } = useToast();

  const steps = [
    { id: 'basics', title: 'Server Basics', component: StepBasics },
    { id: 'mods', title: 'Mods & Modpacks', component: StepMods },
    { id: 'world', title: 'World & Performance', component: StepWorld },
    { id: 'review', title: 'Review & Create', component: StepReview }
  ];

  // Load versions when component mounts or edition changes
  useEffect(() => {
    if (open) {
      loadVersions();
    }
  }, [open, formData.edition]);

  // Ensure we always have some versions available
  useEffect(() => {
    if (versions.length === 0 && !isLoadingVersions) {
      console.log('No versions loaded, setting fallback');
      setVersions(['1.21.1', '1.21', '1.20.6', '1.20.4', '1.20.2']);
    }
  }, [versions.length, isLoadingVersions]);

  // Debug versions state
  useEffect(() => {
    console.log('Versions state updated:', { versions, isLoadingVersions, length: versions.length });
  }, [versions, isLoadingVersions]);

  const loadVersions = async () => {
    setIsLoadingVersions(true);
    try {
      console.log('Loading versions for edition:', formData.edition);
      const response = await api.call<{success: boolean, data: {versions: string[]}, error: string}>(`/api/server/versions?edition=${encodeURIComponent(formData.edition)}`, {
        method: 'GET'
      });
      console.log('Versions response:', response);
      
      if (response.success && response.data && response.data.versions) {
        setVersions(response.data.versions);
      } else {
        throw new Error('Invalid response format');
      }
    } catch (error) {
      console.error('Failed to load versions:', error);
      // Fallback to hardcoded versions
      const fallbackVersions = ['1.21.1', '1.21', '1.20.6', '1.20.4', '1.20.2'];
      console.log('Using fallback versions:', fallbackVersions);
      setVersions(fallbackVersions);
    } finally {
      setIsLoadingVersions(false);
    }
  };

  const updateFormData = (updates: Partial<ServerFormData>) => {
    setFormData(prev => ({ ...prev, ...updates }));
    // Clear related errors when updating
    const newErrors = { ...errors };
    Object.keys(updates).forEach(key => {
      delete newErrors[key];
    });
    setErrors(newErrors);
  };

  const validateCurrentStep = (stepIndex: number): boolean => {
    try {
      const result = validateStep(stepIndex, formData);
      
      if (result.success) {
        setErrors({});
        return true;
      } else {
        const formattedErrors = formatValidationErrors(result.error);
        setErrors(formattedErrors);
        return false;
      }
    } catch (error) {
      console.error('Validation error:', error);
      setErrors({ general: 'Validation failed' });
      return false;
    }
  };

  const handleNext = () => {
    if (validateCurrentStep(currentStep)) {
      setCurrentStep(prev => Math.min(prev + 1, steps.length - 1));
    }
  };

  const handlePrevious = () => {
    setCurrentStep(prev => Math.max(prev - 1, 0));
  };

  const handleCreate = async () => {
    if (!validateCurrentStep(currentStep)) {
      toast({
        title: "Validation Error",
        description: "Please fix the errors before creating the server",
        variant: "destructive"
      });
      return;
    }

    setIsCreating(true);
    setCreationProgress(0);
    setCreationStage('Preparing server creation...');

    try {
      // Prepare server creation data
      const serverData = {
        name: formData.name,
        loader: formData.edition.toLowerCase(),
        version: formData.version,
        minecraft_version: formData.version,
        paths: {
          world: './world',
          mods: './mods',
          config: './config',
          java_path: formData.javaPath || 'java'
        },
        max_players: 20,
        memory: formData.memory.max * 1024, // Convert GB to MB
        world_settings: {
          world_name: formData.name,
          difficulty: 'normal',
          gamemode: 'survival'
        },
        gpu_pregeneration: formData.gpuPregeneration,
        crash_isolation: formData.crashIsolation,
        modpack: formData.modpack,
        individual_mods: formData.individualMods
      };

      // Update stages with realistic timing
      setTimeout(() => setCreationStage('Installing core server files...'), 1000);
      setTimeout(() => setCreationStage('Installing modpack and mods...'), 3000);
      setTimeout(() => setCreationStage('Validating configuration...'), 5000);
      setTimeout(() => setCreationStage('Finalizing server setup...'), 7000);
      setTimeout(() => setCreationStage('Server created successfully!'), 9000);

      // Start progress simulation that's synchronized with stages
      const progressInterval = setInterval(() => {
        setCreationProgress(prev => {
          if (prev >= 95) {
            // Slow down near the end to ensure stage completion
            const increment = Math.min(Math.random() * 2, 100 - prev);
            return Math.min(prev + increment, 100);
          } else if (prev >= 80) {
            // Moderate speed in the middle
            const increment = Math.min(Math.random() * 4, 100 - prev);
            return Math.min(prev + increment, 100);
          } else {
            // Normal speed at the beginning
            const increment = Math.min(Math.random() * 6, 100 - prev);
            return Math.min(prev + increment, 100);
          }
        });
      }, 500);

      // Call the actual API
      const newServer = await api.createServer(serverData);

      // Wait for progress to reach 100% and final stage to complete
      await new Promise(resolve => {
        let attempts = 0;
        const maxAttempts = 100; // 20 seconds max wait
        
        const checkProgress = () => {
          attempts++;
          if (attempts >= maxAttempts || (creationProgress >= 100 && creationStage.includes('Server created successfully'))) {
            resolve(undefined);
          } else {
            setTimeout(checkProgress, 200);
          }
        };
        setTimeout(checkProgress, 2000); // Start checking after 2 seconds
      });

      // Complete the progress
      clearInterval(progressInterval);
      setCreationProgress(100);
      setCreationStage('Server created successfully!');

      // Wait a bit more to show the completion message
      await new Promise(resolve => setTimeout(resolve, 1500));

      // Now call the callback to refresh the server list
      console.log('Calling onServerCreated with:', newServer);
      onServerCreated?.(newServer);
      console.log('onServerCreated called successfully');

      setTimeout(() => {
        onOpenChange(false);
        onClose?.();
        
        // Reset form
        setFormData(initialFormData);
        setCurrentStep(0);
        setErrors({});
        setIsCreating(false);
        setCreationProgress(0);
        setCreationStage('');
      }, 1000);

    } catch (error) {
      console.error('Failed to create server:', error);
      
      let errorMessage = 'Failed to create server';
      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null) {
        // Try to extract error message from API response
        const errorObj = error as any;
        if (errorObj.error) {
          errorMessage = errorObj.error;
        } else if (errorObj.message) {
          errorMessage = errorObj.message;
        }
      }
      
      toast({
        title: "Creation Failed",
        description: errorMessage,
        variant: "destructive"
      });
      setIsCreating(false);
      setCreationProgress(0);
      setCreationStage('');
    }
  };

  const handleClose = () => {
    if (!isCreating) {
      onOpenChange(false);
      onClose?.();
      // Reset form
      setFormData(initialFormData);
      setCurrentStep(0);
      setErrors({});
    }
  };

  if (isCreating) {
    return (
      <ProgressPane
        progress={creationProgress}
        stage={creationStage}
        onCancel={() => {
          if (creationProgress >= 100) {
            // Server creation completed successfully, close the wizard
            setIsCreating(false);
            setCreationProgress(0);
            setCreationStage('');
            onOpenChange(false);
            onClose?.();
          } else {
            // Cancel during creation
            setIsCreating(false);
            setCreationProgress(0);
            setCreationStage('');
          }
        }}
      />
    );
  }

  const CurrentStepComponent = steps[currentStep].component;

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-5xl max-h-[95vh] overflow-y-auto">
        <DialogHeader className="space-y-4">
          <DialogTitle className="text-2xl font-bold text-center">Create New Server</DialogTitle>
          <p className="text-muted-foreground text-center">
            Set up your Minecraft server with our guided wizard
          </p>
        </DialogHeader>

        {/* Enhanced Progress indicator */}
        <div className="space-y-4">
          <div className="flex justify-between items-center text-sm">
            <span className="font-medium">Step {currentStep + 1} of {steps.length}</span>
            <span className="text-muted-foreground">{Math.round(((currentStep + 1) / steps.length) * 100)}% Complete</span>
          </div>
          <Progress value={((currentStep + 1) / steps.length) * 100} className="h-3" />
          <div className="flex justify-between text-sm">
            {steps.map((step, index) => (
              <div
                key={step.id}
                className={`flex flex-col items-center space-y-1 ${
                  index <= currentStep 
                    ? 'text-primary font-medium' 
                    : 'text-muted-foreground'
                }`}
              >
                <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold ${
                  index < currentStep 
                    ? 'bg-primary text-primary-foreground' 
                    : index === currentStep
                    ? 'bg-primary/20 text-primary border-2 border-primary'
                    : 'bg-muted text-muted-foreground'
                }`}>
                  {index < currentStep ? <CheckCircle className="w-4 h-4" /> : index + 1}
                </div>
                <span className="text-xs text-center max-w-20">{step.title}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Current step content */}
        <div className="py-6">
          <CurrentStepComponent
            formData={formData}
            updateFormData={updateFormData}
            errors={errors}
            versions={versions}
            isLoadingVersions={isLoadingVersions}
            onValidate={() => validateCurrentStep(currentStep)}
          />
        </div>

        {/* Enhanced Navigation */}
        <div className="flex justify-between items-center pt-6 border-t bg-muted/30 -mx-6 -mb-6 px-6 py-4 rounded-b-lg">
          <Button 
            variant="outline" 
            onClick={handlePrevious} 
            disabled={currentStep === 0}
            className="min-w-24"
          >
            ← Previous
          </Button>
          
          <div className="flex items-center space-x-4">
            {Object.keys(errors).length > 0 && (
              <div className="flex items-center space-x-2 text-sm text-destructive">
                <AlertTriangle className="w-4 h-4" />
                <span>{Object.keys(errors).length} error{Object.keys(errors).length > 1 ? 's' : ''} found</span>
              </div>
            )}
            
            <div className="flex gap-2">
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              {currentStep === steps.length - 1 ? (
                <Button 
                  onClick={handleCreate} 
                  disabled={Object.keys(errors).length > 0 || isCreating}
                  className="bg-green-600 hover:bg-green-700 min-w-32"
                  size="lg"
                >
                  {isCreating ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Creating...
                    </>
                  ) : (
                    <>
                      <CheckCircle className="h-4 w-4 mr-2" />
                      Create Server
                    </>
                  )}
                </Button>
              ) : (
                <Button 
                  onClick={handleNext}
                  disabled={Object.keys(errors).length > 0}
                  className="min-w-24"
                  size="lg"
                >
                  Next →
                </Button>
              )}
            </div>
          </div>
        </div>

        {/* Error summary */}
        {Object.keys(errors).length > 0 && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <strong>Please fix the following errors:</strong>
              <ul className="mt-2 list-disc list-inside">
                {Object.entries(errors).map(([key, error]) => (
                  <li key={key} className="text-sm">{error}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}
      </DialogContent>
    </Dialog>
  );
}