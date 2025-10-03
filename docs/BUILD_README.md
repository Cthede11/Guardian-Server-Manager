# Build Directory

This directory contains all build artifacts and installers for the Guardian project.

## Directory Structure

```
build/
├── executables/     # Compiled executables (.exe files)
├── installers/      # Installer packages and scripts
├── temp/           # Temporary build files
├── logs/           # Build and runtime logs
└── README.md       # This file
```

## Contents

### executables/
Contains all compiled executables:
- `hostd.exe` - Main backend daemon
- `gpu-worker.exe` - GPU processing worker
- `guardian-ui.exe` - Desktop application (when built)

### installers/
Contains installer packages and scripts:
- `nsis-installer.nsi` - NSIS installer script
- `wix-installer.wxs` - WiX installer script
- `guardian-custom-installer.nsi` - Custom installer script
- Various installer bundles (MSI, NSIS, etc.)

### temp/
Temporary files created during the build process.

### logs/
Build and runtime logs:
- `guardian_debug.log` - Debug logs
- `gpu-error*.txt` - GPU worker error logs
- `gpu-output*.txt` - GPU worker output logs

## Usage

- **Development**: Run `scripts/build/build-desktop.ps1` to build and organize artifacts
- **Cleanup**: Run `scripts/cleanup.ps1` to clean up temporary files and organize artifacts
- **Installation**: Use the installers in the `installers/` directory

## Notes

- This directory is ignored by Git (see `.gitignore`)
- Build artifacts are automatically organized here by build scripts
- Temporary files are cleaned up regularly
- All executables are copied here for easy distribution
