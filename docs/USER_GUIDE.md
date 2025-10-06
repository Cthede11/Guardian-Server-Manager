# Guardian Server Manager - User Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [First Run Setup](#first-run-setup)
3. [Server Management](#server-management)
4. [Mod Management](#mod-management)
5. [Performance Monitoring](#performance-monitoring)
6. [Backup Management](#backup-management)
7. [Settings Configuration](#settings-configuration)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Features](#advanced-features)

## Getting Started

### System Requirements

**Minimum Requirements:**
- Windows 10/11, macOS 10.15+, or Linux (Ubuntu 20.04+)
- 8GB RAM (16GB recommended)
- 2GB free disk space
- Java 17+ (for Minecraft servers)
- Modern web browser (Chrome, Firefox, Safari, Edge)

**Recommended Requirements:**
- 16GB+ RAM
- SSD storage
- Dedicated GPU (for GPU-accelerated features)
- Stable internet connection

### Installation

1. **Download the Application**
   - Download the latest release from the GitHub releases page
   - Choose the appropriate installer for your operating system

2. **Install Guardian Server Manager**
   - **Windows**: Run the `.exe` installer and follow the setup wizard
   - **macOS**: Open the `.dmg` file and drag the app to Applications
   - **Linux**: Extract the `.AppImage` file and make it executable

3. **Launch the Application**
   - The application will start automatically after installation
   - On first run, you'll be guided through the setup wizard

## First Run Setup

When you first launch Guardian Server Manager, you'll be presented with a setup wizard to configure your environment.

### Step 1: API Keys Configuration

**CurseForge API Key:**
1. Visit [CurseForge API](https://console.curseforge.com/)
2. Create an account and generate an API key
3. Enter the API key in the setup wizard

**Modrinth API Key (Optional):**
1. Visit [Modrinth API](https://modrinth.com/developers)
2. Create an account and generate an API key
3. Enter the API key for enhanced mod search capabilities

### Step 2: Java Configuration

**Automatic Detection:**
- Guardian will attempt to automatically detect Java installations
- Select the appropriate Java version (17+ recommended)

**Manual Configuration:**
- If auto-detection fails, browse to your Java installation directory
- Ensure the selected Java version is compatible with your Minecraft version

### Step 3: Directory Setup

**Server Directory:**
- Choose where to store your Minecraft servers
- Default: `~/GuardianServers/`
- Ensure the directory has sufficient space (10GB+ recommended)

**Backup Directory:**
- Choose where to store server backups
- Default: `~/GuardianBackups/`
- Consider using a different drive for redundancy

### Step 4: GPU Settings

**Enable GPU Acceleration:**
- Check this option if you have a compatible GPU
- GPU acceleration improves chunk generation and world processing
- Requires WebGPU support in your browser

**GPU Worker Configuration:**
- Set the maximum number of GPU workers (1-8)
- Higher values use more GPU memory but improve performance

### Step 5: Theme and Preferences

**Theme Selection:**
- **Dark**: Default dark theme (recommended)
- **Light**: Light theme for bright environments
- **System**: Follows your operating system theme

**Notification Settings:**
- Configure which events trigger notifications
- Choose notification types (toast, system, sound)

## Server Management

### Creating a New Server

1. **Navigate to Servers**
   - Click "Servers" in the sidebar
   - Click "Create New Server" button

2. **Basic Configuration**
   - **Server Name**: Choose a descriptive name
   - **Description**: Optional description for your server
   - **Minecraft Version**: Select from available versions
   - **Modpack**: Choose a modpack (optional)

3. **Server Settings**
   - **Max Players**: Maximum number of concurrent players
   - **MOTD**: Message of the day displayed in server list
   - **Difficulty**: Peaceful, Easy, Normal, or Hard
   - **Gamemode**: Survival, Creative, Adventure, or Spectator
   - **PvP**: Enable/disable player vs player combat
   - **Online Mode**: Enable/disable Mojang authentication

4. **Advanced Settings**
   - **JVM Memory**: Allocate RAM for the server (2GB+ recommended)
   - **JVM Flags**: Custom Java Virtual Machine arguments
   - **View Distance**: How far players can see (4-32 chunks)
   - **Simulation Distance**: How far the server simulates (4-32 chunks)

5. **Create Server**
   - Review your configuration
   - Click "Create Server" to proceed

### Starting and Stopping Servers

**Starting a Server:**
1. Navigate to your server in the server list
2. Click the "Start" button
3. Monitor the console for startup progress
4. The server status will change to "Running" when ready

**Stopping a Server:**
1. Click the "Stop" button on your server
2. Choose stop method:
   - **Graceful**: Saves world and kicks players (recommended)
   - **Force**: Immediate shutdown (use only if necessary)

**Restarting a Server:**
1. Click the "Restart" button
2. The server will stop gracefully and restart automatically

### Server Console

**Viewing Console Output:**
1. Click on your server to open the detail view
2. Navigate to the "Console" tab
3. View real-time server output and logs

**Sending Commands:**
1. Type commands in the console input field
2. Press Enter to execute
3. Common commands:
   - `/say <message>`: Broadcast a message
   - `/kick <player>`: Kick a player
   - `/ban <player>`: Ban a player
   - `/op <player>`: Give operator privileges
   - `/whitelist add <player>`: Add player to whitelist

### Player Management

**Viewing Players:**
1. Navigate to the "Players" tab in your server
2. See all online and recently connected players
3. View player statistics and playtime

**Player Actions:**
- **Kick**: Remove player temporarily
- **Ban**: Permanently ban player
- **Whitelist**: Add/remove from whitelist
- **Op**: Grant/revoke operator privileges

## Mod Management

### Installing Mods

**Search for Mods:**
1. Navigate to the "Mods" tab in your server
2. Use the search bar to find mods
3. Filter by Minecraft version, mod loader, and category
4. Browse results from CurseForge and Modrinth

**Install a Mod:**
1. Click on a mod to view details
2. Check compatibility and dependencies
3. Click "Install" to add the mod
4. The mod will be downloaded and installed automatically

**Bulk Mod Installation:**
1. Select multiple mods from search results
2. Click "Install Selected" to install all at once
3. Review compatibility warnings before proceeding

### Managing Installed Mods

**View Installed Mods:**
1. Navigate to the "Mods" tab
2. Switch to "Installed" view
3. See all currently installed mods

**Update Mods:**
1. Look for mods with update indicators
2. Click "Update" next to outdated mods
3. Review changelog before updating

**Remove Mods:**
1. Click the "Remove" button next to a mod
2. Confirm removal
3. The mod will be uninstalled and removed

### Modpack Management

**Installing Modpacks:**
1. Navigate to "Modpacks" in the main menu
2. Browse available modpacks
3. Click "Install" on a modpack
4. Choose server to install to
5. Wait for installation to complete

**Managing Modpacks:**
- **Update**: Update to latest version
- **Remove**: Uninstall modpack
- **Export**: Create a backup of modpack configuration

## Performance Monitoring

### Real-time Metrics

**Viewing Performance:**
1. Navigate to the "Analytics" tab in your server
2. View real-time performance graphs:
   - **TPS (Ticks Per Second)**: Server performance indicator
   - **Memory Usage**: RAM consumption
   - **CPU Usage**: Processor utilization
   - **Disk I/O**: Storage read/write activity
   - **Network**: Data transfer rates

**Performance Indicators:**
- **Green**: Optimal performance
- **Yellow**: Moderate performance issues
- **Red**: Critical performance problems

### Performance Optimization

**Memory Optimization:**
1. Navigate to server settings
2. Adjust JVM memory allocation
3. Monitor memory usage after changes
4. Consider adding more RAM if needed

**JVM Tuning:**
1. Access advanced server settings
2. Modify JVM flags for better performance
3. Common optimizations:
   - `-XX:+UseG1GC`: Use G1 garbage collector
   - `-XX:+UnlockExperimentalVMOptions`: Enable experimental features
   - `-XX:MaxGCPauseMillis=200`: Limit GC pause time

**GPU Acceleration:**
1. Enable GPU workers in settings
2. Monitor GPU usage in analytics
3. Adjust worker count based on performance

## Backup Management

### Creating Backups

**Manual Backup:**
1. Navigate to the "Backups" tab in your server
2. Click "Create Backup"
3. Enter backup name and description
4. Choose what to include:
   - **World Data**: Player builds and terrain
   - **Mods**: Installed mods and configurations
   - **Server Config**: Server.properties and other configs
5. Click "Create Backup"

**Automatic Backups:**
1. Go to server settings
2. Enable "Automatic Backups"
3. Configure backup schedule:
   - **Frequency**: Daily, weekly, or custom
   - **Retention**: How many backups to keep
   - **Compression**: Enable/disable compression

### Managing Backups

**Viewing Backups:**
1. Navigate to the "Backups" tab
2. See all available backups with details:
   - Creation date and time
   - Backup size
   - What's included

**Restoring Backups:**
1. Click on a backup to view details
2. Click "Restore" to restore the backup
3. Choose what to restore:
   - **World Data**: Restore player builds
   - **Mods**: Restore mod configuration
   - **Server Config**: Restore server settings
4. Confirm restoration

**Backup Storage:**
- Backups are stored in your configured backup directory
- Consider using cloud storage for important backups
- Regular cleanup of old backups saves disk space

## Settings Configuration

### Application Settings

**General Settings:**
1. Click the settings icon in the sidebar
2. Configure general preferences:
   - **Theme**: Dark, Light, or System
   - **Language**: Interface language
   - **Auto-start**: Start servers on application launch
   - **Minimize to tray**: Keep running in system tray

**Notification Settings:**
1. Navigate to "Notifications" in settings
2. Configure notification preferences:
   - **Server Events**: Start, stop, crash notifications
   - **Player Events**: Join, leave, ban notifications
   - **System Events**: Updates, errors, warnings
   - **Notification Types**: Toast, system, sound

### Workspace Settings

**Directory Configuration:**
1. Go to "Workspace" in settings
2. Configure directories:
   - **Servers Directory**: Where servers are stored
   - **Backups Directory**: Where backups are stored
   - **Logs Directory**: Where logs are stored
   - **Temp Directory**: Temporary files location

**API Configuration:**
1. Navigate to "API Keys" in settings
2. Update API keys:
   - **CurseForge API Key**: For mod downloads
   - **Modrinth API Key**: For enhanced mod search
   - **Mojang API Key**: For player authentication

### Server-Specific Settings

**Individual Server Settings:**
1. Open a server's detail view
2. Click "Settings" tab
3. Configure server-specific options:
   - **Startup Parameters**: Custom JVM arguments
   - **Environment Variables**: Custom environment setup
   - **Port Configuration**: Server and RCON ports
   - **Resource Limits**: CPU and memory limits

## Troubleshooting

### Common Issues

**Server Won't Start:**
1. Check Java installation and version
2. Verify server directory permissions
3. Review console output for error messages
4. Ensure sufficient disk space and RAM
5. Check port availability

**Mod Installation Fails:**
1. Verify mod compatibility with Minecraft version
2. Check for conflicting mods
3. Ensure mod loader is correct (Forge/Fabric)
4. Review dependency requirements
5. Check available disk space

**Performance Issues:**
1. Monitor resource usage in Analytics tab
2. Adjust JVM memory allocation
3. Optimize JVM flags
4. Consider reducing view distance
5. Check for problematic mods

**Backup/Restore Issues:**
1. Verify backup directory permissions
2. Check available disk space
3. Ensure backup files are not corrupted
4. Review backup contents before restore

### Getting Help

**Log Files:**
1. Navigate to "Logs" in settings
2. View application and server logs
3. Look for error messages and warnings
4. Include relevant logs when reporting issues

**Support Resources:**
- **Documentation**: Check this user guide and API reference
- **GitHub Issues**: Report bugs and request features
- **Community Discord**: Get help from other users
- **FAQ**: Check frequently asked questions

## Advanced Features

### GPU Acceleration

**Enabling GPU Workers:**
1. Ensure you have a compatible GPU
2. Enable GPU acceleration in settings
3. Configure worker count based on GPU memory
4. Monitor GPU usage in performance analytics

**GPU Job Types:**
- **Chunk Generation**: Accelerated world generation
- **World Processing**: Terrain modification and optimization
- **Mod Processing**: GPU-accelerated mod operations

### Compatibility Analysis

**Mod Compatibility:**
1. Navigate to "Compatibility" tab in server
2. View compatibility analysis results
3. See conflicts and recommendations
4. Apply automatic fixes when available

**Risk Assessment:**
1. Review mod risk scores
2. Check stability predictions
3. Follow recommendations for safer configurations

### API Integration

**Using the API:**
1. Enable API access in settings
2. Generate API tokens for external tools
3. Use API endpoints for automation
4. Integrate with external monitoring tools

**Webhook Configuration:**
1. Set up webhooks for server events
2. Configure external service integrations
3. Monitor server status remotely

### Command Line Interface

**CLI Usage:**
```bash
# Start a server
guardian server start <server-id>

# Stop a server
guardian server stop <server-id>

# Install a mod
guardian mod install <server-id> <mod-id>

# Create backup
guardian backup create <server-id> --name "Backup Name"
```

**Automation Scripts:**
- Create batch/shell scripts for common tasks
- Schedule automatic backups and updates
- Monitor server health with external tools

---

This user guide provides comprehensive instructions for using Guardian Server Manager. For additional help or advanced configuration options, refer to the API reference or contact support.
