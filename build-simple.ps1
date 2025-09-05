# Simple Build Script for Guardian
Write-Host "🚀 Building Guardian Minecraft Server Manager..." -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Step 1: Build hostd
Write-Host "📦 Building hostd..." -ForegroundColor Yellow
Set-Location "hostd"
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ hostd build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ hostd built successfully" -ForegroundColor Green
Set-Location ".."

# Step 2: Build gpu-worker
Write-Host "🎮 Building gpu-worker..." -ForegroundColor Yellow
Set-Location "gpu-worker"
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ gpu-worker build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ gpu-worker built successfully" -ForegroundColor Green
Set-Location ".."

# Step 3: Build frontend
Write-Host "🎨 Building frontend..." -ForegroundColor Yellow
Set-Location "guardian-ui"
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Frontend build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Frontend built successfully" -ForegroundColor Green

# Step 4: Copy executables to Tauri target directory
Write-Host "📋 Copying executables..." -ForegroundColor Yellow
$tauriTargetDir = "src-tauri\target\release"
if (-not (Test-Path $tauriTargetDir)) {
    New-Item -ItemType Directory -Path $tauriTargetDir -Force | Out-Null
}

# Copy hostd.exe
if (Test-Path "..\hostd\target\release\hostd.exe") {
    Copy-Item "..\hostd\target\release\hostd.exe" -Destination $tauriTargetDir -Force
    Write-Host "✅ Copied hostd.exe" -ForegroundColor Green
} else {
    Write-Host "❌ hostd.exe not found" -ForegroundColor Red
}

# Copy gpu-worker.exe
if (Test-Path "..\gpu-worker\target\release\gpu-worker.exe") {
    Copy-Item "..\gpu-worker\target\release\gpu-worker.exe" -Destination $tauriTargetDir -Force
    Write-Host "✅ Copied gpu-worker.exe" -ForegroundColor Green
} else {
    Write-Host "❌ gpu-worker.exe not found" -ForegroundColor Red
}

# Copy configs
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

# Step 5: Build Tauri application
Write-Host "🖥️ Building Tauri application..." -ForegroundColor Yellow
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tauri build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Tauri application built successfully" -ForegroundColor Green

Set-Location ".."

Write-Host ""
Write-Host "🎉 Build completed successfully!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Check for final executables
$guardianExe = "guardian-ui\src-tauri\target\release\guardian.exe"
$hostdExe = "guardian-ui\src-tauri\target\release\hostd.exe"
$gpuWorkerExe = "guardian-ui\src-tauri\target\release\gpu-worker.exe"

Write-Host "📋 Final Results:" -ForegroundColor Cyan
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

Write-Host ""
Write-Host "🚀 Ready to run!" -ForegroundColor Green
Write-Host "Test with: .\guardian-ui\src-tauri\target\release\guardian.exe" -ForegroundColor White