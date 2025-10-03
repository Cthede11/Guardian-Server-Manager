# Guardian Implementation Plan

## 🎯 Phase 1: Core Infrastructure (Week 1-2)

### **1.1 File Structure Implementation**

#### **Backend Changes (hostd)**
```rust
// Add to hostd/src/config.rs
pub struct GuardianConfig {
    pub app_data_dir: PathBuf,
    pub servers_dir: PathBuf,
    pub shared_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl GuardianConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let base_dir = dirs::data_dir()
            .ok_or("Could not find data directory")?
            .join("Guardian");
        
        Ok(Self {
            app_data_dir: base_dir.clone(),
            servers_dir: base_dir.join("servers"),
            shared_dir: base_dir.join("shared"),
            templates_dir: base_dir.join("templates"),
            logs_dir: base_dir.join("logs"),
        })
    }
}
```

#### **Frontend Changes**
```typescript
// Add to src/lib/config.ts
export interface AppConfig {
  dataDirectory: string;
  serversDirectory: string;
  sharedDirectory: string;
  templatesDirectory: string;
  logsDirectory: string;
  javaInstallations: JavaInstallation[];
  defaultServerSettings: ServerSettings;
}

export interface JavaInstallation {
  id: string;
  version: string;
  path: string;
  isDefault: boolean;
}
```

### **1.2 Server Configuration System**

#### **Server Configuration Schema**
```json
{
  "id": "server-uuid",
  "name": "My Server",
  "type": "vanilla|forge|fabric|paper|purpur",
  "version": "1.21.1",
  "java": {
    "path": "C:\\Program Files\\Java\\jdk-21\\bin\\java.exe",
    "args": "-Xmx4G -Xms2G -XX:+UseG1GC",
    "version": "21"
  },
  "network": {
    "serverPort": 25565,
    "rconPort": 25575,
    "rconPassword": "encrypted_password",
    "queryPort": 25565
  },
  "paths": {
    "world": "./world",
    "mods": "./mods",
    "config": "./config",
    "logs": "./logs",
    "backups": "./backups"
  },
  "settings": {
    "autoStart": false,
    "autoRestart": true,
    "maxRestarts": 3,
    "backupInterval": 24,
    "backupRetention": 7
  },
  "created": "2024-01-15T10:00:00Z",
  "lastModified": "2024-01-15T10:00:00Z",
  "lastStarted": "2024-01-15T10:00:00Z"
}
```

### **1.3 Directory Management**

#### **Create Directory Structure**
```typescript
// Add to src/lib/file-manager.ts
export class FileManager {
  async createServerDirectory(serverId: string, config: ServerConfig): Promise<void> {
    const serverDir = path.join(this.config.serversDirectory, serverId);
    
    // Create directory structure
    await fs.mkdir(serverDir, { recursive: true });
    await fs.mkdir(path.join(serverDir, 'world'), { recursive: true });
    await fs.mkdir(path.join(serverDir, 'mods'), { recursive: true });
    await fs.mkdir(path.join(serverDir, 'config'), { recursive: true });
    await fs.mkdir(path.join(serverDir, 'logs'), { recursive: true });
    await fs.mkdir(path.join(serverDir, 'backups'), { recursive: true });
    await fs.mkdir(path.join(serverDir, 'temp'), { recursive: true });
    
    // Create configuration file
    await fs.writeFile(
      path.join(serverDir, 'guardian.json'),
      JSON.stringify(config, null, 2)
    );
  }
}
```

## 🎯 Phase 2: Server Management (Week 3-4)

### **2.1 Server Creation Wizard Enhancement**

