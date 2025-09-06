# Guardian - Enterprise Minecraft Server Hosting Platform

A comprehensive, production-ready hosting platform designed specifically for modded Minecraft servers. Guardian provides non-destructive crash prevention, GPU-accelerated world generation, intelligent mod compatibility management, multi-tenancy, AI-powered predictive analytics, and enterprise-grade security.

## 🌟 Key Features

### 🛡️ Non-Destructive Stability
- **Freeze/Quarantine System**: Instead of deleting problematic entities or blocks, Guardian freezes them and logs repair tickets
- **SafeTick Wrappers**: Prevents crashes by intercepting dangerous ticks without data loss
- **Automatic Thaw**: Reintroduces frozen objects once compatibility patches are applied

### ⚡ GPU Acceleration
- **Chunk Generation**: Offloads density/noise calculations and terrain generation to GPU
- **World Supervisor**: GPU-powered batch scans for anomaly detection and hotspot analysis
- **Deterministic Results**: Ensures identical output across different hardware configurations

### 🔧 Mod Compatibility Engine
- **Runtime Patching**: Applies compatibility fixes without redistributing modified mods
- **License-Aware**: Respects mod licenses, only bakes patches when permitted
- **Rules DSL**: YAML-based configuration for managing mod conflicts and fixes

### 📦 Advanced Modpack Management
- **Visual Mod Browser**: Tile-based interface for browsing mods from CurseForge and Modrinth
- **Real-Time API Integration**: Direct integration with CurseForge and Modrinth APIs
- **Comprehensive Version Support**: Support for Minecraft versions 1.12.2 through latest (1.21.1+)
- **Mod Compatibility Database**: Community-driven compatibility information and conflict resolution
- **Smart Mod Search**: Advanced filtering by version, loader, category, and compatibility
- **One-Click Installation**: Direct download links to official mod platforms
- **Modpack Creation Tools**: Build custom modpacks with dependency resolution

### 🚀 High Availability
- **Watchdog Supervision**: Automatic crash detection and instant restart
- **Blue/Green Deployments**: Zero-downtime updates with automatic rollback
- **Snapshot System**: Journaling saves with deduplicated backups

### 🏢 Multi-Tenancy
- **Isolated Instances**: Complete resource isolation between tenants
- **Scalable Architecture**: Support for hundreds of concurrent server instances
- **Resource Management**: CPU, memory, disk, and network quotas per tenant
- **Tenant Administration**: Self-service portal for tenant management

### 🔌 Plugin System
- **Hot-Reloading**: Install and update plugins without server restart
- **Sandboxed Execution**: Secure plugin isolation with resource limits
- **Event System**: Plugin communication through event bus
- **API Access**: Comprehensive API for plugin development

### 🔗 Webhook Integration
- **Event Notifications**: Real-time webhook delivery for server events
- **Retry Logic**: Automatic retry with exponential backoff
- **Signature Verification**: HMAC-SHA256 signature validation
- **Delivery Tracking**: Complete audit trail of webhook deliveries

### 📊 Compliance & Security
- **GDPR Compliance**: Data protection and privacy controls
- **SOC2 Ready**: Security and compliance framework
- **Audit Logging**: Comprehensive audit trails
- **Data Retention**: Automated data lifecycle management

### 🌐 Community Features
- **Mod Database**: Community-driven mod compatibility information
- **Sharing Platform**: Share configurations, mod packs, and rules
- **Compatibility Reports**: Crowdsourced mod compatibility testing
- **Rating System**: Community ratings and reviews

### 🤖 AI-Powered Analytics
- **Crash Prediction**: ML-based crash probability prediction
- **Performance Optimization**: Automated performance tuning
- **Anomaly Detection**: Real-time anomaly detection and alerting
- **Recommendation Engine**: Personalized optimization recommendations

### 🖥️ Modern Desktop Application
- **Cross-Platform Desktop App**: Built with Tauri for Windows, macOS, and Linux
- **Intuitive User Interface**: Modern, responsive design with dark/light themes
- **Real-Time Server Management**: Live server monitoring and control
- **API Key Management**: Secure storage and validation of CurseForge/Modrinth API keys
- **Theme Customization**: Multiple color schemes and theme options
- **Error Handling**: Comprehensive error boundaries and user feedback
- **Offline Capability**: Core functionality works without internet connection

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Players   │───▶│ MC Server    │───▶│ GPU Worker  │
│             │    │ (NeoForge)   │    │ (Rust/wgpu) │
└─────────────┘    └──────┬───────┘    └─────────────┘
                          │
                    ┌─────▼─────┐
                    │ Guardian  │
                    │ Agent     │
                    │ (Java)    │
                    └─────┬─────┘
                          │
                    ┌─────▼─────┐
                    │ Host      │
                    │ Daemon    │
                    │ (Rust)    │
                    └─────┬─────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   ┌────▼────┐      ┌─────▼─────┐      ┌────▼────┐
   │ Multi-  │      │   AI/ML   │      │Community│
   │Tenancy  │      │ Analytics │      │Features │
   └─────────┘      └───────────┘      └─────────┘
        │                 │                 │
   ┌────▼────┐      ┌─────▼─────┐      ┌────▼────┐
   │Plugins  │      │Compliance │      │Webhooks │
   │System   │      │& Security │      │& Events │
   └─────────┘      └───────────┘      └─────────┘
