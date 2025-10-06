# Guardian Backend Startup Script
# This script ensures the database is initialized and the backend starts properly

Write-Host "Starting Guardian Backend..." -ForegroundColor Green

# Set the database URL
$env:DATABASE_URL = "sqlite:data/guardian.db"

# Create data directory if it doesn't exist
if (!(Test-Path "data")) {
    Write-Host "Creating data directory..." -ForegroundColor Yellow
    New-Item -Path "data" -ItemType Directory -Force | Out-Null
}

# Check if database exists, if not initialize it
if (!(Test-Path "data/guardian.db")) {
    Write-Host "Database not found. Initializing database..." -ForegroundColor Yellow
    Set-Location "hostd"
    cargo run --release --bin init_db
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to initialize database!" -ForegroundColor Red
        exit 1
    }
    Set-Location ".."
    Write-Host "Database initialized successfully!" -ForegroundColor Green
} else {
    Write-Host "Database already exists." -ForegroundColor Green
}

# Start the backend
Write-Host "Starting backend server..." -ForegroundColor Green
Set-Location "hostd"
cargo run --release --bin hostd
