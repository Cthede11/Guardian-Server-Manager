import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { 
  HardDrive, 
  Plus, 
    // Download,
  // Upload,
  RefreshCw,
  Search,
  Filter,
  Calendar,
  Clock,
  FileText,
  AlertTriangle,
  CheckCircle,
  Play,
  Pause,
  Trash2,
  MoreHorizontal,
  Settings,
  Archive,
  Database,
  Cloud,
  Shield
} from 'lucide-react';
import { api } from '@/lib/api';
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
import { SnapshotsTable } from '@/components/Tables/SnapshotsTable';
import { RestoreWizard } from '@/components/Backups/RestoreWizard';
import { ErrorEmptyState } from '@/components/ui/EmptyState';

interface BackupsPageProps {
  className?: string;
}

export const Backups: React.FC<BackupsPageProps> = ({ className = '' }) => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  
  const [snapshots, setSnapshots] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState('all');
  const [restoreWizardOpen, setRestoreWizardOpen] = useState(false);
  const [selectedSnapshot, setSelectedSnapshot] = useState<any>(null);

  // Fetch snapshots data
  const fetchSnapshots = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const response = await api.getBackups(serverId);
      if (response.ok && response.data) {
        setSnapshots(response.data as any[]);
      } else {
        console.error('Failed to fetch backups:', response.error);
        setSnapshots([]);
      }
    } catch (error) {
      console.error('Error fetching snapshots:', error);
      setSnapshots([]);
    } finally {
      setIsLoading(false);
    }
  };


  useEffect(() => {
    fetchSnapshots();
    
    // Refresh data every 60 seconds
    const interval = setInterval(fetchSnapshots, 60000);
    return () => clearInterval(interval);
  }, [serverId]);

  const handleCreateSnapshot = async () => {
    setIsLoading(true);
    try {
      // Simulate creating a snapshot
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      const newSnapshot = {
        id: `snapshot-${Date.now()}`,
        name: `Manual Backup ${new Date().toLocaleString()}`,
        type: 'manual',
        status: 'completed',
        size: Math.floor(Math.random() * 3000) + 1500,
        timestamp: Date.now(),
        description: 'Manual backup created by user',
        compression: Math.floor(Math.random() * 20) + 25,
        checksum: `sha256:${Math.random().toString(36).substring(2, 15)}`,
        verified: true,
        retention: 7,
        tags: []
      };
      
      setSnapshots(prev => [newSnapshot, ...prev]);
    } catch (error) {
      console.error('Error creating snapshot:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRestoreSnapshot = (snapshot: any) => {
    setSelectedSnapshot(snapshot);
    setRestoreWizardOpen(true);
  };

  const handleDeleteSnapshot = async (snapshotId: string) => {
    try {
      // Simulate deletion
      await new Promise(resolve => setTimeout(resolve, 1000));
      setSnapshots(prev => prev.filter(s => s.id !== snapshotId));
    } catch (error) {
      console.error('Error deleting snapshot:', error);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'text-green-400';
      case 'in_progress':
        return 'text-blue-400';
      case 'failed':
        return 'text-red-400';
      case 'verifying':
        return 'text-yellow-400';
      default:
        return 'text-gray-400';
    }
  };

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

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its backups."
        />
      </div>
    );
  }

  return (
    <div className={`p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Backups</h2>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <HardDrive className="h-3 w-3" />
              {snapshots.filter(s => s.status === 'completed').length} Snapshots
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <Database className="h-3 w-3" />
              {formatBytes(snapshots.reduce((sum, s) => sum + s.size, 0))} Total
            </Badge>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={handleCreateSnapshot}
            disabled={isLoading}
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Snapshot
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={fetchSnapshots}
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
            placeholder="Search snapshots..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Types</SelectItem>
            <SelectItem value="manual">Manual</SelectItem>
            <SelectItem value="scheduled">Scheduled</SelectItem>
            <SelectItem value="pre-update">Pre-update</SelectItem>
            <SelectItem value="emergency">Emergency</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="snapshots" className="space-y-4">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="snapshots" className="flex items-center gap-2">
            <HardDrive className="h-4 w-4" />
            Snapshots
          </TabsTrigger>
          <TabsTrigger value="settings" className="flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Settings
          </TabsTrigger>
          <TabsTrigger value="storage" className="flex items-center gap-2">
            <Cloud className="h-4 w-4" />
            Storage
          </TabsTrigger>
        </TabsList>

        <TabsContent value="snapshots" className="space-y-4">
          <SnapshotsTable
            snapshots={snapshots}
            searchQuery={searchQuery}
            filterType={filterType}
            onRestore={handleRestoreSnapshot}
            onDelete={handleDeleteSnapshot}
            onDownload={(snapshotId) => {
              console.log('Download snapshot:', snapshotId);
            }}
            onVerify={(snapshotId) => {
              console.log('Verify snapshot:', snapshotId);
            }}
          />
        </TabsContent>

        <TabsContent value="settings" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Calendar className="h-5 w-5" />
                  Schedule Settings
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Backup Frequency</label>
                  <Select defaultValue="daily">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="hourly">Every Hour</SelectItem>
                      <SelectItem value="daily">Daily</SelectItem>
                      <SelectItem value="weekly">Weekly</SelectItem>
                      <SelectItem value="monthly">Monthly</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">Retention Policy</label>
                  <Select defaultValue="7">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">1 Day</SelectItem>
                      <SelectItem value="7">7 Days</SelectItem>
                      <SelectItem value="30">30 Days</SelectItem>
                      <SelectItem value="90">90 Days</SelectItem>
                      <SelectItem value="unlimited">Unlimited</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">Compression Level</label>
                  <Select defaultValue="medium">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="low">Low (Fast)</SelectItem>
                      <SelectItem value="medium">Medium (Balanced)</SelectItem>
                      <SelectItem value="high">High (Small)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5" />
                  Security Settings
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Encryption</label>
                  <Select defaultValue="enabled">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="enabled">Enabled</SelectItem>
                      <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">Checksum Verification</label>
                  <Select defaultValue="enabled">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="enabled">Enabled</SelectItem>
                      <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="space-y-2">
                  <label className="text-sm font-medium">Auto-cleanup</label>
                  <Select defaultValue="enabled">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="enabled">Enabled</SelectItem>
                      <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="storage" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Cloud className="h-5 w-5" />
                  Storage Usage
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Total Storage</span>
                    <span className="text-sm font-medium">
                      {formatBytes(snapshots.reduce((sum, s) => sum + s.size, 0))}
                    </span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div 
                      className="bg-blue-500 h-2 rounded-full" 
                      style={{ width: '65%' }}
                    />
                  </div>
                  
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <span>Snapshots</span>
                      <span>{snapshots.length} files</span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                      <span>Compression</span>
                      <span>~{Math.floor(snapshots.reduce((sum, s) => sum + s.compression, 0) / snapshots.length)}%</span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                      <span>Verified</span>
                      <span>{snapshots.filter(s => s.verified).length}/{snapshots.length}</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Archive className="h-5 w-5" />
                  Storage Targets
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-2">
                      <HardDrive className="h-4 w-4" />
                      <span className="text-sm font-medium">Local Storage</span>
                    </div>
                    <Badge variant="outline">Active</Badge>
                  </div>
                  
                  <div className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-2">
                      <Cloud className="h-4 w-4" />
                      <span className="text-sm font-medium">Cloud Backup</span>
                    </div>
                    <Badge variant="outline">Configured</Badge>
                  </div>
                  
                  <div className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-2">
                      <Database className="h-4 w-4" />
                      <span className="text-sm font-medium">Network Storage</span>
                    </div>
                    <Badge variant="outline">Available</Badge>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>

      {/* Restore Wizard Modal */}
      <RestoreWizard
        isOpen={restoreWizardOpen}
        onClose={() => setRestoreWizardOpen(false)}
        snapshot={selectedSnapshot}
        onRestore={(options) => {
          console.log('Restore with options:', options);
          setRestoreWizardOpen(false);
        }}
      />
    </div>
  );
};

export default Backups;
