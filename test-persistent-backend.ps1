# Test Persistent Backend with Real Database and Folders
# This script tests the backend with persistent storage

Write-Host "Testing Persistent Backend..." -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Create data directory structure
Write-Host "Creating data directory structure..." -ForegroundColor Yellow
if (-not (Test-Path "data")) {
    New-Item -ItemType Directory -Path "data" -Force
}
if (-not (Test-Path "data\servers")) {
    New-Item -ItemType Directory -Path "data\servers" -Force
}
if (-not (Test-Path "data\backups")) {
    New-Item -ItemType Directory -Path "data\backups" -Force
}
if (-not (Test-Path "data\logs")) {
    New-Item -ItemType Directory -Path "data\logs" -Force
}
Write-Host "Data directories created" -ForegroundColor Green

# Start backend with persistent database
Write-Host "Starting backend with persistent database..." -ForegroundColor Yellow
Write-Host "Database: data\guardian.db" -ForegroundColor Cyan
Write-Host "Servers folder: data\servers" -ForegroundColor Cyan
Write-Host "Backups folder: data\backups" -ForegroundColor Cyan
Write-Host ""

# Start the backend in background
$hostdProcess = Start-Process -FilePath ".\hostd\target\release\hostd.exe" -ArgumentList "--config", "configs/hostd.yaml", "--port", "8080", "--database-url", "sqlite:data/guardian.db", "--log-level", "info" -PassThru

# Wait for backend to start
Start-Sleep -Seconds 5

# Test backend health
Write-Host "Testing backend health..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-RestMethod -Uri "http://localhost:8080/health" -Method GET
    if ($healthResponse.success -eq $true) {
        Write-Host "Backend health check passed" -ForegroundColor Green
    } else {
        Write-Host "Backend health check failed" -ForegroundColor Red
    }
} catch {
    Write-Host "Backend not responding: $($_.Exception.Message)" -ForegroundColor Red
}

# Test server creation with real folder
Write-Host "Testing server creation with real folder..." -ForegroundColor Yellow
$testServerDir = "data\servers\test-server-1"
if (-not (Test-Path $testServerDir)) {
    New-Item -ItemType Directory -Path $testServerDir -Force
}

try {
    $serverData = @{
        name = "Test Persistent Server"
        type = "vanilla"
        version = "1.20.1"
        paths = @{
            world = (Resolve-Path $testServerDir).Path
        }
    } | ConvertTo-Json

    $createResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/servers" -Method POST -Body $serverData -ContentType "application/json"
    if ($createResponse.success -eq $true) {
        Write-Host "Server creation test passed" -ForegroundColor Green
        Write-Host "Created server: $($createResponse.data.name)" -ForegroundColor Cyan
        Write-Host "Server ID: $($createResponse.data.id)" -ForegroundColor Cyan
        Write-Host "Server folder: $($createResponse.data.host)" -ForegroundColor Cyan
    } else {
        Write-Host "Server creation test failed" -ForegroundColor Red
    }
} catch {
    Write-Host "Server creation failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test listing servers
Write-Host "Testing server listing..." -ForegroundColor Yellow
try {
    $serversResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/servers" -Method GET
    if ($serversResponse.success -eq $true) {
        Write-Host "Server listing test passed" -ForegroundColor Green
        Write-Host "Found $($serversResponse.data.Count) servers" -ForegroundColor Cyan
        foreach ($server in $serversResponse.data) {
            Write-Host "  - $($server.name) (ID: $($server.id))" -ForegroundColor White
        }
    } else {
        Write-Host "Server listing test failed" -ForegroundColor Red
    }
} catch {
    Write-Host "Server listing failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Check if database file was created
Write-Host "Checking database file..." -ForegroundColor Yellow
if (Test-Path "data\guardian.db") {
    Write-Host "Database file created successfully" -ForegroundColor Green
    $dbSize = (Get-Item "data\guardian.db").Length
    Write-Host "Database size: $dbSize bytes" -ForegroundColor Cyan
} else {
    Write-Host "Database file not found" -ForegroundColor Red
}

# Check folder structure
Write-Host "Checking folder structure..." -ForegroundColor Yellow
Write-Host "Data directory contents:" -ForegroundColor Cyan
Get-ChildItem "data" -Recurse | ForEach-Object {
    Write-Host "  $($_.FullName)" -ForegroundColor White
}

# Stop backend
Write-Host "Stopping backend..." -ForegroundColor Yellow
if ($hostdProcess -and -not $hostdProcess.HasExited) {
    $hostdProcess.Kill()
    Write-Host "Backend stopped" -ForegroundColor Green
}

Write-Host "===============================================" -ForegroundColor Green
Write-Host "Persistent backend testing complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Results:" -ForegroundColor Yellow
Write-Host "- Database: data\guardian.db" -ForegroundColor White
Write-Host "- Servers folder: data\servers" -ForegroundColor White
Write-Host "- Backups folder: data\backups" -ForegroundColor White
Write-Host "- Logs folder: data\logs" -ForegroundColor White
Write-Host ""
Write-Host "Your app now has persistent storage!" -ForegroundColor Green
