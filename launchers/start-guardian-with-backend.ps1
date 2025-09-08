Write-Host "Starting Guardian with Backend..." -ForegroundColor Green

# Get the directory where this script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Join-Path $ScriptDir ".."

# Start the backend first
Write-Host "Starting backend..." -ForegroundColor Yellow
$BackendPath = Join-Path $ProjectRoot "build\executables\hostd.exe"
Start-Process -FilePath $BackendPath -WindowStyle Hidden

# Wait a moment for backend to start
Start-Sleep -Seconds 3

# Check if backend is running
Write-Host "Checking backend status..." -ForegroundColor Yellow
try {
    $Response = Invoke-WebRequest -Uri "http://localhost:8080/health" -Method GET -TimeoutSec 5
    if ($Response.StatusCode -eq 200) {
        Write-Host "Backend is running successfully" -ForegroundColor Green
    } else {
        Write-Host "Warning: Backend responded with status $($Response.StatusCode)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "Warning: Backend may not be running properly - $($_.Exception.Message)" -ForegroundColor Red
}

# Start the Guardian UI
Write-Host "Starting Guardian UI..." -ForegroundColor Yellow
Set-Location (Join-Path $ProjectRoot "guardian-ui")
npm run tauri dev