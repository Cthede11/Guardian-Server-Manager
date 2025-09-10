# Test build script to diagnose Tauri build issues
Write-Host "=== GUARDIAN BUILD DIAGNOSTIC ===" -ForegroundColor Green

# Check if we're in the right directory
$CurrentDir = Get-Location
Write-Host "Current directory: $CurrentDir" -ForegroundColor Blue

# Check if key files exist
$KeyFiles = @(
    "package.json",
    "src-tauri/Cargo.toml",
    "src-tauri/tauri.conf.json",
    "dist/index.html"
)

Write-Host "Checking key files..." -ForegroundColor Yellow
foreach ($file in $KeyFiles) {
    if (Test-Path $file) {
        Write-Host "✓ Found: $file" -ForegroundColor Green
    } else {
        Write-Host "✗ Missing: $file" -ForegroundColor Red
    }
}

# Check if backend binaries exist
Write-Host "Checking backend binaries..." -ForegroundColor Yellow
$BackendFiles = @(
    "../hostd/target/release/hostd.exe",
    "../gpu-worker/target/release/gpu-worker.exe"
)

foreach ($file in $BackendFiles) {
    if (Test-Path $file) {
        Write-Host "✓ Found: $file" -ForegroundColor Green
    } else {
        Write-Host "✗ Missing: $file" -ForegroundColor Red
    }
}

# Check Node.js and npm
Write-Host "Checking prerequisites..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js not found" -ForegroundColor Red
}

try {
    $npmVersion = npm --version
    Write-Host "✓ npm: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ npm not found" -ForegroundColor Red
}

# Try to run the build command and capture output
Write-Host "Attempting to run Tauri build..." -ForegroundColor Yellow
try {
    $buildOutput = npm run tauri build 2>&1
    Write-Host "Build output:" -ForegroundColor Cyan
    Write-Host $buildOutput
} catch {
    Write-Host "Build failed with error: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Diagnostic completed!" -ForegroundColor Green