import React, { useState } from 'react';
import { 
  X, 
  Map, 
  Target,
  Settings,
  Zap,
  Cpu,
  HardDrive,
  Clock,
  CheckCircle,
  AlertTriangle,
  Plus,
  Minus,
  Layers,
  Activity,
  Gauge,
  Database,
  Server,
  Monitor
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import { Separator } from '@/components/ui/separator';

interface RegionSelectorProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (regionData: any) => void;
  className?: string;
}

export const RegionSelector: React.FC<RegionSelectorProps> = ({
  isOpen,
  onClose,
  onCreate,
  className = ''
}) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [isCreating, setIsCreating] = useState(false);
  
  // Region data
  const [regionData, setRegionData] = useState({
    name: '',
    dimension: 'overworld',
    centerX: 0,
    centerZ: 0,
    radius: 100,
    priority: 'normal',
    gpuAccelerated: true,
    description: '',
    tags: [] as string[],
    autoStart: true,
    pauseOnLowTPS: true,
    maxChunksPerSecond: 25,
    memoryLimit: 2048
  });

  const steps = [
    { id: 1, title: 'Region Info', description: 'Basic region details' },
    { id: 2, title: 'Coordinates', description: 'Center and radius' },
    { id: 3, title: 'Settings', description: 'Performance and options' },
    { id: 4, title: 'Review', description: 'Confirm and create' }
  ];

  const dimensions = [
    { value: 'overworld', label: 'Overworld', icon: <Map className="h-4 w-4" />, description: 'Main world dimension' },
    { value: 'nether', label: 'Nether', icon: <Target className="h-4 w-4" />, description: 'Nether dimension' },
    { value: 'end', label: 'End', icon: <Layers className="h-4 w-4" />, description: 'End dimension' }
  ];

  const priorities = [
    { value: 'low', label: 'Low', color: 'bg-gray-500/20 text-gray-400' },
    { value: 'normal', label: 'Normal', color: 'bg-blue-500/20 text-blue-400' },
    { value: 'high', label: 'High', color: 'bg-yellow-500/20 text-yellow-400' },
    { value: 'critical', label: 'Critical', color: 'bg-red-500/20 text-red-400' }
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

  const handleCreate = async () => {
    setIsCreating(true);
    try {
      // Simulate creation
      await new Promise(resolve => setTimeout(resolve, 2000));
      onCreate(regionData);
    } catch (error) {
      console.error('Error creating pregen job:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleClose = () => {
    setCurrentStep(1);
    setRegionData({
      name: '',
      dimension: 'overworld',
      centerX: 0,
      centerZ: 0,
      radius: 100,
      priority: 'normal',
      gpuAccelerated: true,
      description: '',
      tags: [],
      autoStart: true,
      pauseOnLowTPS: true,
      maxChunksPerSecond: 25,
      memoryLimit: 2048
    });
    onClose();
  };

  const handleTagAdd = (tag: string) => {
    if (tag && !regionData.tags.includes(tag)) {
      setRegionData(prev => ({
        ...prev,
        tags: [...prev.tags, tag]
      }));
    }
  };

  const handleTagRemove = (tag: string) => {
    setRegionData(prev => ({
      ...prev,
      tags: prev.tags.filter(t => t !== tag)
    }));
  };

  const calculateTotalChunks = () => {
    const radius = regionData.radius;
    return Math.floor(Math.PI * radius * radius);
  };

  const getDimensionIcon = (dimension: string) => {
    const dim = dimensions.find(d => d.value === dimension);
    return dim?.icon || <Map className="h-4 w-4" />;
  };

  const getDimensionDescription = (dimension: string) => {
    const dim = dimensions.find(d => d.value === dimension);
    return dim?.description || 'Main world dimension';
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Map className="h-5 w-5" />
            Create Pregen Region
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
          {/* Step 1: Region Info */}
          {currentStep === 1 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Target className="h-5 w-5" />
                    Region Details
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label htmlFor="region-name">Region Name</Label>
                    <Input
                      id="region-name"
                      placeholder="Enter region name"
                      value={regionData.name}
                      onChange={(e) => setRegionData(prev => ({ ...prev, name: e.target.value }))}
                    />
                  </div>
                  
                  <div>
                    <Label htmlFor="region-description">Description</Label>
                    <Textarea
                      id="region-description"
                      placeholder="Describe this region (optional)"
                      value={regionData.description}
                      onChange={(e) => setRegionData(prev => ({ ...prev, description: e.target.value }))}
                    />
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Layers className="h-5 w-5" />
                    Dimension
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 gap-3">
                    {dimensions.map((dimension) => (
                      <div
                        key={dimension.value}
                        className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                          regionData.dimension === dimension.value 
                            ? 'border-primary bg-primary/10' 
                            : 'border-muted hover:border-muted-foreground'
                        }`}
                        onClick={() => setRegionData(prev => ({ ...prev, dimension: dimension.value }))}
                      >
                        <div className="flex items-center gap-3">
                          {dimension.icon}
                          <div>
                            <p className="font-medium">{dimension.label}</p>
                            <p className="text-sm text-muted-foreground">{dimension.description}</p>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <AlertTriangle className="h-5 w-5" />
                    Priority
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 gap-3">
                    {priorities.map((priority) => (
                      <div
                        key={priority.value}
                        className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                          regionData.priority === priority.value 
                            ? 'border-primary bg-primary/10' 
                            : 'border-muted hover:border-muted-foreground'
                        }`}
                        onClick={() => setRegionData(prev => ({ ...prev, priority: priority.value }))}
                      >
                        <div className="flex items-center gap-2">
                          <Badge className={`text-xs ${priority.color}`}>
                            {priority.label}
                          </Badge>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Step 2: Coordinates */}
          {currentStep === 2 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Target className="h-5 w-5" />
                    Region Coordinates
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label htmlFor="center-x">Center X</Label>
                      <Input
                        id="center-x"
                        type="number"
                        value={regionData.centerX}
                        onChange={(e) => setRegionData(prev => ({ ...prev, centerX: parseInt(e.target.value) || 0 }))}
                      />
                    </div>
                    <div>
                      <Label htmlFor="center-z">Center Z</Label>
                      <Input
                        id="center-z"
                        type="number"
                        value={regionData.centerZ}
                        onChange={(e) => setRegionData(prev => ({ ...prev, centerZ: parseInt(e.target.value) || 0 }))}
                      />
                    </div>
                  </div>
                  
                  <div>
                    <Label htmlFor="radius">Radius (blocks)</Label>
                    <Input
                      id="radius"
                      type="number"
                      min="1"
                      max="10000"
                      value={regionData.radius}
                      onChange={(e) => setRegionData(prev => ({ ...prev, radius: parseInt(e.target.value) || 100 }))}
                    />
                    <p className="text-xs text-muted-foreground mt-1">
                      Larger radius = more chunks to generate
                    </p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Database className="h-5 w-5" />
                    Region Statistics
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label className="text-sm font-medium">Total Chunks</Label>
                      <p className="text-2xl font-bold text-primary">
                        {calculateTotalChunks().toLocaleString()}
                      </p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Estimated Time</Label>
                      <p className="text-2xl font-bold text-primary">
                        {Math.ceil(calculateTotalChunks() / (regionData.maxChunksPerSecond * 60))} min
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Step 3: Settings */}
          {currentStep === 3 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Performance Settings
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label htmlFor="max-chunks-per-second">Max Chunks Per Second</Label>
                    <Select 
                      value={regionData.maxChunksPerSecond.toString()} 
                      onValueChange={(value) => setRegionData(prev => ({ ...prev, maxChunksPerSecond: parseInt(value) }))}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="10">10 chunks/s (Low)</SelectItem>
                        <SelectItem value="25">25 chunks/s (Medium)</SelectItem>
                        <SelectItem value="50">50 chunks/s (High)</SelectItem>
                        <SelectItem value="100">100 chunks/s (Maximum)</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div>
                    <Label htmlFor="memory-limit">Memory Limit (MB)</Label>
                    <Select 
                      value={regionData.memoryLimit.toString()} 
                      onValueChange={(value) => setRegionData(prev => ({ ...prev, memoryLimit: parseInt(value) }))}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="1024">1 GB</SelectItem>
                        <SelectItem value="2048">2 GB</SelectItem>
                        <SelectItem value="4096">4 GB</SelectItem>
                        <SelectItem value="8192">8 GB</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Zap className="h-5 w-5" />
                    GPU Acceleration
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center space-x-2">
                    <Checkbox 
                      id="gpu-accelerated"
                      checked={regionData.gpuAccelerated}
                      onCheckedChange={(checked) => setRegionData(prev => ({ ...prev, gpuAccelerated: !!checked }))}
                    />
                    <Label htmlFor="gpu-accelerated">Enable GPU acceleration</Label>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    GPU acceleration can significantly speed up chunk generation
                  </p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Activity className="h-5 w-5" />
                    Behavior Options
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-3">
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="auto-start"
                        checked={regionData.autoStart}
                        onCheckedChange={(checked) => setRegionData(prev => ({ ...prev, autoStart: !!checked }))}
                      />
                      <Label htmlFor="auto-start">Start immediately after creation</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="pause-on-low-tps"
                        checked={regionData.pauseOnLowTPS}
                        onCheckedChange={(checked) => setRegionData(prev => ({ ...prev, pauseOnLowTPS: !!checked }))}
                      />
                      <Label htmlFor="pause-on-low-tps">Pause when TPS drops below 18</Label>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Plus className="h-5 w-5" />
                    Tags
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center gap-2">
                    <Input
                      placeholder="Add tag"
                      onKeyPress={(e) => {
                        if (e.key === 'Enter') {
                          handleTagAdd(e.currentTarget.value);
                          e.currentTarget.value = '';
                        }
                      }}
                    />
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={(e) => {
                        const input = e.currentTarget.previousElementSibling as HTMLInputElement;
                        handleTagAdd(input.value);
                        input.value = '';
                      }}
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  
                  {regionData.tags.length > 0 && (
                    <div className="flex flex-wrap gap-2">
                      {regionData.tags.map((tag, index) => (
                        <Badge key={index} variant="outline" className="flex items-center gap-1">
                          {tag}
                          <button
                            onClick={() => handleTagRemove(tag)}
                            className="ml-1 hover:text-red-500"
                          >
                            <X className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          )}

          {/* Step 4: Review */}
          {currentStep === 4 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <CheckCircle className="h-5 w-5" />
                    Region Summary
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label className="text-sm font-medium">Name</Label>
                      <p className="text-sm text-muted-foreground">{regionData.name}</p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Dimension</Label>
                      <div className="flex items-center gap-2">
                        {getDimensionIcon(regionData.dimension)}
                        <span className="text-sm">{regionData.dimension}</span>
                      </div>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Priority</Label>
                      <Badge className={`text-xs ${priorities.find(p => p.value === regionData.priority)?.color}`}>
                        {regionData.priority}
                      </Badge>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Center</Label>
                      <p className="text-sm text-muted-foreground">
                        {regionData.centerX}, {regionData.centerZ}
                      </p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Radius</Label>
                      <p className="text-sm text-muted-foreground">{regionData.radius} blocks</p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Total Chunks</Label>
                      <p className="text-sm text-muted-foreground">
                        {calculateTotalChunks().toLocaleString()}
                      </p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">GPU Acceleration</Label>
                      <p className="text-sm text-muted-foreground">
                        {regionData.gpuAccelerated ? 'Enabled' : 'Disabled'}
                      </p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Max Speed</Label>
                      <p className="text-sm text-muted-foreground">
                        {regionData.maxChunksPerSecond} chunks/s
                      </p>
                    </div>
                  </div>
                  
                  {regionData.description && (
                    <div>
                      <Label className="text-sm font-medium">Description</Label>
                      <p className="text-sm text-muted-foreground">{regionData.description}</p>
                    </div>
                  )}
                  
                  {regionData.tags.length > 0 && (
                    <div>
                      <Label className="text-sm font-medium">Tags</Label>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {regionData.tags.map((tag, index) => (
                          <Badge key={index} variant="outline" className="text-xs">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
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
                onClick={handleCreate}
                disabled={isCreating || !regionData.name || !regionData.radius}
              >
                {isCreating ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                    Creating...
                  </>
                ) : (
                  <>
                    <Map className="h-4 w-4 mr-2" />
                    Create Region
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

export default RegionSelector;
