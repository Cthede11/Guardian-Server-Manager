import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
// import { Input } from '@/components/ui/input';
// import { Label } from '@/components/ui/label';
// import { Textarea } from '@/components/ui/textarea';
// import { Switch } from '@/components/ui/switch';
// import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { apiClient as api } from '@/lib/api';
import { 
  Database, 
  HardDrive, 
  Cloud, 
  AlertTriangle,
  CheckCircle,
  Info,
  Edit,
  Trash2,
  X,
  Plus,
  TestTube,
  RefreshCw,
  Download,
  Shield,
  Lock,
  Unlock,
  Network,
  Zap,
  Package
} from 'lucide-react';

interface BackupTargetData {
  id: string;
  name: string;
  description: string;
  type: 'local' | 's3' | 'gcs' | 'azure' | 'ftp' | 'sftp' | 'nfs' | 'smb';
  status: 'active' | 'inactive' | 'error' | 'testing';
  endpoint: string;
  credentials: {
    accessKey?: string;
    secretKey?: string;
    username?: string;
    password?: string;
    token?: string;
  };
  settings: {
    bucket?: string;
    region?: string;
    path?: string;
    compression: boolean;
    encryption: boolean;
    retention: number;
    maxSize: string;
    chunkSize: string;
    threads: number;
    timeout: number;
    retries: number;
  };
  health: {
    lastTested: string | null;
    lastBackup: string | null;
    totalBackups: number;
    totalSize: string;
    availableSpace: string;
    responseTime: number;
    successRate: number;
  };
  createdAt: string;
  updatedAt: string;
}

