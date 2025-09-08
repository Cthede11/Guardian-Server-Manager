# Build script for Guardian Desktop Application
# This script builds both the backend (hostd) and frontend (guardian-ui) and packages them as a desktop app

Write-Host "[BUILD] Building Guardian Desktop Application..." -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Build the backend (hostd)
Write-Host "[BACKEND] Building backend (hostd)..." -ForegroundColor Yellow
Set-Location "$ProjectRoot/hostd"
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Backend build failed"
    }
    Write-Host "[SUCCESS] Backend built successfully" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Backend build failed: $_" -ForegroundColor Red
    exit 1
}

# Build the frontend (guardian-ui)
Write-Host "[FRONTEND] Building frontend (guardian-ui)..." -ForegroundColor Yellow
Set-Location "$ProjectRoot/guardian-ui"
try {
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "Frontend build failed"
    }
    Write-Host "[SUCCESS] Frontend built successfully" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Frontend build failed: $_" -ForegroundColor Red
    exit 1
}

# Build the desktop app with Tauri
Write-Host "[DESKTOP] Building desktop application..." -ForegroundColor Yellow
try {
    npm run tauri:build
    if ($LASTEXITCODE -ne 0) {
        throw "Desktop app build failed"
    }
    Write-Host "[SUCCESS] Desktop application built successfully" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Desktop app build failed: $_" -ForegroundColor Red
    exit 1
}

# Copy build artifacts to centralized build directory
Write-Host "[COPY] Organizing build artifacts..." -ForegroundColor Yellow
$BackendBinary = "$ProjectRoot/hostd/target/release/hostd.exe"
$GpuWorkerBinary = "$ProjectRoot/gpu-worker/target/release/gpu-worker.exe"
$BuildDir = "$ProjectRoot/build"
$ExecutablesDir = "$BuildDir/executables"
$InstallersDir = "$BuildDir/installers"
$TauriOutputDir = "$ProjectRoot/guardian-ui/src-tauri/target/release/bundle"

# Ensure build directories exist
New-Item -ItemType Directory -Path $ExecutablesDir, $InstallersDir -Force | Out-Null

# Copy backend binary
if (Test-Path $BackendBinary) {
    Copy-Item $BackendBinary "$ExecutablesDir/hostd.exe" -Force
    Write-Host "[SUCCESS] Backend binary copied to build/executables/" -ForegroundColor Green
} else {
    Write-Host "[WARNING] Backend binary not found at $BackendBinary" -ForegroundColor Yellow
}

# Copy GPU worker binary
if (Test-Path $GpuWorkerBinary) {
    Copy-Item $GpuWorkerBinary "$ExecutablesDir/gpu-worker.exe" -Force
    Write-Host "[SUCCESS] GPU worker binary copied to build/executables/" -ForegroundColor Green
} else {
    Write-Host "[WARNING] GPU worker binary not found at $GpuWorkerBinary" -ForegroundColor Yellow
}

# Copy Tauri installers if they exist
if (Test-Path $TauriOutputDir) {
    $BundleDirs = Get-ChildItem -Path $TauriOutputDir -Directory
    foreach ($BundleDir in $BundleDirs) {
        $BundlePath = $BundleDir.FullName
        Write-Host "Copying Tauri bundle from: $BundlePath" -ForegroundColor Blue
        
        # Copy all files from the bundle to installers directory
        $BundleName = $BundleDir.Name
        $TargetDir = "$InstallersDir/$BundleName"
        New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
        Copy-Item "$BundlePath/*" $TargetDir -Recurse -Force
        
        # Create a batch file to start the backend
        $StartScript = @"
@echo off
cd /d "%~dp0"
start /B hostd.exe --port 8080 --database-url sqlite:guardian.db
"@
        Set-Content -Path "$TargetDir/start-backend.bat" -Value $StartScript
    }
    Write-Host "[SUCCESS] Tauri bundles copied to build/installers/" -ForegroundColor Green
}

Write-Host "[COMPLETE] Guardian Desktop Application build completed!" -ForegroundColor Green
Write-Host "[OUTPUT] Executables: $ExecutablesDir" -ForegroundColor Blue
Write-Host "[OUTPUT] Installers: $InstallersDir" -ForegroundColor Blue

# Return to project root
Set-Location $ProjectRoot
