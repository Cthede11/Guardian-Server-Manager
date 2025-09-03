# Simple Guardian Platform Test Script
param([string]$Command = "help")

function Write-Info { param([string]$msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success { param([string]$msg) Write-Host "[SUCCESS] $msg" -ForegroundColor Green }
function Write-Warning { param([string]$msg) Write-Host "[WARNING] $msg" -ForegroundColor Yellow }
function Write-Error { param([string]$msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }

function Test-Docker {
    Write-Info "Checking Docker..."
    try {
        docker info | Out-Null
        Write-Success "Docker is running"
        return $true
    }
    catch {
        Write-Error "Docker is not running. Please start Docker Desktop."
        return $false
    }
}

function Start-Platform {
    Write-Info "Starting Guardian Platform..."
    if (-not (Test-Path "docker-compose.yml")) {
        Write-Error "docker-compose.yml not found. Run from project root."
        return $false
    }
    
    try {
        docker-compose up -d
        Write-Success "Platform started!"
        return $true
    }
    catch {
        Write-Error "Failed to start platform"
        return $false
    }
}

function Wait-ForWeb {
    Write-Info "Waiting for web interface..."
    $attempts = 0
    do {
        $attempts++
        try {
            Invoke-WebRequest -Uri "http://localhost:8080" -TimeoutSec 5 | Out-Null
            Write-Success "Web interface is ready at http://localhost:8080"
            return $true
        }
        catch {
            if ($attempts -ge 15) {
                Write-Error "Web interface failed to start"
                return $false
            }
            Start-Sleep -Seconds 2
        }
    } while ($attempts -lt 15)
    return $false
}

function Show-Status {
    Write-Info "Platform Status:"
    docker-compose ps
    try {
        Invoke-WebRequest -Uri "http://localhost:8080" -TimeoutSec 5 | Out-Null
        Write-Success "Web interface accessible at http://localhost:8080"
    }
    catch {
        Write-Warning "Web interface not accessible"
    }
}

function Stop-Platform {
    Write-Info "Stopping platform..."
    docker-compose down
    Write-Success "Platform stopped"
}

function Show-Help {
    Write-Host "Guardian Platform Test Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\scripts\simple-test.ps1 [COMMAND]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  start    - Start the platform"
    Write-Host "  stop     - Stop the platform"
    Write-Host "  status   - Show status"
    Write-Host "  help     - Show this help"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\scripts\simple-test.ps1 start"
    Write-Host "  .\scripts\simple-test.ps1 status"
}

# Main logic
switch ($Command.ToLower()) {
    "start" {
        if (Test-Docker) {
            Start-Platform
            Wait-ForWeb
            Show-Status
        }
    }
    "stop" { Stop-Platform }
    "status" { Show-Status }
    "help" { Show-Help }
    default { Show-Help }
}
