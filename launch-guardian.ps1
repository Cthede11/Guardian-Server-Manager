# Guardian Launcher Script
Write-Host "🚀 Launching Guardian Minecraft Server Manager..." -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "guardian-ui")) {
    Write-Host "❌ Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Try to run the Tauri application first
$guardianExe = "guardian-ui\src-tauri\target\release\guardian.exe"
if (Test-Path $guardianExe) {
    Write-Host "✅ Found Guardian executable, launching..." -ForegroundColor Green
    Start-Process -FilePath $guardianExe -WorkingDirectory "guardian-ui\src-tauri\target\release"
    Write-Host "🎯 Guardian launched successfully!" -ForegroundColor Green
    exit 0
}

# If Tauri app not found, try running backend and frontend separately
Write-Host "⚠️  Tauri app not found, trying alternative launch method..." -ForegroundColor Yellow

# Check if backend exists
$hostdExe = "hostd\target\release\hostd.exe"
if (Test-Path $hostdExe) {
    Write-Host "✅ Found backend executable" -ForegroundColor Green
    
    # Start backend in background
    Write-Host "🚀 Starting backend..." -ForegroundColor Yellow
    Start-Process -FilePath $hostdExe -ArgumentList "--config", "configs\hostd.yaml", "--port", "8080" -WorkingDirectory "hostd\target\release" -WindowStyle Minimized
    
    # Wait a moment for backend to start
    Start-Sleep -Seconds 3
    
    # Start frontend
    Write-Host "🎨 Starting frontend..." -ForegroundColor Yellow
    Set-Location "guardian-ui"
    npm run dev
    
    Write-Host "🎯 Guardian launched! Open your browser to http://localhost:8080" -ForegroundColor Green
} else {
    Write-Host "❌ Backend executable not found. Please build the application first." -ForegroundColor Red
    Write-Host "Run: .\build-simple.ps1" -ForegroundColor Yellow
    exit 1
}
