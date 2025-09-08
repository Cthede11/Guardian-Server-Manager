# Guardian Minecraft Server Manager - Build & Install Guide

## 🚀 Overview

This guide will walk you through building and installing the Guardian Minecraft Server Manager application from source. Guardian is a comprehensive Minecraft server management platform with GPU acceleration, modpack management, and real-time monitoring.

## 📋 Prerequisites

Before building Guardian, ensure you have the following installed:

### Required Software
- **Rust** (latest stable version)
  ```bash
  # Install from https://rustup.rs/ or run:
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Node.js** (v18 or later)
  ```bash
  # Download from https://nodejs.org/ or use a package manager
  ```
- **npm** (comes with Node.js)
- **Git** (for cloning the repository)

### Optional but Recommended
- **Tauri CLI** (for building desktop applications)
  ```bash
  npm install -g @tauri-apps/cli
  ```
- **Visual Studio Build Tools** (Windows) or **Xcode Command Line Tools** (macOS)

## 🏗️ Building the Application

### Step 1: Clone the Repository

```bash
git clone <repository-url>
cd modded-manager
```

### Step 2: Build Backend Components

#### Build hostd (Main Backend Service)
```bash
cd hostd
cargo build --release
cd ..
```

#### Build gpu-worker (GPU Acceleration Service)
```bash
cd gpu-worker
cargo build --release
cd ..
```

### Step 3: Build Frontend

```bash
cd guardian-ui
npm install
npm run build
```

### Step 4: Build Tauri Desktop Application

```bash
# Ensure you're in the guardian-ui directory
npm run tauri:build
```

This will create:
- `src-tauri/target/release/guardian.exe` (Windows)
- `src-tauri/target/release/bundle/msi/Guardian_1.0.0_x64_en-US.msi` (Windows Installer)

## 📦 Installation Methods

### Method 1: MSI Installer (Recommended for Windows)

1. Navigate to the installer:
   ```bash
   cd guardian-ui/src-tauri/target/release/bundle/msi/
   ```

2. Run the MSI installer:
   ```bash
   # Double-click or run:
   Guardian_1.0.0_x64_en-US.msi
   ```

3. Follow the installation wizard to install Guardian system-wide.

### Method 2: Portable Installation

1. Copy the release directory:
   ```bash
   # Copy the entire release directory to your desired location
   cp -r guardian-ui/src-tauri/target/release/ /path/to/guardian/
   ```

2. Run the application:
   ```bash
   cd /path/to/guardian/
   ./guardian.exe
   ```

### Method 3: Development Mode

For development or testing:

1. Start the backend:
   ```bash
   cd hostd/target/release
   ./hostd.exe --config ../../../configs/hostd.yaml --port 8080
   ```

2. Start the frontend (in a new terminal):
   ```bash
   cd guardian-ui
   npm run dev
   ```

3. Open your browser to `http://localhost:8080`

## 🔧 Configuration

### Backend Configuration

The backend uses a YAML configuration file located at `configs/hostd.yaml`. Key settings include:

```yaml
minecraft:
  loader: "fabric"
  version: "1.21.1"
  java:
    heap_gb: 4
    flags: "-Xmx4G -Xms2G"

paths:
  mods_dir: "data/mods"
  config_dir: "configs"
  world_dir: "data/servers"
  backup_dir: "data/backups"

gpu:
  enabled: true
  worker_ipc: "tcp://127.0.0.1:9091"
  batch_size_chunks: 16
  max_cache_size: 1000
  health_check_interval_seconds: 30
  fallback_to_cpu: true
```

### Environment Variables

Optional environment variables:

```bash
# CurseForge API Key (for mod downloads)
export CURSEFORGE_API_KEY="your-api-key-here"

# Database URL (defaults to SQLite)
export DATABASE_URL="sqlite:data/guardian.db"

# HTTP Port (defaults to 8080)
export PORT=8080
```

## 🚀 Running the Application

### Windows

#### Using the Launcher Script
```powershell
# Run the PowerShell launcher
.\launch-guardian.ps1

# Or run the batch file
launch-guardian.bat
```

#### Direct Execution
```powershell
cd guardian-ui\src-tauri\target\release
.\guardian.exe
```

### Linux/macOS

```bash
cd guardian-ui/src-tauri/target/release
./guardian
```

## 🛠️ Troubleshooting

### Common Issues

#### Build Failures

1. **Rust compilation errors:**
   ```bash
   # Update Rust
   rustup update
   
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

2. **Node.js/npm issues:**
   ```bash
   # Clear npm cache
   npm cache clean --force
   
   # Delete node_modules and reinstall
   rm -rf node_modules package-lock.json
   npm install
   ```

3. **Tauri build errors:**
   ```bash
   # Install Tauri CLI globally
   npm install -g @tauri-apps/cli
   
   # Check Tauri requirements
   npx tauri info
   ```

#### Runtime Issues

1. **Backend won't start:**
   - Check if port 8080 is available
   - Verify configuration file exists
   - Check database permissions

2. **Frontend won't load:**
   - Ensure backend is running
   - Check browser console for errors
   - Verify CORS settings

3. **GPU worker issues:**
   - Check GPU drivers are up to date
   - Verify OpenGL/Vulkan support
   - Check system requirements

### Logs and Debugging

#### Backend Logs
```bash
# Run with debug logging
./hostd.exe --config configs/hostd.yaml --log-level debug
```

#### Frontend Logs
- Open browser developer tools (F12)
- Check Console tab for errors
- Check Network tab for API calls

#### System Logs
- Windows: Event Viewer
- Linux: `journalctl -u guardian`
- macOS: Console.app

## 📁 Directory Structure

After building, your directory structure should look like:

```
modded-manager/
├── hostd/
│   └── target/release/
│       └── hostd.exe
├── gpu-worker/
│   └── target/release/
│       └── gpu-worker.exe
├── guardian-ui/
│   ├── src-tauri/target/release/
│   │   ├── guardian.exe
│   │   ├── hostd.exe
│   │   ├── gpu-worker.exe
│   │   ├── configs/
│   │   └── data/
│   └── src-tauri/target/release/bundle/msi/
│       └── Guardian_1.0.0_x64_en-US.msi
└── configs/
    └── hostd.yaml
```

## 🔄 Updating the Application

### From Source

1. Pull latest changes:
   ```bash
   git pull origin main
   ```

2. Rebuild the application:
   ```bash
   # Clean previous builds
   cargo clean
   rm -rf guardian-ui/node_modules
   
   # Rebuild
   ./build-simple.ps1  # or follow manual build steps
   ```

### From Installer

1. Download the latest MSI installer
2. Run the installer (it will update the existing installation)

## 🧪 Testing the Installation

### Quick Test

1. Start Guardian
2. Open browser to `http://localhost:8080`
3. Check that the dashboard loads
4. Try creating a test server
5. Verify GPU worker is running (check system processes)

### API Test

```bash
# Test backend health
curl http://localhost:8080/health

# Test server list
curl http://localhost:8080/api/servers
```

## 📞 Support

If you encounter issues:

1. Check this guide first
2. Review the troubleshooting section
3. Check GitHub issues
4. Create a new issue with:
   - Operating system
   - Error messages
   - Steps to reproduce
   - Log files

## 🎯 Next Steps

After successful installation:

1. **Configure your first server** in the Guardian dashboard
2. **Set up modpacks** using the modpack manager
3. **Configure GPU acceleration** for better performance
4. **Set up automated backups** for your worlds
5. **Explore the monitoring features** for server health

---

**Happy Minecraft Server Managing! 🎮**
