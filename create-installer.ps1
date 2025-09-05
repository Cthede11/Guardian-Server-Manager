# Create Complete Installer with Persistent Storage
# This script creates a production-ready installer with all components

Write-Host "Creating Complete Guardian Installer..." -ForegroundColor Green
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

# Build all components
Write-Host "Building hostd backend..." -ForegroundColor Yellow
cd hostd
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build hostd" -ForegroundColor Red
    exit 1
}
cd ..

Write-Host "Building gpu-worker..." -ForegroundColor Yellow
cd gpu-worker
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build gpu-worker" -ForegroundColor Red
    exit 1
}
cd ..

Write-Host "Building frontend..." -ForegroundColor Yellow
cd guardian-ui
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build frontend" -ForegroundColor Red
    exit 1
}
cd ..

# Create data directory structure for installer
Write-Host "Creating data directory structure..." -ForegroundColor Yellow
$installerDataDir = "installer-data"
if (Test-Path $installerDataDir) {
    Remove-Item -Recurse -Force $installerDataDir
}
New-Item -ItemType Directory -Path $installerDataDir -Force
New-Item -ItemType Directory -Path "$installerDataDir\servers" -Force
New-Item -ItemType Directory -Path "$installerDataDir\backups" -Force
New-Item -ItemType Directory -Path "$installerDataDir\logs" -Force
New-Item -ItemType Directory -Path "$installerDataDir\gpu-cache" -Force

# Create empty database file for installer
New-Item -ItemType File -Path "$installerDataDir\guardian.db" -Force

Write-Host "Data structure created" -ForegroundColor Green

# Update Tauri config to include installer data
Write-Host "Updating Tauri configuration..." -ForegroundColor Yellow
$tauriConfig = Get-Content "guardian-ui\src-tauri\tauri.conf.json" | ConvertFrom-Json
$tauriConfig.bundle.resources += @("../../$installerDataDir")
$tauriConfig | ConvertTo-Json -Depth 10 | Set-Content "guardian-ui\src-tauri\tauri.conf.json"

# Build Tauri application
Write-Host "Building Tauri application..." -ForegroundColor Yellow
cd guardian-ui
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build Tauri application" -ForegroundColor Red
    exit 1
}
cd ..

# Copy additional files to the installer
Write-Host "Preparing installer files..." -ForegroundColor Yellow
$tauriTargetDir = "guardian-ui\src-tauri\target\release"
$tauriResourceDir = "guardian-ui\src-tauri\target\release\bundle\msi\Guardian_1.0.0_x64_en-US\resources"

# Copy executables
Copy-Item "hostd\target\release\hostd.exe" -Destination $tauriTargetDir -Force
Copy-Item "gpu-worker\target\release\gpu-worker.exe" -Destination $tauriTargetDir -Force

# Copy configuration files
Copy-Item "configs" -Destination $tauriTargetDir -Recurse -Force

# Copy data structure
Copy-Item $installerDataDir -Destination $tauriTargetDir -Recurse -Force

# If MSI was created, copy to the MSI resources directory
if (Test-Path $tauriResourceDir) {
    Copy-Item "hostd\target\release\hostd.exe" -Destination $tauriResourceDir -Force
    Copy-Item "gpu-worker\target\release\gpu-worker.exe" -Destination $tauriResourceDir -Force
    Copy-Item "configs" -Destination $tauriResourceDir -Recurse -Force
    Copy-Item $installerDataDir -Destination $tauriResourceDir -Recurse -Force
}

# Create post-installation script
Write-Host "Creating post-installation script..." -ForegroundColor Yellow
$postInstallScript = @"
# Guardian Post-Installation Script
# This script runs after installation to set up persistent storage

Write-Host "Setting up Guardian data directories..." -ForegroundColor Green

# Get the installation directory
`$installDir = Split-Path -Parent `$MyInvocation.MyCommand.Path
`$dataDir = Join-Path `$installDir "data"

# Create data directories if they don't exist
if (-not (Test-Path `$dataDir)) {
    New-Item -ItemType Directory -Path `$dataDir -Force
    Write-Host "Created data directory: `$dataDir" -ForegroundColor Green
}

if (-not (Test-Path (Join-Path `$dataDir "servers"))) {
    New-Item -ItemType Directory -Path (Join-Path `$dataDir "servers") -Force
    Write-Host "Created servers directory" -ForegroundColor Green
}

if (-not (Test-Path (Join-Path `$dataDir "backups"))) {
    New-Item -ItemType Directory -Path (Join-Path `$dataDir "backups") -Force
    Write-Host "Created backups directory" -ForegroundColor Green
}

if (-not (Test-Path (Join-Path `$dataDir "logs"))) {
    New-Item -ItemType Directory -Path (Join-Path `$dataDir "logs") -Force
    Write-Host "Created logs directory" -ForegroundColor Green
}

# Create empty database file if it doesn't exist
`$dbFile = Join-Path `$dataDir "guardian.db"
if (-not (Test-Path `$dbFile)) {
    New-Item -ItemType File -Path `$dbFile -Force
    Write-Host "Created database file: `$dbFile" -ForegroundColor Green
}

Write-Host "Guardian setup complete!" -ForegroundColor Green
Write-Host "Data directory: `$dataDir" -ForegroundColor Cyan
"@

$postInstallScript | Out-File -FilePath "guardian-ui\src-tauri\target\release\setup-data.ps1" -Encoding UTF8

# Clean up temporary installer data
Remove-Item -Recurse -Force $installerDataDir

Write-Host "===============================================" -ForegroundColor Green
Write-Host "Installer creation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Installer location: guardian-ui\src-tauri\target\release\bundle\msi\" -ForegroundColor Cyan
Write-Host "Application location: guardian-ui\src-tauri\target\release\" -ForegroundColor Cyan
Write-Host ""
Write-Host "What's included:" -ForegroundColor Yellow
Write-Host "- Guardian.exe (main application)" -ForegroundColor White
Write-Host "- hostd.exe (backend service)" -ForegroundColor White
Write-Host "- gpu-worker.exe (GPU acceleration)" -ForegroundColor White
Write-Host "- configs/ (configuration files)" -ForegroundColor White
Write-Host "- data/ (persistent storage structure)" -ForegroundColor White
Write-Host "- setup-data.ps1 (post-installation setup)" -ForegroundColor White
Write-Host ""
Write-Host "After installation:" -ForegroundColor Yellow
Write-Host "1. Run the MSI installer" -ForegroundColor White
Write-Host "2. Run setup-data.ps1 to create data directories" -ForegroundColor White
Write-Host "3. Launch Guardian.exe" -ForegroundColor White
Write-Host "4. Start managing Minecraft servers!" -ForegroundColor White
