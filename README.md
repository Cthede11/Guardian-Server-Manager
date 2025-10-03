# Guardian - Minecraft Server Management Platform

A comprehensive server management platform designed for modded Minecraft servers. Guardian provides server administration tools, mod management capabilities, and real-time monitoring through a modern desktop application.

## Current Features

### Desktop Application
- **Cross-Platform Desktop App**: Built with Tauri for Windows, macOS, and Linux
- **Modern User Interface**: React-based UI with TypeScript and Vite
- **Theme System**: Dark/light themes with multiple color schemes
- **Server Management**: Real-time server monitoring and control
- **Settings Management**: Comprehensive configuration options
- **Error Handling**: Robust error boundaries and user feedback

### Mod Management
- **API Integration**: Direct integration with CurseForge and Modrinth APIs
- **Visual Mod Browser**: Tile-based interface for browsing mods
- **Comprehensive Version Support**: Support for Minecraft versions 1.12.2 through 1.21.1
- **Smart Search**: Advanced filtering by version, loader, and category
- **Direct Downloads**: Links to official mod platforms for downloads
- **API Key Management**: Secure storage and validation of API keys

### Server Administration
- **Server Creation**: Wizard-based server setup process
- **Server Monitoring**: Real-time status and performance tracking
- **Server Deletion**: Complete cleanup including file system removal
- **Folder Management**: Direct access to server directories
- **Configuration Management**: YAML-based server configuration

### Backend Services
- **Rust Backend**: High-performance server management daemon
- **Database Integration**: SQLite database for persistent storage
- **API Endpoints**: RESTful API for frontend communication
- **Process Management**: Child process supervision and management
- **File System Operations**: Cross-platform file management

## In Development

### Advanced Server Features
- **GPU Acceleration**: Offload world generation to GPU using wgpu
- **Crash Prevention**: Non-destructive freeze/quarantine system for problematic entities
- **Mod Compatibility Engine**: Runtime patching and conflict resolution
- **Multi-Tenancy**: Support for multiple isolated server instances
- **Plugin System**: Hot-reloadable plugins with sandboxed execution

### AI and Analytics
- **Crash Prediction**: ML-based crash probability prediction
- **Performance Optimization**: Automated performance tuning
- **Anomaly Detection**: Real-time anomaly detection and alerting
- **Predictive Maintenance**: Proactive server health management

### Enterprise Features
- **High Availability**: Watchdog supervision and automatic restart
- **Blue/Green Deployments**: Zero-downtime updates
- **Compliance**: GDPR, SOC2, and audit logging
- **Webhook Integration**: Real-time event notifications
- **Community Features**: Mod compatibility database and sharing

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Desktop   │───▶│   Backend    │───▶│   Database   │
│   App       │    │   (Rust)     │    │   (SQLite)  │
└─────────────┘    └──────┬───────┘    └─────────────┘
                          │
                    ┌─────▼─────┐
                    │  Minecraft │
                    │  Server    │
                    │  Process   │
                    └────────────┘
```

## Quick Start

### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+
- Git

### Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/Cthede11/Guardian-Server-Manager.git
   cd Guardian-Server-Manager
   ```

2. **Install Dependencies**
   ```bash
   # Install frontend dependencies
   cd guardian-ui
   npm install
   
   # Install backend dependencies
   cd ../hostd
   cargo build --release
   ```

3. **API Keys (Optional)**
   - The app works out of the box with default API keys
   - For higher rate limits, you can add your own keys:
     - CurseForge API key from [CurseForge Developer Portal](https://docs.curseforge.com/#authentication)
     - Modrinth token from [Modrinth Settings](https://modrinth.com/settings/tokens)
   - Configure in the desktop app: Settings → API Keys

4. **Build and Run**
   ```bash
   # Build the desktop application
   cd guardian-ui
   npm run tauri build
   
   # Run the backend server
   cd ../hostd
   cargo run --release
   ```

5. **Launch the Application**
   - Windows: Run `Guardian.exe` from the build output
   - macOS: Run `Guardian.app` from the build output
   - Linux: Run the generated binary

## API Requirements

- **CurseForge API Key**: Required for mod browsing and search functionality
- **Modrinth API Token**: Optional but recommended for higher rate limits
- **Rate Limits**: 
  - CurseForge: 100 requests per minute
  - Modrinth: 300 requests per minute

## Project Structure

```
Guardian-Server-Manager/
├── guardian-ui/          # Desktop application (React + Tauri)
├── hostd/                # Backend server (Rust)
├── gpu-worker/           # GPU acceleration worker (Rust + wgpu)
├── guardian-agent/       # Minecraft server agent (Java)
├── configs/              # Configuration templates
├── docs/                 # Documentation
├── scripts/              # Build and deployment scripts
└── monitoring/           # Prometheus and Grafana configs
```

## Development

### Frontend Development
```bash
cd guardian-ui
npm run dev
```

### Backend Development
```bash
cd hostd
cargo run
```

### Building
```bash
# Build desktop application
cd guardian-ui
npm run tauri build

# Build backend
cd ../hostd
cargo build --release
```

## Configuration

### Server Configuration
Edit `configs/server.yaml` to configure server settings:
```yaml
name: "My Server"
version: "1.21.1"
loader: "vanilla"
max_players: 20
memory: 4096
```

### API Configuration
Configure API keys in the desktop application:
1. Open Settings → API Keys
2. Enter your CurseForge API key
3. Enter your Modrinth token (optional)
4. Test and save configuration

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: See the [docs/](docs/) directory
- **Issues**: Report bugs via GitHub Issues
- **Discussions**: Use GitHub Discussions for questions
- **API Setup**: See [API_SETUP.md](docs/API_SETUP.md) for detailed API configuration

## Roadmap

### Phase 1 (Current)
- Desktop application with basic server management
- Mod browsing and search functionality
- API integration with CurseForge and Modrinth
- Server creation and deletion

### Phase 2 (In Development)
- GPU acceleration for world generation
- Advanced mod compatibility features
- Plugin system for extensibility
- Multi-tenant architecture

### Phase 3 (Planned)
- AI-powered analytics and predictions
- Enterprise features and compliance
- Community features and sharing
- Advanced monitoring and alerting

## Status

**Current Version**: 1.0.0-alpha
**Development Status**: Active development
**Last Updated**: January 2025