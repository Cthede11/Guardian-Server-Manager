# Guardian Tools

This directory contains utility scripts and tools for Guardian.

## Files

- **`build-custom-installer.ps1`** - Builds a custom NSIS installer
- **`guardian-custom-installer.nsi`** - NSIS installer script
- **`cleanup-guardian.ps1`** - Removes Guardian from system

## Usage

### Custom Installer
Run `build-custom-installer.ps1` to create a custom installer that:
- Installs Guardian with backend services
- Creates desktop shortcut to launcher
- Creates start menu shortcut to launcher
- Sets up proper uninstaller

### Cleanup
Run `cleanup-guardian.ps1` to remove Guardian from your system (registry entries, files, etc.)

## Requirements

- **NSIS** - Required for custom installer (download from https://nsis.sourceforge.io/)
- **PowerShell** - Required for all scripts
