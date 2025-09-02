# Guardian - Modded Minecraft Server Hosting Platform

A high-performance, self-healing hosting platform designed specifically for modded Minecraft servers. Guardian provides non-destructive crash prevention, GPU-accelerated world generation, and intelligent mod compatibility management.

## Key Features

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

### 🚀 High Availability
- **Watchdog Supervision**: Automatic crash detection and instant restart
- **Blue/Green Deployments**: Zero-downtime updates with automatic rollback
- **Snapshot System**: Journaling saves with deduplicated backups

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
                    │ Watchdog  │
                    │ (Rust)    │
                    └───────────┘
```

## Quick Start

1. **Clone and Setup**
   ```bash
   git clone <repository>
   cd guardian
   ./scripts/setup.sh
   ```

2. **Configure Your Server**
   ```bash
   cp configs/server.yaml.example configs/server.yaml
   # Edit server.yaml with your modpack details
   ```

3. **Start the Platform**
   ```bash
   docker-compose up -d
   ```

4. **Access Dashboard**
   Open http://localhost:8080 for the web interface

## Components

- **guardian-agent/**: Java/Kotlin agent with NeoForge/Forge integration
- **gpu-worker/**: Rust sidecar using wgpu for GPU acceleration
- **hostd/**: Rust watchdog daemon for process supervision
- **proxy/**: Velocity proxy for multi-instance sharding
- **configs/**: Configuration templates and example rules

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development setup and contribution guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

