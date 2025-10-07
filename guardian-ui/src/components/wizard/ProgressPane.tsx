import React from 'react';
import { motion } from 'framer-motion';
import { 
  CheckCircle, 
  Loader2, 
  X, 
  Download,
  Settings,
  Package,
  Shield,
  CheckSquare
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';

interface ProgressPaneProps {
  progress: number;
  stage: string;
  onCancel: () => void;
}

const progressStages = [
  { id: 'preparing', label: 'Preparing', icon: Settings, threshold: 0 },
  { id: 'core', label: 'Installing Core', icon: Download, threshold: 20 },
  { id: 'modpack', label: 'Installing Modpack', icon: Package, threshold: 40 },
  { id: 'mods', label: 'Installing Mods', icon: Package, threshold: 60 },
  { id: 'validating', label: 'Validating', icon: Shield, threshold: 80 },
  { id: 'finalizing', label: 'Finalizing', icon: CheckSquare, threshold: 100 }
];

export const ProgressPane: React.FC<ProgressPaneProps> = ({
  progress,
  stage,
  onCancel
}) => {
  const getCurrentStage = () => {
    for (let i = progressStages.length - 1; i >= 0; i--) {
      if (progress >= progressStages[i].threshold) {
        return progressStages[i];
      }
    }
    return progressStages[0];
  };

  const getStageStatus = (stageThreshold: number) => {
    if (progress > stageThreshold) return 'completed';
    if (progress === stageThreshold) return 'current';
    return 'pending';
  };

  const currentStage = getCurrentStage();

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-4"
    >
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Loader2 className="h-5 w-5 animate-spin" />
            Creating Server
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Progress Bar */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Progress</span>
              <span>{Math.round(progress)}%</span>
            </div>
            <Progress value={progress} className="h-2" />
          </div>

          {/* Current Stage */}
          <div className="text-center">
            <div className="flex items-center justify-center gap-2 mb-2">
              {progress < 100 ? (
                <Loader2 className="h-5 w-5 animate-spin" />
              ) : (
                <CheckCircle className="h-5 w-5 text-green-500" />
              )}
              <span className="font-medium">{stage}</span>
            </div>
            <p className="text-sm text-muted-foreground">
              {progress < 100 
                ? 'Please wait while your server is being created...' 
                : 'Server created successfully!'
              }
            </p>
          </div>

          {/* Stage Progress */}
          <div className="space-y-3">
            {progressStages.map((stage, index) => {
              const status = getStageStatus(stage.threshold);
              const Icon = stage.icon;
              
              return (
                <div key={stage.id} className="flex items-center gap-3">
                  <div className={`
                    flex items-center justify-center w-8 h-8 rounded-full border-2 transition-colors
                    ${status === 'completed' 
                      ? 'bg-green-500 border-green-500 text-white' 
                      : status === 'current'
                      ? 'bg-primary border-primary text-primary-foreground'
                      : 'border-muted-foreground text-muted-foreground'
                    }
                  `}>
                    {status === 'completed' ? (
                      <CheckCircle className="h-4 w-4" />
                    ) : status === 'current' ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <Icon className="h-4 w-4" />
                    )}
                  </div>
                  <div className="flex-1">
                    <p className={`text-sm font-medium ${
                      status === 'completed' ? 'text-green-600' : 
                      status === 'current' ? 'text-primary' : 
                      'text-muted-foreground'
                    }`}>
                      {stage.label}
                    </p>
                    {status === 'current' && (
                      <p className="text-xs text-muted-foreground">
                        In progress...
                      </p>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Estimated Time */}
          {progress < 100 && (
            <Alert>
              <AlertDescription className="text-center">
                <strong>Estimated time remaining:</strong> {
                  progress < 20 ? '2-3 minutes' :
                  progress < 60 ? '1-2 minutes' :
                  progress < 80 ? '30-60 seconds' :
                  '10-30 seconds'
                }
              </AlertDescription>
            </Alert>
          )}

          {/* Actions */}
          <div className="flex justify-center">
            {progress < 100 || !stage.includes('Server created successfully') ? (
              <Button variant="outline" onClick={onCancel}>
                <X className="h-4 w-4 mr-2" />
                Cancel
              </Button>
            ) : (
              <Button onClick={onCancel} className="bg-green-600 hover:bg-green-700">
                <CheckCircle className="h-4 w-4 mr-2" />
                Continue
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
};
