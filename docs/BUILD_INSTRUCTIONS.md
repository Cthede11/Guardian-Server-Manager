# Guardian Build Instructions

## Enhanced Build Script

The `scripts/build/build-all.ps1` script has been enhanced to build the full Guardian Minecraft Server Manager application with all the critical fixes applied.

## Features

### ✅ **Comprehensive Build Process**
- Builds hostd backend with process fixes
- Builds GPU worker with process fixes  
- Builds Guardian UI frontend with race condition fixes
- Builds Tauri desktop application with cleanup handlers
- Copies all configuration files
- Organizes build artifacts properly

### ✅ **Error Handling & Logging**
- Comprehensive error handling with detailed logging
- Timestamped log messages with color coding
- Prerequisites checking before build
- Graceful failure handling with proper exit codes

### ✅ **Build Artifacts Organization**
- Executables: `build/executables/`
- Installers: `build/installers/`
- Configuration files: `build/executables/configs/`
- Launcher scripts: `build/`
- Version information: `build/version.json`
- Build summary: `build/BUILD_SUMMARY.txt`

## Usage

### Basic Build
```powershell
.\scripts\build-all.ps1
```

### Build with Options
```powershell
# Verbose output
.\scripts\build-all.ps1 -Verbose

# Skip cleanup step
.\scripts\build-all.ps1 -SkipCleanup

# Skip tests (if any)
.\scripts\build-all.ps1 -SkipTests
```

### Build with Execution Policy Bypass
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-all.ps1 -Verbose
```

## Prerequisites

The script checks for these prerequisites:
- `cargo` - Rust build tool
- `npm` - Node.js package manager  
- `node` - Node.js runtime

## Build Process

1. **Prerequisites Check** - Verifies all required tools are available
2. **Cleanup** - Removes temporary files and organizes project structure
3. **Backend Build** - Builds hostd with CREATE_NO_WINDOW fixes
4. **GPU Worker Build** - Builds gpu-worker with CREATE_NO_WINDOW fixes
5. **Frontend Build** - Builds Guardian UI with race condition fixes
6. **Configuration Copy** - Copies all config files to build directory
7. **Tauri Build** - Builds desktop application with cleanup handlers
8. **Artifact Organization** - Organizes all build outputs
9. **Launcher Copy** - Copies production launcher scripts
10. **Version Info** - Creates version and build summary files

## Output Structure

```
build/
├── executables/
│   ├── hostd.exe
│   ├── gpu-worker.exe
│   └── configs/
│       ├── hostd.yaml
│       ├── server.yaml
│       ├── rules.yaml
│       └── test.yaml
├── installers/
│   ├── nsis/
│   │   └── Guardian_1.0.0_x64-setup.exe
│   └── msi/
│       └── Guardian_1.0.0_x64_en-US.msi
├── start-guardian-with-backend.ps1
├── start-guardian-production.bat
├── version.json
└── BUILD_SUMMARY.txt
```

## Fixes Applied

The build script ensures all critical fixes are included:

### ✅ **Console Window Elimination**
- All process spawning uses `CREATE_NO_WINDOW` flag (0x08000000)
- `Stdio::null()` prevents console output
- Windows-specific process creation flags applied

### ✅ **Backend Connection Race Conditions**
- Race condition prevention with `AtomicBool`
- Single backend instance management
- Proper initialization deduplication

### ✅ **Process Cleanup**
- Process cleanup handlers on window close
- Child process tracking in app state
- Graceful shutdown with force-kill fallback

### ✅ **Professional Startup Experience**
- No console windows flash during startup
- Clean process lifecycle management
- Production-ready launcher scripts

## Testing the Build

### 1. Verify Prerequisites
```powershell
# Check if all tools are available
cargo --version
npm --version
node --version
```

### 2. Run Build Script
```powershell
# Run the enhanced build script
.\scripts\build-all.ps1 -Verbose
```

### 3. Test Built Application
```powershell
# Test the built application
.\build\start-guardian-production.bat
```

### 4. Verify Fixes
- No console windows should appear
- Backend should connect reliably
- All processes should terminate when app closes
- Professional startup experience

## Troubleshooting

### Execution Policy Issues
```powershell
# Set execution policy for current session
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process

# Or run with bypass flag
powershell -ExecutionPolicy Bypass -File .\scripts\build-all.ps1
```

### Build Failures
- Check prerequisites are installed
- Ensure all source files are present
- Check for any compilation errors
- Verify disk space is available

### Missing Dependencies
```powershell
# Install Node.js dependencies
cd guardian-ui
npm install

# Install Rust dependencies (if needed)
cd hostd
cargo build
```

## Success Criteria

After running the build script, you should have:

✅ **Complete Build Artifacts**
- All executables built successfully
- Installers generated properly
- Configuration files copied
- Launcher scripts available

✅ **All Fixes Applied**
- Console windows eliminated
- Race conditions prevented
- Process cleanup implemented
- Professional experience delivered

✅ **Production Ready**
- Clean build process
- Proper error handling
- Comprehensive logging
- Organized output structure

The Guardian Minecraft Server Manager is now ready for production deployment with all critical issues resolved!
