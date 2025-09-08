import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  AlertTriangle, 
  AlertCircle, 
  XCircle, 
  CheckCircle, 
  RefreshCw,
  Zap,
  Users,
  Activity,
  Cpu,
  Clock,
  ExternalLink
} from 'lucide-react';
import { api } from '@/lib/api';

interface ShardingWarning {
  id: string;
  type: 'load_imbalance' | 'shard_failure' | 'connection_issue' | 'capacity_warning' | 'performance_degradation';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  affectedShards: string[];
  impact: string;
  suggestedActions: string[];
  status: 'active' | 'acknowledged' | 'resolved';
  createdAt: string;
  lastUpdated: string;
  autoResolve: boolean;
  estimatedResolutionTime?: number;
}

interface WarningStats {
  total: number;
  critical: number;
  high: number;
  medium: number;
  low: number;
  resolved: number;
  autoResolved: number;
}

export const ShardingWarnings: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const [warnings, setWarnings] = useState<ShardingWarning[]>([]);
  const [stats, setStats] = useState<WarningStats>({
    total: 0,
    critical: 0,
    high: 0,
    medium: 0,
    low: 0,
    resolved: 0,
    autoResolved: 0
  });
  const [isLoading, setIsLoading] = useState(false);
  const [selectedSeverity, setSelectedSeverity] = useState<string>('all');
  const [selectedStatus, setSelectedStatus] = useState<string>('active');

  const fetchWarnings = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      // Real API call to get sharding warnings
      const response = await api.getShardingTopology();
      if (response.ok && response.data) {
        const topologyData = response.data as any;
        const warnings: ShardingWarning[] = topologyData.warnings || [];
        
        setWarnings(warnings);
        
        // Calculate stats from real data
        const stats: WarningStats = {
          total: warnings.length,
          critical: warnings.filter(w => w.severity === 'critical').length,
          high: warnings.filter(w => w.severity === 'high').length,
          medium: warnings.filter(w => w.severity === 'medium').length,
          low: warnings.filter(w => w.severity === 'low').length,
          resolved: warnings.filter(w => w.status === 'resolved').length,
          autoResolved: warnings.filter(w => w.autoResolve).length
        };
        
        setStats(stats);
      } else {
        // If no data available, show empty state
        setWarnings([]);
        setStats({
          total: 0,
          critical: 0,
          high: 0,
          medium: 0,
          low: 0,
          resolved: 0,
          autoResolved: 0
        });
      }
    } catch (error) {
      console.error('Failed to fetch sharding warnings:', error);
      setWarnings([]);
      setStats({
        total: 0,
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        resolved: 0,
        autoResolved: 0
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (serverId) {
      fetchWarnings();
      
      // Auto-refresh every 30 seconds
      const interval = setInterval(fetchWarnings, 30000);
      return () => clearInterval(interval);
    }
  }, [serverId]);

  const getSeverityIcon = (severity: ShardingWarning['severity']) => {
    switch (severity) {
      case 'critical': return <XCircle className="h-5 w-5 text-red-500" />;
      case 'high': return <AlertTriangle className="h-5 w-5 text-orange-500" />;
      case 'medium': return <AlertCircle className="h-5 w-5 text-yellow-500" />;
      case 'low': return <AlertTriangle className="h-5 w-5 text-blue-500" />;
      default: return <AlertTriangle className="h-5 w-5 text-gray-500" />;
    }
  };

  const getSeverityColor = (severity: ShardingWarning['severity']) => {
    switch (severity) {
      case 'critical': return 'destructive';
      case 'high': return 'default';
      case 'medium': return 'secondary';
      case 'low': return 'outline';
      default: return 'outline';
    }
  };

  const getStatusColor = (status: ShardingWarning['status']) => {
    switch (status) {
      case 'active': return 'bg-red-100 text-red-800';
      case 'acknowledged': return 'bg-yellow-100 text-yellow-800';
      case 'resolved': return 'bg-green-100 text-green-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getTypeIcon = (type: ShardingWarning['type']) => {
    switch (type) {
      case 'load_imbalance': return <Activity className="h-4 w-4" />;
      case 'shard_failure': return <XCircle className="h-4 w-4" />;
      case 'connection_issue': return <ExternalLink className="h-4 w-4" />;
      case 'capacity_warning': return <Users className="h-4 w-4" />;
      case 'performance_degradation': return <Cpu className="h-4 w-4" />;
      default: return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const handleAcknowledge = async (warningId: string) => {
    setWarnings(prev => prev.map(warning => 
      warning.id === warningId 
        ? { ...warning, status: 'acknowledged' as const, lastUpdated: new Date().toISOString() }
        : warning
    ));
  };

  const handleResolve = async (warningId: string) => {
    setWarnings(prev => prev.map(warning => 
      warning.id === warningId 
        ? { ...warning, status: 'resolved' as const, lastUpdated: new Date().toISOString() }
        : warning
    ));
  };

  const filteredWarnings = warnings.filter(warning => {
    const severityMatch = selectedSeverity === 'all' || warning.severity === selectedSeverity;
    const statusMatch = selectedStatus === 'all' || warning.status === selectedStatus;
    return severityMatch && statusMatch;
  });

  const activeWarnings = warnings.filter(w => w.status === 'active');
  const criticalWarnings = activeWarnings.filter(w => w.severity === 'critical');

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Critical Alerts */}
      {criticalWarnings.length > 0 && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            <strong>{criticalWarnings.length} critical warning(s) require immediate attention!</strong>
            <div className="mt-2 space-y-1">
              {criticalWarnings.map(warning => (
                <div key={warning.id} className="text-sm">
                  • {warning.title}
                </div>
              ))}
            </div>
          </AlertDescription>
        </Alert>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Warnings</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">
              {stats.resolved} resolved
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Critical</CardTitle>
            <XCircle className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">{stats.critical}</div>
            <p className="text-xs text-muted-foreground">
              Immediate action required
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">High Priority</CardTitle>
            <AlertTriangle className="h-4 w-4 text-orange-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-orange-500">{stats.high}</div>
            <p className="text-xs text-muted-foreground">
              {stats.medium + stats.low} medium/low
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Auto-Resolve</CardTitle>
            <Zap className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-500">{stats.autoResolved}</div>
            <p className="text-xs text-muted-foreground">
              Self-healing enabled
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium">Severity:</label>
          <select 
            value={selectedSeverity} 
            onChange={(e) => setSelectedSeverity(e.target.value)}
            className="px-3 py-1 border border-border rounded-md text-sm"
          >
            <option value="all">All</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium">Status:</label>
          <select 
            value={selectedStatus} 
            onChange={(e) => setSelectedStatus(e.target.value)}
            className="px-3 py-1 border border-border rounded-md text-sm"
          >
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="acknowledged">Acknowledged</option>
            <option value="resolved">Resolved</option>
          </select>
        </div>
        <Button variant="outline" size="sm" onClick={fetchWarnings} disabled={isLoading}>
          <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Warnings List */}
      <div className="flex-1 overflow-y-auto">
        <div className="space-y-4">
          {filteredWarnings.map(warning => (
            <Card key={warning.id} className={`${warning.severity === 'critical' ? 'border-red-500' : ''}`}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex items-center space-x-3">
                    {getSeverityIcon(warning.severity)}
                    <div>
                      <CardTitle className="text-lg">{warning.title}</CardTitle>
                      <div className="flex items-center space-x-2 mt-1">
                        <Badge variant={getSeverityColor(warning.severity)}>
                          {warning.severity}
                        </Badge>
                        <Badge variant="outline" className={getStatusColor(warning.status)}>
                          {warning.status}
                        </Badge>
                        {warning.autoResolve && (
                          <Badge variant="secondary">
                            <Zap className="h-3 w-3 mr-1" />
                            Auto-resolve
                          </Badge>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    {warning.estimatedResolutionTime && (
                      <div className="flex items-center space-x-1 text-sm text-muted-foreground">
                        <Clock className="h-4 w-4" />
                        <span>{warning.estimatedResolutionTime}min</span>
                      </div>
                    )}
                    {warning.status === 'active' && (
                      <Button size="sm" variant="outline" onClick={() => handleAcknowledge(warning.id)}>
                        Acknowledge
                      </Button>
                    )}
                    {warning.status === 'acknowledged' && (
                      <Button size="sm" onClick={() => handleResolve(warning.id)}>
                        Resolve
                      </Button>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <p className="text-muted-foreground">{warning.description}</p>
                  
                  <div>
                    <h4 className="font-medium mb-2">Impact:</h4>
                    <p className="text-sm text-muted-foreground">{warning.impact}</p>
                  </div>

                  <div>
                    <h4 className="font-medium mb-2">Affected Shards:</h4>
                    <div className="flex flex-wrap gap-2">
                      {warning.affectedShards.map(shardId => (
                        <Badge key={shardId} variant="outline">
                          {getTypeIcon(warning.type)}
                          <span className="ml-1">{shardId}</span>
                        </Badge>
                      ))}
                    </div>
                  </div>

                  <div>
                    <h4 className="font-medium mb-2">Suggested Actions:</h4>
                    <ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
                      {warning.suggestedActions.map((action, index) => (
                        <li key={index}>{action}</li>
                      ))}
                    </ul>
                  </div>

                  <div className="flex items-center justify-between text-xs text-muted-foreground pt-2 border-t">
                    <span>Created: {new Date(warning.createdAt).toLocaleString()}</span>
                    <span>Updated: {new Date(warning.lastUpdated).toLocaleString()}</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
          
          {filteredWarnings.length === 0 && (
            <div className="text-center text-muted-foreground py-12">
              <CheckCircle className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No warnings match the current filters</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
