# Guardian Server File Structure

## Directory Layout

```
guardian-servers/
├── servers/                    # All server instances
│   ├── {server-id}/           # Individual server directory
│   │   ├── server.properties  # Server configuration
│   │   ├── server.jar         # Server JAR file
│   │   ├── world/             # World data
│   │   ├── mods/              # Mod files
│   │   ├── config/            # Mod configurations
│   │   ├── logs/              # Server logs
│   │   ├── backups/           # Server backups
│   │   └── guardian.json      # Guardian-specific config
│   └── templates/             # Server templates
│       ├── vanilla/           # Vanilla server template
│       ├── forge/             # Forge server template
│       └── fabric/            # Fabric server template
├── shared/                    # Shared resources
│   ├── java/                  # Java installations
│   ├── mods/                  # Shared mod files
│   └── configs/               # Shared configurations
└── guardian/                  # Guardian app data
    ├── config.json            # App configuration
    ├── servers.json           # Server registry
    └── logs/                  # App logs
```

## Server Configuration Format

Each server has a `guardian.json` file with:

```json
{
  "id": "server-uuid",
  "name": "Server Name",
  "type": "vanilla|forge|fabric",
  "version": "1.20.1",
  "java": {
    "path": "/path/to/java",
    "args": "-Xmx4G -Xms2G"
  },
  "paths": {
    "world": "./world",
    "mods": "./mods",
    "config": "./config"
  },
  "settings": {
    "port": 25565,
    "rcon": {
      "enabled": true,
      "port": 25575,
      "password": "password"
    }
  },
  "created": "2024-01-15T10:00:00Z",
  "lastModified": "2024-01-15T10:00:00Z"
}
```

## Server Management Features

1. **Server Creation Wizard**
   - Choose server type (Vanilla, Forge, Fabric)
   - Select Minecraft version
   - Configure Java settings
   - Set up file paths

2. **Server Templates**
   - Pre-configured server setups
   - Easy server creation from templates
   - Custom template creation

3. **File Management**
   - Browse server files
   - Upload/download mods
   - Edit configuration files
   - Manage world data

4. **Backup System**
   - Automated backups
   - Manual backup creation
   - Backup restoration
   - Backup scheduling

5. **Performance Monitoring**
   - Real-time metrics
   - Performance analysis
   - Resource usage tracking
   - Optimization suggestions
