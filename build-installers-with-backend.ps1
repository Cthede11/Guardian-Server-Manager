# Build Guardian Installers with Backend Integration
Write-Host "Building Guardian Installers with Backend Integration..." -ForegroundColor Green
Write-Host ""

# Check if we're in the right directory
if (-not (Test-Path "guardian-ui\src-tauri\tauri.conf.json")) {
    Write-Host "Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Build the project first
Write-Host "Building Guardian project..." -ForegroundColor Cyan
cd guardian-ui

# Build frontend
Write-Host "Building frontend..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Frontend build failed!" -ForegroundColor Red
    exit 1
}

# Build Tauri app
Write-Host "Building Tauri application..." -ForegroundColor Yellow
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri build failed!" -ForegroundColor Red
    exit 1
}

cd ..

# Build backend services
Write-Host "Building backend services..." -ForegroundColor Cyan

# Build hostd
Write-Host "Building hostd..." -ForegroundColor Yellow
cd hostd
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Hostd build failed!" -ForegroundColor Red
    exit 1
}
cd ..

# Build gpu-worker
Write-Host "Building gpu-worker..." -ForegroundColor Yellow
cd gpu-worker
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "GPU worker build failed!" -ForegroundColor Red
    exit 1
}
cd ..

# Copy backend executables to Tauri resources
Write-Host "Copying backend executables..." -ForegroundColor Cyan
Copy-Item "hostd\target\release\hostd.exe" "guardian-ui\src-tauri\" -Force
Copy-Item "gpu-worker\target\release\gpu-worker.exe" "guardian-ui\src-tauri\" -Force

# Copy launcher scripts to Tauri resources
Write-Host "Copying launcher scripts..." -ForegroundColor Cyan
Copy-Item "launchers\start-guardian-with-backend.bat" "guardian-ui\src-tauri\" -Force
Copy-Item "launchers\start-guardian-with-backend.ps1" "guardian-ui\src-tauri\" -Force
Copy-Item "launchers\create-desktop-shortcut.ps1" "guardian-ui\src-tauri\" -Force
Copy-Item "launchers\update-desktop-shortcut.ps1" "guardian-ui\src-tauri\" -Force

# Copy post-installation setup scripts
Write-Host "Copying post-installation setup scripts..." -ForegroundColor Cyan
Copy-Item "post-install-setup.ps1" "guardian-ui\src-tauri\" -Force
Copy-Item "Setup-Guardian-Shortcuts.bat" "guardian-ui\src-tauri\" -Force

# Copy configs to Tauri resources
Write-Host "Copying configuration files..." -ForegroundColor Cyan
Copy-Item "configs" "guardian-ui\src-tauri\" -Recurse -Force

# Build the installers
Write-Host "Building installers..." -ForegroundColor Cyan
cd guardian-ui
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Installer build failed!" -ForegroundColor Red
    exit 1
}

cd ..

# Check if installers were created
$nsisInstaller = "guardian-ui\src-tauri\target\release\bundle\nsis\Guardian_1.0.0_x64-setup.exe"
$msiInstaller = "guardian-ui\src-tauri\target\release\bundle\msi\Guardian_1.0.0_x64_en-US.msi"

if (Test-Path $nsisInstaller) {
    Write-Host "✅ NSIS installer created: $nsisInstaller" -ForegroundColor Green
} else {
    Write-Host "❌ NSIS installer not found" -ForegroundColor Red
}

if (Test-Path $msiInstaller) {
    Write-Host "✅ MSI installer created: $msiInstaller" -ForegroundColor Green
} else {
    Write-Host "❌ MSI installer not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "Installation Features:" -ForegroundColor Yellow
Write-Host "  - Desktop shortcut points to launcher (starts with backend)" -ForegroundColor White
Write-Host "  - Start menu shortcuts include launcher options" -ForegroundColor White
Write-Host "  - Backend services automatically bundled" -ForegroundColor White
Write-Host "  - Configuration files included" -ForegroundColor White
Write-Host "  - Data directory created on installation" -ForegroundColor White
Write-Host ""
Write-Host "After installation, users can:" -ForegroundColor Cyan
Write-Host "  - Double-click desktop shortcut to start Guardian with backend" -ForegroundColor White
Write-Host "  - Use Start Menu shortcuts for different launch options" -ForegroundColor White
Write-Host "  - Server creation will work immediately" -ForegroundColor White