```

## Quick Start

1. **Clone and Setup**
   ```bash
   git clone https://github.com/Cthede11/Guardian-Server-Manager.git
   cd Guardian-Server-Manager
   chmod +x scripts/build.sh
   ./scripts/build.sh
   ```

2. **Configure API Keys**
   - **CurseForge API**: Get your API key from [CurseForge Developer Portal](https://docs.curseforge.com/#authentication)
   - **Modrinth API**: Get your token from [Modrinth Settings](https://modrinth.com/settings/tokens) (optional)
   - **Desktop App**: Open Settings → API Keys to configure your keys
   - **Detailed Guide**: See [API_SETUP.md](API_SETUP.md) for step-by-step instructions

3. **Configure Your Server**
   ```bash
   # Edit server.yaml with your modpack details
   nano configs/server.yaml
   
   # Edit rules.yaml for mod compatibility
   nano configs/rules.yaml
   ```

4. **Start the Platform**
   ```bash
   # Using Docker Compose (Recommended)
   docker-compose up -d
   
   # Or manually
   cd guardian-dist
   ./start.sh
   ```

5. **Access Services**
   - **Desktop Application**: Launch Guardian.exe (Windows) or Guardian.app (macOS)
   - **Web Dashboard**: http://localhost:8080
   - **API Documentation**: http://localhost:8080/docs
   - **Prometheus Metrics**: http://localhost:9090
   - **Grafana Dashboard**: http://localhost:3000 (admin/admin)
   - **Minecraft Server**: localhost:25565

### Default Credentials
- **Admin Username**: admin
- **Admin Password**: admin123
- **Grafana**: admin/admin

### API Requirements
- **CurseForge API Key**: Required for mod browsing and search functionality
- **Modrinth API Token**: Optional but recommended for higher rate limits (300 req/min)
- **Rate Limits**: 
  - CurseForge: 100 requests/minute
  - Modrinth: 300 requests/minute (same with or without token)
- **Compliance**: Full compliance with CurseForge 3rd Party API Terms and Conditions

## Components

- **guardian-ui/**: Cross-platform desktop application built with React, TypeScript, and Tauri
- **guardian-agent/**: Java/Kotlin agent with NeoForge/Forge integration
- **gpu-worker/**: Rust sidecar using wgpu for GPU acceleration
- **hostd/**: Rust host daemon with multi-tenancy, plugins, webhooks, compliance, community features, and AI analytics
- **configs/**: Configuration templates and example rules
- **docs/**: Comprehensive documentation including API specs and production deployment guides
- **tests/**: Unit tests, integration tests, and performance tests
- **monitoring/**: Prometheus and Grafana configurations

### Desktop Application Features

- **Modern UI Framework**: Built with React 18, TypeScript, and Vite
- **Cross-Platform**: Tauri-based desktop app for Windows, macOS, and Linux
- **Theme System**: Dark/light themes with multiple color schemes
- **API Integration**: Direct integration with CurseForge and Modrinth APIs
- **Mod Management**: Visual mod browser with tile-based interface
- **Server Management**: Real-time server monitoring and control
- **Settings Management**: Comprehensive configuration options
- **Error Handling**: Robust error boundaries and user feedback

## Production Features

### 🔐 Security & Authentication
- JWT-based authentication with role-based access control
- Rate limiting and API security
- Multi-factor authentication support
- Audit logging and compliance reporting

### 🏢 Enterprise Multi-Tenancy
- Complete tenant isolation with resource quotas
- Self-service tenant management portal
- Scalable architecture supporting hundreds of instances
- Resource monitoring and alerting

### 🔌 Extensible Plugin System
- Hot-reloadable plugins with sandboxed execution
- Event-driven plugin communication
- Comprehensive plugin API
- Plugin marketplace and sharing

### 📡 Webhook & Integration Platform
- Real-time event notifications
- Retry logic with exponential backoff
- HMAC signature verification
- Complete delivery audit trails

### 📊 Compliance & Data Protection
- GDPR, SOC2, and HIPAA compliance frameworks
- Automated data retention policies
- Consent management system
- Privacy controls and data anonymization

### 🌐 Community Platform
- Mod compatibility database
- Configuration and mod pack sharing
- Community ratings and reviews
- Crowdsourced compatibility testing

### 🤖 AI-Powered Intelligence
- Machine learning crash prediction
- Automated performance optimization
- Real-time anomaly detection
- Personalized recommendations

## Documentation

- **[API Documentation](docs/api/openapi.yaml)**: Complete OpenAPI 3.0 specification
- **[Production Deployment Guide](docs/PRODUCTION_DEPLOYMENT.md)**: Enterprise deployment instructions
- **[Development Guide](DEVELOPMENT.md)**: Development setup and contribution guidelines
- **[Architecture Overview](docs/ARCHITECTURE.md)**: Detailed system architecture

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development setup and contribution guidelines.

## 📁 Project Structure

- **`launchers/`** - All launcher scripts and desktop shortcut tools
- **`tools/`** - Build tools, installers, and cleanup utilities  
- **`guardian-ui/`** - Tauri desktop application
- **`hostd/`** - Backend service (Rust)
- **`gpu-worker/`** - GPU acceleration service (Rust)
- **`configs/`** - Configuration files

## 🖥️ Desktop App Usage

### Quick Launch
Double-click `Launch-Guardian.bat` in the project root to start Guardian with backend services.

### Advanced Launch
Run `launchers\start-guardian-with-backend.ps1` for detailed output and error handling.

### Desktop Shortcut
Run `launchers\create-desktop-shortcut.ps1` to create a desktop shortcut.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

