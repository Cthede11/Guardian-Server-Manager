import React, { useState } from 'react';
import { 
  X, 
  Calendar, 
  Clock,
  Timer,
  Repeat,
  Target,
  Settings,
  HardDrive,
  Server,
  Zap,
  Users,
  AlertTriangle,
  CheckCircle,
  Plus,
  Minus
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

interface CreateEventModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (eventData: any) => void;
  className?: string;
}

export const CreateEventModal: React.FC<CreateEventModalProps> = ({
  isOpen,
  onClose,
  onCreate,
  className = ''
}) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [isCreating, setIsCreating] = useState(false);
  
  // Event data
  const [eventData, setEventData] = useState({
    name: '',
    type: 'custom',
    priority: 'normal',
    scheduledAt: '',
    duration: 60,
    description: '',
    command: '',
    repeat: false,
    repeatInterval: 'daily',
    repeatCount: 1,
    tags: [] as string[],
    notifyPlayers: true,
    backupBefore: false,
    stopServer: false
  });

  const steps = [
    { id: 1, title: 'Basic Info', description: 'Event name and type' },
    { id: 2, title: 'Schedule', description: 'When and how often' },
    { id: 3, title: 'Configuration', description: 'Commands and settings' },
    { id: 4, title: 'Review', description: 'Confirm and create' }
  ];

  const eventTypes = [
    { value: 'backup', label: 'Backup', icon: <HardDrive className="h-4 w-4" />, description: 'Create server backup' },
    { value: 'restart', label: 'Restart', icon: <Server className="h-4 w-4" />, description: 'Restart server' },
    { value: 'maintenance', label: 'Maintenance', icon: <Settings className="h-4 w-4" />, description: 'Server maintenance' },
    { value: 'update', label: 'Update', icon: <Zap className="h-4 w-4" />, description: 'Update server/mods' },
    { value: 'custom', label: 'Custom', icon: <Target className="h-4 w-4" />, description: 'Custom command' }
  ];

  const priorities = [
    { value: 'low', label: 'Low', color: 'bg-gray-500/20 text-gray-400' },
    { value: 'normal', label: 'Normal', color: 'bg-blue-500/20 text-blue-400' },
    { value: 'high', label: 'High', color: 'bg-yellow-500/20 text-yellow-400' },
    { value: 'critical', label: 'Critical', color: 'bg-red-500/20 text-red-400' }
  ];

  const repeatIntervals = [
    { value: 'hourly', label: 'Hourly' },
    { value: 'daily', label: 'Daily' },
    { value: 'weekly', label: 'Weekly' },
    { value: 'monthly', label: 'Monthly' }
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
      onCreate(eventData);
    } catch (error) {
      console.error('Error creating event:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleClose = () => {
    setCurrentStep(1);
    setEventData({
      name: '',
      type: 'custom',
      priority: 'normal',
      scheduledAt: '',
      duration: 60,
      description: '',
      command: '',
      repeat: false,
      repeatInterval: 'daily',
      repeatCount: 1,
      tags: [],
      notifyPlayers: true,
      backupBefore: false,
      stopServer: false
    });
    onClose();
  };

  const handleTagAdd = (tag: string) => {
    if (tag && !eventData.tags.includes(tag)) {
      setEventData(prev => ({
        ...prev,
        tags: [...prev.tags, tag]
      }));
    }
  };

  const handleTagRemove = (tag: string) => {
    setEventData(prev => ({
      ...prev,
      tags: prev.tags.filter(t => t !== tag)
    }));
  };

  const formatDate = (dateString: string) => {
    if (!dateString) return 'Not set';
    return new Date(dateString).toLocaleString();
  };

  const getTypeIcon = (type: string) => {
    const eventType = eventTypes.find(t => t.value === type);
    return eventType?.icon || <Target className="h-4 w-4" />;
  };

  const getTypeDescription = (type: string) => {
    const eventType = eventTypes.find(t => t.value === type);
    return eventType?.description || 'Custom event';
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            Create Event
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
          {/* Step 1: Basic Info */}
          {currentStep === 1 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Target className="h-5 w-5" />
                    Event Details
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label htmlFor="event-name">Event Name</Label>
                    <Input
                      id="event-name"
                      placeholder="Enter event name"
                      value={eventData.name}
                      onChange={(e) => setEventData(prev => ({ ...prev, name: e.target.value }))}
                    />
                  </div>
                  
                  <div>
                    <Label htmlFor="event-description">Description</Label>
                    <Textarea
                      id="event-description"
                      placeholder="Describe what this event does"
                      value={eventData.description}
                      onChange={(e) => setEventData(prev => ({ ...prev, description: e.target.value }))}
                    />
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Event Type
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 gap-3">
                    {eventTypes.map((type) => (
                      <div
                        key={type.value}
                        className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                          eventData.type === type.value 
                            ? 'border-primary bg-primary/10' 
                            : 'border-muted hover:border-muted-foreground'
                        }`}
                        onClick={() => setEventData(prev => ({ ...prev, type: type.value }))}
                      >
                        <div className="flex items-center gap-3">
                          {type.icon}
                          <div>
                            <p className="font-medium">{type.label}</p>
                            <p className="text-sm text-muted-foreground">{type.description}</p>
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
                          eventData.priority === priority.value 
                            ? 'border-primary bg-primary/10' 
                            : 'border-muted hover:border-muted-foreground'
                        }`}
                        onClick={() => setEventData(prev => ({ ...prev, priority: priority.value }))}
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

          {/* Step 2: Schedule */}
          {currentStep === 2 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Clock className="h-5 w-5" />
                    Schedule
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label htmlFor="scheduled-at">Scheduled Date & Time</Label>
                    <Input
                      id="scheduled-at"
                      type="datetime-local"
                      value={eventData.scheduledAt}
                      onChange={(e) => setEventData(prev => ({ ...prev, scheduledAt: e.target.value }))}
                    />
                  </div>
                  
                  <div>
                    <Label htmlFor="duration">Duration (minutes)</Label>
                    <Input
                      id="duration"
                      type="number"
                      min="1"
                      max="1440"
                      value={eventData.duration}
                      onChange={(e) => setEventData(prev => ({ ...prev, duration: parseInt(e.target.value) || 60 }))}
                    />
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Repeat className="h-5 w-5" />
                    Recurrence
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center space-x-2">
                    <Checkbox 
                      id="repeat"
                      checked={eventData.repeat}
                      onCheckedChange={(checked) => setEventData(prev => ({ ...prev, repeat: !!checked }))}
                    />
                    <Label htmlFor="repeat">Make this event recurring</Label>
                  </div>
                  
                  {eventData.repeat && (
                    <div className="space-y-4">
                      <div>
                        <Label htmlFor="repeat-interval">Repeat Interval</Label>
                        <Select 
                          value={eventData.repeatInterval} 
                          onValueChange={(value) => setEventData(prev => ({ ...prev, repeatInterval: value }))}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {repeatIntervals.map((interval) => (
                              <SelectItem key={interval.value} value={interval.value}>
                                {interval.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                      
                      <div>
                        <Label htmlFor="repeat-count">Repeat Count (0 = infinite)</Label>
                        <Input
                          id="repeat-count"
                          type="number"
                          min="0"
                          value={eventData.repeatCount}
                          onChange={(e) => setEventData(prev => ({ ...prev, repeatCount: parseInt(e.target.value) || 0 }))}
                        />
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          )}

          {/* Step 3: Configuration */}
          {currentStep === 3 && (
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Target className="h-5 w-5" />
                    Command Configuration
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label htmlFor="command">Command to Execute</Label>
                    <Input
                      id="command"
                      placeholder="Enter command (e.g., /say Server restart in 5 minutes)"
                      value={eventData.command}
                      onChange={(e) => setEventData(prev => ({ ...prev, command: e.target.value }))}
                    />
                    <p className="text-xs text-muted-foreground mt-1">
                      Leave empty for automatic actions based on event type
                    </p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Event Settings
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-3">
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="notify-players"
                        checked={eventData.notifyPlayers}
                        onCheckedChange={(checked) => setEventData(prev => ({ ...prev, notifyPlayers: !!checked }))}
                      />
                      <Label htmlFor="notify-players">Notify players before event</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="backup-before"
                        checked={eventData.backupBefore}
                        onCheckedChange={(checked) => setEventData(prev => ({ ...prev, backupBefore: !!checked }))}
                      />
                      <Label htmlFor="backup-before">Create backup before event</Label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <Checkbox 
                        id="stop-server"
                        checked={eventData.stopServer}
                        onCheckedChange={(checked) => setEventData(prev => ({ ...prev, stopServer: !!checked }))}
                      />
                      <Label htmlFor="stop-server">Stop server during event</Label>
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
                  
                  {eventData.tags.length > 0 && (
                    <div className="flex flex-wrap gap-2">
                      {eventData.tags.map((tag, index) => (
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
                    Event Summary
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label className="text-sm font-medium">Name</Label>
                      <p className="text-sm text-muted-foreground">{eventData.name}</p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Type</Label>
                      <div className="flex items-center gap-2">
                        {getTypeIcon(eventData.type)}
                        <span className="text-sm">{eventData.type}</span>
                      </div>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Priority</Label>
                      <Badge className={`text-xs ${priorities.find(p => p.value === eventData.priority)?.color}`}>
                        {eventData.priority}
                      </Badge>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Scheduled</Label>
                      <p className="text-sm text-muted-foreground">{formatDate(eventData.scheduledAt)}</p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Duration</Label>
                      <p className="text-sm text-muted-foreground">{eventData.duration} minutes</p>
                    </div>
                    <div>
                      <Label className="text-sm font-medium">Recurring</Label>
                      <p className="text-sm text-muted-foreground">
                        {eventData.repeat ? `${eventData.repeatInterval} (${eventData.repeatCount === 0 ? 'infinite' : eventData.repeatCount} times)` : 'No'}
                      </p>
                    </div>
                  </div>
                  
                  {eventData.description && (
                    <div>
                      <Label className="text-sm font-medium">Description</Label>
                      <p className="text-sm text-muted-foreground">{eventData.description}</p>
                    </div>
                  )}
                  
                  {eventData.command && (
                    <div>
                      <Label className="text-sm font-medium">Command</Label>
                      <p className="text-sm text-muted-foreground font-mono bg-muted p-2 rounded">
                        {eventData.command}
                      </p>
                    </div>
                  )}
                  
                  {eventData.tags.length > 0 && (
                    <div>
                      <Label className="text-sm font-medium">Tags</Label>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {eventData.tags.map((tag, index) => (
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
                disabled={isCreating || !eventData.name || !eventData.scheduledAt}
              >
                {isCreating ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                    Creating...
                  </>
                ) : (
                  <>
                    <Calendar className="h-4 w-4 mr-2" />
                    Create Event
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

export default CreateEventModal;