#### **Multi-Step Wizard Implementation**
```typescript
// Update src/app/layout/Sidebar.tsx
const AddServerWizard: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [formData, setFormData] = useState<ServerCreationData>({
    // Step 1: Basic Info
    name: '',
    type: 'vanilla',
    version: '1.21.1',
    
    // Step 2: Java Configuration
    javaPath: '',
    javaArgs: '-Xmx4G -Xms2G',
    memory: 4096,
    
    // Step 3: Network Configuration
    serverPort: 25565,
    rconPort: 25575,
    rconPassword: '',
    queryPort: 25565,
    
    // Step 4: File Paths
    paths: {
      world: './world',
      mods: './mods',
      config: './config'
    },
    
    // Step 5: Advanced Settings
    settings: {
      autoStart: false,
      autoRestart: true,
      maxRestarts: 3,
      backupInterval: 24,
      backupRetention: 7
    }
  });

  const steps = [
    { id: 1, title: 'Basic Info', description: 'Server name and type' },
    { id: 2, title: 'Java Config', description: 'Java version and memory' },
    { id: 3, title: 'Network', description: 'Ports and RCON' },
    { id: 4, title: 'File Paths', description: 'Directory structure' },
    { id: 5, title: 'Advanced', description: 'Auto-start and backups' }
  ];
};
```

### **2.2 Server Process Management**

#### **Backend Process Control**
```rust
// Add to hostd/src/minecraft.rs
pub struct MinecraftServer {
    pub id: String,
    pub config: ServerConfig,
    pub process: Option<Child>,
    pub status: ServerStatus,
    pub health: ServerHealth,
}

impl MinecraftServer {
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::new(&self.config.java.path);
        cmd.args(self.config.java.args.split_whitespace());
        cmd.arg("-jar");
        cmd.arg("server.jar");
        cmd.arg("nogui");
        cmd.current_dir(&self.config.paths.server_dir);
        
        self.process = Some(cmd.spawn()?);
        self.status = ServerStatus::Starting;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
            process.wait()?;
        }
        self.status = ServerStatus::Stopped;
        Ok(())
    }
}
```

### **2.3 Real-time Monitoring**

#### **WebSocket Integration**
```typescript
// Add to src/lib/websocket.ts
export class RealtimeConnection {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  
  connect(serverId: string) {
    this.ws = new WebSocket(`ws://localhost:8080/ws/servers/${serverId}`);
    
    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.handleMessage(data);
    };
    
    this.ws.onclose = () => {
      this.handleReconnect(serverId);
    };
  }
  
  private handleMessage(data: any) {
    switch (data.type) {
      case 'console':
        this.updateConsole(data.message);
        break;
      case 'metrics':
        this.updateMetrics(data.metrics);
        break;
      case 'players':
        this.updatePlayers(data.players);
        break;
      case 'health':
        this.updateHealth(data.health);
        break;
    }
  }
}
```

## 🎯 Phase 3: Advanced Features (Week 5-6)

### **3.1 Backup System**

#### **Automated Backup Management**
```typescript
// Add to src/lib/backup-manager.ts
export class BackupManager {
  async createBackup(serverId: string, type: 'manual' | 'scheduled'): Promise<BackupInfo> {
    const serverDir = path.join(this.config.serversDirectory, serverId);
    const backupId = generateId();
    const backupDir = path.join(serverDir, 'backups', backupId);
    
    // Create backup directory
    await fs.mkdir(backupDir, { recursive: true });
    
    // Copy world data
    await this.copyDirectory(
      path.join(serverDir, 'world'),
      path.join(backupDir, 'world')
    );
    
    // Copy server configuration
    await fs.copyFile(
      path.join(serverDir, 'server.properties'),
      path.join(backupDir, 'server.properties')
    );
    
    // Create backup metadata
    const backupInfo: BackupInfo = {
      id: backupId,
      serverId,
      type,
      createdAt: new Date().toISOString(),
      size: await this.calculateSize(backupDir),
      status: 'completed'
    };
    
    await fs.writeFile(
      path.join(backupDir, 'backup.json'),
      JSON.stringify(backupInfo, null, 2)
    );
    
    return backupInfo;
  }
}
```

### **3.2 Mod Management**

#### **Mod Installation and Compatibility**
```typescript
// Add to src/lib/mod-manager.ts
export class ModManager {
  async installMod(serverId: string, modFile: File): Promise<void> {
    const serverDir = path.join(this.config.serversDirectory, serverId);
    const modsDir = path.join(serverDir, 'mods');
    
    // Validate mod file
    const modInfo = await this.validateMod(modFile);
    if (!modInfo.isValid) {
      throw new Error(`Invalid mod file: ${modInfo.error}`);
    }
    
    // Check for conflicts
    const conflicts = await this.checkConflicts(serverId, modInfo);
    if (conflicts.length > 0) {
      throw new Error(`Mod conflicts detected: ${conflicts.join(', ')}`);
    }
    
    // Install mod
    await fs.copyFile(modFile.path, path.join(modsDir, modFile.name));
    
    // Update mod registry
    await this.updateModRegistry(serverId, modInfo);
  }
  
