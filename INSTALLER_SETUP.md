# Guardian Installer Setup with Backend Integration

This document explains how to set up Guardian installers that automatically launch with backend services.

## What the Installers Include

### NSIS Installer (`Guardian_1.0.0_x64-setup.exe`)
- **Main Application**: `guardian.exe`
- **Backend Services**: `hostd.exe`, `gpu-worker.exe`
- **Configuration Files**: `configs/` directory
- **Launcher Scripts**: All launcher scripts for different launch methods
- **Post-Installation Setup**: `post-install-setup.ps1` script

### MSI Installer (`Guardian_1.0.0_x64_en-US.msi`)
- Same contents as NSIS installer
- Windows Installer format for enterprise environments

## Post-Installation Setup

After installing Guardian, users need to run the post-installation setup script to configure shortcuts to launch with backend services.

### Automatic Setup (Recommended)
1. Install Guardian using either installer
2. Navigate to the installation directory (usually `C:\Program Files\Guardian - Minecraft Server Manager`)
3. Right-click on `post-install-setup.ps1` and select "Run with PowerShell"
4. Follow the prompts to configure shortcuts

### Manual Setup
If the automatic setup doesn't work, users can manually configure shortcuts:

1. **Desktop Shortcut**: Right-click the desktop shortcut → Properties → Change target to:
   ```
   C:\Program Files\Guardian - Minecraft Server Manager\start-guardian-with-backend.bat
   ```

2. **Start Menu Shortcuts**: Similar process for start menu shortcuts

## What Happens After Setup

Once configured, the shortcuts will:
- ✅ **Start Backend Services**: Automatically launch `hostd.exe` and `gpu-worker.exe`
- ✅ **Initialize Database**: Create database if it doesn't exist
- ✅ **Test API Connection**: Verify backend is responding
- ✅ **Launch Guardian**: Start the main application
- ✅ **Clean Shutdown**: Stop backend services when Guardian closes

## Troubleshooting

### Backend Services Not Starting
- Check if `hostd.exe` and `gpu-worker.exe` are in the installation directory
- Verify the database file exists in the installation directory
- Run the launcher scripts manually to see error messages

### Shortcuts Not Working
- Ensure the target path points to the correct installation directory
- Check that the launcher scripts have execute permissions
- Try running the post-installation setup script again

### Server Creation Fails
- Verify that `hostd.exe` is running (check Task Manager)
- Test the API connection at `http://localhost:8080/api/health`
- Check the installation directory for error logs

## Building the Installers

To build the installers with backend integration:

```powershell
# Run from project root
.\build-installers-with-backend.ps1
```

This script will:
1. Build the frontend and Tauri application
2. Build the backend services (hostd, gpu-worker)
3. Copy all necessary files to the Tauri resources
4. Build both NSIS and MSI installers
5. Verify the installers were created successfully

## File Structure After Installation

```
C:\Program Files\Guardian - Minecraft Server Manager\
├── guardian.exe                    # Main application
├── hostd.exe                      # Backend service
├── gpu-worker.exe                 # GPU acceleration service
├── configs\                       # Configuration files
│   ├── hostd.yaml
│   ├── server.yaml
│   └── ...
├── start-guardian-with-backend.bat    # Batch launcher
├── start-guardian-with-backend.ps1    # PowerShell launcher
├── create-desktop-shortcut.ps1        # Shortcut creation tool
├── update-desktop-shortcut.ps1        # Shortcut update tool
├── post-install-setup.ps1             # Post-installation setup
└── data\                          # Database directory (created on first run)
    └── guardian.db
```

## Benefits

- **One-Click Launch**: Users can start Guardian with backend services with a single click
- **No Manual Setup**: Backend services start automatically
- **Server Creation Works**: No more "Failed to fetch" errors
- **Professional Installation**: Proper Windows installer with uninstaller
- **Multiple Launch Options**: Batch file, PowerShell, and direct executable options
