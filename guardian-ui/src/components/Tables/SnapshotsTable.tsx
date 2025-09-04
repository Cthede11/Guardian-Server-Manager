import React, { useState } from 'react';
import { 
  HardDrive, 
  Download, 
  Upload,
  MoreHorizontal,
  Trash2,
  Eye,
  Shield,
  Clock,
  AlertTriangle,
  CheckCircle,
  Play,
  Archive,
  Calendar,
  Database
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

interface SnapshotsTableProps {
  snapshots: any[];
  searchQuery: string;
  filterType: string;
  onRestore: (snapshot: any) => void;
  onDelete: (snapshotId: string) => void;
  onDownload: (snapshotId: string) => void;
  onVerify: (snapshotId: string) => void;
  className?: string;
}

export const SnapshotsTable: React.FC<SnapshotsTableProps> = ({
  snapshots,
  searchQuery,
  filterType,
  onRestore,
  onDelete,
  onDownload,
  onVerify
}) => {
  const [sortBy] = useState('timestamp');
  const [sortOrder] = useState<'asc' | 'desc'>('desc');

  // Filter and sort snapshots
  const filteredSnapshots = snapshots
    .filter(snapshot => {
      const matchesSearch = snapshot.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           snapshot.description.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesFilter = filterType === 'all' || snapshot.type === filterType;
      
      return matchesSearch && matchesFilter;
    })
    .sort((a, b) => {
      let aValue = a[sortBy];
      let bValue = b[sortBy];
      
      if (sortBy === 'timestamp') {
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



  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="h-4 w-4" />;
      case 'in_progress':
        return <Play className="h-4 w-4" />;
      case 'failed':
        return <AlertTriangle className="h-4 w-4" />;
      case 'verifying':
        return <Shield className="h-4 w-4" />;
      default:
        return <Clock className="h-4 w-4" />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'manual':
        return 'bg-blue-500/20 text-blue-400';
      case 'scheduled':
        return 'bg-green-500/20 text-green-400';
      case 'pre-update':
        return 'bg-yellow-500/20 text-yellow-400';
      case 'emergency':
        return 'bg-red-500/20 text-red-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes >= 1024 * 1024 * 1024) {
      return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    } else if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    } else {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const getRelativeTime = (timestamp: number) => {
    const now = Date.now();
    const diff = now - timestamp;
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);
    
    if (days > 0) {
      return `${days} day${days > 1 ? 's' : ''} ago`;
    } else if (hours > 0) {
      return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    } else {
      return 'Just now';
    }
  };

  if (filteredSnapshots.length === 0) {
    return (
      <Card>
        <CardContent className="text-center py-12">
          <HardDrive className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p className="text-muted-foreground">
            {searchQuery || filterType !== 'all' 
              ? 'No snapshots found matching your criteria' 
              : 'No snapshots available'}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            Create your first snapshot to get started
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
            <HardDrive className="h-5 w-5" />
            Snapshots ({filteredSnapshots.length})
          </CardTitle>
          
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline">
              <Download className="h-4 w-4 mr-2" />
              Export List
            </Button>
            <Button size="sm" variant="outline">
              <Upload className="h-4 w-4 mr-2" />
              Import
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {filteredSnapshots.map((snapshot) => (
            <div
              key={snapshot.id}
              className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors"
            >
              <div className="flex items-center gap-4 flex-1">
                {/* Snapshot Icon/Status */}
                <div className="flex items-center gap-2">
                  <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
                    <HardDrive className="h-5 w-5" />
                  </div>
                  
                  {getStatusIcon(snapshot.status)}
                </div>

                {/* Snapshot Info */}
                <div className="flex-1 space-y-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium">{snapshot.name}</h3>
                    <Badge 
                      variant="outline" 
                      className={`text-xs ${getTypeColor(snapshot.type)}`}
                    >
                      {snapshot.type}
                    </Badge>
                    {snapshot.verified && (
                      <Badge variant="outline" className="text-xs text-green-400">
                        <Shield className="h-3 w-3 mr-1" />
                        Verified
                      </Badge>
                    )}
                    {snapshot.tags.length > 0 && (
                      <Badge variant="outline" className="text-xs">
                        {snapshot.tags.join(', ')}
                      </Badge>
                    )}
                  </div>
                  
                  <p className="text-sm text-muted-foreground">
                    {snapshot.description}
                  </p>
                  
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {formatDate(snapshot.timestamp)}
                    </span>
                    <span className="flex items-center gap-1">
                      <Database className="h-3 w-3" />
                      {formatBytes(snapshot.size)}
                    </span>
                    <span className="flex items-center gap-1">
                      <Archive className="h-3 w-3" />
                      {snapshot.compression}% compressed
                    </span>
                    <span className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      {getRelativeTime(snapshot.timestamp)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => onRestore(snapshot)}
                  disabled={snapshot.status !== 'completed'}
                >
                  <Upload className="h-4 w-4 mr-1" />
                  Restore
                </Button>
                
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button size="sm" variant="ghost">
                      <MoreHorizontal className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => onDownload(snapshot.id)}>
                      <Download className="h-4 w-4 mr-2" />
                      Download
                    </DropdownMenuItem>
                    
                    <DropdownMenuItem onClick={() => onVerify(snapshot.id)}>
                      <Shield className="h-4 w-4 mr-2" />
                      Verify
                    </DropdownMenuItem>
                    
                    <DropdownMenuItem>
                      <Eye className="h-4 w-4 mr-2" />
                      View Details
                    </DropdownMenuItem>
                    
                    <DropdownMenuSeparator />
                    
                    <DropdownMenuItem 
                      onClick={() => onDelete(snapshot.id)}
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

export default SnapshotsTable;
