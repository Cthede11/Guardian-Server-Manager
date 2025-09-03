#!/bin/bash

# Guardian Platform Test Script
# This script helps you test the Guardian Platform

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is in use
port_in_use() {
    lsof -i :$1 >/dev/null 2>&1
}

# Function to wait for a service to be ready
wait_for_service() {
    local url=$1
    local max_attempts=30
    local attempt=1
    
    print_status "Waiting for service at $url..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            print_success "Service is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "Service failed to start within expected time"
    return 1
}

# Function to run basic health checks
run_health_checks() {
    print_status "Running health checks..."
    
    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    
    # Check if Docker Compose is available
    if ! command_exists docker-compose; then
        print_error "Docker Compose is not installed. Please install Docker Compose and try again."
        exit 1
    fi
    
    # Check if required ports are available
    if port_in_use 8080; then
        print_warning "Port 8080 is already in use. The web interface might not start properly."
    fi
    
    if port_in_use 25565; then
        print_warning "Port 25565 is already in use. Minecraft servers might not start properly."
    fi
    
    print_success "Health checks completed"
}

# Function to start the platform
start_platform() {
    print_status "Starting Guardian Platform..."
    
    # Build if needed
    if [ ! -f "docker-compose.yml" ]; then
        print_error "docker-compose.yml not found. Please run from the project root directory."
        exit 1
    fi
    
    # Start services
    docker-compose up -d
    
    print_success "Platform started successfully!"
}

# Function to test web interface
test_web_interface() {
    print_status "Testing web interface..."
    
    # Wait for web server to be ready
    if wait_for_service "http://localhost:8080"; then
        print_success "Web interface is accessible at http://localhost:8080"
        
        # Test main endpoints
        local endpoints=(
            "http://localhost:8080"
            "http://localhost:8080/server_management.html"
            "http://localhost:8080/performance.html"
            "http://localhost:8080/backup.html"
            "http://localhost:8080/deployment.html"
            "http://localhost:8080/plugins.html"
            "http://localhost:8080/users.html"
            "http://localhost:8080/settings.html"
        )
        
        for endpoint in "${endpoints[@]}"; do
            if curl -s "$endpoint" >/dev/null 2>&1; then
                print_success "✓ $endpoint is accessible"
            else
                print_error "✗ $endpoint is not accessible"
            fi
        done
    else
        print_error "Web interface is not accessible"
        return 1
    fi
}

# Function to test API endpoints
test_api_endpoints() {
    print_status "Testing API endpoints..."
    
    local api_endpoints=(
        "http://localhost:8080/api/health"
        "http://localhost:8080/api/servers"
        "http://localhost:8080/api/users"
        "http://localhost:8080/api/plugins"
    )
    
    for endpoint in "${api_endpoints[@]}"; do
        if curl -s "$endpoint" >/dev/null 2>&1; then
            print_success "✓ $endpoint is accessible"
        else
            print_warning "✗ $endpoint is not accessible (this might be expected if not implemented yet)"
        fi
    done
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    # Test Rust components
    if [ -d "hostd" ]; then
        cd hostd
        if cargo test >/dev/null 2>&1; then
            print_success "✓ Rust unit tests passed"
        else
            print_warning "✗ Some Rust unit tests failed"
        fi
        cd ..
    fi
    
    # Test Java components
    if [ -d "guardian-agent" ]; then
        cd guardian-agent
        if ./gradlew test >/dev/null 2>&1; then
            print_success "✓ Java unit tests passed"
        else
            print_warning "✗ Some Java unit tests failed"
        fi
        cd ..
    fi
}

# Function to show platform status
show_status() {
    print_status "Platform Status:"
    
    # Check Docker containers
    if docker-compose ps | grep -q "Up"; then
        print_success "Docker containers are running:"
        docker-compose ps
    else
        print_warning "No Docker containers are running"
    fi
    
    # Check web interface
    if curl -s "http://localhost:8080" >/dev/null 2>&1; then
        print_success "Web interface is accessible at http://localhost:8080"
    else
        print_warning "Web interface is not accessible"
    fi
}

# Function to stop the platform
stop_platform() {
    print_status "Stopping Guardian Platform..."
    docker-compose down
    print_success "Platform stopped successfully!"
}

# Function to show logs
show_logs() {
    print_status "Showing platform logs..."
    docker-compose logs --tail=50
}

# Function to clean up
cleanup() {
    print_status "Cleaning up..."
    docker-compose down -v
    docker system prune -f
    print_success "Cleanup completed!"
}

# Function to show help
show_help() {
    echo "Guardian Platform Test Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start       Start the Guardian Platform"
    echo "  stop        Stop the Guardian Platform"
    echo "  test        Run all tests"
    echo "  web         Test web interface only"
    echo "  api         Test API endpoints only"
    echo "  unit        Run unit tests only"
    echo "  status      Show platform status"
    echo "  logs        Show platform logs"
    echo "  cleanup     Stop platform and clean up"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start    # Start the platform"
    echo "  $0 test     # Run all tests"
    echo "  $0 status   # Check if platform is running"
}

# Main script logic
case "${1:-help}" in
    "start")
        run_health_checks
        start_platform
        wait_for_service "http://localhost:8080"
        show_status
        ;;
    "stop")
        stop_platform
        ;;
    "test")
        run_health_checks
        start_platform
        test_web_interface
        test_api_endpoints
        run_unit_tests
        show_status
        ;;
    "web")
        test_web_interface
        ;;
    "api")
        test_api_endpoints
        ;;
    "unit")
        run_unit_tests
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs
        ;;
    "cleanup")
        cleanup
        ;;
    "help"|*)
        show_help
        ;;
esac