  async checkConflicts(serverId: string, modInfo: ModInfo): Promise<string[]> {
    const existingMods = await this.getInstalledMods(serverId);
    const conflicts: string[] = [];
    
    for (const existingMod of existingMods) {
      if (this.hasConflict(modInfo, existingMod)) {
        conflicts.push(existingMod.name);
      }
    }
    
    return conflicts;
  }
}
```

### **3.3 Performance Monitoring**

#### **Real-time Metrics Collection**
```typescript
// Add to src/lib/metrics-collector.ts
export class MetricsCollector {
  private metrics: Map<string, MetricData> = new Map();
  
  async collectMetrics(serverId: string): Promise<MetricData> {
    const server = await this.getServer(serverId);
    if (!server) return null;
    
    const metrics: MetricData = {
      timestamp: Date.now(),
      tps: await this.getTPS(server),
      memory: await this.getMemoryUsage(server),
      cpu: await this.getCPUUsage(server),
      players: await this.getPlayerCount(server),
      world: await this.getWorldMetrics(server)
    };
    
    this.metrics.set(serverId, metrics);
    return metrics;
  }
  
  private async getTPS(server: MinecraftServer): Promise<number> {
    // Query server for TPS data
    const response = await this.sendCommand(server, 'tps');
    return this.parseTPS(response);
  }
}
```

## 🎯 Phase 4: User Experience (Week 7-8)

### **4.1 Dashboard Enhancement**

#### **Comprehensive Dashboard**
```typescript
// Update src/app/pages/Servers/Overview.tsx
export const Overview: React.FC = () => {
  const { selectedServer } = useServersStore();
  const { metrics, health } = useRealtimeStore();
  
  return (
    <div className="space-y-6">
      {/* Server Status Card */}
      <ServerStatusCard server={selectedServer} />
      
      {/* Real-time Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="TPS"
          value={metrics?.tps || 0}
          trend={metrics?.tpsTrend}
          icon={<Activity className="h-4 w-4" />}
        />
        <MetricCard
          title="Players"
          value={`${metrics?.playersOnline || 0}/${metrics?.playersMax || 0}`}
          trend={metrics?.playersTrend}
          icon={<Users className="h-4 w-4" />}
        />
        <MetricCard
          title="Memory"
          value={`${metrics?.memoryUsed || 0}MB`}
          trend={metrics?.memoryTrend}
          icon={<MemoryStick className="h-4 w-4" />}
        />
        <MetricCard
          title="Tick Time"
          value={`${metrics?.tickTime || 0}ms`}
          trend={metrics?.tickTrend}
          icon={<Clock className="h-4 w-4" />}
        />
      </div>
      
      {/* Health Status */}
      <HealthStatusCard health={health} />
      
      {/* Recent Activity */}
      <ActivityFeed serverId={selectedServer?.id} />
    </div>
  );
};
```

### **4.2 Settings Management**

#### **Comprehensive Settings System**
```typescript
// Add to src/components/Settings/ServerSettings.tsx
export const ServerSettings: React.FC = () => {
  const { selectedServer, updateServerSettings } = useServersStore();
  const [settings, setSettings] = useState<ServerSettings>({});
  
  const handleSave = async () => {
    try {
      await updateServerSettings(selectedServer.id, settings);
      toast.success('Settings saved successfully');
    } catch (error) {
      toast.error('Failed to save settings');
    }
  };
  
  return (
    <Tabs defaultValue="general" className="space-y-4">
      <TabsList>
        <TabsTrigger value="general">General</TabsTrigger>
        <TabsTrigger value="java">Java</TabsTrigger>
        <TabsTrigger value="network">Network</TabsTrigger>
        <TabsTrigger value="world">World</TabsTrigger>
        <TabsTrigger value="mods">Mods</TabsTrigger>
        <TabsTrigger value="backups">Backups</TabsTrigger>
      </TabsList>
      
      <TabsContent value="general">
        <GeneralSettings settings={settings} onChange={setSettings} />
      </TabsContent>
      
      <TabsContent value="java">
        <JavaSettings settings={settings} onChange={setSettings} />
      </TabsContent>
      
      {/* Other tabs... */}
      
      <div className="flex justify-end space-x-2">
        <Button variant="outline" onClick={handleCancel}>
          Cancel
        </Button>
        <Button onClick={handleSave}>
          Save Changes
        </Button>
      </div>
    </Tabs>
  );
};
```

## 🎯 Phase 5: Production Readiness (Week 9-10)

### **5.1 Error Handling & Logging**

#### **Comprehensive Error Handling**
```typescript
// Add to src/lib/error-handler.ts
export class ErrorHandler {
  static handle(error: Error, context: string): void {
    console.error(`[${context}] ${error.message}`, error);
    
    // Log to file
    this.logToFile(error, context);
    
    // Show user-friendly message
    this.showUserMessage(error, context);
    
    // Report to analytics (if enabled)
    this.reportError(error, context);
  }
  
