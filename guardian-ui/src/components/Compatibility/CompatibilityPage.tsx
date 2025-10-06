import React, { useState, useEffect } from 'react';
import { 
  AlertTriangle, 
  CheckCircle, 
  X, 
  Download,
  Eye,
  MoreHorizontal,
  Zap,
  Package,
  FileText,
  RefreshCw,
  Search,
  Filter,
  Shield,
  Activity,
  TrendingUp,
  AlertCircle,
  Info
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';
import { apiClient as api } from '@/lib/api';

interface CompatibilityIssue {
  id: string;
  mod_id: string;
  issue_type: 'incompatibility' | 'missing_dependency' | 'performance_impact' | 'version_mismatch';
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  affected_mods: string[];
  recommendations: CompatibilityRecommendation[];
}

interface CompatibilityRecommendation {
  id: string;
  mod_id: string;
  action: 'remove' | 'update' | 'install' | 'downgrade' | 'configure';
  description: string;
  priority: number;
}

interface RiskAnalysis {
  mod_id: string;
  overall_score: number;
  risk_level: 'minimal' | 'low' | 'medium' | 'high' | 'critical';
  incompatibility_score: number;
  dependency_score: number;
  performance_score: number;
  stability_score: number;
  recommendations: string[];
}

interface CompatibilityPageProps {
  serverId: string;
  className?: string;
}

export const CompatibilityPage: React.FC<CompatibilityPageProps> = ({
  serverId,
  className = ''
}) => {
  const [issues, setIssues] = useState<CompatibilityIssue[]>([]);
  const [recommendations, setRecommendations] = useState<CompatibilityRecommendation[]>([]);
  const [riskAnalysis, setRiskAnalysis] = useState<RiskAnalysis[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterSeverity, setFilterSeverity] = useState('all');
  const [filterType, setFilterType] = useState('all');

  // Fetch compatibility data
  const fetchCompatibilityData = async () => {
    if (!serverId) return;
    
    setIsLoading(true);
    try {
      const [issuesRes, recommendationsRes, riskRes] = await Promise.all([
        api.getCompatibilityIssues(serverId),
        api.getCompatibilityRecommendations(serverId),
        api.getServerRiskAnalysis(serverId)
      ]);

      if (issuesRes.ok && issuesRes.data) {
        setIssues(issuesRes.data as CompatibilityIssue[]);
      }
      if (recommendationsRes.ok && recommendationsRes.data) {
        setRecommendations(recommendationsRes.data as CompatibilityRecommendation[]);
      }
      if (riskRes.ok && riskRes.data) {
        setRiskAnalysis(riskRes.data as RiskAnalysis[]);
      }
    } catch (error) {
      console.error('Error fetching compatibility data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchCompatibilityData();
    
    // Refresh data every 30 seconds
    const interval = setInterval(fetchCompatibilityData, 30000);
    return () => clearInterval(interval);
  }, [serverId]);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'text-red-400 bg-red-500/20 border-red-500/30';
      case 'high':
        return 'text-orange-400 bg-orange-500/20 border-orange-500/30';
      case 'medium':
        return 'text-yellow-400 bg-yellow-500/20 border-yellow-500/30';
      case 'low':
        return 'text-blue-400 bg-blue-500/20 border-blue-500/30';
      default:
        return 'text-gray-400 bg-gray-500/20 border-gray-500/30';
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return <AlertCircle className="h-4 w-4" />;
      case 'high':
        return <AlertTriangle className="h-4 w-4" />;
      case 'medium':
        return <AlertTriangle className="h-4 w-4" />;
      case 'low':
        return <Info className="h-4 w-4" />;
      default:
        return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const getIssueTypeIcon = (type: string) => {
    switch (type) {
      case 'incompatibility':
        return <X className="h-4 w-4" />;
      case 'missing_dependency':
        return <Download className="h-4 w-4" />;
      case 'performance_impact':
        return <Activity className="h-4 w-4" />;
      case 'version_mismatch':
        return <FileText className="h-4 w-4" />;
      default:
        return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const getActionIcon = (action: string) => {
    switch (action) {
      case 'remove':
        return <X className="h-4 w-4" />;
      case 'update':
        return <Zap className="h-4 w-4" />;
      case 'install':
        return <Download className="h-4 w-4" />;
      case 'downgrade':
        return <TrendingUp className="h-4 w-4" />;
      case 'configure':
        return <Shield className="h-4 w-4" />;
      default:
        return <CheckCircle className="h-4 w-4" />;
    }
  };

  const getRiskLevelColor = (level: string) => {
    switch (level) {
      case 'critical':
        return 'text-red-400 bg-red-500/20';
      case 'high':
        return 'text-orange-400 bg-orange-500/20';
      case 'medium':
        return 'text-yellow-400 bg-yellow-500/20';
      case 'low':
        return 'text-blue-400 bg-blue-500/20';
      case 'minimal':
        return 'text-green-400 bg-green-500/20';
      default:
        return 'text-gray-400 bg-gray-500/20';
    }
  };

  const handleApplyFix = async (recommendationId: string) => {
    try {
      const response = await api.applyCompatibilityFix(serverId, recommendationId);
      if (response.ok) {
        // Refresh data after applying fix
        fetchCompatibilityData();
      }
    } catch (error) {
      console.error('Error applying fix:', error);
    }
  };

  const filteredIssues = issues.filter(issue => {
    const matchesSearch = issue.mod_id.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         issue.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesSeverity = filterSeverity === 'all' || issue.severity === filterSeverity;
    const matchesType = filterType === 'all' || issue.issue_type === filterType;
    return matchesSearch && matchesSeverity && matchesType;
  });

  const filteredRecommendations = recommendations.filter(rec => {
    const matchesSearch = rec.mod_id.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         rec.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesSearch;
  });

  const filteredRiskAnalysis = riskAnalysis.filter(risk => {
    const matchesSearch = risk.mod_id.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesSearch;
  });

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-2xl font-bold">Compatibility Analysis</h2>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="flex items-center gap-1">
              <AlertTriangle className="h-3 w-3" />
              {issues.length} Issues
            </Badge>
            <Badge variant="outline" className="flex items-center gap-1">
              <Zap className="h-3 w-3" />
              {recommendations.length} Fixes
            </Badge>
          </div>
        </div>
        
        <Button
          size="sm"
          variant="outline"
          onClick={fetchCompatibilityData}
          disabled={isLoading}
        >
          <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search issues and recommendations..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Select value={filterSeverity} onValueChange={setFilterSeverity}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Severities</SelectItem>
            <SelectItem value="critical">Critical</SelectItem>
            <SelectItem value="high">High</SelectItem>
            <SelectItem value="medium">Medium</SelectItem>
            <SelectItem value="low">Low</SelectItem>
          </SelectContent>
        </Select>

        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Types</SelectItem>
            <SelectItem value="incompatibility">Incompatibility</SelectItem>
            <SelectItem value="missing_dependency">Missing Dependency</SelectItem>
            <SelectItem value="performance_impact">Performance</SelectItem>
            <SelectItem value="version_mismatch">Version Mismatch</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="issues" className="space-y-4">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="issues" className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4" />
            Issues
            {issues.length > 0 && (
              <Badge variant="destructive" className="ml-1 text-xs">
                {issues.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="recommendations" className="flex items-center gap-2">
            <Zap className="h-4 w-4" />
            Auto-Fix
            {recommendations.length > 0 && (
              <Badge variant="outline" className="ml-1 text-xs">
                {recommendations.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="risk" className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            Risk Analysis
          </TabsTrigger>
        </TabsList>

        {/* Issues Tab */}
        <TabsContent value="issues" className="space-y-4">
          {filteredIssues.length === 0 ? (
            <Card>
              <CardContent className="text-center py-12">
                <CheckCircle className="h-12 w-12 text-green-400 mx-auto mb-4" />
                <p className="text-muted-foreground">No compatibility issues detected</p>
                <p className="text-xs text-muted-foreground mt-1">
                  All mods are compatible and dependencies are satisfied
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-4">
              {filteredIssues.map((issue) => (
                <Card key={issue.id} className="border-l-4 border-l-red-500">
                  <CardContent className="p-6">
                    <div className="flex items-start justify-between">
                      <div className="flex-1 space-y-3">
                        {/* Issue Header */}
                        <div className="flex items-center gap-3">
                          <div className="flex items-center gap-2">
                            {getIssueTypeIcon(issue.issue_type)}
                            <h3 className="font-medium capitalize">
                              {issue.issue_type.replace('_', ' ')}
                            </h3>
                          </div>
                          
                          <Badge 
                            variant="outline" 
                            className={`text-xs ${getSeverityColor(issue.severity)}`}
                          >
                            {getSeverityIcon(issue.severity)}
                            <span className="ml-1 capitalize">{issue.severity}</span>
                          </Badge>
                        </div>

                        {/* Affected Mods */}
                        <div className="space-y-1">
                          <h4 className="text-sm font-medium">Affected Mods:</h4>
                          <div className="flex flex-wrap gap-2">
                            {issue.affected_mods.map((mod, index) => (
                              <Badge key={index} variant="secondary" className="text-xs">
                                <Package className="h-3 w-3 mr-1" />
                                {mod}
                              </Badge>
                            ))}
                          </div>
                        </div>

                        {/* Description */}
                        <p className="text-sm text-muted-foreground">
                          {issue.description}
                        </p>

                        {/* Recommendations */}
                        {issue.recommendations.length > 0 && (
                          <div className="space-y-2">
                            <h4 className="text-sm font-medium">Recommended Actions:</h4>
                            <div className="space-y-1">
                              {issue.recommendations.map((rec, index) => (
                                <div key={index} className="flex items-center gap-2 text-sm">
                                  {getActionIcon(rec.action)}
                                  <span>{rec.description}</span>
                                  <Badge variant="outline" className="text-xs">
                                    Priority {rec.priority}
                                  </Badge>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </TabsContent>

        {/* Recommendations Tab */}
        <TabsContent value="recommendations" className="space-y-4">
          {filteredRecommendations.length === 0 ? (
            <Card>
              <CardContent className="text-center py-12">
                <CheckCircle className="h-12 w-12 text-green-400 mx-auto mb-4" />
                <p className="text-muted-foreground">No fixes available</p>
                <p className="text-xs text-muted-foreground mt-1">
                  All compatibility issues have been resolved
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-4">
              {filteredRecommendations.map((rec) => (
                <Card key={rec.id} className="hover:bg-muted/50 transition-colors">
                  <CardContent className="p-6">
                    <div className="flex items-center justify-between">
                      <div className="flex-1 space-y-2">
                        <div className="flex items-center gap-3">
                          {getActionIcon(rec.action)}
                          <h3 className="font-medium capitalize">
                            {rec.action.replace('_', ' ')} {rec.mod_id}
                          </h3>
                          <Badge variant="outline" className="text-xs">
                            Priority {rec.priority}
                          </Badge>
                        </div>
                        <p className="text-sm text-muted-foreground">
                          {rec.description}
                        </p>
                      </div>
                      
                      <Button
                        size="sm"
                        onClick={() => handleApplyFix(rec.id)}
                        className="ml-4"
                      >
                        <Zap className="h-4 w-4 mr-1" />
                        Apply Fix
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </TabsContent>

        {/* Risk Analysis Tab */}
        <TabsContent value="risk" className="space-y-4">
          {filteredRiskAnalysis.length === 0 ? (
            <Card>
              <CardContent className="text-center py-12">
                <Shield className="h-12 w-12 text-green-400 mx-auto mb-4" />
                <p className="text-muted-foreground">No risk analysis available</p>
                <p className="text-xs text-muted-foreground mt-1">
                  Risk analysis will be available after mods are loaded
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-4">
              {filteredRiskAnalysis.map((risk) => (
                <Card key={risk.mod_id} className="hover:bg-muted/50 transition-colors">
                  <CardContent className="p-6">
                    <div className="flex items-start justify-between">
                      <div className="flex-1 space-y-3">
                        <div className="flex items-center gap-3">
                          <Package className="h-5 w-5" />
                          <h3 className="font-medium text-lg">{risk.mod_id}</h3>
                          <Badge 
                            variant="outline" 
                            className={`text-xs ${getRiskLevelColor(risk.risk_level)}`}
                          >
                            {risk.risk_level.toUpperCase()}
                          </Badge>
                        </div>

                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                          <div className="text-center">
                            <div className="text-2xl font-bold text-blue-400">
                              {(risk.overall_score * 100).toFixed(0)}%
                            </div>
                            <div className="text-xs text-muted-foreground">Overall Risk</div>
                          </div>
                          <div className="text-center">
                            <div className="text-lg font-semibold text-red-400">
                              {(risk.incompatibility_score * 100).toFixed(0)}%
                            </div>
                            <div className="text-xs text-muted-foreground">Incompatibility</div>
                          </div>
                          <div className="text-center">
                            <div className="text-lg font-semibold text-yellow-400">
                              {(risk.dependency_score * 100).toFixed(0)}%
                            </div>
                            <div className="text-xs text-muted-foreground">Dependencies</div>
                          </div>
                          <div className="text-center">
                            <div className="text-lg font-semibold text-orange-400">
                              {(risk.performance_score * 100).toFixed(0)}%
                            </div>
                            <div className="text-xs text-muted-foreground">Performance</div>
                          </div>
                        </div>

                        {risk.recommendations.length > 0 && (
                          <div className="space-y-1">
                            <h4 className="text-sm font-medium">Recommendations:</h4>
                            <ul className="text-sm text-muted-foreground space-y-1">
                              {risk.recommendations.map((rec, index) => (
                                <li key={index} className="flex items-start gap-2">
                                  <span className="text-blue-400 mt-1">•</span>
                                  <span>{rec}</span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default CompatibilityPage;
