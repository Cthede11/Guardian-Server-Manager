# Start Backend with Persistent Database
# This script starts the backend with a real database file and proper folder structure

Write-Host "Starting Guardian Backend with Persistent Database..." -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Create data directory if it doesn't exist
if (-not (Test-Path "data")) {
    New-Item -ItemType Directory -Path "data" -Force
    Write-Host "Created data directory" -ForegroundColor Green
}

# Create servers directory for managed servers
if (-not (Test-Path "data\servers")) {
    New-Item -ItemType Directory -Path "data\servers" -Force
    Write-Host "Created servers directory" -ForegroundColor Green
}

# Create backups directory
if (-not (Test-Path "data\backups")) {
    New-Item -ItemType Directory -Path "data\backups" -Force
    Write-Host "Created backups directory" -ForegroundColor Green
}

# Create logs directory
if (-not (Test-Path "data\logs")) {
    New-Item -ItemType Directory -Path "data\logs" -Force
    Write-Host "Created logs directory" -ForegroundColor Green
}

# Start backend with persistent database
Write-Host "Starting hostd with persistent database..." -ForegroundColor Yellow
Write-Host "Database: data\guardian.db" -ForegroundColor Cyan
Write-Host "Servers folder: data\servers" -ForegroundColor Cyan
Write-Host "Backups folder: data\backups" -ForegroundColor Cyan
Write-Host ""

# Start the backend
.\hostd\target\release\hostd.exe --config "configs/hostd.yaml" --port 8080 --database-url "sqlite:data/guardian.db" --log-level info
