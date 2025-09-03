import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { 
  MessageSquare, 
  UserX, 
  Ban, 
  MapPin, 
  Gauge,
  MoreHorizontal,
  Search,
  RefreshCw,
  Users,
  Clock,
  Shield,
  AlertTriangle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { useServersStore } from '@/store/servers';
import type { Player } from '@/lib/types';
import { PlayersTableLoading, LoadingState } from '@/components/ui/LoadingStates';
import { NoPlayersEmptyState, SearchEmptyState, ErrorEmptyState } from '@/components/ui/EmptyState';
import { useLoadingState } from '@/components/ui/LoadingStates';
import { notifications } from '@/lib/notifications';
import { handleApiError } from '@/lib/error-handler';

interface PlayersTableProps {
  className?: string;
}

export const PlayersTable: React.FC<PlayersTableProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [players, setPlayers] = useState<Player[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const { isLoading, error, startLoading, stopLoading, setLoadingError } = useLoadingState();
  
  // Dialog states
  const [messageDialog, setMessageDialog] = useState<{ open: boolean; player: Player | null }>({ open: false, player: null });
  const [kickDialog, setKickDialog] = useState<{ open: boolean; player: Player | null }>({ open: false, player: null });
  const [banDialog, setBanDialog] = useState<{ open: boolean; player: Player | null }>({ open: false, player: null });
  const [teleportDialog, setTeleportDialog] = useState<{ open: boolean; player: Player | null }>({ open: false, player: null });
  const [throttleDialog, setThrottleDialog] = useState<{ open: boolean; player: Player | null }>({ open: false, player: null });
  
  // Form states
  const [messageText, setMessageText] = useState('');
  const [kickReason, setKickReason] = useState('');
  const [banReason, setBanReason] = useState('');
  const [teleportCoords, setTeleportCoords] = useState({ x: '', y: '', z: '' });
  const [throttleDuration, setThrottleDuration] = useState('');

  // Fetch players data
  const fetchPlayers = async () => {
    if (!serverId) return;
    
    startLoading();
    try {
      const response = await fetch(`/api/v1/servers/${serverId}/players/online`);
      if (response.ok) {
        const data = await response.json();
        setPlayers(data);
      } else {
        // Use mock data for demo
        setPlayers(generateMockPlayers());
      }
    } catch (error) {
      console.error('Error fetching players:', error);
      setLoadingError(handleApiError(error, 'fetching players'));
      // Use mock data for demo
      setPlayers(generateMockPlayers());
    } finally {
      stopLoading();
    }
  };

  // Generate mock players for demo
  const generateMockPlayers = (): Player[] => {
    const mockPlayers = [
      { uuid: '1', name: 'Steve', ping: 45, location: { x: 100, y: 64, z: 200, dimension: 'overworld' }, playtime: 3600, isOp: true },
      { uuid: '2', name: 'Alex', ping: 32, location: { x: -150, y: 70, z: 300, dimension: 'overworld' }, playtime: 2400, isOp: false },
      { uuid: '3', name: 'Notch', ping: 67, location: { x: 0, y: 120, z: 0, dimension: 'overworld' }, playtime: 7200, isOp: true },
      { uuid: '4', name: 'Herobrine', ping: 23, location: { x: 50, y: 64, z: -100, dimension: 'nether' }, playtime: 1800, isOp: false },
      { uuid: '5', name: 'Dinnerbone', ping: 89, location: { x: 200, y: 80, z: 150, dimension: 'overworld' }, playtime: 4800, isOp: true },
      { uuid: '6', name: 'Jeb_', ping: 12, location: { x: -50, y: 64, z: 250, dimension: 'overworld' }, playtime: 6000, isOp: true },
      { uuid: '7', name: 'Grumm', ping: 156, location: { x: 75, y: 65, z: -75, dimension: 'overworld' }, playtime: 1200, isOp: false },
      { uuid: '8', name: 'CaptainSparklez', ping: 34, location: { x: 300, y: 90, z: 400, dimension: 'overworld' }, playtime: 9000, isOp: false }
    ];
    
    return mockPlayers;
  };

  useEffect(() => {
    fetchPlayers();
    
    // Refresh players every 10 seconds
    const interval = setInterval(fetchPlayers, 10000);
    return () => clearInterval(interval);
  }, [serverId]);

  // Handle player actions
  const handlePlayerAction = async (action: string, player: Player, data?: any) => {
    if (!serverId) return;
    
    setActionLoading(player.uuid);
    try {
      const response = await fetch(`/api/v1/servers/${serverId}/players/${player.uuid}/actions/${action}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data || {}),
      });

      if (response.ok) {
        // Show success notification
        switch (action) {
          case 'message':
            notifications.success(`Message sent to ${player.name}`);
            break;
          case 'kick':
            notifications.playerKicked(player.name);
            break;
          case 'ban':
            notifications.playerBanned(player.name);
            break;
          case 'teleport':
            notifications.playerTeleported(player.name);
            break;
          case 'throttle':
            notifications.playerThrottled(player.name);
            break;
          default:
            notifications.success(`Action completed for ${player.name}`);
        }
        
        // Close dialogs
        setMessageDialog({ open: false, player: null });
        setKickDialog({ open: false, player: null });
        setBanDialog({ open: false, player: null });
        setTeleportDialog({ open: false, player: null });
        setThrottleDialog({ open: false, player: null });
        
        // Clear form data
        setMessageText('');
        setKickReason('');
        setBanReason('');
        setTeleportCoords({ x: '', y: '', z: '' });
        setThrottleDuration('');
        
        // Refresh players list
        fetchPlayers();
      } else {
        const error = await response.json().catch(() => ({ message: 'Unknown error' }));
        notifications.error(`Failed to ${action} player ${player.name}`, { description: error.message });
      }
    } catch (error) {
      console.error(`Error ${action}ing player:`, error);
      notifications.error(`Failed to ${action} player ${player.name}`, { description: 'Network error' });
    } finally {
      setActionLoading(null);
    }
  };

  const filteredPlayers = players.filter(player =>
    player.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const formatPlaytime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const getPingColor = (ping: number) => {
    if (ping < 50) return 'text-green-400';
    if (ping < 100) return 'text-yellow-400';
    return 'text-red-400';
  };

  const getDimensionColor = (dimension: string) => {
    switch (dimension) {
      case 'overworld':
        return 'text-green-400';
      case 'nether':
        return 'text-red-400';
      case 'end':
        return 'text-purple-400';
      default:
        return 'text-gray-400';
    }
  };

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its players."
        />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="Failed to load players"
          description={error.message}
          onRetry={() => {
            setLoadingError(null);
            fetchPlayers();
          }}
        />
      </div>
    );
  }

  if (isLoading && players.length === 0) {
    return (
      <div className="p-6 space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold">Players</h2>
          <Badge variant="outline" className="flex items-center gap-1">
            <Users className="h-3 w-3" />
            Loading...
          </Badge>
        </div>
        <PlayersTableLoading count={5} />
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Players</h2>
          <Badge variant="outline" className="flex items-center gap-1">
            <Users className="h-3 w-3" />
            {players.length} Online
          </Badge>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={fetchPlayers}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Search */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search players..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
      </div>

      {/* Players Table */}
      <Card>
        <CardHeader>
          <CardTitle>Online Players</CardTitle>
        </CardHeader>
        <CardContent>
          {filteredPlayers.length === 0 ? (
            searchQuery ? (
              <SearchEmptyState
                query={searchQuery}
                onClear={() => setSearchQuery('')}
              />
            ) : (
              <NoPlayersEmptyState onRefresh={fetchPlayers} />
            )
          ) : (
            <div className="space-y-2">
              {filteredPlayers.map((player) => (
                <div
                  key={player.uuid}
                  className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors"
                >
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 bg-primary/10 rounded-full flex items-center justify-center">
                      <span className="text-sm font-medium">
                        {player.name.charAt(0).toUpperCase()}
                      </span>
                    </div>
                    
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{player.name}</span>
                        {player.isOp && (
                          <Badge variant="secondary" className="text-xs">
                            <Shield className="h-3 w-3 mr-1" />
                            OP
                          </Badge>
                        )}
                      </div>
                      
                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span className={`flex items-center gap-1 ${getPingColor(player.ping)}`}>
                          <Gauge className="h-3 w-3" />
                          {player.ping}ms
                        </span>
                        
                        <span className="flex items-center gap-1">
                          <MapPin className="h-3 w-3" />
                          <span className={getDimensionColor(player.location.dimension)}>
                            {player.location.dimension}
                          </span>
                          ({player.location.x}, {player.location.y}, {player.location.z})
                        </span>
                        
                        <span className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {formatPlaytime(player.playtime)}
                        </span>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-2">
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button
                          size="sm"
                          variant="ghost"
                          disabled={actionLoading === player.uuid}
                        >
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem
                          onClick={() => setMessageDialog({ open: true, player })}
                        >
                          <MessageSquare className="h-4 w-4 mr-2" />
                          Send Message
                        </DropdownMenuItem>
                        
                        <DropdownMenuItem
                          onClick={() => setTeleportDialog({ open: true, player })}
                        >
                          <MapPin className="h-4 w-4 mr-2" />
                          Teleport
                        </DropdownMenuItem>
                        
                        <DropdownMenuSeparator />
                        
                        <DropdownMenuItem
                          onClick={() => setKickDialog({ open: true, player })}
                          className="text-yellow-600"
                        >
                          <UserX className="h-4 w-4 mr-2" />
                          Kick Player
                        </DropdownMenuItem>
                        
                        <DropdownMenuItem
                          onClick={() => setBanDialog({ open: true, player })}
                          className="text-red-600"
                        >
                          <Ban className="h-4 w-4 mr-2" />
                          Ban Player
                        </DropdownMenuItem>
                        
                        <DropdownMenuItem
                          onClick={() => setThrottleDialog({ open: true, player })}
                          className="text-orange-600"
                        >
                          <AlertTriangle className="h-4 w-4 mr-2" />
                          Throttle
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Message Dialog */}
      <Dialog open={messageDialog.open} onOpenChange={(open) => setMessageDialog({ open, player: null })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Send Message to {messageDialog.player?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="message">Message</Label>
              <Textarea
                id="message"
                placeholder="Enter your message..."
                value={messageText}
                onChange={(e) => setMessageText(e.target.value)}
                rows={3}
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => setMessageDialog({ open: false, player: null })}
              >
                Cancel
              </Button>
              <Button
                onClick={() => messageDialog.player && handlePlayerAction('message', messageDialog.player, { message: messageText })}
                disabled={!messageText.trim() || actionLoading === messageDialog.player?.uuid}
              >
                {actionLoading === messageDialog.player?.uuid ? 'Sending...' : 'Send Message'}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Kick Dialog */}
      <Dialog open={kickDialog.open} onOpenChange={(open) => setKickDialog({ open, player: null })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Kick {kickDialog.player?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="kickReason">Reason (optional)</Label>
              <Input
                id="kickReason"
                placeholder="Enter kick reason..."
                value={kickReason}
                onChange={(e) => setKickReason(e.target.value)}
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => setKickDialog({ open: false, player: null })}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => kickDialog.player && handlePlayerAction('kick', kickDialog.player, { reason: kickReason })}
                disabled={actionLoading === kickDialog.player?.uuid}
              >
                {actionLoading === kickDialog.player?.uuid ? 'Kicking...' : 'Kick Player'}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Ban Dialog */}
      <Dialog open={banDialog.open} onOpenChange={(open) => setBanDialog({ open, player: null })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Ban {banDialog.player?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="banReason">Reason (optional)</Label>
              <Input
                id="banReason"
                placeholder="Enter ban reason..."
                value={banReason}
                onChange={(e) => setBanReason(e.target.value)}
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => setBanDialog({ open: false, player: null })}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => banDialog.player && handlePlayerAction('ban', banDialog.player, { reason: banReason })}
                disabled={actionLoading === banDialog.player?.uuid}
              >
                {actionLoading === banDialog.player?.uuid ? 'Banning...' : 'Ban Player'}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Teleport Dialog */}
      <Dialog open={teleportDialog.open} onOpenChange={(open) => setTeleportDialog({ open, player: null })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Teleport {teleportDialog.player?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <div>
                <Label htmlFor="teleportX">X Coordinate</Label>
                <Input
                  id="teleportX"
                  placeholder="X"
                  value={teleportCoords.x}
                  onChange={(e) => setTeleportCoords(prev => ({ ...prev, x: e.target.value }))}
                />
              </div>
              <div>
                <Label htmlFor="teleportY">Y Coordinate</Label>
                <Input
                  id="teleportY"
                  placeholder="Y"
                  value={teleportCoords.y}
                  onChange={(e) => setTeleportCoords(prev => ({ ...prev, y: e.target.value }))}
                />
              </div>
              <div>
                <Label htmlFor="teleportZ">Z Coordinate</Label>
                <Input
                  id="teleportZ"
                  placeholder="Z"
                  value={teleportCoords.z}
                  onChange={(e) => setTeleportCoords(prev => ({ ...prev, z: e.target.value }))}
                />
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => setTeleportDialog({ open: false, player: null })}
              >
                Cancel
              </Button>
              <Button
                onClick={() => teleportDialog.player && handlePlayerAction('teleport', teleportDialog.player, { 
                  x: parseFloat(teleportCoords.x), 
                  y: parseFloat(teleportCoords.y), 
                  z: parseFloat(teleportCoords.z) 
                })}
                disabled={!teleportCoords.x || !teleportCoords.y || !teleportCoords.z || actionLoading === teleportDialog.player?.uuid}
              >
                {actionLoading === teleportDialog.player?.uuid ? 'Teleporting...' : 'Teleport'}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Throttle Dialog */}
      <Dialog open={throttleDialog.open} onOpenChange={(open) => setThrottleDialog({ open, player: null })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Throttle {throttleDialog.player?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="throttleDuration">Duration (seconds)</Label>
              <Input
                id="throttleDuration"
                placeholder="Enter duration in seconds..."
                value={throttleDuration}
                onChange={(e) => setThrottleDuration(e.target.value)}
                type="number"
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => setThrottleDialog({ open: false, player: null })}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => throttleDialog.player && handlePlayerAction('throttle', throttleDialog.player, { duration: parseInt(throttleDuration) })}
                disabled={!throttleDuration || actionLoading === throttleDialog.player?.uuid}
              >
                {actionLoading === throttleDialog.player?.uuid ? 'Throttling...' : 'Throttle Player'}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default PlayersTable;
