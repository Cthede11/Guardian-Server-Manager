# Build Custom Guardian Installer
Write-Host "Building Custom Guardian Installer..." -ForegroundColor Green

# Check if NSIS is installed
$nsisPath = Get-Command "makensis.exe" -ErrorAction SilentlyContinue
if (-not $nsisPath) {
    Write-Host "NSIS not found in PATH. Please install NSIS and add it to your PATH." -ForegroundColor Red
    Write-Host "Download from: https://nsis.sourceforge.io/Download" -ForegroundColor Yellow
    exit 1
}

# Build the project first
Write-Host "Building Guardian project..." -ForegroundColor Cyan
cd guardian-ui
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Frontend build failed!" -ForegroundColor Red
    exit 1
}

# Build Tauri app
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri build failed!" -ForegroundColor Red
    exit 1
}

cd ..

# Build backend services
Write-Host "Building backend services..." -ForegroundColor Cyan
cd hostd
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Hostd build failed!" -ForegroundColor Red
    exit 1
}

cd ..
cd gpu-worker
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "GPU worker build failed!" -ForegroundColor Red
    exit 1
}

cd ..

# Compile the NSIS installer
Write-Host "Compiling NSIS installer..." -ForegroundColor Cyan
makensis guardian-custom-installer.nsi

if ($LASTEXITCODE -eq 0) {
    Write-Host "Custom installer created successfully!" -ForegroundColor Green
    Write-Host "Installer location: Guardian_Setup.exe" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "This installer will:" -ForegroundColor Yellow
    Write-Host "  - Install Guardian with backend services" -ForegroundColor White
    Write-Host "  - Create desktop shortcut to Launch-Guardian.bat" -ForegroundColor White
    Write-Host "  - Create start menu shortcut to Launch-Guardian.bat" -ForegroundColor White
    Write-Host "  - Set up proper uninstaller" -ForegroundColor White
} else {
    Write-Host "NSIS compilation failed!" -ForegroundColor Red
    exit 1
}
