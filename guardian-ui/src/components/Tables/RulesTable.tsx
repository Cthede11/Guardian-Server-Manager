import React, { useState } from 'react';
import { 
  Shield, 
  Settings, 
  Edit, 
  Trash2,
  MoreHorizontal,
  Plus,
  Play,
  Pause,
  Clock,
  Zap,
  AlertTriangle,
  CheckCircle,
  Activity
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';
import { Switch } from '@/components/ui/switch';

interface RulesTableProps {
  rules: any[];
  searchQuery: string;
  onRuleToggle: (ruleId: string) => void;
  onRuleEdit: (ruleId: string) => void;
  onRuleDelete: (ruleId: string) => void;
  className?: string;
}

export const RulesTable: React.FC<RulesTableProps> = ({
  rules,
  searchQuery,
  onRuleToggle,
  onRuleEdit,
  onRuleDelete,
}) => {
  const [sortBy] = useState('priority');
  const [sortOrder] = useState<'asc' | 'desc'>('asc');

  // Filter and sort rules
  const filteredRules = rules
    .filter(rule => {
      return rule.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
             rule.description.toLowerCase().includes(searchQuery.toLowerCase());
    })
    .sort((a, b) => {
      let aValue = a[sortBy];
      let bValue = b[sortBy];
      
      if (sortBy === 'name') {
        aValue = aValue.toLowerCase();
        bValue = bValue.toLowerCase();
      }
      
      if (sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

  const getPriorityColor = (priority: number) => {
    if (priority === 1) return 'bg-red-500/20 text-red-400';
    if (priority === 2) return 'bg-yellow-500/20 text-yellow-400';
    if (priority === 3) return 'bg-blue-500/20 text-blue-400';
    return 'bg-gray-500/20 text-gray-400';
  };

  const getConditionIcon = (type: string) => {
    switch (type) {
      case 'tps':
        return <Activity className="h-3 w-3" />;
      case 'chunk_entities':
        return <Zap className="h-3 w-3" />;
      case 'redstone_updates':
        return <AlertTriangle className="h-3 w-3" />;
      default:
        return <CheckCircle className="h-3 w-3" />;
    }
  };

  const getActionIcon = (type: string) => {
    switch (type) {
      case 'freeze_chunk':
        return <Pause className="h-3 w-3" />;
      case 'despawn_entities':
        return <Trash2 className="h-3 w-3" />;
      case 'throttle_redstone':
        return <Zap className="h-3 w-3" />;
      default:
        return <Settings className="h-3 w-3" />;
    }
  };

  const formatDate = (date: Date | null) => {
    if (!date) return 'Never';
    return new Date(date).toLocaleString();
  };

  if (filteredRules.length === 0) {
    return (
      <Card>
        <CardContent className="text-center py-12">
          <Shield className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p className="text-muted-foreground">
            {searchQuery 
              ? 'No rules found matching your search' 
              : 'No rules configured'}
          </p>
          <Button className="mt-4">
            <Plus className="h-4 w-4 mr-2" />
            Create Rule
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Performance Rules ({filteredRules.length})
          </CardTitle>
          
          <Button>
            <Plus className="h-4 w-4 mr-2" />
            Create Rule
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {filteredRules.map((rule) => (
            <div
              key={rule.id}
              className="p-4 border rounded-lg hover:bg-muted/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 space-y-3">
                  {/* Rule Header */}
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                      {rule.enabled ? (
                        <Play className="h-4 w-4 text-green-400" />
                      ) : (
                        <Pause className="h-4 w-4 text-gray-400" />
                      )}
                      <h3 className="font-medium">{rule.name}</h3>
                    </div>
                    
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getPriorityColor(rule.priority)}`}
                    >
                      Priority {rule.priority}
                    </Badge>
                    
                    <Badge variant="outline" className="text-xs">
                      {rule.triggerCount} triggers
                    </Badge>
                  </div>

                  {/* Rule Description */}
                  <p className="text-sm text-muted-foreground">
                    {rule.description}
                  </p>

                  {/* Conditions */}
                  <div className="space-y-2">
                    <h4 className="text-sm font-medium">Conditions:</h4>
                    <div className="flex flex-wrap gap-2">
                      {rule.conditions.map((condition: any, index: number) => (
                        <Badge key={index} variant="secondary" className="text-xs">
                          {getConditionIcon(condition.type)}
                          <span className="ml-1">
                            {condition.type} {condition.operator} {condition.value}
                          </span>
                        </Badge>
                      ))}
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="space-y-2">
                    <h4 className="text-sm font-medium">Actions:</h4>
                    <div className="flex flex-wrap gap-2">
                      {rule.actions.map((action: any, index: number) => (
                        <Badge key={index} variant="outline" className="text-xs">
                          {getActionIcon(action.type)}
                          <span className="ml-1">
                            {action.type}
                            {action.duration && ` (${action.duration}s)`}
                            {action.count && ` (${action.count})`}
                            {action.factor && ` (${action.factor}x)`}
                          </span>
                        </Badge>
                      ))}
                    </div>
                  </div>

                  {/* Last Triggered */}
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      Last triggered: {formatDate(rule.lastTriggered)}
                    </span>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 ml-4">
                  <Switch
                    checked={rule.enabled}
                    onCheckedChange={() => onRuleToggle(rule.id)}
                  />
                  
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button size="sm" variant="ghost">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => onRuleEdit(rule.id)}>
                        <Edit className="h-4 w-4 mr-2" />
                        Edit Rule
                      </DropdownMenuItem>
                      
                      <DropdownMenuItem>
                        <Settings className="h-4 w-4 mr-2" />
                        Test Rule
                      </DropdownMenuItem>
                      
                      <DropdownMenuSeparator />
                      
                      <DropdownMenuItem 
                        onClick={() => onRuleDelete(rule.id)}
                        className="text-red-600"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Delete Rule
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

export default RulesTable;
