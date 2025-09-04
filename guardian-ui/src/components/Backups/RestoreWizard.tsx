import React, { useState } from 'react';
import { 
  Upload, 
  AlertTriangle, 
  CheckCircle,
  HardDrive,
  Database,
  Shield,
  Settings
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
// Unused import removed
// import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
// Unused import removed
// import { Separator } from '@/components/ui/separator';

interface RestoreWizardProps {
  isOpen: boolean;
  onClose: () => void;
  snapshot: any;
  onRestore: (options: any) => void;
  className?: string;
}

export const RestoreWizard: React.FC<RestoreWizardProps> = ({
  isOpen,
  onClose,
  snapshot,
  onRestore,
}) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [isRestoring, setIsRestoring] = useState(false);
  
  // Restore options
  const [restoreOptions, setRestoreOptions] = useState({
    targetServer: 'current',
    restoreType: 'full',
    includeWorld: true,
    includePlayerData: true,
    includeMods: true,
    includeConfigs: true,
    includePlugins: true,
    backupCurrent: true,
    notifyPlayers: true,
    scheduleRestore: false,
    restoreTime: '',
    confirmationCode: ''
  });

  const steps = [
    { id: 1, title: 'Snapshot Info', description: 'Review snapshot details' },
    { id: 2, title: 'Restore Options', description: 'Configure restore settings' },
    { id: 3, title: 'Confirmation', description: 'Confirm and execute restore' }
  ];

  const handleNext = () => {
    if (currentStep < steps.length) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrevious = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleRestore = async () => {
    setIsRestoring(true);
    try {
      // Simulate restore process
      await new Promise(resolve => setTimeout(resolve, 3000));
      onRestore(restoreOptions);
    } catch (error) {
      console.error('Error during restore:', error);
    } finally {
      setIsRestoring(false);
    }
  };

  const handleClose = () => {
    setCurrentStep(1);
    setRestoreOptions({
      targetServer: 'current',
      restoreType: 'full',
      includeWorld: true,
      includePlayerData: true,
      includeMods: true,
      includeConfigs: true,
      includePlugins: true,
      backupCurrent: true,
      notifyPlayers: true,
      scheduleRestore: false,
      restoreTime: '',
      confirmationCode: ''
    });
    onClose();
  };

  const formatBytes = (bytes: number) => {
    if (bytes >= 1024 * 1024 * 1024) {
      return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    } else if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    } else {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'text-green-400';
      case 'in_progress':
        return 'text-blue-400';
      case 'failed':
        return 'text-red-400';
      case 'verifying':
        return 'text-yellow-400';
      default:
        return 'text-gray-400';
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'manual':
        return 'bg-blue-500/20 text-blue-400';
      case 'scheduled':
        return 'bg-green-500/20 text-green-400';
      case 'pre-update':
        return 'bg-yellow-500/20 text-yellow-400';
      case 'emergency':
        return 'bg-red-500/20 text-red-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  if (!snapshot) return null;

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            Restore Snapshot
          </DialogTitle>
        </DialogHeader>

        {/* Progress Steps */}
        <div className="flex items-center justify-between mb-6">
          {steps.map((step, index) => (
            <div key={step.id} className="flex items-center">
              <div className={`flex items-center justify-center w-8 h-8 rounded-full border-2 ${
                currentStep >= step.id 
                  ? 'bg-primary border-primary text-primary-foreground' 
                  : 'border-muted-foreground text-muted-foreground'
              }`}>
                {currentStep > step.id ? (
                  <CheckCircle className="h-4 w-4" />
                ) : (
                  <span className="text-sm font-medium">{step.id}</span>
                )}
              </div>
              <div className="ml-3">
                <p className={`text-sm font-medium ${
                  currentStep >= step.id ? 'text-foreground' : 'text-muted-foreground'
                }`}>
                  {step.title}
                </p>
                <p className="text-xs text-muted-foreground">{step.description}</p>
              </div>
              {index < steps.length - 1 && (
                <div className={`w-16 h-0.5 mx-4 ${
                  currentStep > step.id ? 'bg-primary' : 'bg-muted-foreground'
                }`} />
              )}
            </div>
          ))}
        </div>

        {/* Step Content */}
        <div className="space-y-6">
          {/* Step 1: Snapshot Info */}
          {currentStep === 1 && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <HardDrive className="h-5 w-5" />
                  Snapshot Details
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label className="text-sm font-medium">Name</Label>
                    <p className="text-sm text-muted-foreground">{snapshot.name}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-medium">Type</Label>
                    <Badge className={`text-xs ${getTypeColor(snapshot.type)}`}>
                      {snapshot.type}
                    </Badge>
                  </div>
                  <div>
                    <Label className="text-sm font-medium">Size</Label>
                    <p className="text-sm text-muted-foreground">{formatBytes(snapshot.size)}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-medium">Created</Label>
                    <p className="text-sm text-muted-foreground">{formatDate(snapshot.timestamp)}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-medium">Status</Label>
                    <p className={`text-sm ${getStatusColor(snapshot.status)}`}>
                      {snapshot.status}
                    </p>
                  </div>
                  <div>
                    <Label className="text-sm font-medium">Compression</Label>
                    <p className="text-sm text-muted-foreground">{snapshot.compression}%</p>
                  </div>
                </div>
                
                <div>
                  <Label className="text-sm font-medium">Description</Label>
                  <p className="text-sm text-muted-foreground">{snapshot.description}</p>
                </div>
                
                {snapshot.tags.length > 0 && (
                  <div>
                    <Label className="text-sm font-medium">Tags</Label>
                    <div className="flex flex-wrap gap-1 mt-1">
                      {snapshot.tags.map((tag: string, index: number) => (
                        <Badge key={index} variant="outline" className="text-xs">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          {/* Step 2: Restore Options */}
          {currentStep === 2 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Restore Configuration
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label htmlFor="target-server">Target Server</Label>
                      <Select 
                        value={restoreOptions.targetServer} 
                        onValueChange={(value) => setRestoreOptions(prev => ({ ...prev, targetServer: value }))}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="current">Current Server</SelectItem>
                          <SelectItem value="new">New Server</SelectItem>
                          <SelectItem value="other">Other Server</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    
                    <div>
                      <Label htmlFor="restore-type">Restore Type</Label>
                      <Select 
                        value={restoreOptions.restoreType} 
                        onValueChange={(value) => setRestoreOptions(prev => ({ ...prev, restoreType: value }))}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="full">Full Restore</SelectItem>
                          <SelectItem value="selective">Selective Restore</SelectItem>
                          <SelectItem value="world-only">World Only</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Database className="h-5 w-5" />
                    Data Selection
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-3">
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="include-world"
                        checked={restoreOptions.includeWorld}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, includeWorld: !!checked }))}
                      />
                      <Label htmlFor="include-world">World Data</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="include-players"
                        checked={restoreOptions.includePlayerData}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, includePlayerData: !!checked }))}
                      />
                      <Label htmlFor="include-players">Player Data</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="include-mods"
                        checked={restoreOptions.includeMods}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, includeMods: !!checked }))}
                      />
                      <Label htmlFor="include-mods">Mods</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="include-configs"
                        checked={restoreOptions.includeConfigs}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, includeConfigs: !!checked }))}
                      />
                      <Label htmlFor="include-configs">Configurations</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="include-plugins"
                        checked={restoreOptions.includePlugins}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, includePlugins: !!checked }))}
                      />
                      <Label htmlFor="include-plugins">Plugins</Label>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Shield className="h-5 w-5" />
                    Safety Options
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-3">
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="backup-current"
                        checked={restoreOptions.backupCurrent}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, backupCurrent: !!checked }))}
                      />
                      <Label htmlFor="backup-current">Backup current state before restore</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="notify-players"
                        checked={restoreOptions.notifyPlayers}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, notifyPlayers: !!checked }))}
                      />
                      <Label htmlFor="notify-players">Notify players before restore</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="schedule-restore"
                        checked={restoreOptions.scheduleRestore}
                        onCheckedChange={(checked) => setRestoreOptions(prev => ({ ...prev, scheduleRestore: !!checked }))}
                      />
                      <Label htmlFor="schedule-restore">Schedule restore for later</Label>
                    </div>
                  </div>
                  
                  {restoreOptions.scheduleRestore && (
                    <div>
                      <Label htmlFor="restore-time">Restore Time</Label>
                      <Input
                        id="restore-time"
                        type="datetime-local"
                        value={restoreOptions.restoreTime}
                        onChange={(e) => setRestoreOptions(prev => ({ ...prev, restoreTime: e.target.value }))}
                      />
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          )}

          {/* Step 3: Confirmation */}
          {currentStep === 3 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <AlertTriangle className="h-5 w-5 text-yellow-500" />
                    Restore Confirmation
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                    <p className="text-sm text-yellow-400">
                      <strong>Warning:</strong> This action will restore the server to the state captured in this snapshot. 
                      All current data will be replaced. Make sure you have a backup of the current state.
                    </p>
                  </div>
                  
                  <div className="space-y-2">
                    <h4 className="font-medium">Restore Summary:</h4>
                    <ul className="text-sm text-muted-foreground space-y-1">
                      <li>• Snapshot: {snapshot.name}</li>
                      <li>• Target: {restoreOptions.targetServer}</li>
                      <li>• Type: {restoreOptions.restoreType}</li>
                      <li>• Backup current: {restoreOptions.backupCurrent ? 'Yes' : 'No'}</li>
                      {restoreOptions.scheduleRestore && (
                        <li>• Scheduled for: {restoreOptions.restoreTime}</li>
                      )}
                    </ul>
                  </div>
                  
                  <div>
                    <Label htmlFor="confirmation-code">Confirmation Code</Label>
                    <Input
                      id="confirmation-code"
                      placeholder="Type 'RESTORE' to confirm"
                      value={restoreOptions.confirmationCode}
                      onChange={(e) => setRestoreOptions(prev => ({ ...prev, confirmationCode: e.target.value }))}
                    />
                  </div>
                </CardContent>
              </Card>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {currentStep > 1 && (
              <Button variant="outline" onClick={handlePrevious}>
                Previous
              </Button>
            )}
          </div>
          
          <div className="flex items-center gap-2">
            <Button variant="outline" onClick={handleClose}>
              Cancel
            </Button>
            
            {currentStep < steps.length ? (
              <Button onClick={handleNext}>
                Next
              </Button>
            ) : (
              <Button
                onClick={handleRestore}
                disabled={isRestoring || restoreOptions.confirmationCode !== 'RESTORE'}
                className="bg-red-600 hover:bg-red-700"
              >
                {isRestoring ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                    Restoring...
                  </>
                ) : (
                  <>
                    <Upload className="h-4 w-4 mr-2" />
                    Restore Snapshot
                  </>
                )}
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default RestoreWizard;
