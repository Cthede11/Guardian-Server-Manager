import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
// import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
// import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { apiClient as api } from '@/lib/api';
import { 
  FileText, 
  Download, 
  Bug, 
  Activity, 
  HardDrive, 
  Cpu, 
  MemoryStick, 
  Network,
  CheckCircle,
  Loader2,
  Plus,
  Trash2
} from 'lucide-react';

interface BundleComponent {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  size: string;
  estimatedTime: string;
  required: boolean;
  selected: boolean;
}

interface BundleTemplate {
  id: string;
  name: string;
  description: string;
  components: string[];
}

export const CreateBundleModal: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [currentStep, setCurrentStep] = useState(1);
  const [bundleName, setBundleName] = useState('');
  const [bundleDescription, setBundleDescription] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');
  const [selectedComponents, setSelectedComponents] = useState<string[]>([]);
  const [customComponents, setCustomComponents] = useState<Array<{name: string, description: string}>>([]);
  const [isCreating, setIsCreating] = useState(false);
  const [createdBundle, setCreatedBundle] = useState<{id: string, name: string, size: string} | null>(null);

  const bundleTemplates: BundleTemplate[] = [
    {
      id: 'system',
      name: 'System Diagnostic',
      description: 'Complete system logs, crash dumps, and performance metrics',
      components: ['logs', 'crash-dumps', 'performance', 'system-info']
    },
    {
      id: 'crash',
      name: 'Crash Analysis',
      description: 'Crash logs, stack traces, and memory dumps from recent crashes',
      components: ['crash-dumps', 'stack-traces', 'memory-dumps', 'logs']
    },
    {
      id: 'performance',
      name: 'Performance Analysis',
      description: 'Performance metrics, profiling data, and optimization reports',
      components: ['performance', 'profiling', 'metrics', 'system-info']
    },
    {
      id: 'custom',
      name: 'Custom Bundle',
      description: 'Select specific components for your diagnostic needs',
      components: []
    }
  ];

  const bundleComponents: BundleComponent[] = [
    {
      id: 'logs',
      name: 'Server Logs',
      description: 'Complete server log files from the last 7 days',
      icon: <FileText className="h-4 w-4" />,
      size: '~50MB',
      estimatedTime: '2 minutes',
      required: false,
      selected: false
    },
    {
      id: 'crash-dumps',
      name: 'Crash Dumps',
      description: 'Memory dumps and crash reports from recent crashes',
      icon: <Bug className="h-4 w-4" />,
      size: '~200MB',
      estimatedTime: '5 minutes',
      required: false,
      selected: false
    },
    {
      id: 'stack-traces',
      name: 'Stack Traces',
      description: 'Detailed stack traces from all recent crashes',
      icon: <Activity className="h-4 w-4" />,
      size: '~10MB',
      estimatedTime: '1 minute',
      required: false,
      selected: false
    },
    {
      id: 'memory-dumps',
      name: 'Memory Dumps',
      description: 'Heap dumps and memory analysis data',
      icon: <MemoryStick className="h-4 w-4" />,
      size: '~500MB',
      estimatedTime: '10 minutes',
      required: false,
      selected: false
    },
    {
      id: 'performance',
      name: 'Performance Metrics',
      description: 'TPS, memory usage, and performance profiling data',
      icon: <Cpu className="h-4 w-4" />,
      size: '~20MB',
      estimatedTime: '3 minutes',
      required: false,
      selected: false
    },
    {
      id: 'profiling',
      name: 'Profiling Data',
      description: 'Detailed performance profiling and optimization reports',
      icon: <Activity className="h-4 w-4" />,
      size: '~100MB',
      estimatedTime: '7 minutes',
      required: false,
      selected: false
    },
    {
      id: 'system-info',
      name: 'System Information',
      description: 'Hardware specs, OS info, and system configuration',
      icon: <HardDrive className="h-4 w-4" />,
      size: '~5MB',
      estimatedTime: '1 minute',
      required: false,
      selected: false
    },
    {
      id: 'network',
      name: 'Network Diagnostics',
      description: 'Network configuration and connectivity diagnostics',
      icon: <Network className="h-4 w-4" />,
      size: '~15MB',
      estimatedTime: '2 minutes',
      required: false,
      selected: false
    }
  ];

  const handleTemplateSelect = (templateId: string) => {
    setSelectedTemplate(templateId);
    const template = bundleTemplates.find(t => t.id === templateId);
    if (template && template.components.length > 0) {
      setSelectedComponents(template.components);
    } else {
      setSelectedComponents([]);
    }
  };

  const handleComponentToggle = (componentId: string) => {
    setSelectedComponents(prev => 
      prev.includes(componentId)
        ? prev.filter(id => id !== componentId)
        : [...prev, componentId]
    );
  };

  const addCustomComponent = () => {
    setCustomComponents(prev => [...prev, { name: '', description: '' }]);
  };

  const removeCustomComponent = (index: number) => {
    setCustomComponents(prev => prev.filter((_, i) => i !== index));
  };

  const updateCustomComponent = (index: number, field: 'name' | 'description', value: string) => {
    setCustomComponents(prev => prev.map((comp, i) => 
      i === index ? { ...comp, [field]: value } : comp
    ));
  };

  const calculateBundleSize = () => {
    const selectedComps = bundleComponents.filter(comp => selectedComponents.includes(comp.id));
    const totalSize = selectedComps.reduce((acc, comp) => {
      const size = parseInt(comp.size.replace(/[^\d]/g, ''));
      return acc + size;
    }, 0);
    return `${totalSize}MB`;
  };

  const calculateEstimatedTime = () => {
    const selectedComps = bundleComponents.filter(comp => selectedComponents.includes(comp.id));
    const totalTime = selectedComps.reduce((acc, comp) => {
      const time = parseInt(comp.estimatedTime.replace(/[^\d]/g, ''));
      return acc + time;
    }, 0);
    return `${totalTime} minutes`;
  };

  const handleCreateBundle = async () => {
    setIsCreating(true);
    try {
      const response = await api.createDiagnosticBundle?.({
        name: bundleName || 'Diagnostic Bundle',
        description: bundleDescription,
        components: selectedComponents,
        customComponents: customComponents
      });
      
      if (response?.ok && response.data) {
        setCreatedBundle({
          id: response.data.id,
          name: response.data.name,
          size: response.data.size
        });
        setCurrentStep(4);
      } else {
        throw new Error('Failed to create diagnostic bundle');
      }
    } catch (error) {
      console.error('Failed to create bundle:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleClose = () => {
    setIsOpen(false);
    setCurrentStep(1);
    setBundleName('');
    setBundleDescription('');
    setSelectedTemplate('');
    setSelectedComponents([]);
    setCustomComponents([]);
    setCreatedBundle(null);
  };

  const getStepIcon = (step: number) => {
    if (currentStep > step) return <CheckCircle className="h-5 w-5 text-green-500" />;
    if (currentStep === step) return <div className="h-5 w-5 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-xs font-bold">{step}</div>;
    return <div className="h-5 w-5 rounded-full bg-muted text-muted-foreground flex items-center justify-center text-xs font-bold">{step}</div>;
  };

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Create Bundle
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Diagnostic Bundle</DialogTitle>
          <DialogDescription>
            Generate a comprehensive diagnostic bundle for troubleshooting
          </DialogDescription>
        </DialogHeader>

        {/* Progress Steps */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center space-x-2">
            {getStepIcon(1)}
            <span className="text-sm font-medium">Basic Info</span>
          </div>
          <div className="flex-1 h-px bg-border mx-4"></div>
          <div className="flex items-center space-x-2">
            {getStepIcon(2)}
            <span className="text-sm font-medium">Template</span>
          </div>
          <div className="flex-1 h-px bg-border mx-4"></div>
          <div className="flex items-center space-x-2">
            {getStepIcon(3)}
            <span className="text-sm font-medium">Components</span>
          </div>
          <div className="flex-1 h-px bg-border mx-4"></div>
          <div className="flex items-center space-x-2">
            {getStepIcon(4)}
            <span className="text-sm font-medium">Complete</span>
          </div>
        </div>

        {/* Step 1: Basic Info */}
        {currentStep === 1 && (
          <div className="space-y-4">
            <div>
              <Label htmlFor="bundle-name">Bundle Name</Label>
              <Input
                id="bundle-name"
                value={bundleName}
                onChange={(e) => setBundleName(e.target.value)}
                placeholder="Enter bundle name"
              />
            </div>
            <div>
              <Label htmlFor="bundle-description">Description</Label>
              <Textarea
                id="bundle-description"
                value={bundleDescription}
                onChange={(e) => setBundleDescription(e.target.value)}
                placeholder="Enter bundle description"
                rows={3}
              />
            </div>
            <div className="flex justify-end">
              <Button onClick={() => setCurrentStep(2)}>
                Next
              </Button>
            </div>
          </div>
        )}

        {/* Step 2: Template Selection */}
        {currentStep === 2 && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {bundleTemplates.map(template => (
                <Card 
                  key={template.id} 
                  className={`cursor-pointer transition-colors ${
                    selectedTemplate === template.id ? 'ring-2 ring-primary' : ''
                  }`}
                  onClick={() => handleTemplateSelect(template.id)}
                >
                  <CardHeader>
                    <CardTitle className="text-sm">{template.name}</CardTitle>
                    <CardDescription>{template.description}</CardDescription>
                  </CardHeader>
                </Card>
              ))}
            </div>
            <div className="flex justify-between">
              <Button variant="outline" onClick={() => setCurrentStep(1)}>
                Back
              </Button>
              <Button onClick={() => setCurrentStep(3)}>
                Next
              </Button>
            </div>
          </div>
        )}

        {/* Step 3: Component Selection */}
        {currentStep === 3 && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {bundleComponents.map(component => (
                <Card key={component.id} className="p-4">
                  <div className="flex items-start space-x-3">
                    <Checkbox
                      checked={selectedComponents.includes(component.id)}
                      onCheckedChange={() => handleComponentToggle(component.id)}
                    />
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-2">
                        {component.icon}
                        <h3 className="font-medium">{component.name}</h3>
                      </div>
                      <p className="text-sm text-muted-foreground mb-2">{component.description}</p>
                      <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                        <span>Size: {component.size}</span>
                        <span>Time: {component.estimatedTime}</span>
                      </div>
                    </div>
                  </div>
                </Card>
              ))}
            </div>

            {/* Custom Components */}
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="font-medium">Custom Components</h3>
                <Button size="sm" variant="outline" onClick={addCustomComponent}>
                  <Plus className="h-4 w-4 mr-2" />
                  Add Custom
                </Button>
              </div>
              {customComponents.map((comp, index) => (
                <Card key={index} className="p-4">
                  <div className="flex items-center space-x-3">
                    <div className="flex-1 space-y-2">
                      <Input
                        placeholder="Component name"
                        value={comp.name}
                        onChange={(e) => updateCustomComponent(index, 'name', e.target.value)}
                      />
                      <Input
                        placeholder="Component description"
                        value={comp.description}
                        onChange={(e) => updateCustomComponent(index, 'description', e.target.value)}
                      />
                    </div>
                    <Button size="sm" variant="outline" onClick={() => removeCustomComponent(index)}>
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </Card>
              ))}
            </div>

            {/* Bundle Summary */}
            <Card className="p-4">
              <h3 className="font-medium mb-2">Bundle Summary</h3>
              <div className="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Components:</span>
                  <span className="ml-2 font-medium">{selectedComponents.length}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Estimated Size:</span>
                  <span className="ml-2 font-medium">{calculateBundleSize()}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Estimated Time:</span>
                  <span className="ml-2 font-medium">{calculateEstimatedTime()}</span>
                </div>
              </div>
            </Card>

            <div className="flex justify-between">
              <Button variant="outline" onClick={() => setCurrentStep(2)}>
                Back
              </Button>
              <Button onClick={handleCreateBundle} disabled={selectedComponents.length === 0}>
                Create Bundle
              </Button>
            </div>
          </div>
        )}

        {/* Step 4: Complete */}
        {currentStep === 4 && createdBundle && (
          <div className="space-y-4 text-center">
            <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center">
              <CheckCircle className="h-8 w-8 text-green-500" />
            </div>
            <div>
              <h3 className="text-lg font-semibold">Bundle Created Successfully!</h3>
              <p className="text-muted-foreground">
                Your diagnostic bundle "{createdBundle.name}" has been created and is ready for download.
              </p>
            </div>
            <Card className="p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <FileText className="h-8 w-8 text-blue-500" />
                  <div className="text-left">
                    <h4 className="font-medium">{createdBundle.name}</h4>
                    <p className="text-sm text-muted-foreground">Size: {createdBundle.size}</p>
                  </div>
                </div>
                <Button>
                  <Download className="h-4 w-4 mr-2" />
                  Download
                </Button>
              </div>
            </Card>
            <div className="flex justify-end">
              <Button onClick={handleClose}>
                Close
              </Button>
            </div>
          </div>
        )}

        {/* Loading State */}
        {isCreating && (
          <div className="space-y-4 text-center">
            <div className="mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center">
              <Loader2 className="h-8 w-8 text-blue-500 animate-spin" />
            </div>
            <div>
              <h3 className="text-lg font-semibold">Creating Bundle...</h3>
              <p className="text-muted-foreground">
                Please wait while we collect and package your diagnostic data.
              </p>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
};