  private static logToFile(error: Error, context: string): void {
    const logEntry = {
      timestamp: new Date().toISOString(),
      context,
      message: error.message,
      stack: error.stack,
      userAgent: navigator.userAgent
    };
    
    // Write to log file
    fs.appendFileSync(
      path.join(this.config.logsDirectory, 'error.log'),
      JSON.stringify(logEntry) + '\n'
    );
  }
}
```

### **5.2 Performance Optimization**

#### **Code Splitting and Lazy Loading**
```typescript
// Update src/app/routes.tsx
const Overview = lazy(() => import('./pages/Servers/Overview'));
const Console = lazy(() => import('./pages/Servers/Console'));
const Players = lazy(() => import('./pages/Servers/Players'));
const World = lazy(() => import('./pages/Servers/World'));
const ModsRules = lazy(() => import('./pages/Servers/ModsRules'));
const Performance = lazy(() => import('./pages/Servers/Performance'));
const Backups = lazy(() => import('./pages/Servers/Backups'));
const Events = lazy(() => import('./pages/Servers/Events'));
const Pregen = lazy(() => import('./pages/Servers/Pregen'));
const Sharding = lazy(() => import('./pages/Servers/Sharding'));
const Diagnostics = lazy(() => import('./pages/Servers/Diagnostics'));
const Settings = lazy(() => import('./pages/Servers/Settings'));
```

### **5.3 Testing & Quality Assurance**

#### **Comprehensive Testing Suite**
```typescript
// Add to src/__tests__/integration/server-management.test.ts
describe('Server Management Integration', () => {
  test('should create server successfully', async () => {
    const serverData = {
      name: 'Test Server',
      type: 'vanilla',
      version: '1.21.1'
    };
    
    const result = await api.createServer(serverData);
    expect(result.ok).toBe(true);
    expect(result.data).toBeDefined();
  });
  
  test('should start server successfully', async () => {
    const server = await createTestServer();
    const result = await api.startServer(server.id);
    expect(result.ok).toBe(true);
  });
  
  test('should handle server creation errors gracefully', async () => {
    const invalidData = { name: '', type: 'invalid' };
    const result = await api.createServer(invalidData);
    expect(result.ok).toBe(false);
    expect(result.error).toBeDefined();
  });
});
```

## 🚀 Deployment Strategy

### **1. Build Process**
```bash
# Frontend build
npm run build

# Tauri build
npm run tauri:build

# Package for distribution
npm run package:all
```

### **2. Installation Process**
1. **Download**: User downloads MSI installer
2. **Install**: One-click installation with admin privileges
3. **Setup**: Automatic directory structure creation
4. **Configure**: First-run configuration wizard
5. **Ready**: App ready for server management

### **3. Update Process**
1. **Check**: Automatic update checking on startup
2. **Download**: Background download of new version
3. **Install**: Seamless update installation
4. **Restart**: Automatic app restart with new version

This implementation plan provides a clear roadmap for building a professional-grade Minecraft server management platform that scales from individual users to enterprise deployments.
