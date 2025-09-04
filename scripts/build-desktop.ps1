# Build script for Guardian Desktop Application
# This script builds both the backend (hostd) and frontend (guardian-ui) and packages them as a desktop app

Write-Host "🚀 Building Guardian Desktop Application..." -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Build the backend (hostd)
Write-Host "📦 Building backend (hostd)..." -ForegroundColor Yellow
Set-Location "$ProjectRoot/hostd"
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Backend build failed"
    }
    Write-Host "✅ Backend built successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Backend build failed: $_" -ForegroundColor Red
    exit 1
}

# Build the frontend (guardian-ui)
Write-Host "🎨 Building frontend (guardian-ui)..." -ForegroundColor Yellow
Set-Location "$ProjectRoot/guardian-ui"
try {
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "Frontend build failed"
    }
    Write-Host "✅ Frontend built successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Frontend build failed: $_" -ForegroundColor Red
    exit 1
}

# Build the desktop app with Tauri
Write-Host "🖥️ Building desktop application..." -ForegroundColor Yellow
try {
    npm run tauri:build
    if ($LASTEXITCODE -ne 0) {
        throw "Desktop app build failed"
    }
    Write-Host "✅ Desktop application built successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Desktop app build failed: $_" -ForegroundColor Red
    exit 1
}

# Copy the backend binary to the output directory
Write-Host "📋 Copying backend binary..." -ForegroundColor Yellow
$BackendBinary = "$ProjectRoot/hostd/target/release/hostd.exe"
$OutputDir = "$ProjectRoot/guardian-ui/src-tauri/target/release/bundle"

if (Test-Path $BackendBinary) {
    # Find the output directory (could be msi, nsis, or appimage)
    $BundleDirs = Get-ChildItem -Path $OutputDir -Directory
    foreach ($BundleDir in $BundleDirs) {
        $BundlePath = $BundleDir.FullName
        Write-Host "Copying to: $BundlePath" -ForegroundColor Blue
        
        # Copy the binary
        Copy-Item $BackendBinary "$BundlePath/hostd.exe" -Force
        
        # Create a batch file to start the backend
        $StartScript = @"
@echo off
cd /d "%~dp0"
start /B hostd.exe --port 8080 --database-url sqlite:guardian.db
"@
        Set-Content -Path "$BundlePath/start-backend.bat" -Value $StartScript
    }
    Write-Host "✅ Backend binary copied successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️ Backend binary not found at $BackendBinary" -ForegroundColor Yellow
}

Write-Host "🎉 Guardian Desktop Application build completed!" -ForegroundColor Green
Write-Host "📁 Output directory: $OutputDir" -ForegroundColor Blue

# Return to project root
Set-Location $ProjectRoot
