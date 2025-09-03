import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { 
  Calendar, 
  Plus, 
  Clock,
  RefreshCw,
  Search,
  Filter,
  Play,
  Pause,
  Square,
  Edit,
  Trash2,
  MoreHorizontal,
  Bell,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Zap,
  Server,
  Users,
  HardDrive,
  Settings,
  Repeat,
  Timer,
  Target,
  Activity
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';
import { useServersStore } from '@/store/servers';
import { EventsTable } from '@/components/Tables/EventsTable';
import { CreateEventModal } from '@/components/Events/CreateEventModal';

interface EventsPageProps {
  className?: string;
}

export const Events: React.FC<EventsPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [events, setEvents] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState('all');
  const [filterType, setFilterType] = useState('all');
  const [createEventOpen, setCreateEventOpen] = useState(false);

  // Fetch events data
  const fetchEvents = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const response = await fetch(`http://localhost:8080/api/v1/servers/${serverId}/events`);
      if (response.ok) {
        const data = await response.json();
        setEvents(data);
      } else {
        // Use mock data for demo
        setEvents(generateMockEvents());
      }
    } catch (error) {
      console.error('Error fetching events:', error);
      // Use mock data for demo
      setEvents(generateMockEvents());
    } finally {
      setIsLoading(false);
    }
  };

  // Generate mock events for demo
  const generateMockEvents = () => {
    const now = Date.now();
    const events = [];
    
    // Generate various types of events
    const types = ['backup', 'restart', 'maintenance', 'update', 'custom'];
    const statuses = ['scheduled', 'running', 'completed', 'failed', 'cancelled'];
    const priorities = ['low', 'normal', 'high', 'critical'];
    
    for (let i = 0; i < 20; i++) {
      const type = types[Math.floor(Math.random() * types.length)];
      const status = statuses[Math.floor(Math.random() * statuses.length)];
      const priority = priorities[Math.floor(Math.random() * priorities.length)];
      const timestamp = now + (i * 3600000) + Math.random() * 86400000; // Future events
      
      events.push({
        id: `event-${i + 1}`,
        name: `${type.charAt(0).toUpperCase() + type.slice(1)} Event ${i + 1}`,
        type,
        status,
        priority,
        scheduledAt: timestamp,
        duration: Math.floor(Math.random() * 120) + 30, // 30-150 minutes
        description: `Scheduled ${type} event for server maintenance`,
        command: type === 'custom' ? `/say Server maintenance in progress` : null,
        repeat: Math.random() > 0.7, // 30% are recurring
        repeatInterval: Math.random() > 0.7 ? 'daily' : 'weekly',
        lastRun: status === 'completed' ? timestamp - 86400000 : null,
        nextRun: status === 'scheduled' ? timestamp : null,
        createdBy: 'admin',
        createdAt: timestamp - 86400000,
        tags: type === 'maintenance' ? ['maintenance', 'scheduled'] : type === 'update' ? ['update', 'mod-update'] : []
      });
    }
    
    return events.sort((a, b) => a.scheduledAt - b.scheduledAt);
  };

  useEffect(() => {
    fetchEvents();
    
    // Refresh data every 30 seconds
    const interval = setInterval(fetchEvents, 30000);
    return () => clearInterval(interval);
  }, [serverId]);

  const handleCreateEvent = async (eventData: any) => {
    try {
      // Simulate creating an event
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const newEvent = {
        id: `event-${Date.now()}`,
        ...eventData,
        status: 'scheduled',
        createdAt: Date.now(),
        createdBy: 'admin'
      };
      
      setEvents(prev => [newEvent, ...prev]);
      setCreateEventOpen(false);
    } catch (error) {
      console.error('Error creating event:', error);
    }
  };

  const handleEventAction = async (eventId: string, action: string) => {
    try {
      // Simulate event action
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setEvents(prev => prev.map(event => {
        if (event.id === eventId) {
          switch (action) {
            case 'start':
              return { ...event, status: 'running', lastRun: Date.now() };
            case 'pause':
              return { ...event, status: 'paused' };
            case 'stop':
              return { ...event, status: 'cancelled' };
            case 'complete':
              return { ...event, status: 'completed', lastRun: Date.now() };
            default:
              return event;
          }
        }
        return event;
      }));
    } catch (error) {
      console.error('Error performing event action:', error);
    }
  };

  const handleDeleteEvent = async (eventId: string) => {
    try {
      // Simulate deletion
      await new Promise(resolve => setTimeout(resolve, 1000));
      setEvents(prev => prev.filter(e => e.id !== eventId));
    } catch (error) {
      console.error('Error deleting event:', error);
    }
  };

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

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view events</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Events</h2>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <Calendar className="h-3 w-3" />
              {events.filter(e => e.status === 'scheduled').length} Scheduled
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <Activity className="h-3 w-3" />
              {events.filter(e => e.status === 'running').length} Running
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <Repeat className="h-3 w-3" />
              {events.filter(e => e.repeat).length} Recurring
            </Badge>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setCreateEventOpen(true)}
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Event
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={fetchEvents}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search events..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="scheduled">Scheduled</SelectItem>
            <SelectItem value="running">Running</SelectItem>
            <SelectItem value="completed">Completed</SelectItem>
            <SelectItem value="failed">Failed</SelectItem>
            <SelectItem value="cancelled">Cancelled</SelectItem>
          </SelectContent>
        </Select>
        
        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Types</SelectItem>
            <SelectItem value="backup">Backup</SelectItem>
            <SelectItem value="restart">Restart</SelectItem>
            <SelectItem value="maintenance">Maintenance</SelectItem>
            <SelectItem value="update">Update</SelectItem>
            <SelectItem value="custom">Custom</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="events" className="space-y-4">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="events" className="flex items-center gap-2">
            <Calendar className="h-4 w-4" />
            Events
          </TabsTrigger>
          <TabsTrigger value="schedule" className="flex items-center gap-2">
            <Timer className="h-4 w-4" />
            Schedule
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-2">
            <Clock className="h-4 w-4" />
            History
          </TabsTrigger>
        </TabsList>

        <TabsContent value="events" className="space-y-4">
          <EventsTable
            events={events}
            searchQuery={searchQuery}
            filterStatus={filterStatus}
            filterType={filterType}
            onEventAction={handleEventAction}
            onDelete={handleDeleteEvent}
            onEdit={(eventId) => {
              console.log('Edit event:', eventId);
            }}
          />
        </TabsContent>

        <TabsContent value="schedule" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Calendar className="h-5 w-5" />
                  Upcoming Events
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {events
                    .filter(e => e.status === 'scheduled' && e.scheduledAt > Date.now())
                    .slice(0, 5)
                    .map((event) => (
                      <div key={event.id} className="flex items-center justify-between p-3 border rounded-lg">
                        <div className="flex items-center gap-3">
                          <div className="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center">
                            {getStatusIcon(event.status)}
                          </div>
                          <div>
                            <p className="font-medium text-sm">{event.name}</p>
                            <p className="text-xs text-muted-foreground">
                              {formatDate(event.scheduledAt)}
                            </p>
                          </div>
                        </div>
                        <Badge className={`text-xs ${getTypeColor(event.type)}`}>
                          {event.type}
                        </Badge>
                      </div>
                    ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Running Events
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {events
                    .filter(e => e.status === 'running')
                    .slice(0, 5)
                    .map((event) => (
                      <div key={event.id} className="flex items-center justify-between p-3 border rounded-lg">
                        <div className="flex items-center gap-3">
                          <div className="w-8 h-8 bg-green-500/10 rounded-lg flex items-center justify-center">
                            <Play className="h-4 w-4 text-green-400" />
                          </div>
                          <div>
                            <p className="font-medium text-sm">{event.name}</p>
                            <p className="text-xs text-muted-foreground">
                              Running for {Math.floor((Date.now() - (event.lastRun || event.scheduledAt)) / 60000)} minutes
                            </p>
                          </div>
                        </div>
                        <Badge className={`text-xs ${getPriorityColor(event.priority)}`}>
                          {event.priority}
                        </Badge>
                      </div>
                    ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="history" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Clock className="h-5 w-5" />
                Event History
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {events
                  .filter(e => e.status === 'completed' || e.status === 'failed')
                  .slice(0, 10)
                  .map((event) => (
                    <div key={event.id} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 bg-muted/50 rounded-lg flex items-center justify-center">
                          {getStatusIcon(event.status)}
                        </div>
                        <div>
                          <p className="font-medium text-sm">{event.name}</p>
                          <p className="text-xs text-muted-foreground">
                            {event.lastRun ? formatDate(event.lastRun) : 'Never run'}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Badge className={`text-xs ${getTypeColor(event.type)}`}>
                          {event.type}
                        </Badge>
                        <Badge className={`text-xs ${getStatusColor(event.status)}`}>
                          {event.status}
                        </Badge>
                      </div>
                    </div>
                  ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Create Event Modal */}
      <CreateEventModal
        isOpen={createEventOpen}
        onClose={() => setCreateEventOpen(false)}
        onCreate={handleCreateEvent}
      />
    </div>
  );
};

export default Events;