export const BackupTargets: React.FC = () => {
  const [targets, setTargets] = useState<BackupTargetData[]>([]);

  // const [isLoading, setIsLoading] = useState(false);
  // const [hasChanges, setHasChanges] = useState(false);
  // const [selectedTarget, setSelectedTarget] = useState<BackupTargetData | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [isTesting, setIsTesting] = useState<string | null>(null);
  // const [showCredentials, setShowCredentials] = useState<string | null>(null);

  const fetchData = async () => {
    // setIsLoading(true);
    try {
      const response = await api.getBackupTargets?.();
      if (response?.ok && response.data) {
        setTargets(response.data as BackupTargetData[]);
      }
      // setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch backup targets:', error);
    } finally {
      // setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'local': return <HardDrive className="h-4 w-4" />;
      case 's3': return <Cloud className="h-4 w-4" />;
      case 'gcs': return <Cloud className="h-4 w-4" />;
      case 'azure': return <Cloud className="h-4 w-4" />;
      case 'ftp': return <Network className="h-4 w-4" />;
      case 'sftp': return <Network className="h-4 w-4" />;
      case 'nfs': return <Network className="h-4 w-4" />;
      case 'smb': return <Network className="h-4 w-4" />;
      default: return <Database className="h-4 w-4" />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'local': return 'bg-blue-500';
      case 's3': return 'bg-orange-500';
      case 'gcs': return 'bg-blue-600';
      case 'azure': return 'bg-blue-700';
      case 'ftp': return 'bg-green-500';
      case 'sftp': return 'bg-green-600';
      case 'nfs': return 'bg-purple-500';
      case 'smb': return 'bg-purple-600';
      default: return 'bg-gray-500';
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'local': return 'Local';
      case 's3': return 'AWS S3';
      case 'gcs': return 'Google Cloud';
      case 'azure': return 'Azure Blob';
      case 'ftp': return 'FTP';
      case 'sftp': return 'SFTP';
      case 'nfs': return 'NFS';
      case 'smb': return 'SMB';
      default: return 'Unknown';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-500';
      case 'inactive': return 'text-gray-500';
      case 'error': return 'text-red-500';
      case 'testing': return 'text-yellow-500';
      default: return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return <CheckCircle className="h-4 w-4" />;
      case 'inactive': return <X className="h-4 w-4" />;
      case 'error': return <AlertTriangle className="h-4 w-4" />;
      case 'testing': return <RefreshCw className="h-4 w-4 animate-spin" />;
      default: return <Info className="h-4 w-4" />;
    }
  };

  const handleCreateTarget = async () => {
    setIsCreating(true);
    try {
      const response = await api.createBackupTarget?.({
        name: 'New Target',
        description: 'A new backup target',
        type: 'local',
        endpoint: '',
        credentials: {},
        settings: {
          compression: true,
          encryption: false,
          retention: 30,
          maxSize: '10G',
          chunkSize: '64M',
          threads: 4,
          timeout: 300,
          retries: 3
        }
      });
      
      if (response?.ok && response.data) {
        setTargets(prev => [...prev, response.data as BackupTargetData]);
      } else {
        throw new Error('Failed to create backup target');
      }
    } catch (error) {
      console.error('Failed to create backup target:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleDeleteTarget = async (id: string) => {
    try {
      const response = await api.deleteBackupTarget?.(id);
      if (response?.ok) {
        setTargets(prev => prev.filter(target => target.id !== id));
      } else {
        throw new Error('Failed to delete backup target');
      }
    } catch (error) {
      console.error('Failed to delete backup target:', error);
    }
  };

  const handleTestTarget = async (id: string) => {
    setIsTesting(id);
    try {
      const response = await api.testBackupTarget?.(id);
      if (response?.ok && response.data) {
        setTargets(prev => prev.map(target => 
          target.id === id ? { 
            ...target, 
            status: response.data.status,
            health: response.data.health
          } : target
        ));
      } else {
        throw new Error('Failed to test backup target');
      }
    } catch (error) {
      console.error('Failed to test backup target:', error);
    } finally {
      setIsTesting(null);
    }
  };

  const handleToggleTarget = async (id: string) => {
    try {
      const target = targets.find(t => t.id === id);
      if (!target) return;
      
      const newStatus = target.status === 'active' ? 'inactive' : 'active';
      const response = await api.updateBackupTarget?.(id, { status: newStatus });
      
      if (response?.ok) {
        setTargets(prev => prev.map(t => 
          t.id === id ? { ...t, status: newStatus } : t
        ));
      } else {
        throw new Error('Failed to update backup target status');
      }
    } catch (error) {
      console.error('Failed to update backup target status:', error);
    }
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Backup Targets */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Database className="h-5 w-5" />
              <span>Backup Targets</span>
            </div>
            <Button onClick={handleCreateTarget} disabled={isCreating}>
              <Plus className="h-4 w-4 mr-2" />
              {isCreating ? 'Creating...' : 'Add Target'}
            </Button>
          </CardTitle>
          <CardDescription>
            Configure backup storage destinations
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {targets.map((target) => (
              <div key={target.id} className="p-4 border rounded-lg">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center space-x-3">
                    <div className={`w-8 h-8 rounded-lg ${getTypeColor(target.type)} flex items-center justify-center text-white`}>
                      {getTypeIcon(target.type)}
                    </div>
                    <div>
                      <div className="font-medium">{target.name}</div>
                      <div className="text-sm text-muted-foreground">{target.description}</div>
                      <div className="flex items-center space-x-2 mt-1">
                        <Badge variant="outline">{getTypeLabel(target.type)}</Badge>
                        <div className={`flex items-center space-x-1 ${getStatusColor(target.status)}`}>
                          {getStatusIcon(target.status)}
                          <span className="text-sm capitalize">{target.status}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleTestTarget(target.id)}
                      disabled={isTesting === target.id}
                    >
                      <TestTube className="h-4 w-4 mr-2" />
                      {isTesting === target.id ? 'Testing...' : 'Test'}
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {/* setSelectedTarget(target) */}}
                    >
                      <Edit className="h-4 w-4" />
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleToggleTarget(target.id)}
                    >
                      {target.status === 'active' ? <Unlock className="h-4 w-4" /> : <Lock className="h-4 w-4" />}
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDeleteTarget(target.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Endpoint</div>
                    <div className="text-sm text-muted-foreground font-mono">{target.endpoint}</div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Health</div>
                    <div className="text-sm text-muted-foreground">
                      {target.health.lastTested ? (
                        <div className="flex items-center space-x-1">
                          <CheckCircle className="h-3 w-3 text-green-500" />
                          <span>Last tested: {new Date(target.health.lastTested).toLocaleDateString()}</span>
                        </div>
                      ) : (
                        <div className="flex items-center space-x-1">
                          <AlertTriangle className="h-3 w-3 text-yellow-500" />
                          <span>Never tested</span>
                        </div>
                      )}
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Backups</div>
                    <div className="text-sm text-muted-foreground">
                      {target.health.totalBackups} backups ({target.health.totalSize})
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="text-sm font-medium">Performance</div>
                    <div className="text-sm text-muted-foreground">
                      {target.health.responseTime}ms • {target.health.successRate}% success
                    </div>
                  </div>
                </div>
                
                <div className="mt-4 pt-4 border-t">
                  <div className="flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">
                      Available space: {target.health.availableSpace}
                    </div>
                    <div className="flex items-center space-x-2">
                      {target.settings.compression && (
                        <Badge variant="secondary" className="text-xs">
                          <Package className="h-3 w-3 mr-1" />
                          Compressed
                        </Badge>
                      )}
                      {target.settings.encryption && (
                        <Badge variant="secondary" className="text-xs">
                          <Shield className="h-3 w-3 mr-1" />
                          Encrypted
                        </Badge>
                      )}
                      <Badge variant="secondary" className="text-xs">
                        {target.settings.retention} days retention
                      </Badge>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Zap className="h-5 w-5" />
            <span>Quick Actions</span>
          </CardTitle>
          <CardDescription>
            Common backup target operations
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <TestTube className="h-6 w-6" />
              <span>Test All Targets</span>
            </Button>
            
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <RefreshCw className="h-6 w-6" />
              <span>Refresh Status</span>
            </Button>
            
            <Button variant="outline" className="h-20 flex flex-col items-center justify-center space-y-2">
              <Download className="h-6 w-6" />
              <span>Export Config</span>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
