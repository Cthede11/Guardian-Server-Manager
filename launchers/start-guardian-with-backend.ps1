Write-Host "Starting Guardian with Backend..." -ForegroundColor Green

# Get the directory where this script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Join-Path $ScriptDir ".."

# Start the backend first
Write-Host "Starting backend..." -ForegroundColor Yellow
$BackendPath = Join-Path $ProjectRoot "build\executables\hostd.exe"

# Use this pattern instead of WindowStyle Hidden
$ProcessStartInfo = New-Object System.Diagnostics.ProcessStartInfo
$ProcessStartInfo.FileName = $BackendPath
$ProcessStartInfo.CreateNoWindow = $true
$ProcessStartInfo.UseShellExecute = $false
$ProcessStartInfo.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden

$BackendProcess = [System.Diagnostics.Process]::Start($ProcessStartInfo)

# Wait a moment for backend to start
Start-Sleep -Seconds 3

# Check if backend is running
Write-Host "Checking backend status..." -ForegroundColor Yellow
try {
    $Response = Invoke-WebRequest -Uri "http://127.0.0.1:52100/api/healthz" -Method GET -TimeoutSec 5
    if ($Response.StatusCode -eq 200) {
        Write-Host "Backend is running successfully" -ForegroundColor Green
    } else {
        Write-Host "Warning: Backend responded with status $($Response.StatusCode)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "Warning: Backend may not be running properly - $($_.Exception.Message)" -ForegroundColor Red
}

# Function to cleanup processes on exit
function Cleanup-Processes {
    Write-Host "Cleaning up processes..." -ForegroundColor Yellow
    if ($BackendProcess -and !$BackendProcess.HasExited) {
        Write-Host "Terminating backend process..." -ForegroundColor Yellow
        $BackendProcess.Kill()
        $BackendProcess.WaitForExit(5000)
    }
    Write-Host "Cleanup completed" -ForegroundColor Green
}

# Register cleanup on script exit
Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action { Cleanup-Processes }

# Start the Guardian UI
Write-Host "Starting Guardian UI..." -ForegroundColor Yellow
Set-Location (Join-Path $ProjectRoot "guardian-ui")

try {
    npm run tauri dev
} finally {
    # Ensure cleanup happens even if npm fails
    Cleanup-Processes
}