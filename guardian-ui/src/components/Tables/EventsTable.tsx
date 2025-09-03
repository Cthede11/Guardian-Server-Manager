import React, { useState } from 'react';
import { 
  Calendar, 
  Play, 
  Pause,
  Square,
  MoreHorizontal,
  Edit,
  Trash2,
  Clock,
  Timer,
  Repeat,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Zap,
  Server,
  Users,
  HardDrive,
  Settings,
  Target,
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

interface EventsTableProps {
  events: any[];
  searchQuery: string;
  filterStatus: string;
  filterType: string;
  onEventAction: (eventId: string, action: string) => void;
  onDelete: (eventId: string) => void;
  onEdit: (eventId: string) => void;
  className?: string;
}

export const EventsTable: React.FC<EventsTableProps> = ({
  events,
  searchQuery,
  filterStatus,
  filterType,
  onEventAction,
  onDelete,
  onEdit,
  className = ''
}) => {
  const [sortBy, setSortBy] = useState('scheduledAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');

  // Filter and sort events
  const filteredEvents = events
    .filter(event => {
      const matchesSearch = event.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           event.description.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesStatus = filterStatus === 'all' || event.status === filterStatus;
      const matchesType = filterType === 'all' || event.type === filterType;
      
      return matchesSearch && matchesStatus && matchesType;
    })
    .sort((a, b) => {
      let aValue = a[sortBy];
      let bValue = b[sortBy];
      
      if (sortBy === 'scheduledAt' || sortBy === 'createdAt') {
        aValue = new Date(aValue).getTime();
        bValue = new Date(bValue).getTime();
      } else if (sortBy === 'name') {
        aValue = aValue.toLowerCase();
        bValue = bValue.toLowerCase();
      }
      
      if (sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'scheduled':
        return 'text-blue-400';
      case 'running':
        return 'text-green-400';
      case 'completed':
        return 'text-gray-400';
      case 'failed':
        return 'text-red-400';
      case 'cancelled':
        return 'text-yellow-400';
      case 'paused':
        return 'text-orange-400';
      default:
        return 'text-gray-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'scheduled':
        return <Clock className="h-4 w-4" />;
      case 'running':
        return <Play className="h-4 w-4" />;
      case 'completed':
        return <CheckCircle className="h-4 w-4" />;
      case 'failed':
        return <XCircle className="h-4 w-4" />;
      case 'cancelled':
        return <Square className="h-4 w-4" />;
      case 'paused':
        return <Pause className="h-4 w-4" />;
      default:
        return <Clock className="h-4 w-4" />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'backup':
        return 'bg-blue-500/20 text-blue-400';
      case 'restart':
        return 'bg-green-500/20 text-green-400';
      case 'maintenance':
        return 'bg-yellow-500/20 text-yellow-400';
      case 'update':
        return 'bg-purple-500/20 text-purple-400';
      case 'custom':
        return 'bg-gray-500/20 text-gray-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'low':
        return 'bg-gray-500/20 text-gray-400';
      case 'normal':
        return 'bg-blue-500/20 text-blue-400';
      case 'high':
        return 'bg-yellow-500/20 text-yellow-400';
      case 'critical':
        return 'bg-red-500/20 text-red-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'backup':
        return <HardDrive className="h-4 w-4" />;
      case 'restart':
        return <Server className="h-4 w-4" />;
      case 'maintenance':
        return <Settings className="h-4 w-4" />;
      case 'update':
        return <Zap className="h-4 w-4" />;
      case 'custom':
        return <Target className="h-4 w-4" />;
      default:
        return <Calendar className="h-4 w-4" />;
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const getRelativeTime = (timestamp: number) => {
    const now = Date.now();
    const diff = timestamp - now;
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);
    
    if (days > 0) {
      return `in ${days} day${days > 1 ? 's' : ''}`;
    } else if (hours > 0) {
      return `in ${hours} hour${hours > 1 ? 's' : ''}`;
    } else if (diff > 0) {
      return 'soon';
    } else {
      return 'overdue';
    }
  };

  const formatDuration = (minutes: number) => {
    if (minutes < 60) {
      return `${minutes}m`;
    } else {
      const hours = Math.floor(minutes / 60);
      const mins = minutes % 60;
      return `${hours}h ${mins}m`;
    }
  };

  if (filteredEvents.length === 0) {
    return (
      <Card>
        <CardContent className="text-center py-12">
          <Calendar className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p className="text-muted-foreground">
            {searchQuery || filterStatus !== 'all' || filterType !== 'all'
              ? 'No events found matching your criteria' 
              : 'No events scheduled'}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            Create your first event to get started
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
            <Calendar className="h-5 w-5" />
            Events ({filteredEvents.length})
          </CardTitle>
          
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline">
              <Calendar className="h-4 w-4 mr-2" />
              Export
            </Button>
            <Button size="sm" variant="outline">
              <Timer className="h-4 w-4 mr-2" />
              Bulk Actions
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {filteredEvents.map((event) => (
            <div
              key={event.id}
              className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors"
            >
              <div className="flex items-center gap-4 flex-1">
                {/* Event Icon/Status */}
                <div className="flex items-center gap-2">
                  <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
                    {getTypeIcon(event.type)}
                  </div>
                  
                  {getStatusIcon(event.status)}
                </div>

                {/* Event Info */}
                <div className="flex-1 space-y-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium">{event.name}</h3>
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getTypeColor(event.type)}`}
                    >
                      {event.type}
                    </Badge>
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getPriorityColor(event.priority)}`}
                    >
                      {event.priority}
                    </Badge>
                    {event.repeat && (
                      <Badge variant="outline" className="text-xs text-blue-400">
                        <Repeat className="h-3 w-3 mr-1" />
                        Recurring
                      </Badge>
                    )}
                    {event.tags.length > 0 && (
                      <Badge variant="outline" className="text-xs">
                        {event.tags.join(', ')}
                      </Badge>
                    )}
                  </div>
                  
                  <p className="text-sm text-muted-foreground">
                    {event.description}
                  </p>
                  
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      {formatDate(event.scheduledAt)}
                    </span>
                    <span className="flex items-center gap-1">
                      <Timer className="h-3 w-3" />
                      {formatDuration(event.duration)}
                    </span>
                    <span className="flex items-center gap-1">
                      <Activity className="h-3 w-3" />
                      {getRelativeTime(event.scheduledAt)}
                    </span>
                    {event.command && (
                      <span className="flex items-center gap-1">
                        <Target className="h-3 w-3" />
                        Custom command
                      </span>
                    )}
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center gap-2">
                {event.status === 'scheduled' && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onEventAction(event.id, 'start')}
                  >
                    <Play className="h-4 w-4 mr-1" />
                    Start
                  </Button>
                )}
                
                {event.status === 'running' && (
                  <>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => onEventAction(event.id, 'pause')}
                    >
                      <Pause className="h-4 w-4 mr-1" />
                      Pause
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => onEventAction(event.id, 'stop')}
                    >
                      <Square className="h-4 w-4 mr-1" />
                      Stop
                    </Button>
                  </>
                )}
                
                {event.status === 'paused' && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onEventAction(event.id, 'start')}
                  >
                    <Play className="h-4 w-4 mr-1" />
                    Resume
                  </Button>
                )}
                
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button size="sm" variant="ghost">
                      <MoreHorizontal className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => onEdit(event.id)}>
                      <Edit className="h-4 w-4 mr-2" />
                      Edit
                    </DropdownMenuItem>
                    
                    <DropdownMenuItem onClick={() => onEventAction(event.id, 'complete')}>
                      <CheckCircle className="h-4 w-4 mr-2" />
                      Mark Complete
                    </DropdownMenuItem>
                    
                    <DropdownMenuItem onClick={() => onEventAction(event.id, 'stop')}>
                      <Square className="h-4 w-4 mr-2" />
                      Cancel
                    </DropdownMenuItem>
                    
                    <DropdownMenuSeparator />
                    
                    <DropdownMenuItem 
                      onClick={() => onDelete(event.id)}
                      className="text-red-600"
                    >
                      <Trash2 className="h-4 w-4 mr-2" />
                      Delete
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

export default EventsTable;
