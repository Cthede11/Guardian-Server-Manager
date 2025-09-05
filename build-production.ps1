# Guardian Production Build Script
# This script builds the complete Guardian application for production distribution

Write-Host "🚀 Building Guardian - Professional Minecraft Server Manager" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Function to check if command exists
function Test-Command {
    param($Command)
    try {
        if (Get-Command $Command -ErrorAction SilentlyContinue) {
            return $true
        }
    }
    catch {
        return $false
    }
    return $false
}

# Check prerequisites
Write-Host "📋 Checking prerequisites..." -ForegroundColor Yellow

if (-not (Test-Command "cargo")) {
    Write-Error "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
}

if (-not (Test-Command "node")) {
    Write-Error "❌ Node.js not found. Please install Node.js from https://nodejs.org/"
    exit 1
}

if (-not (Test-Command "npm")) {
    Write-Error "❌ npm not found. Please install npm"
    exit 1
}

Write-Host "✅ Prerequisites check passed" -ForegroundColor Green

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "guardian-ui/dist") {
    Remove-Item -Recurse -Force "guardian-ui/dist"
}
if (Test-Path "guardian-ui/src-tauri/target") {
    Remove-Item -Recurse -Force "guardian-ui/src-tauri/target"
}
if (Test-Path "hostd/target") {
    Remove-Item -Recurse -Force "hostd/target"
}
if (Test-Path "gpu-worker/target") {
    Remove-Item -Recurse -Force "gpu-worker/target"
}

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
npm install
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

$BuildDir = "guardian-production"
if (Test-Path $BuildDir) {
    Remove-Item -Recurse -Force $BuildDir
}
New-Item -ItemType Directory -Path $BuildDir | Out-Null

# Copy built application
$AppPath = "guardian-ui/src-tauri/target/release/bundle/msi"
if (Test-Path $AppPath) {
    $MsiFiles = Get-ChildItem -Path $AppPath -Filter "*.msi"
    if ($MsiFiles.Count -gt 0) {
        Copy-Item $MsiFiles[0].FullName -Destination $BuildDir
        Write-Host "  ✅ MSI installer copied" -ForegroundColor Green
    }
}

# Copy configuration files
Copy-Item -Path "configs" -Destination "$BuildDir/configs" -Recurse
Write-Host "  ✅ Configuration files copied" -ForegroundColor Green

# Create data directory
New-Item -ItemType Directory -Path "$BuildDir/data" | Out-Null
Write-Host "  ✅ Data directory created" -ForegroundColor Green

# Create README for production
$ReadmeContent = @'
# Guardian - Professional Minecraft Server Manager

## Installation
1. Run the MSI installer to install Guardian
2. Launch Guardian from your Start Menu or Desktop
3. The application will automatically start all required services

## First Time Setup
1. Open Guardian
2. Configure your server settings in the Settings tab
3. Create your first Minecraft server
4. Start managing your servers!

## Features
- Real-time server monitoring
- Advanced performance optimization
- Automated backup management
- GPU-accelerated chunk generation
- Professional server management tools

## Support
For support and documentation, visit our website or contact support.

Copyright (c) 2024 Guardian Team
'@

Set-Content -Path "$BuildDir/README.txt" -Value $ReadmeContent
Write-Host "  ✅ README created" -ForegroundColor Green

Write-Host "🎉 Production build completed successfully!" -ForegroundColor Green
Write-Host "📁 Build output: $BuildDir" -ForegroundColor Cyan
Write-Host "🚀 Ready for distribution!" -ForegroundColor Green
