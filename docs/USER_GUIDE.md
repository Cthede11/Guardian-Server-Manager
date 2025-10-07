# Guardian Server Manager - User Guide

## Getting Started

Guardian Server Manager is a comprehensive tool for managing Minecraft servers with advanced features like mod management, GPU acceleration, and automated backups.

### System Requirements

- **Operating System**: Windows 10/11 (primary), Linux (supported)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 10GB free space minimum
- **Java**: Java 17 or higher (auto-detected)
- **GPU**: Optional, for acceleration features

### Installation

1. Download the latest release from the GitHub repository
2. Extract the files to your desired location
3. Run `guardian-server-manager.exe` (Windows) or `./guardian-server-manager` (Linux)
4. The application will start and open in your default web browser

## First Time Setup

### 1. API Keys Configuration

For full functionality, you'll need API keys from mod providers:

#### CurseForge API Key
1. Go to [CurseForge Developer Portal](https://console.curseforge.com/)
2. Create an account and generate an API key
3. In Guardian, go to Settings → API Keys
4. Enter your CurseForge API key

#### Modrinth API Key
1. Go to [Modrinth Developer Portal](https://modrinth.com/developers)
2. Create an account and generate an API key
3. In Guardian, go to Settings → API Keys
4. Enter your Modrinth API key

### 2. Java Detection

Guardian will automatically detect Java installations on your system. If you have multiple Java versions, you can specify which one to use in server settings.

## Creating Your First Server

### Using the Server Creation Wizard

1. Click "Create Server" on the main dashboard
2. Follow the 4-step wizard:

#### Step 1: Basics
- **Server Name**: Choose a unique name (letters, numbers, hyphens only)
- **Minecraft Version**: Select from available versions
- **Loader**: Choose Vanilla, Fabric, Forge, or Quilt
- **Memory**: Allocate RAM (minimum 1GB, recommended 2-4GB)
- **Port**: Server port (default 25565)

#### Step 2: Mods & Modpacks
- **Modpack**: Select a modpack to install (optional)
- **Individual Mods**: Add specific mods (optional)
- **Search**: Use the search to find mods by name

#### Step 3: World & Performance
- **World Seed**: Enter a seed for world generation
- **World Type**: Choose from Default, Flat, Large Biomes, etc.
- **Difficulty**: Peaceful, Easy, Normal, or Hard
- **Game Rules**: Configure PvP, command blocks, etc.
- **Performance**: Set view distance, simulation distance

#### Step 4: Review & Create
- Review all settings
- Click "Create Server" to begin installation

### Server Creation Process

The creation process includes:
1. **Validation**: Checking all parameters
2. **Loader Installation**: Installing Fabric/Forge/Quilt if needed
3. **Modpack Application**: Installing selected modpack
4. **Mod Installation**: Installing individual mods
5. **Configuration**: Setting up server properties

Progress is shown in real-time with WebSocket updates.

## Managing Servers

### Server Dashboard

Each server has its own dashboard with:

- **Status**: Running, Stopped, Starting, Stopping
- **Players**: Online player count and list
- **Performance**: TPS, memory usage, CPU usage
- **Console**: Real-time server console output
- **Files**: Access to server files and configuration

### Starting and Stopping Servers

1. **Start Server**: Click the "Start" button on the server dashboard
2. **Stop Server**: Click the "Stop" button (graceful shutdown)
3. **Restart Server**: Click the "Restart" button for quick restart

### Server Console

The console provides:
- Real-time server output
- Command input (type commands and press Enter)
- Log filtering and search
- Export logs for debugging

### Server Settings

Access server settings from the server dashboard:

#### General Settings
- Server name and description
- Auto-start on system boot
- Auto-restart on crash
- JVM arguments

#### World Settings
- World seed and type
- Difficulty and game rules
- Player limits and permissions

#### Performance Settings
- Memory allocation
- View distance and simulation distance
- Chunk loading settings

#### Backup Settings
- Automatic backups
- Backup frequency and retention
- Backup compression

## Mod Management

### Installing Mods

1. Go to the Mod Browser
2. Search for mods by name
3. Select the mod and version
4. Choose "Install to Server"
5. Select target server
6. Confirm installation

### Installing Modpacks

1. Go to the Mod Browser
2. Switch to "Modpacks" tab
3. Search for modpacks
4. Select a modpack
5. Choose "Apply to Server"
6. Select target server
7. Confirm application

### Managing Installed Mods

1. Go to your server dashboard
2. Click on "Mods" tab
3. View installed mods
4. Enable/disable mods
5. Uninstall mods
6. Check for updates

### Mod Dependencies

Guardian automatically handles mod dependencies:
- Required dependencies are installed automatically
- Dependency conflicts are detected and reported
- Version compatibility is checked

## GPU Acceleration (Experimental)

### Enabling GPU Acceleration

1. Go to Settings → GPU
2. Toggle "Enable GPU Worker"
3. Configure GPU settings
4. Monitor GPU metrics

### GPU Features

- **Chunk Generation**: Accelerated world generation
- **Lighting Calculations**: Faster lighting updates
- **Pregeneration**: GPU-accelerated world pregeneration
- **Performance Monitoring**: Real-time GPU metrics

### GPU Requirements

- **NVIDIA**: GTX 1060 or better (CUDA support)
- **AMD**: RX 580 or better (OpenCL support)
- **Intel**: Arc A380 or better (experimental)
- **Drivers**: Latest GPU drivers required

## Backups and Restore

### Automatic Backups

Configure automatic backups in server settings:
- **Frequency**: Every 1-168 hours
- **Retention**: Keep 1-365 backups
- **Compression**: Enable/disable compression
- **Location**: Choose backup directory

### Manual Backups

1. Go to server dashboard
2. Click "Backups" tab
3. Click "Create Backup"
4. Enter backup name and description
5. Confirm creation

### Restoring Backups

1. Go to server dashboard
2. Click "Backups" tab
3. Select a backup
4. Click "Restore"
5. Confirm restoration

## Performance Monitoring

### Server Metrics

Monitor server performance with:
- **TPS**: Ticks per second (target: 20)
- **Memory Usage**: RAM consumption
- **CPU Usage**: Processor utilization
- **Player Count**: Online players

### System Metrics

Monitor system resources:
- **CPU Usage**: Overall system CPU
- **Memory Usage**: System RAM usage
- **Disk Usage**: Storage space
- **GPU Usage**: GPU utilization (if enabled)

### Alerts and Notifications

Set up alerts for:
- Low TPS (< 15)
- High memory usage (> 90%)
- Server crashes
- Backup failures

## Troubleshooting

### Common Issues

#### Server Won't Start
1. Check Java version compatibility
2. Verify memory allocation
3. Check port availability
4. Review console logs for errors

#### Mods Not Loading
1. Verify mod compatibility with Minecraft version
2. Check loader compatibility (Fabric/Forge)
3. Review mod dependencies
4. Check for mod conflicts

#### Performance Issues
1. Increase memory allocation
2. Reduce view distance
3. Disable unnecessary mods
4. Enable GPU acceleration (if available)

#### Connection Issues
1. Check firewall settings
2. Verify port forwarding
3. Check server.properties
4. Review network configuration

### Logs and Debugging

#### Server Logs
- Access via server console
- Export logs for analysis
- Filter by log level
- Search for specific errors

#### System Logs
- Check Guardian application logs
- Review GPU worker logs
- Monitor system resource usage
- Check network connectivity

### Getting Help

1. **Documentation**: Check this user guide
2. **GitHub Issues**: Report bugs and request features
3. **Discord Community**: Join the community server
4. **FAQ**: Check frequently asked questions

## Advanced Configuration

### Environment Variables

Configure Guardian using environment variables:

```bash
# API Keys
CURSEFORGE_API_KEY=your_curseforge_key
MODRINTH_API_KEY=your_modrinth_key

# Server Configuration
GUARDIAN_PORT=52100
GUARDIAN_HOST=127.0.0.1

# Database
DATABASE_URL=sqlite:guardian.db

# GPU Configuration
GPU_ENABLED=false
GPU_WORKER_PATH=./gpu-worker.exe

# Logging
RUST_LOG=info
LOG_LEVEL=info
```

### Configuration Files

Guardian uses several configuration files:
- `guardian.yaml`: Main configuration
- `servers/`: Server-specific configurations
- `backups/`: Backup configurations
- `logs/`: Log files

### Custom JVM Arguments

Configure custom JVM arguments for servers:
- **Memory**: `-Xmx4G -Xms2G`
- **Garbage Collection**: `-XX:+UseG1GC`
- **Performance**: `-XX:+UseStringDeduplication`
- **Debugging**: `-Xdebug -Xrunjdwp:transport=dt_socket,server=y,suspend=n,address=5005`

## Security Considerations

### Network Security
- Guardian binds to localhost by default
- No external network access without configuration
- Firewall rules should be configured appropriately

### File System Security
- Path sanitization prevents directory traversal
- Secure file permissions
- Regular security updates

### API Security
- Rate limiting prevents abuse
- Input validation prevents injection
- No sensitive data in error responses

## Best Practices

### Server Management
1. **Regular Backups**: Set up automatic backups
2. **Monitor Performance**: Keep an eye on TPS and memory
3. **Update Regularly**: Keep mods and server updated
4. **Test Changes**: Test mods in a separate server first

### Resource Management
1. **Memory Allocation**: Don't allocate more RAM than available
2. **CPU Usage**: Monitor CPU usage across all servers
3. **Storage Space**: Keep sufficient free space for backups
4. **Network Bandwidth**: Consider bandwidth for multiple servers

### Mod Management
1. **Compatibility**: Check mod compatibility before installing
2. **Dependencies**: Install required dependencies
3. **Updates**: Keep mods updated for security and performance
4. **Testing**: Test mod combinations before production use

## Frequently Asked Questions

### Q: Can I run multiple servers at once?
A: Yes, Guardian supports multiple concurrent servers, limited by your system resources.

### Q: What Minecraft versions are supported?
A: Guardian supports Minecraft 1.16.5 and newer, with automatic detection of available versions.

### Q: Can I use Guardian without API keys?
A: Yes, but you'll be limited to basic server management without mod search and installation features.

### Q: Is GPU acceleration safe?
A: GPU acceleration is experimental and disabled by default. It may cause instability on some systems.

### Q: How do I update Guardian?
A: Download the latest release and replace the executable. Your servers and configurations will be preserved.

### Q: Can I migrate from other server managers?
A: Guardian can import server configurations from some other managers. Check the migration guide for details.

### Q: Does Guardian work on Linux?
A: Yes, Guardian supports Linux, though Windows is the primary platform.

### Q: How much RAM do I need?
A: Minimum 4GB, but 8GB+ is recommended for multiple servers or modded servers.

### Q: Can I use Guardian for production servers?
A: Yes, but ensure you have proper backups, monitoring, and security measures in place.

### Q: Is there a web interface?
A: Yes, Guardian provides a web interface that opens automatically when you start the application.