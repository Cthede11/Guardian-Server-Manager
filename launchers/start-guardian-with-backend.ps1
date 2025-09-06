# Guardian with Backend Startup Script
Write-Host "Starting Guardian with Backend Services..." -ForegroundColor Green
Write-Host ""

# Check if database exists in project root, create if not
if (-not (Test-Path "data\guardian.db")) {
    Write-Host "Creating database..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path "data" -Force | Out-Null
    New-Item -ItemType File -Path "data\guardian.db" -Force | Out-Null
    Write-Host "Database created" -ForegroundColor Green
}

# Change to the release directory
$releaseDir = "guardian-ui\src-tauri\target\release"
Set-Location $releaseDir

# Start the backend service
Write-Host "Starting backend service (hostd)..." -ForegroundColor Cyan
$hostdPath = "..\..\..\hostd\target\release\hostd.exe"
$configPath = "..\..\..\configs\hostd.yaml"
$hostdProcess = Start-Process -FilePath $hostdPath -ArgumentList "--port", "8080", "--log-level", "info", "--config", $configPath -PassThru -WindowStyle Hidden

# Wait for the backend to start
Write-Host "Waiting for backend to initialize..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# Check if hostd is running
if ($hostdProcess -and !$hostdProcess.HasExited) {
    Write-Host "Backend service started successfully (PID: $($hostdProcess.Id))" -ForegroundColor Green
} else {
    Write-Host "Backend service failed to start" -ForegroundColor Red
    Write-Host "You can manually start hostd.exe to enable server management" -ForegroundColor Yellow
}

# Test API connection
Write-Host "Testing API connection..." -ForegroundColor Cyan
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/api/health" -Method GET -TimeoutSec 5
    Write-Host "API connection successful" -ForegroundColor Green
} catch {
    Write-Host "API connection failed: $($_.Exception.Message)" -ForegroundColor Yellow
    Write-Host "Backend may still be starting up..." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Starting Guardian application..." -ForegroundColor Green
Write-Host "Press Ctrl+C to stop both Guardian and the backend service" -ForegroundColor Gray
Write-Host ""

# Start the Guardian application
try {
    $guardianProcess = Start-Process -FilePath "guardian.exe" -Wait -PassThru
} finally {
    # Clean up - kill hostd when Guardian closes
    Write-Host ""
    Write-Host "Cleaning up backend service..." -ForegroundColor Yellow
    if ($hostdProcess -and !$hostdProcess.HasExited) {
        Stop-Process -Id $hostdProcess.Id -Force -ErrorAction SilentlyContinue
        Write-Host "Backend service stopped" -ForegroundColor Green
    }
    Write-Host "Guardian has been closed." -ForegroundColor Green
}