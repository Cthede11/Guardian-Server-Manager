# Guardian Platform Test Script for PowerShell
# This script helps you test the Guardian Platform

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

# Colors for output
$Colors = @{
    Info = "Cyan"
    Success = "Green"
    Warning = "Yellow"
    Error = "Red"
}

function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Colors.Info
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Colors.Success
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Colors.Warning
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Colors.Error
}

function Test-Docker {
    Write-Status "Checking Docker..."
    try {
        docker info | Out-Null
        Write-Success "Docker is running"
        return $true
    }
    catch {
        Write-Error "Docker is not running. Please start Docker Desktop and try again."
        return $false
    }
}

function Test-DockerCompose {
    Write-Status "Checking Docker Compose..."
    try {
        docker-compose --version | Out-Null
        Write-Success "Docker Compose is available"
        return $true
    }
    catch {
        Write-Error "Docker Compose is not installed. Please install Docker Compose and try again."
        return $false
    }
}

function Test-Ports {
    Write-Status "Checking port availability..."
    
    try {
        $port8080 = Get-NetTCPConnection -LocalPort 8080 -ErrorAction SilentlyContinue
        if ($port8080) {
            Write-Warning "Port 8080 is already in use. The web interface might not start properly."
        } else {
            Write-Success "Port 8080 is available"
        }
    }
    catch {
        Write-Warning "Could not check port 8080 status"
    }
    
    try {
        $port25565 = Get-NetTCPConnection -LocalPort 25565 -ErrorAction SilentlyContinue
        if ($port25565) {
            Write-Warning "Port 25565 is already in use. Minecraft servers might not start properly."
        } else {
            Write-Success "Port 25565 is available"
        }
    }
    catch {
        Write-Warning "Could not check port 25565 status"
    }
}

function Start-Platform {
    Write-Status "Starting Guardian Platform..."
    
    if (-not (Test-Path "docker-compose.yml")) {
        Write-Error "docker-compose.yml not found. Please run from the project root directory."
        return $false
    }
    
    try {
        docker-compose up -d
        Write-Success "Platform started successfully!"
        return $true
    }
    catch {
        Write-Error "Failed to start the platform"
        return $false
    }
}

function Wait-ForService {
    param([string]$Url = "http://localhost:8080")
    
    Write-Status "Waiting for web interface to be ready..."
    $attempts = 0
    $maxAttempts = 30
    
    do {
        $attempts++
        try {
            $response = Invoke-WebRequest -Uri $Url -TimeoutSec 5 -ErrorAction Stop
            Write-Success "Web interface is ready!"
            return $true
        }
        catch {
            if ($attempts -ge $maxAttempts) {
                Write-Error "Web interface failed to start within expected time"
                return $false
            }
            Start-Sleep -Seconds 2
        }
    } while ($attempts -lt $maxAttempts)
    
    return $false
}

function Test-WebInterface {
    Write-Status "Testing web interface..."
    
    $endpoints = @(
        "http://localhost:8080",
        "http://localhost:8080/server_management.html",
        "http://localhost:8080/performance.html",
        "http://localhost:8080/backup.html",
        "http://localhost:8080/deployment.html",
        "http://localhost:8080/plugins.html",
        "http://localhost:8080/users.html",
        "http://localhost:8080/settings.html"
    )
    
    foreach ($endpoint in $endpoints) {
        try {
            $response = Invoke-WebRequest -Uri $endpoint -TimeoutSec 5 -ErrorAction Stop
            Write-Success "✓ $endpoint is accessible"
        }
        catch {
            Write-Error "✗ $endpoint is not accessible"
        }
    }
}

function Show-Status {
    Write-Status "Platform Status:"
    
    try {
        docker-compose ps
    }
    catch {
        Write-Warning "No Docker containers are running"
    }
    
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080" -TimeoutSec 5 -ErrorAction Stop
        Write-Success "Web interface is accessible at http://localhost:8080"
    }
    catch {
        Write-Warning "Web interface is not accessible"
    }
}

function Stop-Platform {
    Write-Status "Stopping Guardian Platform..."
    
    try {
        docker-compose down
        Write-Success "Platform stopped successfully!"
    }
    catch {
        Write-Error "Failed to stop the platform"
    }
}

function Show-Logs {
    Write-Status "Showing platform logs..."
    docker-compose logs --tail=50
}

function Invoke-Cleanup {
    Write-Status "Cleaning up..."
    docker-compose down -v
    docker system prune -f
    Write-Success "Cleanup completed!"
}

function Show-Help {
    Write-Host "Guardian Platform Test Script for PowerShell" -ForegroundColor $Colors.Info
    Write-Host ""
    Write-Host "Usage: .\scripts\test-fixed.ps1 [COMMAND]" -ForegroundColor $Colors.Info
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor $Colors.Info
    Write-Host "  start       Start the Guardian Platform" -ForegroundColor $Colors.Info
    Write-Host "  stop        Stop the Guardian Platform" -ForegroundColor $Colors.Info
    Write-Host "  test        Run all tests" -ForegroundColor $Colors.Info
    Write-Host "  web         Test web interface only" -ForegroundColor $Colors.Info
    Write-Host "  status      Show platform status" -ForegroundColor $Colors.Info
    Write-Host "  logs        Show platform logs" -ForegroundColor $Colors.Info
    Write-Host "  cleanup     Stop platform and clean up" -ForegroundColor $Colors.Info
    Write-Host "  help        Show this help message" -ForegroundColor $Colors.Info
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor $Colors.Info
    Write-Host "  .\scripts\test-fixed.ps1 start    # Start the platform" -ForegroundColor $Colors.Info
    Write-Host "  .\scripts\test-fixed.ps1 test     # Run all tests" -ForegroundColor $Colors.Info
    Write-Host "  .\scripts\test-fixed.ps1 status   # Check if platform is running" -ForegroundColor $Colors.Info
}

function Invoke-AllTests {
    if (-not (Test-Docker)) { return }
    if (-not (Test-DockerCompose)) { return }
    Test-Ports
    if (Start-Platform) {
        if (Wait-ForService) {
            Test-WebInterface
            Show-Status
        }
    }
}

# Main script logic
switch ($Command.ToLower()) {
    "start" {
        if (Test-Docker -and Test-DockerCompose) {
            Test-Ports
            Start-Platform
            Wait-ForService
            Show-Status
        }
    }
    "stop" { Stop-Platform }
    "test" { Invoke-AllTests }
    "web" { Test-WebInterface }
    "status" { Show-Status }
    "logs" { Show-Logs }
    "cleanup" { Invoke-Cleanup }
    "help" { Show-Help }
    default { Show-Help }
}
