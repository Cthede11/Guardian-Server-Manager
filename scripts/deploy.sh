#!/bin/bash

# Guardian Server Manager Production Deployment Script
set -e

# Configuration
APP_NAME="guardian-server-manager"
DOCKER_COMPOSE_FILE="docker-compose.prod.yml"
ENV_FILE=".env.production"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   log_error "This script should not be run as root"
   exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    log_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    log_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Create production environment file if it doesn't exist
if [ ! -f "$ENV_FILE" ]; then
    log_info "Creating production environment file..."
    cat > "$ENV_FILE" << EOF
# Guardian Server Manager Production Environment
JWT_SECRET=$(openssl rand -base64 32)
API_KEY=$(openssl rand -base64 32)
DATABASE_URL=sqlite:///data/guardian.db
RUST_LOG=info
VITE_API_URL=http://localhost
EOF
    log_info "Production environment file created at $ENV_FILE"
    log_warn "Please review and update the environment variables as needed"
fi

# Load environment variables
source "$ENV_FILE"

# Create necessary directories
log_info "Creating necessary directories..."
mkdir -p data/servers
mkdir -p data/backups
mkdir -p configs
mkdir -p ssl
mkdir -p logs

# Set proper permissions
chmod 755 data
chmod 755 configs
chmod 755 ssl
chmod 755 logs

# Build and start services
log_info "Building and starting services..."
docker-compose -f "$DOCKER_COMPOSE_FILE" --env-file "$ENV_FILE" up -d --build

# Wait for services to be ready
log_info "Waiting for services to be ready..."
sleep 30

# Check service health
log_info "Checking service health..."

# Check backend health
if curl -f http://localhost:52100/healthz > /dev/null 2>&1; then
    log_info "Backend service is healthy"
else
    log_error "Backend service is not responding"
    exit 1
fi

# Check frontend health
if curl -f http://localhost:3000 > /dev/null 2>&1; then
    log_info "Frontend service is healthy"
else
    log_error "Frontend service is not responding"
    exit 1
fi

# Check nginx health
if curl -f http://localhost > /dev/null 2>&1; then
    log_info "Nginx service is healthy"
else
    log_error "Nginx service is not responding"
    exit 1
fi

# Show running services
log_info "Running services:"
docker-compose -f "$DOCKER_COMPOSE_FILE" ps

# Show logs
log_info "Recent logs:"
docker-compose -f "$DOCKER_COMPOSE_FILE" logs --tail=20

log_info "Deployment completed successfully!"
log_info "Guardian Server Manager is now running at:"
log_info "  - Frontend: http://localhost"
log_info "  - Backend API: http://localhost:52100"
log_info "  - Health Check: http://localhost/healthz"

log_info "To view logs: docker-compose -f $DOCKER_COMPOSE_FILE logs -f"
log_info "To stop services: docker-compose -f $DOCKER_COMPOSE_FILE down"
log_info "To restart services: docker-compose -f $DOCKER_COMPOSE_FILE restart"
