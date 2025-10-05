#!/bin/bash

# Guardian Server Manager Production Deployment Script
# This script handles the complete deployment process for production

set -euo pipefail

# Configuration
APP_NAME="guardian-server-manager"
APP_VERSION="${1:-latest}"
DOCKER_IMAGE="guardian-server-manager:${APP_VERSION}"
COMPOSE_FILE="docker-compose.production.yml"
BACKUP_DIR="/opt/guardian/backups"
LOG_DIR="/opt/guardian/logs"
DATA_DIR="/opt/guardian/data"
CONFIG_DIR="/opt/guardian/configs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Error handling
error_exit() {
    log_error "$1"
    exit 1
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        log_error "This script should not be run as root for security reasons"
        exit 1
    fi
}

# Check system requirements
check_requirements() {
    log_info "Checking system requirements..."
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        error_exit "Docker is not installed. Please install Docker first."
    fi
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        error_exit "Docker Compose is not installed. Please install Docker Compose first."
    fi
    
    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        error_exit "curl is not installed. Please install curl first."
    fi
    
    # Check available disk space (at least 10GB)
    available_space=$(df / | awk 'NR==2 {print $4}')
    if [ "$available_space" -lt 10485760 ]; then
        error_exit "Insufficient disk space. At least 10GB is required."
    fi
    
    log_success "System requirements check passed"
}

# Create necessary directories
create_directories() {
    log_info "Creating necessary directories..."
    
    sudo mkdir -p "$BACKUP_DIR" "$LOG_DIR" "$DATA_DIR" "$CONFIG_DIR"
    sudo chown -R $(whoami):$(whoami) "$BACKUP_DIR" "$LOG_DIR" "$DATA_DIR" "$CONFIG_DIR"
    
    log_success "Directories created successfully"
}

# Backup existing data
backup_existing_data() {
    if [ -d "$DATA_DIR" ] && [ "$(ls -A $DATA_DIR)" ]; then
        log_info "Backing up existing data..."
        
        backup_timestamp=$(date +"%Y%m%d_%H%M%S")
        backup_path="$BACKUP_DIR/backup_$backup_timestamp"
        
        sudo cp -r "$DATA_DIR" "$backup_path"
        log_success "Data backed up to $backup_path"
    fi
}

# Pull latest Docker image
pull_docker_image() {
    log_info "Pulling Docker image: $DOCKER_IMAGE"
    
    if ! docker pull "$DOCKER_IMAGE"; then
        error_exit "Failed to pull Docker image: $DOCKER_IMAGE"
    fi
    
    log_success "Docker image pulled successfully"
}

# Stop existing containers
stop_existing_containers() {
    log_info "Stopping existing containers..."
    
    if docker-compose -f "$COMPOSE_FILE" ps -q | grep -q .; then
        docker-compose -f "$COMPOSE_FILE" down
        log_success "Existing containers stopped"
    else
        log_info "No existing containers to stop"
    fi
}

# Deploy new containers
deploy_containers() {
    log_info "Deploying new containers..."
    
    # Set environment variables
    export JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 32)}"
    export GRAFANA_PASSWORD="${GRAFANA_PASSWORD:-$(openssl rand -base64 16)}"
    export REDIS_URL="redis://redis:6379"
    
    # Deploy with Docker Compose
    if ! docker-compose -f "$COMPOSE_FILE" up -d; then
        error_exit "Failed to deploy containers"
    fi
    
    log_success "Containers deployed successfully"
}

# Wait for services to be ready
wait_for_services() {
    log_info "Waiting for services to be ready..."
    
    # Wait for Guardian API
    max_attempts=30
    attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -f http://localhost:8080/health &> /dev/null; then
            log_success "Guardian API is ready"
            break
        fi
        
        attempt=$((attempt + 1))
        log_info "Waiting for Guardian API... (attempt $attempt/$max_attempts)"
        sleep 10
    done
    
    if [ $attempt -eq $max_attempts ]; then
        error_exit "Guardian API failed to start within expected time"
    fi
    
    # Wait for Redis
    max_attempts=20
    attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if docker exec guardian-redis redis-cli ping &> /dev/null; then
            log_success "Redis is ready"
            break
        fi
        
        attempt=$((attempt + 1))
        log_info "Waiting for Redis... (attempt $attempt/$max_attempts)"
        sleep 5
    done
    
    if [ $attempt -eq $max_attempts ]; then
        error_exit "Redis failed to start within expected time"
    fi
}

# Run health checks
run_health_checks() {
    log_info "Running health checks..."
    
    # Check Guardian API health
    if ! curl -f http://localhost:8080/health; then
        error_exit "Guardian API health check failed"
    fi
    
    # Check Redis health
    if ! docker exec guardian-redis redis-cli ping | grep -q PONG; then
        error_exit "Redis health check failed"
    fi
    
    # Check container status
    if ! docker-compose -f "$COMPOSE_FILE" ps | grep -q "Up"; then
        error_exit "Some containers are not running"
    fi
    
    log_success "All health checks passed"
}

# Setup monitoring
setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Wait for Prometheus to be ready
    max_attempts=20
    attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -f http://localhost:9090/-/healthy &> /dev/null; then
            log_success "Prometheus is ready"
            break
        fi
        
        attempt=$((attempt + 1))
        log_info "Waiting for Prometheus... (attempt $attempt/$max_attempts)"
        sleep 5
    done
    
    # Wait for Grafana to be ready
    max_attempts=20
    attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -f http://localhost:3000/api/health &> /dev/null; then
            log_success "Grafana is ready"
            break
        fi
        
        attempt=$((attempt + 1))
        log_info "Waiting for Grafana... (attempt $attempt/$max_attempts)"
        sleep 5
    done
    
    log_success "Monitoring setup completed"
}

# Display deployment information
display_deployment_info() {
    log_success "Deployment completed successfully!"
    echo
    echo "=== Deployment Information ==="
    echo "Application: $APP_NAME"
    echo "Version: $APP_VERSION"
    echo "Docker Image: $DOCKER_IMAGE"
    echo
    echo "=== Service URLs ==="
    echo "Guardian API: http://localhost:8080"
    echo "Guardian WebSocket: ws://localhost:8081"
    echo "Prometheus: http://localhost:9090"
    echo "Grafana: http://localhost:3000"
    echo
    echo "=== Default Credentials ==="
    echo "Guardian Admin: admin / admin123"
    echo "Grafana Admin: admin / $GRAFANA_PASSWORD"
    echo
    echo "=== Useful Commands ==="
    echo "View logs: docker-compose -f $COMPOSE_FILE logs -f"
    echo "Stop services: docker-compose -f $COMPOSE_FILE down"
    echo "Restart services: docker-compose -f $COMPOSE_FILE restart"
    echo "Update services: docker-compose -f $COMPOSE_FILE pull && docker-compose -f $COMPOSE_FILE up -d"
    echo
}

# Main deployment function
main() {
    log_info "Starting Guardian Server Manager production deployment..."
    log_info "Version: $APP_VERSION"
    echo
    
    check_root
    check_requirements
    create_directories
    backup_existing_data
    pull_docker_image
    stop_existing_containers
    deploy_containers
    wait_for_services
    run_health_checks
    setup_monitoring
    display_deployment_info
}

# Run main function
main "$@"
