import React, { useState } from 'react';
import { 
  X, 
  Plus, 
  Trash2, 
  Play, 
  Pause,
  Settings,
  TestTube,
  Activity,
  Zap,
  AlertTriangle,
  CheckCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface LiveRuleLabProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (rule: any) => void;
  className?: string;
}

export const LiveRuleLab: React.FC<LiveRuleLabProps> = ({
  isOpen,
  onClose,
  onSave,
  className = ''
}) => {
  const [ruleName, setRuleName] = useState('');
  const [ruleDescription, setRuleDescription] = useState('');
  const [priority, setPriority] = useState('2');
  const [conditions, setConditions] = useState<any[]>([]);
  const [actions, setActions] = useState<any[]>([]);
  const [isTesting, setIsTesting] = useState(false);

  const conditionTypes = [
    { value: 'tps', label: 'TPS', icon: <Activity className="h-4 w-4" /> },
    { value: 'chunk_entities', label: 'Chunk Entities', icon: <Zap className="h-4 w-4" /> },
    { value: 'redstone_updates', label: 'Redstone Updates', icon: <AlertTriangle className="h-4 w-4" /> },
    { value: 'memory_usage', label: 'Memory Usage', icon: <Settings className="h-4 w-4" /> },
    { value: 'player_count', label: 'Player Count', icon: <CheckCircle className="h-4 w-4" /> }
  ];

  const actionTypes = [
    { value: 'freeze_chunk', label: 'Freeze Chunk', icon: <Pause className="h-4 w-4" /> },
    { value: 'despawn_entities', label: 'Despawn Entities', icon: <Trash2 className="h-4 w-4" /> },
    { value: 'throttle_redstone', label: 'Throttle Redstone', icon: <Zap className="h-4 w-4" /> },
    { value: 'kick_players', label: 'Kick Players', icon: <AlertTriangle className="h-4 w-4" /> },
    { value: 'restart_server', label: 'Restart Server', icon: <Settings className="h-4 w-4" /> }
  ];

  const operators = [
    { value: '>', label: 'Greater than' },
    { value: '<', label: 'Less than' },
    { value: '>=', label: 'Greater than or equal' },
    { value: '<=', label: 'Less than or equal' },
    { value: '==', label: 'Equal to' },
    { value: '!=', label: 'Not equal to' }
  ];

  const addCondition = () => {
    setConditions([...conditions, {
      type: 'tps',
      operator: '<',
      value: '15'
    }]);
  };

  const removeCondition = (index: number) => {
    setConditions(conditions.filter((_, i) => i !== index));
  };

  const updateCondition = (index: number, field: string, value: string) => {
    const updated = [...conditions];
    updated[index] = { ...updated[index], [field]: value };
    setConditions(updated);
  };

  const addAction = () => {
    setActions([...actions, {
      type: 'freeze_chunk',
      duration: '300'
    }]);
  };

  const removeAction = (index: number) => {
    setActions(actions.filter((_, i) => i !== index));
  };

  const updateAction = (index: number, field: string, value: string) => {
    const updated = [...actions];
    updated[index] = { ...updated[index], [field]: value };
    setActions(updated);
  };

  const handleSave = () => {
    if (!ruleName.trim() || conditions.length === 0 || actions.length === 0) {
      return;
    }

    const rule = {
      name: ruleName,
      description: ruleDescription,
      enabled: true,
      priority: parseInt(priority),
      conditions,
      actions,
      lastTriggered: null,
      triggerCount: 0
    };

    onSave(rule);
    handleClose();
  };

  const handleClose = () => {
    setRuleName('');
    setRuleDescription('');
    setPriority('2');
    setConditions([]);
    setActions([]);
    onClose();
  };

  const handleTest = async () => {
    setIsTesting(true);
    // Simulate testing
    await new Promise(resolve => setTimeout(resolve, 2000));
    setIsTesting(false);
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <TestTube className="h-5 w-5" />
            Live Rule Lab
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-6">
          {/* Basic Rule Info */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Rule Information</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label htmlFor="ruleName">Rule Name</Label>
                <Input
                  id="ruleName"
                  placeholder="Enter rule name..."
                  value={ruleName}
                  onChange={(e) => setRuleName(e.target.value)}
                />
              </div>
              
              <div>
                <Label htmlFor="ruleDescription">Description</Label>
                <Textarea
                  id="ruleDescription"
                  placeholder="Describe what this rule does..."
                  value={ruleDescription}
                  onChange={(e) => setRuleDescription(e.target.value)}
                  rows={3}
                />
              </div>
              
              <div>
                <Label htmlFor="priority">Priority</Label>
                <Select value={priority} onValueChange={setPriority}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="1">1 - Critical</SelectItem>
                    <SelectItem value="2">2 - High</SelectItem>
                    <SelectItem value="3">3 - Medium</SelectItem>
                    <SelectItem value="4">4 - Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          {/* Conditions */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">Conditions</CardTitle>
                <Button size="sm" onClick={addCondition}>
                  <Plus className="h-4 w-4 mr-2" />
                  Add Condition
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {conditions.length === 0 ? (
                <p className="text-muted-foreground text-center py-4">
                  No conditions defined. Add conditions to specify when this rule should trigger.
                </p>
              ) : (
                conditions.map((condition, index) => (
                  <div key={index} className="flex items-center gap-4 p-4 border rounded-lg">
                    <div className="flex-1 grid grid-cols-3 gap-4">
                      <div>
                        <Label>Type</Label>
                        <Select
                          value={condition.type}
                          onValueChange={(value) => updateCondition(index, 'type', value)}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {conditionTypes.map(type => (
                              <SelectItem key={type.value} value={type.value}>
                                <div className="flex items-center gap-2">
                                  {type.icon}
                                  {type.label}
                                </div>
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                      
                      <div>
                        <Label>Operator</Label>
                        <Select
                          value={condition.operator}
                          onValueChange={(value) => updateCondition(index, 'operator', value)}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {operators.map(op => (
                              <SelectItem key={op.value} value={op.value}>
                                {op.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                      
                      <div>
                        <Label>Value</Label>
                        <Input
                          type="number"
                          value={condition.value}
                          onChange={(e) => updateCondition(index, 'value', e.target.value)}
                          placeholder="Enter value..."
                        />
                      </div>
                    </div>
                    
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => removeCondition(index)}
                      className="text-red-400 hover:text-red-300"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                ))
              )}
            </CardContent>
          </Card>

          {/* Actions */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">Actions</CardTitle>
                <Button size="sm" onClick={addAction}>
                  <Plus className="h-4 w-4 mr-2" />
                  Add Action
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {actions.length === 0 ? (
                <p className="text-muted-foreground text-center py-4">
                  No actions defined. Add actions to specify what should happen when conditions are met.
                </p>
              ) : (
                actions.map((action, index) => (
                  <div key={index} className="flex items-center gap-4 p-4 border rounded-lg">
                    <div className="flex-1 grid grid-cols-2 gap-4">
                      <div>
                        <Label>Action Type</Label>
                        <Select
                          value={action.type}
                          onValueChange={(value) => updateAction(index, 'type', value)}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {actionTypes.map(type => (
                              <SelectItem key={type.value} value={type.value}>
                                <div className="flex items-center gap-2">
                                  {type.icon}
                                  {type.label}
                                </div>
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                      
                      <div>
                        <Label>Parameter</Label>
                        <Input
                          type="number"
                          value={action.duration || action.count || action.factor || ''}
                          onChange={(e) => {
                            const field = action.type === 'freeze_chunk' ? 'duration' :
                                         action.type === 'despawn_entities' ? 'count' :
                                         action.type === 'throttle_redstone' ? 'factor' : 'value';
                            updateAction(index, field, e.target.value);
                          }}
                          placeholder="Enter parameter..."
                        />
                      </div>
                    </div>
                    
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => removeAction(index)}
                      className="text-red-400 hover:text-red-300"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                ))
              )}
            </CardContent>
          </Card>

          {/* Actions */}
          <div className="flex items-center justify-between">
            <Button
              variant="outline"
              onClick={handleTest}
              disabled={isTesting || conditions.length === 0 || actions.length === 0}
            >
              {isTesting ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                  Testing...
                </>
              ) : (
                <>
                  <TestTube className="h-4 w-4 mr-2" />
                  Test Rule
                </>
              )}
            </Button>
            
            <div className="flex items-center gap-2">
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                onClick={handleSave}
                disabled={!ruleName.trim() || conditions.length === 0 || actions.length === 0}
              >
                Save Rule
              </Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default LiveRuleLab;
