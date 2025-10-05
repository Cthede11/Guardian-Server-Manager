# Guardian Server Manager Production Deployment Script for Windows
param(
    [switch]$Force,
    [switch]$SkipBuild
)

# Configuration
$APP_NAME = "guardian-server-manager"
$DOCKER_COMPOSE_FILE = "docker-compose.prod.yml"
$ENV_FILE = ".env.production"

# Colors for output
$RED = "Red"
$GREEN = "Green"
$YELLOW = "Yellow"

# Logging functions
function Log-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $GREEN
}

function Log-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor $YELLOW
}

function Log-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $RED
}

# Check if running as administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Log-Error "This script requires administrator privileges. Please run as administrator."
    exit 1
}

# Check if Docker is installed
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Log-Error "Docker is not installed. Please install Docker Desktop first."
    exit 1
}

# Check if Docker Compose is installed
if (-not (Get-Command docker-compose -ErrorAction SilentlyContinue)) {
    Log-Error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
}

# Create production environment file if it doesn't exist
if (-not (Test-Path $ENV_FILE)) {
    Log-Info "Creating production environment file..."
    
    # Generate random secrets
    $JWT_SECRET = [System.Convert]::ToBase64String([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
    $API_KEY = [System.Convert]::ToBase64String([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
    
    $envContent = @"
# Guardian Server Manager Production Environment
JWT_SECRET=$JWT_SECRET
API_KEY=$API_KEY
DATABASE_URL=sqlite:///data/guardian.db
RUST_LOG=info
VITE_API_URL=http://localhost
"@
    
    $envContent | Out-File -FilePath $ENV_FILE -Encoding UTF8
    Log-Info "Production environment file created at $ENV_FILE"
    Log-Warn "Please review and update the environment variables as needed"
}

# Load environment variables
Get-Content $ENV_FILE | ForEach-Object {
    if ($_ -match "^([^=]+)=(.*)$") {
        [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

# Create necessary directories
Log-Info "Creating necessary directories..."
$directories = @("data\servers", "data\backups", "configs", "ssl", "logs")
foreach ($dir in $directories) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}

# Build and start services
if (-not $SkipBuild) {
    Log-Info "Building and starting services..."
    docker-compose -f $DOCKER_COMPOSE_FILE --env-file $ENV_FILE up -d --build
} else {
    Log-Info "Starting services (skipping build)..."
    docker-compose -f $DOCKER_COMPOSE_FILE --env-file $ENV_FILE up -d
}

# Wait for services to be ready
Log-Info "Waiting for services to be ready..."
Start-Sleep -Seconds 30

# Check service health
Log-Info "Checking service health..."

# Check backend health
try {
    $response = Invoke-WebRequest -Uri "http://localhost:52100/healthz" -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Log-Info "Backend service is healthy"
    } else {
        Log-Error "Backend service returned status code: $($response.StatusCode)"
        exit 1
    }
} catch {
    Log-Error "Backend service is not responding: $($_.Exception.Message)"
    exit 1
}

# Check frontend health
try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000" -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Log-Info "Frontend service is healthy"
    } else {
        Log-Error "Frontend service returned status code: $($response.StatusCode)"
        exit 1
    }
} catch {
    Log-Error "Frontend service is not responding: $($_.Exception.Message)"
    exit 1
}

# Check nginx health
try {
    $response = Invoke-WebRequest -Uri "http://localhost" -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Log-Info "Nginx service is healthy"
    } else {
        Log-Error "Nginx service returned status code: $($response.StatusCode)"
        exit 1
    }
} catch {
    Log-Error "Nginx service is not responding: $($_.Exception.Message)"
    exit 1
}

# Show running services
Log-Info "Running services:"
docker-compose -f $DOCKER_COMPOSE_FILE ps

# Show logs
Log-Info "Recent logs:"
docker-compose -f $DOCKER_COMPOSE_FILE logs --tail=20

Log-Info "Deployment completed successfully!"
Log-Info "Guardian Server Manager is now running at:"
Log-Info "  - Frontend: http://localhost"
Log-Info "  - Backend API: http://localhost:52100"
Log-Info "  - Health Check: http://localhost/healthz"

Log-Info "To view logs: docker-compose -f $DOCKER_COMPOSE_FILE logs -f"
Log-Info "To stop services: docker-compose -f $DOCKER_COMPOSE_FILE down"
Log-Info "To restart services: docker-compose -f $DOCKER_COMPOSE_FILE restart"
