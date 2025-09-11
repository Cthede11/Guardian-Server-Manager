import React, { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useVirtualizer } from '@tanstack/react-virtual';
import { 
  MessageSquare, 
  UserX, 
  Ban, 
  MapPin, 
  MoreHorizontal,
  Search,
  RefreshCw,
  Users,
  Clock,
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
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { useServers } from '@/store/servers-new';
import { usePlayerData, liveStore } from '@/store/live';
import type { Player } from '@/lib/types';
import { PlayersTableLoading } from '@/components/ui/LoadingStates';
import { NoPlayersEmptyState, SearchEmptyState, ErrorEmptyState } from '@/components/ui/EmptyState';
import { useLoadingState } from '@/components/ui/LoadingStates';
import { notifications } from '@/lib/notifications';
import { handleApiError } from '@/lib/error-handler';
import { apiClient as api } from '@/lib/api';

interface PlayersTableProps {
  className?: string;
}

export const PlayersTable: React.FC<PlayersTableProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServers();
  const server = serverId ? getServerById(serverId) : null;
  
  // Use live store for player data
  const players = usePlayerData(serverId || '');
  
  const [searchQuery, setSearchQuery] = useState('');
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const { isLoading, error, startLoading, stopLoading, setLoadingError } = useLoadingState();
  
  const parentRef = useRef<HTMLDivElement>(null);
  
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

  // Filter players with memoization for performance
  const filteredPlayers = useMemo(() => {
    return players.filter(player =>
      player.name.toLowerCase().includes(searchQuery.toLowerCase())
    );
  }, [players, searchQuery]);

  // Virtualizer for performance with large player lists
  const rowVirtualizer = useVirtualizer({
    count: filteredPlayers.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80, // Player row height
    overscan: 5,
  });

  // Memoized player row component to prevent unnecessary re-renders
  const PlayerRow = React.memo<{ player: Player; virtualItem: any }>(({ player, virtualItem }) => {
    return (
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: `${virtualItem.size}px`,
          transform: `translateY(${virtualItem.start}px)`,
        }}
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
            </div>
            
            <div className="flex items-center gap-4 text-sm text-muted-foreground">
              <span className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                {player.online ? 'Online' : 'Offline'}
              </span>
              
              <span className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                {formatPlaytime(player.playtime || 0)}
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
    );
  });

  // Fetch players data (for initial load and refresh)
  const fetchPlayers = useCallback(async () => {
    if (!serverId) return;
    
    startLoading();
    try {
      const response = await api.getPlayers(serverId);
      if (response.ok) {
        // Update live store with real data
        liveStore.getState().updatePlayers(serverId, response.data as any);
      } else {
        // If server is stopped, show empty list
        liveStore.getState().updatePlayers(serverId, []);
      }
    } catch (error) {
      console.error('Error fetching players:', error);
      handleApiError(error as Error, 'fetching players');
      setLoadingError(new Error('Failed to fetch players'));
      // If server is stopped, show empty list
      liveStore.getState().updatePlayers(serverId, []);
    } finally {
      stopLoading();
    }
  }, [serverId, startLoading, stopLoading, setLoadingError]);


  useEffect(() => {
    fetchPlayers();
    
    // Refresh players every 10 seconds
    const interval = setInterval(fetchPlayers, 10000);
    return () => clearInterval(interval);
  }, [fetchPlayers]);

  // Handle player actions
  const handlePlayerAction = async (action: string, player: Player, data?: any) => {
    if (!serverId) return;
    
    setActionLoading(player.uuid);
    try {
      const response = await api.playerAction(serverId, player.uuid, action as any, data);

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
        const error = { message: response.error || 'Unknown error' };
        notifications.error(`Failed to ${action} player ${player.name}`, { description: error.message });
      }
    } catch (error) {
      console.error(`Error ${action}ing player:`, error);
      notifications.error(`Failed to ${action} player ${player.name}`, { description: 'Network error' });
    } finally {
      setActionLoading(null);
    }
  };

  // Get virtual items for rendering
  const virtualItems = rowVirtualizer.getVirtualItems();
  const paddingTop = virtualItems.length ? virtualItems[0].start : 0;
  // const paddingBottom = virtualItems.length ? rowVirtualizer.getTotalSize() - virtualItems[virtualItems.length - 1].end : 0;

  const formatPlaytime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  // Ping and dimension color functions removed - not used

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
            setLoadingError(new Error('Retrying...'));
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
              <NoPlayersEmptyState onRefresh={fetchPlayers} serverStatus={server?.status} />
            )
          ) : (
            <div 
              ref={parentRef}
              className="h-96 overflow-auto"
            >
              <div
                style={{
                  height: rowVirtualizer.getTotalSize(),
                  position: 'relative',
                }}
              >
                <div style={{ transform: `translateY(${paddingTop}px)` }}>
                  {virtualItems.map((virtualItem) => {
                    const player = filteredPlayers[virtualItem.index];
                    return (
                      <PlayerRow
                        key={player.uuid}
                        player={player}
                        virtualItem={virtualItem}
                      />
                    );
                  })}
                </div>
              </div>
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
