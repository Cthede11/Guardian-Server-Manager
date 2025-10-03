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

### Step 2: Quick Build (Recommended)

Use the automated build script that handles everything:

```powershell
# Windows PowerShell
.\scripts\build-all.ps1
```

This will:
- Clean up any existing build artifacts
- Build all backend components (hostd, gpu-worker)
- Build the frontend (guardian-ui)
- Build the Tauri desktop application
- Organize all build artifacts in the `build/` directory

### Step 3: Manual Build (Alternative)

If you prefer to build components individually:

#### Build Backend Components
```bash
# Build hostd (Main Backend Service)
cd hostd
cargo build --release
cd ..

# Build gpu-worker (GPU Acceleration Service)
cd gpu-worker
cargo build --release
cd ..
```

#### Build Frontend
```bash
cd guardian-ui
npm install
npm run build
```

#### Build Tauri Desktop Application
```bash
# Ensure you're in the guardian-ui directory
npm run tauri:build
```

#### Organize Build Artifacts
```powershell
# Run cleanup to organize everything
.\scripts\cleanup.ps1
```

### Build Output

All build artifacts are organized in the `build/` directory:
- `build/executables/` - All compiled executables (.exe files)
- `build/installers/` - Installer packages and scripts
- `build/logs/` - Build and runtime logs
- `build/temp/` - Temporary build files

## 📦 Installation Methods

### Method 1: MSI Installer (Recommended for Windows)

1. Navigate to the installer:
   ```bash
   cd build/installers/msi/
   ```

2. Run the MSI installer:
   ```bash
   # Double-click or run:
   Guardian_1.0.0_x64_en-US.msi
   ```

3. Follow the installation wizard to install Guardian system-wide.

### Method 2: Portable Installation

1. Copy the executables:
   ```bash
   # Copy all executables to your desired location
   cp -r build/executables/ /path/to/guardian/
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
   cd build/executables
   ./hostd.exe --config ../../configs/hostd.yaml --port 8080
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

#### Using the Build Scripts
```powershell
# Run the complete build and organization
.\scripts\build-all.ps1

# Or just run the desktop build
.\scripts\build-desktop.ps1
```

#### Direct Execution
```powershell
cd build\executables
.\guardian.exe
```

### Linux/macOS

```bash
cd build/executables
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
├── build/                    # 🆕 Centralized build directory
│   ├── executables/         # All compiled executables
│   │   ├── hostd.exe
│   │   ├── gpu-worker.exe
│   │   └── guardian.exe
│   ├── installers/          # Installer packages and scripts
│   │   ├── msi/
│   │   │   └── Guardian_1.0.0_x64_en-US.msi
│   │   ├── nsis-installer.nsi
│   │   └── wix-installer.wxs
│   ├── logs/               # Build and runtime logs
│   └── temp/               # Temporary build files
├── scripts/                 # Build and maintenance scripts
│   ├── build-all.ps1       # Master build script
│   ├── build-desktop.ps1   # Desktop build script
│   └── cleanup.ps1         # Cleanup script
├── hostd/                  # Backend source
├── gpu-worker/            # GPU worker source
├── guardian-ui/           # Frontend source
└── configs/               # Configuration files
    └── hostd.yaml
```

## 🔄 Updating the Application

### From Source

1. Pull latest changes:
   ```bash
   git pull origin main
   ```

2. Rebuild the application:
   ```powershell
   # Clean and rebuild everything
   .\scripts\build-all.ps1
   
   # Or clean manually and rebuild
   cargo clean
   Remove-Item -Recurse -Force guardian-ui\node_modules
   .\scripts\build-desktop.ps1
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
