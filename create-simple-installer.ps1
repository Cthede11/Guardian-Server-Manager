# Simple Installer Script for Guardian
# This creates a portable installation without MSI

Write-Host "Creating Guardian Installer..." -ForegroundColor Green

# Create installer directory
$installerDir = "Guardian-Installer"
if (Test-Path $installerDir) {
    Remove-Item $installerDir -Recurse -Force
}
New-Item -ItemType Directory -Path $installerDir

# Copy the executable
Write-Host "Copying Guardian executable..." -ForegroundColor Yellow
Copy-Item "guardian-ui\src-tauri\target\release\guardian.exe" "$installerDir\Guardian.exe"

# Copy backend executables
Write-Host "Copying backend services..." -ForegroundColor Yellow
Copy-Item "hostd\target\release\hostd.exe" "$installerDir\hostd.exe"
Copy-Item "gpu-worker\target\release\gpu-worker.exe" "$installerDir\gpu-worker.exe"

# Copy configuration files
Write-Host "Copying configuration files..." -ForegroundColor Yellow
Copy-Item "configs" "$installerDir\configs" -Recurse

# Create data directory structure
Write-Host "Creating data directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path "$installerDir\data" -Force
New-Item -ItemType Directory -Path "$installerDir\data\servers" -Force
New-Item -ItemType Directory -Path "$installerDir\data\backups" -Force
New-Item -ItemType Directory -Path "$installerDir\data\logs" -Force
New-Item -ItemType Directory -Path "$installerDir\data\gpu-cache" -Force

# Create empty database file
New-Item -ItemType File -Path "$installerDir\data\guardian.db" -Force

# Create startup script
Write-Host "Creating startup script..." -ForegroundColor Yellow
$startupScript = @"
@echo off
echo Starting Guardian Minecraft Server Manager...
echo.
echo Backend services will start automatically.
echo Press any key to launch the application...
pause >nul
start "" "Guardian.exe"
"@
$startupScript | Out-File -FilePath "$installerDir\Start-Guardian.bat" -Encoding ASCII

# Create README
Write-Host "Creating README..." -ForegroundColor Yellow
$readme = @"
# Guardian Minecraft Server Manager

## Quick Start
1. Double-click "Start-Guardian.bat" to launch the application
2. Or double-click "Guardian.exe" to run directly

## Features
- Professional Minecraft server management
- GPU-accelerated chunk generation
- Real-time monitoring and diagnostics
- Automated backups and snapshots
- Multi-server support

## System Requirements
- Windows 10/11
- 4GB+ RAM
- DirectX 11 compatible GPU (for GPU acceleration)

## Support
For issues or questions, check the console output or contact support.

## Version
Guardian v1.0.0
"@
$readme | Out-File -FilePath "$installerDir\README.txt" -Encoding ASCII

Write-Host "`nInstaller created successfully!" -ForegroundColor Green
Write-Host "Location: $installerDir" -ForegroundColor Cyan
Write-Host "`nTo install:" -ForegroundColor Yellow
Write-Host "1. Copy the entire '$installerDir' folder to your desired location" -ForegroundColor White
Write-Host "2. Run 'Start-Guardian.bat' to launch the application" -ForegroundColor White
Write-Host "`nThe application is now ready for distribution!" -ForegroundColor Green
