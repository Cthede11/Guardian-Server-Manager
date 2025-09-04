import React, { useState } from 'react';
import { 
  Package, 
  Settings, 
  ToggleLeft, 
  MoreHorizontal,
  Download,
  Upload,
  Trash2,
  Eye,
  AlertTriangle,
  CheckCircle,
  Clock,
  MemoryStick
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

interface ModsTableProps {
  mods: any[];
  searchQuery: string;
  filterType: string;
  onModToggle: (modId: string) => void;
  onModConfigure: (modId: string) => void;
  className?: string;
}

export const ModsTable: React.FC<ModsTableProps> = ({
  mods,
  searchQuery,
  filterType,
  onModToggle,
  onModConfigure,
}) => {
  const [sortBy] = useState('name');
  const [sortOrder] = useState<'asc' | 'desc'>('asc');

  // Filter and sort mods
  const filteredMods = mods
    .filter(mod => {
      const matchesSearch = mod.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           mod.description.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesFilter = filterType === 'all' ||
                           (filterType === 'enabled' && mod.status === 'enabled') ||
                           (filterType === 'disabled' && mod.status === 'disabled') ||
                           (filterType === 'conflicts' && mod.conflicts.length > 0);
      
      return matchesSearch && matchesFilter;
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

  // Unused function removed
  // const getStatusColor = (status: string) => {
  //   switch (status) {
  //     case 'enabled':
  //       return 'text-green-400';
  //     case 'disabled':
  //       return 'text-gray-400';
  //     case 'error':
  //       return 'text-red-400';
  //     default:
  //       return 'text-gray-400';
  //   }
  // };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'content':
        return 'bg-blue-500/20 text-blue-400';
      case 'utility':
        return 'bg-green-500/20 text-green-400';
      case 'optimization':
        return 'bg-purple-500/20 text-purple-400';
      case 'library':
        return 'bg-yellow-500/20 text-yellow-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  const formatBytes = (bytes: number) => {
    return `${bytes.toFixed(1)} MB`;
  };

  const formatDate = (date: Date) => {
    return new Date(date).toLocaleDateString();
  };

  if (filteredMods.length === 0) {
    return (
      <Card>
        <CardContent className="text-center py-12">
          <Package className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p className="text-muted-foreground">
            {searchQuery || filterType !== 'all' 
              ? 'No mods found matching your criteria' 
              : 'No mods installed'}
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Package className="h-5 w-5" />
            Installed Mods ({filteredMods.length})
          </CardTitle>
          
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline">
              <Download className="h-4 w-4 mr-2" />
              Install Mod
            </Button>
            <Button size="sm" variant="outline">
              <Upload className="h-4 w-4 mr-2" />
              Export List
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {filteredMods.map((mod) => (
            <div
              key={mod.id}
              className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors"
            >
              <div className="flex items-center gap-4 flex-1">
                {/* Mod Icon/Status */}
                <div className="flex items-center gap-2">
                  <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
                    <Package className="h-5 w-5" />
                  </div>
                  
                  {mod.status === 'enabled' ? (
                    <CheckCircle className="h-4 w-4 text-green-400" />
                  ) : (
                    <ToggleLeft className="h-4 w-4 text-gray-400" />
                  )}
                </div>

                {/* Mod Info */}
                <div className="flex-1 space-y-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium">{mod.name}</h3>
                    <Badge variant="outline" className="text-xs">
                      {mod.version}
                    </Badge>
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getCategoryColor(mod.category)}`}
                    >
                      {mod.category}
                    </Badge>
                    {mod.conflicts.length > 0 && (
                      <Badge variant="destructive" className="text-xs">
                        <AlertTriangle className="h-3 w-3 mr-1" />
                        Conflicts
                      </Badge>
                    )}
                  </div>
                  
                  <p className="text-sm text-muted-foreground">
                    {mod.description}
                  </p>
                  
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <span>by {mod.author}</span>
                    {mod.status === 'enabled' && (
                      <>
                        <span className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {mod.loadTime}ms
                        </span>
                        <span className="flex items-center gap-1">
                          <MemoryStick className="h-3 w-3" />
                          {formatBytes(mod.memoryUsage)}
                        </span>
                      </>
                    )}
                    <span>Updated {formatDate(mod.lastUpdated)}</span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center gap-2">
                <Switch
                  checked={mod.status === 'enabled'}
                  onCheckedChange={() => onModToggle(mod.id)}
                />
                
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button size="sm" variant="ghost">
                      <MoreHorizontal className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => onModConfigure(mod.id)}>
                      <Settings className="h-4 w-4 mr-2" />
                      Configure
                    </DropdownMenuItem>
                    
                    <DropdownMenuItem>
                      <Eye className="h-4 w-4 mr-2" />
                      View Details
                    </DropdownMenuItem>
                    
                    <DropdownMenuSeparator />
                    
                    <DropdownMenuItem>
                      <Download className="h-4 w-4 mr-2" />
                      Update
                    </DropdownMenuItem>
                    
                    <DropdownMenuItem className="text-red-600">
                      <Trash2 className="h-4 w-4 mr-2" />
                      Uninstall
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

export default ModsTable;
