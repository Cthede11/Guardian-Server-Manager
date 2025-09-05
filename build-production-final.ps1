# Guardian Production Build Script - Final Version
# This script builds the complete production application for real server testing

Write-Host "Building Guardian Production Application..." -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "guardian-ui\src-tauri\target") {
    Remove-Item -Recurse -Force "guardian-ui\src-tauri\target"
}
if (Test-Path "hostd\target") {
    Remove-Item -Recurse -Force "hostd\target"
}
if (Test-Path "gpu-worker\target") {
    Remove-Item -Recurse -Force "gpu-worker\target"
}

# Build hostd backend
Write-Host "Building hostd backend..." -ForegroundColor Yellow
cd hostd
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build hostd" -ForegroundColor Red
    exit 1
}
cd ..

# Build gpu-worker
Write-Host "Building gpu-worker..." -ForegroundColor Yellow
cd gpu-worker
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build gpu-worker" -ForegroundColor Red
    exit 1
}
cd ..

# Build frontend
Write-Host "Building frontend..." -ForegroundColor Yellow
cd guardian-ui
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build frontend" -ForegroundColor Red
    exit 1
}
cd ..

# Build Tauri application
Write-Host "Building Tauri application..." -ForegroundColor Yellow
cd guardian-ui
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build Tauri application" -ForegroundColor Red
    exit 1
}
cd ..

# Copy executables to Tauri target directory
Write-Host "Copying executables..." -ForegroundColor Yellow
$tauriTargetDir = "guardian-ui\src-tauri\target\release"
$tauriResourceDir = "guardian-ui\src-tauri\target\release\bundle\msi\Guardian_1.0.0_x64_en-US\resources"

# Copy hostd and gpu-worker executables
Copy-Item "hostd\target\release\hostd.exe" -Destination $tauriTargetDir -Force
Copy-Item "gpu-worker\target\release\gpu-worker.exe" -Destination $tauriTargetDir -Force

# Copy configuration files
Copy-Item "configs" -Destination $tauriTargetDir -Recurse -Force
Copy-Item "data" -Destination $tauriTargetDir -Recurse -Force

# If MSI was created, copy to the MSI resources directory
if (Test-Path $tauriResourceDir) {
    Copy-Item "hostd\target\release\hostd.exe" -Destination $tauriResourceDir -Force
    Copy-Item "gpu-worker\target\release\gpu-worker.exe" -Destination $tauriResourceDir -Force
    Copy-Item "configs" -Destination $tauriResourceDir -Recurse -Force
    Copy-Item "data" -Destination $tauriResourceDir -Recurse -Force
}

Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host "Application location: guardian-ui\src-tauri\target\release" -ForegroundColor Cyan
Write-Host "MSI installer: guardian-ui\src-tauri\target\release\bundle\msi\" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Test the application: .\test-production-app.ps1" -ForegroundColor White
Write-Host "2. Install the MSI: Run the installer in the bundle\msi folder" -ForegroundColor White
Write-Host "3. Test with real Minecraft servers" -ForegroundColor White