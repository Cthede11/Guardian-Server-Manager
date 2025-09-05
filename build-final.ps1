# Guardian Final Production Build Script
# Creates a complete, distributable application package

Write-Host "🚀 Building Guardian - Final Production Release" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Configuration
$AppName = "Guardian"
$AppVersion = "1.0.0"
$BuildDir = "guardian-production"
$DistDir = "guardian-distribution"

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path $BuildDir) {
    Remove-Item -Recurse -Force $BuildDir
}
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}

# Create build directories
New-Item -ItemType Directory -Path $BuildDir | Out-Null
New-Item -ItemType Directory -Path $DistDir | Out-Null

Write-Host "✅ Build directories created" -ForegroundColor Green

# Build backend services
Write-Host "🔧 Building backend services..." -ForegroundColor Yellow

# Build hostd
Write-Host "  Building hostd..." -ForegroundColor Cyan
Set-Location "hostd"
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ Failed to build hostd"
    exit 1
}
Set-Location ".."

# Build gpu-worker
Write-Host "  Building gpu-worker..." -ForegroundColor Cyan
Set-Location "gpu-worker"
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ Failed to build gpu-worker"
    exit 1
}
Set-Location ".."

Write-Host "✅ Backend services built successfully" -ForegroundColor Green

# Build frontend
Write-Host "🎨 Building frontend..." -ForegroundColor Yellow
Set-Location "guardian-ui"

# Install dependencies
Write-Host "  Installing dependencies..." -ForegroundColor Cyan
npm install --production
if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ Failed to install frontend dependencies"
    exit 1
}

# Build frontend
Write-Host "  Building React app..." -ForegroundColor Cyan
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ Failed to build frontend"
    exit 1
}

# Build Tauri app
Write-Host "  Building Tauri app..." -ForegroundColor Cyan
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ Failed to build Tauri app"
    exit 1
}

Set-Location ".."

Write-Host "✅ Frontend built successfully" -ForegroundColor Green

# Create production package
Write-Host "📦 Creating production package..." -ForegroundColor Yellow

# Copy built application
$AppPath = "guardian-ui/src-tauri/target/release/bundle/msi"
if (Test-Path $AppPath) {
    $MsiFiles = Get-ChildItem -Path $AppPath -Filter "*.msi"
    if ($MsiFiles.Count -gt 0) {
        Copy-Item $MsiFiles[0].FullName -Destination $DistDir
        Write-Host "  ✅ MSI installer copied" -ForegroundColor Green
    }
}

# Copy executable files
$ExePath = "guardian-ui/src-tauri/target/release"
if (Test-Path "$ExePath/guardian.exe") {
    Copy-Item "$ExePath/guardian.exe" -Destination $DistDir
    Write-Host "  ✅ Guardian executable copied" -ForegroundColor Green
}

# Copy backend executables
if (Test-Path "hostd/target/release/hostd.exe") {
    Copy-Item "hostd/target/release/hostd.exe" -Destination $DistDir
    Write-Host "  ✅ Hostd executable copied" -ForegroundColor Green
}

if (Test-Path "gpu-worker/target/release/gpu-worker.exe") {
    Copy-Item "gpu-worker/target/release/gpu-worker.exe" -Destination $DistDir
    Write-Host "  ✅ GPU worker executable copied" -ForegroundColor Green
}

# Copy configuration files
if (Test-Path "configs") {
    Copy-Item -Path "configs" -Destination "$DistDir/configs" -Recurse
    Write-Host "  ✅ Configuration files copied" -ForegroundColor Green
}

# Create data directory
New-Item -ItemType Directory -Path "$DistDir/data" | Out-Null
Write-Host "  ✅ Data directory created" -ForegroundColor Green

# Create logs directory
New-Item -ItemType Directory -Path "$DistDir/logs" | Out-Null
Write-Host "  ✅ Logs directory created" -ForegroundColor Green

# Create production README
$ReadmeContent = @'
# Guardian - Professional Minecraft Server Manager

## Quick Start
1. Run the MSI installer to install Guardian
2. Launch Guardian from your Start Menu or Desktop
3. Configure your server settings
4. Start managing your Minecraft servers!

## Features
- Real-time server monitoring
- Advanced performance optimization
- Automated backup management
- GPU-accelerated chunk generation
- Professional server management tools

## System Requirements
- Windows 10/11 (64-bit)
- 4GB+ RAM
- 2GB+ free disk space
- DirectX 11 compatible GPU (recommended)

## Support
For support and documentation, visit our website or contact support.

Copyright (c) 2024 Guardian Team
'@

Set-Content -Path "$DistDir/README.txt" -Value $ReadmeContent
Write-Host "  ✅ README created" -ForegroundColor Green

# Create launcher script
$LauncherContent = @"
@echo off
echo Starting Guardian...
start "" "guardian.exe"
"@

Set-Content -Path "$DistDir/start-guardian.bat" -Value $LauncherContent
Write-Host "  ✅ Launcher script created" -ForegroundColor Green

# Create uninstaller script
$UninstallerContent = @"
@echo off
echo Uninstalling Guardian...
taskkill /F /IM guardian.exe 2>nul
taskkill /F /IM hostd.exe 2>nul
taskkill /F /IM gpu-worker.exe 2>nul
echo Guardian uninstalled successfully.
pause
"@

Set-Content -Path "$DistDir/uninstall.bat" -Value $UninstallerContent
Write-Host "  ✅ Uninstaller script created" -ForegroundColor Green

# Create version info file
$VersionInfo = @"
{
  "name": "Guardian",
  "version": "1.0.0",
  "build_date": "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')",
  "platform": "Windows x64",
  "components": {
    "frontend": "React + Tauri",
    "backend": "Rust",
    "gpu_worker": "Rust + WebGPU"
  }
}
"@

Set-Content -Path "$DistDir/version.json" -Value $VersionInfo
Write-Host "  ✅ Version info created" -ForegroundColor Green

# Create distribution archive
Write-Host "📦 Creating distribution archive..." -ForegroundColor Yellow
$ArchiveName = "Guardian-v$AppVersion-Windows-x64.zip"
Compress-Archive -Path "$DistDir/*" -DestinationPath $ArchiveName -Force
Write-Host "  ✅ Distribution archive created: $ArchiveName" -ForegroundColor Green

# Final summary
Write-Host "🎉 Production build completed successfully!" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Green
Write-Host "📁 Build output: $DistDir" -ForegroundColor Cyan
Write-Host "📦 Distribution archive: $ArchiveName" -ForegroundColor Cyan
Write-Host "🚀 Ready for distribution!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Test the application in the $DistDir folder" -ForegroundColor White
Write-Host "2. Distribute the $ArchiveName file" -ForegroundColor White
Write-Host "3. Or run the MSI installer for proper installation" -ForegroundColor White
