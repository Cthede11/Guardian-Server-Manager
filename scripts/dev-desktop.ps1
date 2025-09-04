# Development script for Guardian Desktop Application
# This script starts both the backend (hostd) and frontend (guardian-ui) in development mode

Write-Host "[DEV] Starting Guardian Desktop Application in Development Mode..." -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Start the backend (hostd) in a separate process
Write-Host "[BACKEND] Starting backend (hostd)..." -ForegroundColor Yellow
Set-Location "$ProjectRoot/hostd"

# Build the backend first
try {
    cargo build
    if ($LASTEXITCODE -ne 0) {
        throw "Backend build failed"
    }
    Write-Host "[SUCCESS] Backend built successfully" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Backend build failed: $_" -ForegroundColor Red
    exit 1
}

# Start the backend in the background
$BackendProcess = Start-Process -FilePath "cargo" -ArgumentList "run", "--", "--port", "8080", "--database-url", "sqlite:guardian.db" -PassThru -WindowStyle Hidden
Write-Host "[SUCCESS] Backend started (PID: $($BackendProcess.Id))" -ForegroundColor Green

# Wait a moment for the backend to start
Start-Sleep -Seconds 3

# Start the frontend in development mode
Write-Host "[FRONTEND] Starting frontend (guardian-ui)..." -ForegroundColor Yellow
Set-Location "$ProjectRoot/guardian-ui"

# Set environment variable to use real data instead of mock data
$env:VITE_USE_MOCK_DATA = "false"

try {
    # Start the Tauri development server
    npm run tauri:dev
} catch {
    Write-Host "[ERROR] Frontend development server failed: $_" -ForegroundColor Red
    # Clean up the backend process
    if ($BackendProcess -and !$BackendProcess.HasExited) {
        Stop-Process -Id $BackendProcess.Id -Force
        Write-Host "[CLEANUP] Backend process stopped" -ForegroundColor Yellow
    }
    exit 1
}

# Clean up function
function Cleanup {
    Write-Host "[CLEANUP] Cleaning up processes..." -ForegroundColor Yellow
    if ($BackendProcess -and !$BackendProcess.HasExited) {
        Stop-Process -Id $BackendProcess.Id -Force
        Write-Host "[SUCCESS] Backend process stopped" -ForegroundColor Green
    }
}

# Register cleanup function to run on script exit
Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action { Cleanup }

Write-Host "[COMPLETE] Guardian Desktop Application development environment started!" -ForegroundColor Green
Write-Host "[INFO] Frontend: http://localhost:5173" -ForegroundColor Blue
Write-Host "[INFO] Backend API: http://localhost:8080" -ForegroundColor Blue
Write-Host "[INFO] Desktop App: Tauri development window" -ForegroundColor Blue
Write-Host "Press Ctrl+C to stop all services" -ForegroundColor Yellow
