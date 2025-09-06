# Guardian Launchers

This directory contains all the launcher scripts for Guardian.

## Files

- **`start-guardian-with-backend.bat`** - Batch file launcher (Windows)
- **`start-guardian-with-backend.ps1`** - PowerShell launcher with full features
- **`create-desktop-shortcut.ps1`** - Creates desktop shortcut
- **`update-desktop-shortcut.ps1`** - Updates existing desktop shortcut

## Usage

### Quick Start
Double-click `Launch-Guardian.bat` in the project root to start Guardian with backend services.

### Advanced Usage
Run `start-guardian-with-backend.ps1` directly for more detailed output and error handling.

### Desktop Shortcut
- Run `create-desktop-shortcut.ps1` to create a desktop shortcut
- Run `update-desktop-shortcut.ps1` to update an existing shortcut

## What the Launcher Does

1. **Database Check** - Creates the database if it doesn't exist
2. **Backend Startup** - Starts `hostd.exe` on port 8080
3. **API Test** - Verifies the backend is responding
4. **Guardian Launch** - Starts the main application
5. **Clean Shutdown** - Stops backend when you close Guardian
