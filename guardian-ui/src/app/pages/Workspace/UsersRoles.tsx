import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { 
  Users, 
  UserPlus, 
  UserMinus, 
  Shield, 
  Crown, 
  User, 
  Settings, 
  AlertTriangle,
  CheckCircle,
  Info,
  Edit,
  Trash2,
  Save,
  X,
  Search,
  Filter,
  MoreHorizontal,
  Lock,
  Unlock,
  Eye,
  EyeOff,
  Copy,
  Mail,
  Phone,
  Calendar,
  MapPin,
  Globe,
  Key,
  Database,
  Server,
  Activity,
  Clock,
  RefreshCw
} from 'lucide-react';

interface UserData {
  id: string;
  username: string;
  email: string;
  firstName: string;
  lastName: string;
  role: 'owner' | 'admin' | 'moderator' | 'member' | 'guest';
  status: 'active' | 'inactive' | 'suspended' | 'pending';
  permissions: string[];
  lastLoginAt: string | null;
  createdAt: string;
  isOnline: boolean;
  avatar?: string;
  phone?: string;
  location?: string;
  timezone?: string;
  language?: string;
  theme?: string;
  notifications?: boolean;
  twoFactorEnabled?: boolean;
  apiAccess?: boolean;
  serverAccess?: string[];
}

interface RoleData {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  color: string;
  isDefault: boolean;
  userCount: number;
  createdAt: string;
  updatedAt: string;
}

