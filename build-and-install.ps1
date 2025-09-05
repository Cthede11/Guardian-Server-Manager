# Guardian Build and Install Script
# This script builds and installs the complete Guardian application

Write-Host "🚀 Guardian Minecraft Server Manager - Build & Install" -ForegroundColor Green
Write-Host "=====================================================" -ForegroundColor Green
Write-Host ""

# Function to check if command succeeded
function Test-CommandSuccess {
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Command failed with exit code $LASTEXITCODE" -ForegroundColor Red
        exit 1
    }
}

# Step 1: Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "guardian-ui\src-tauri\target") {
    Remove-Item -Recurse -Force "guardian-ui\src-tauri\target" -ErrorAction SilentlyContinue
}
if (Test-Path "hostd\target") {
    Remove-Item -Recurse -Force "hostd\target" -ErrorAction SilentlyContinue
}
if (Test-Path "gpu-worker\target") {
    Remove-Item -Recurse -Force "gpu-worker\target" -ErrorAction SilentlyContinue
}
Write-Host "✅ Cleanup completed" -ForegroundColor Green
Write-Host ""

# Step 2: Build hostd backend
Write-Host "📦 Building hostd backend..." -ForegroundColor Yellow
Set-Location "hostd"
cargo build --release
Test-CommandSuccess
Write-Host "✅ hostd backend built successfully" -ForegroundColor Green
Set-Location ".."
Write-Host ""

# Step 3: Build gpu-worker
Write-Host "🎮 Building gpu-worker..." -ForegroundColor Yellow
Set-Location "gpu-worker"
cargo build --release
Test-CommandSuccess
Write-Host "✅ gpu-worker built successfully" -ForegroundColor Green
Set-Location ".."
Write-Host ""

# Step 4: Build frontend
Write-Host "🎨 Building frontend..." -ForegroundColor Yellow
Set-Location "guardian-ui"
npm run build
Test-CommandSuccess
Write-Host "✅ Frontend built successfully" -ForegroundColor Green
Write-Host ""

# Step 5: Build Tauri application
Write-Host "🖥️ Building Tauri application..." -ForegroundColor Yellow
npm run tauri:build
Test-CommandSuccess
Write-Host "✅ Tauri application built successfully" -ForegroundColor Green
Write-Host ""

# Step 6: Copy executables and resources
Write-Host "📋 Copying executables and resources..." -ForegroundColor Yellow
$tauriTargetDir = "src-tauri\target\release"
$tauriResourceDir = "src-tauri\target\release\bundle\msi\Guardian_1.0.0_x64_en-US\resources"

# Copy hostd and gpu-worker executables
if (Test-Path "..\hostd\target\release\hostd.exe") {
    Copy-Item "..\hostd\target\release\hostd.exe" -Destination $tauriTargetDir -Force
    Write-Host "✅ Copied hostd.exe" -ForegroundColor Green
} else {
    Write-Host "⚠️ hostd.exe not found" -ForegroundColor Yellow
}

if (Test-Path "..\gpu-worker\target\release\gpu-worker.exe") {
    Copy-Item "..\gpu-worker\target\release\gpu-worker.exe" -Destination $tauriTargetDir -Force
    Write-Host "✅ Copied gpu-worker.exe" -ForegroundColor Green
} else {
    Write-Host "⚠️ gpu-worker.exe not found" -ForegroundColor Yellow
}

# Copy configuration files
if (Test-Path "..\configs") {
    Copy-Item "..\configs" -Destination $tauriTargetDir -Recurse -Force
    Write-Host "✅ Copied configs" -ForegroundColor Green
}

# Create data directory
$dataDir = "$tauriTargetDir\data"
if (-not (Test-Path $dataDir)) {
    New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
    New-Item -ItemType Directory -Path "$dataDir\servers" -Force | Out-Null
    New-Item -ItemType Directory -Path "$dataDir\backups" -Force | Out-Null
    New-Item -ItemType Directory -Path "$dataDir\logs" -Force | Out-Null
    New-Item -ItemType Directory -Path "$dataDir\mods" -Force | Out-Null
    Write-Host "✅ Created data directories" -ForegroundColor Green
}

# If MSI was created, copy to the MSI resources directory
if (Test-Path $tauriResourceDir) {
    Write-Host "📦 MSI installer found, copying resources..." -ForegroundColor Cyan
    
    if (Test-Path "..\hostd\target\release\hostd.exe") {
        Copy-Item "..\hostd\target\release\hostd.exe" -Destination $tauriResourceDir -Force
    }
    if (Test-Path "..\gpu-worker\target\release\gpu-worker.exe") {
        Copy-Item "..\gpu-worker\target\release\gpu-worker.exe" -Destination $tauriResourceDir -Force
    }
    if (Test-Path "..\configs") {
        Copy-Item "..\configs" -Destination $tauriResourceDir -Recurse -Force
    }
    
    Write-Host "✅ MSI resources updated" -ForegroundColor Green
}

Set-Location ".."
Write-Host ""

# Step 7: Display results
Write-Host "🎉 Build completed successfully!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host ""

# Check for executables
$guardianExe = "guardian-ui\src-tauri\target\release\guardian.exe"
$hostdExe = "guardian-ui\src-tauri\target\release\hostd.exe"
$gpuWorkerExe = "guardian-ui\src-tauri\target\release\gpu-worker.exe"

Write-Host "📋 Build Results:" -ForegroundColor Cyan
if (Test-Path $guardianExe) {
    Write-Host "✅ Guardian.exe: $guardianExe" -ForegroundColor Green
} else {
    Write-Host "❌ Guardian.exe: Not found" -ForegroundColor Red
}

if (Test-Path $hostdExe) {
    Write-Host "✅ hostd.exe: $hostdExe" -ForegroundColor Green
} else {
    Write-Host "❌ hostd.exe: Not found" -ForegroundColor Red
}

if (Test-Path $gpuWorkerExe) {
    Write-Host "✅ gpu-worker.exe: $gpuWorkerExe" -ForegroundColor Green
} else {
    Write-Host "❌ gpu-worker.exe: Not found" -ForegroundColor Red
}

# Check for MSI installer
$msiPath = "guardian-ui\src-tauri\target\release\bundle\msi\Guardian_1.0.0_x64_en-US.msi"
if (Test-Path $msiPath) {
    Write-Host "✅ MSI Installer: $msiPath" -ForegroundColor Green
} else {
    Write-Host "⚠️ MSI Installer: Not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🚀 Next Steps:" -ForegroundColor Yellow
Write-Host "1. Test the application: .\guardian-ui\src-tauri\target\release\guardian.exe" -ForegroundColor White
Write-Host "2. Install MSI: Run the installer in guardian-ui\src-tauri\target\release\bundle\msi\" -ForegroundColor White
Write-Host "3. Start backend: .\guardian-ui\src-tauri\target\release\hostd.exe --config configs\hostd.yaml" -ForegroundColor White
Write-Host ""
Write-Host "🎯 Guardian is ready for production use!" -ForegroundColor Green
