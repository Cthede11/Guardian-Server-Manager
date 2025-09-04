import React from 'react';
import { 
  AlertTriangle, 
  CheckCircle, 
  X, 
  Download,
  Eye,
  MoreHorizontal,
  Zap,
  Package,
  FileText
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

interface ConflictsListProps {
  conflicts: any[];
  onResolve: (conflictId: string) => void;
  onIgnore: (conflictId: string) => void;
  className?: string;
}

export const ConflictsList: React.FC<ConflictsListProps> = ({
  conflicts,
  onResolve,
  onIgnore,
  // className = ''
}) => {
  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'high':
        return 'text-red-400 bg-red-500/20';
      case 'medium':
        return 'text-yellow-400 bg-yellow-500/20';
      case 'low':
        return 'text-blue-400 bg-blue-500/20';
      default:
        return 'text-gray-400 bg-gray-500/20';
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'high':
        return <AlertTriangle className="h-4 w-4" />;
      case 'medium':
        return <AlertTriangle className="h-4 w-4" />;
      case 'low':
        return <CheckCircle className="h-4 w-4" />;
      default:
        return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const getConflictTypeIcon = (type: string) => {
    switch (type) {
      case 'mod_conflict':
        return <Package className="h-4 w-4" />;
      case 'dependency_missing':
        return <Download className="h-4 w-4" />;
      case 'version_mismatch':
        return <FileText className="h-4 w-4" />;
      default:
        return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const getResolutionIcon = (resolution: string) => {
    switch (resolution) {
      case 'disable_one':
        return <X className="h-4 w-4" />;
      case 'install_dependency':
        return <Download className="h-4 w-4" />;
      case 'update_mod':
        return <Zap className="h-4 w-4" />;
      default:
        return <CheckCircle className="h-4 w-4" />;
    }
  };

  if (conflicts.length === 0) {
    return (
      <Card>
        <CardContent className="text-center py-12">
          <CheckCircle className="h-12 w-12 text-green-400 mx-auto mb-4" />
          <p className="text-muted-foreground">No conflicts detected</p>
          <p className="text-xs text-muted-foreground mt-1">
            All mods are compatible and dependencies are satisfied
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertTriangle className="h-5 w-5" />
          Mod Conflicts ({conflicts.length})
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {conflicts.map((conflict) => (
            <div
              key={conflict.id}
              className="p-4 border rounded-lg hover:bg-muted/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 space-y-3">
                  {/* Conflict Header */}
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                      {getConflictTypeIcon(conflict.type)}
                      <h3 className="font-medium">{conflict.type.replace('_', ' ').toUpperCase()}</h3>
                    </div>
                    
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getSeverityColor(conflict.severity)}`}
                    >
                      {getSeverityIcon(conflict.severity)}
                      <span className="ml-1 capitalize">{conflict.severity}</span>
                    </Badge>
                  </div>

                  {/* Affected Mods */}
                  <div className="space-y-1">
                    <h4 className="text-sm font-medium">Affected Mods:</h4>
                    <div className="flex flex-wrap gap-2">
                      {conflict.mods.map((mod: string, index: number) => (
                        <Badge key={index} variant="secondary" className="text-xs">
                          <Package className="h-3 w-3 mr-1" />
                          {mod}
                        </Badge>
                      ))}
                    </div>
                  </div>

                  {/* Description */}
                  <p className="text-sm text-muted-foreground">
                    {conflict.description}
                  </p>

                  {/* Impact */}
                  <div className="space-y-1">
                    <h4 className="text-sm font-medium">Impact:</h4>
                    <p className="text-sm text-muted-foreground">
                      {conflict.impact}
                    </p>
                  </div>

                  {/* Suggested Action */}
                  <div className="space-y-1">
                    <h4 className="text-sm font-medium">Suggested Action:</h4>
                    <div className="flex items-center gap-2">
                      {getResolutionIcon(conflict.resolution)}
                      <span className="text-sm">{conflict.suggestedAction}</span>
                    </div>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 ml-4">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onResolve(conflict.id)}
                    className="text-green-400 border-green-400 hover:bg-green-400 hover:text-white"
                  >
                    <CheckCircle className="h-4 w-4 mr-1" />
                    Resolve
                  </Button>
                  
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button size="sm" variant="ghost">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem>
                        <Eye className="h-4 w-4 mr-2" />
                        View Details
                      </DropdownMenuItem>
                      
                      <DropdownMenuItem>
                        <Download className="h-4 w-4 mr-2" />
                        Auto-Fix
                      </DropdownMenuItem>
                      
                      <DropdownMenuSeparator />
                      
                      <DropdownMenuItem 
                        onClick={() => onIgnore(conflict.id)}
                        className="text-yellow-600"
                      >
                        <X className="h-4 w-4 mr-2" />
                        Ignore
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

export default ConflictsList;