export const UsersRoles: React.FC = () => {
  const [users, setUsers] = useState<UserData[]>([
    {
      id: '1',
      username: 'admin',
      email: 'admin@guardian.com',
      firstName: 'Admin',
      lastName: 'User',
      role: 'owner',
      status: 'active',
      permissions: ['*'],
      lastLoginAt: '2024-01-15T10:30:00Z',
      createdAt: '2024-01-01T00:00:00Z',
      isOnline: true,
      twoFactorEnabled: true,
      apiAccess: true,
      serverAccess: ['*']
    },
    {
      id: '2',
      username: 'moderator1',
      email: 'mod@guardian.com',
      firstName: 'Moderator',
      lastName: 'One',
      role: 'moderator',
      status: 'active',
      permissions: ['read', 'write', 'moderate'],
      lastLoginAt: '2024-01-14T15:45:00Z',
      createdAt: '2024-01-01T00:00:00Z',
      isOnline: false,
      twoFactorEnabled: false,
      apiAccess: false,
      serverAccess: ['server-1', 'server-2']
    }
  ]);

  const [roles, setRoles] = useState<RoleData[]>([
    {
      id: '1',
      name: 'Owner',
      description: 'Full system access and control',
      permissions: ['*'],
      color: 'bg-purple-500',
      isDefault: false,
      userCount: 1,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z'
    },
    {
      id: '2',
      name: 'Admin',
      description: 'Administrative access to all servers',
      permissions: ['read', 'write', 'execute', 'admin'],
      color: 'bg-red-500',
      isDefault: false,
      userCount: 0,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z'
    },
    {
      id: '3',
      name: 'Moderator',
      description: 'Moderate servers and manage players',
      permissions: ['read', 'write', 'moderate'],
      color: 'bg-blue-500',
      isDefault: false,
      userCount: 1,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z'
    },
    {
      id: '4',
      name: 'Member',
      description: 'Basic access to assigned servers',
      permissions: ['read', 'write'],
      color: 'bg-green-500',
      isDefault: true,
      userCount: 0,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z'
    },
    {
      id: '5',
      name: 'Guest',
      description: 'Read-only access to public servers',
      permissions: ['read'],
      color: 'bg-gray-500',
      isDefault: true,
      userCount: 0,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z'
    }
  ]);

  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [selectedUser, setSelectedUser] = useState<UserData | null>(null);
  const [selectedRole, setSelectedRole] = useState<RoleData | null>(null);
  const [isCreatingUser, setIsCreatingUser] = useState(false);
  const [isCreatingRole, setIsCreatingRole] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterRole, setFilterRole] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');

  const fetchData = async () => {
    setIsLoading(true);
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to fetch users and roles:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'owner': return 'bg-purple-500';
      case 'admin': return 'bg-red-500';
      case 'moderator': return 'bg-blue-500';
      case 'member': return 'bg-green-500';
      case 'guest': return 'bg-gray-500';
      default: return 'bg-gray-500';
    }
  };

  const getRoleLabel = (role: string) => {
    switch (role) {
      case 'owner': return 'Owner';
      case 'admin': return 'Admin';
      case 'moderator': return 'Moderator';
      case 'member': return 'Member';
      case 'guest': return 'Guest';
      default: return 'Unknown';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-500';
      case 'inactive': return 'text-gray-500';
      case 'suspended': return 'text-red-500';
      case 'pending': return 'text-yellow-500';
      default: return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return <CheckCircle className="h-4 w-4" />;
      case 'inactive': return <X className="h-4 w-4" />;
      case 'suspended': return <AlertTriangle className="h-4 w-4" />;
      case 'pending': return <Clock className="h-4 w-4" />;
      default: return <Info className="h-4 w-4" />;
    }
  };

  const getOnlineStatus = (isOnline: boolean) => {
    return isOnline ? (
      <div className="flex items-center space-x-1 text-green-500">
        <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
        <span className="text-sm">Online</span>
      </div>
    ) : (
      <div className="flex items-center space-x-1 text-gray-500">
        <div className="w-2 h-2 bg-gray-500 rounded-full" />
        <span className="text-sm">Offline</span>
      </div>
    );
  };

  const filteredUsers = users.filter(user => {
    const matchesSearch = user.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         `${user.firstName} ${user.lastName}`.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesRole = filterRole === 'all' || user.role === filterRole;
    const matchesStatus = filterStatus === 'all' || user.status === filterStatus;
    return matchesSearch && matchesRole && matchesStatus;
  });

  const handleCreateUser = () => {
    setIsCreatingUser(true);
    // Mock user creation
    setTimeout(() => {
      const newUser: UserData = {
        id: Date.now().toString(),
        username: 'newuser',
        email: 'newuser@guardian.com',
        firstName: 'New',
        lastName: 'User',
        role: 'member',
        status: 'pending',
        permissions: ['read'],
        lastLoginAt: null,
        createdAt: new Date().toISOString(),
        isOnline: false,
        twoFactorEnabled: false,
        apiAccess: false,
        serverAccess: []
      };
      setUsers(prev => [...prev, newUser]);
      setIsCreatingUser(false);
    }, 1000);
  };

  const handleCreateRole = () => {
    setIsCreatingRole(true);
    // Mock role creation
    setTimeout(() => {
      const newRole: RoleData = {
        id: Date.now().toString(),
        name: 'New Role',
        description: 'A new role',
        permissions: ['read'],
        color: 'bg-blue-500',
        isDefault: false,
        userCount: 0,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      };
      setRoles(prev => [...prev, newRole]);
      setIsCreatingRole(false);
    }, 1000);
  };

  const handleDeleteUser = (id: string) => {
    setUsers(prev => prev.filter(user => user.id !== id));
  };

  const handleDeleteRole = (id: string) => {
    setRoles(prev => prev.filter(role => role.id !== id));
  };

  const handleToggleUserStatus = (id: string) => {
    setUsers(prev => prev.map(user => 
      user.id === id ? { 
        ...user, 
        status: user.status === 'active' ? 'inactive' : 'active' 
      } : user
    ));
  };

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Users Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Users className="h-5 w-5" />
              <span>Users</span>
            </div>
            <Button onClick={handleCreateUser} disabled={isCreatingUser}>
              <UserPlus className="h-4 w-4 mr-2" />
              {isCreatingUser ? 'Creating...' : 'Add User'}
            </Button>
          </CardTitle>
          <CardDescription>
            Manage user accounts and permissions
          </CardDescription>
        </CardHeader>
        <CardContent>
          {/* Search and Filters */}
          <div className="flex items-center space-x-4 mb-6">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search users..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <Select value={filterRole} onValueChange={setFilterRole}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="Filter by role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Roles</SelectItem>
                <SelectItem value="owner">Owner</SelectItem>
                <SelectItem value="admin">Admin</SelectItem>
                <SelectItem value="moderator">Moderator</SelectItem>
                <SelectItem value="member">Member</SelectItem>
                <SelectItem value="guest">Guest</SelectItem>
              </SelectContent>
            </Select>
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="Filter by status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="active">Active</SelectItem>
                <SelectItem value="inactive">Inactive</SelectItem>
                <SelectItem value="suspended">Suspended</SelectItem>
                <SelectItem value="pending">Pending</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Users List */}
          <div className="space-y-4">
            {filteredUsers.map((user) => (
              <div key={user.id} className="flex items-center justify-between p-4 border rounded-lg">
                <div className="flex items-center space-x-4">
                  <div className="w-10 h-10 bg-muted rounded-full flex items-center justify-center">
                    <User className="h-5 w-5" />
                  </div>
                  <div>
                    <div className="font-medium">{user.username}</div>
                    <div className="text-sm text-muted-foreground">{user.email}</div>
                    <div className="flex items-center space-x-2 mt-1">
                      <Badge className={`${getRoleColor(user.role)} text-white`}>
                        {getRoleLabel(user.role)}
                      </Badge>
                      <div className={`flex items-center space-x-1 ${getStatusColor(user.status)}`}>
                        {getStatusIcon(user.status)}
                        <span className="text-sm capitalize">{user.status}</span>
                      </div>
                      {getOnlineStatus(user.isOnline)}
                    </div>
                  </div>
                </div>
                
                <div className="flex items-center space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSelectedUser(user)}
                  >
                    <Edit className="h-4 w-4" />
                  </Button>
                  
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleToggleUserStatus(user.id)}
                  >
                    {user.status === 'active' ? <Unlock className="h-4 w-4" /> : <Lock className="h-4 w-4" />}
                  </Button>
                  
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDeleteUser(user.id)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Roles Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Shield className="h-5 w-5" />
              <span>Roles</span>
            </div>
            <Button onClick={handleCreateRole} disabled={isCreatingRole}>
              <UserPlus className="h-4 w-4 mr-2" />
              {isCreatingRole ? 'Creating...' : 'Add Role'}
            </Button>
          </CardTitle>
          <CardDescription>
            Manage user roles and permissions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {roles.map((role) => (
              <div key={role.id} className="p-4 border rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center space-x-2">
                    <div className={`w-3 h-3 rounded-full ${role.color}`} />
                    <span className="font-medium">{role.name}</span>
                    {role.isDefault && (
                      <Badge variant="outline" className="text-xs">Default</Badge>
                    )}
                  </div>
                  <div className="flex items-center space-x-1">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setSelectedRole(role)}
                    >
                      <Edit className="h-3 w-3" />
                    </Button>
                    {!role.isDefault && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDeleteRole(role.id)}
                      >
                        <Trash2 className="h-3 w-3" />
                      </Button>
                    )}
                  </div>
                </div>
                
                <p className="text-sm text-muted-foreground mb-3">{role.description}</p>
                
                <div className="space-y-2">
                  <div className="text-sm">
                    <span className="text-muted-foreground">Users: </span>
                    <span className="font-medium">{role.userCount}</span>
                  </div>
                  
                  <div className="text-sm">
                    <span className="text-muted-foreground">Permissions: </span>
                    <div className="flex flex-wrap gap-1 mt-1">
                      {role.permissions.slice(0, 3).map((permission) => (
                        <Badge key={permission} variant="secondary" className="text-xs">
                          {permission}
                        </Badge>
                      ))}
                      {role.permissions.length > 3 && (
                        <Badge variant="secondary" className="text-xs">
                          +{role.permissions.length - 3} more
                        </Badge>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
