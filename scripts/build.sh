#!/bin/bash

# Guardian Build Script
set -e

echo "🛡️ Building Guardian Platform..."

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

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v java &> /dev/null; then
        print_error "Java is not installed. Please install Java 21 or later."
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Please install Rust."
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        print_warning "Docker is not installed. Container builds will be skipped."
    fi
    
    print_success "Dependencies check completed"
}

# Build Guardian Agent (Java)
build_guardian_agent() {
    print_status "Building Guardian Agent..."
    
    cd guardian-agent
    
    if [ ! -f "gradlew" ]; then
        print_error "Gradle wrapper not found. Please run 'gradle wrapper' first."
        exit 1
    fi
    
    ./gradlew build
    cp build/libs/guardian-agent-*.jar ../guardian-agent.jar
    
    cd ..
    print_success "Guardian Agent built successfully"
}

# Build GPU Worker (Rust)
build_gpu_worker() {
    print_status "Building GPU Worker..."
    
    cd gpu-worker
    cargo build --release
    cp target/release/libgpu_worker.so ../libgpu_worker.so
    cp target/release/gpu-worker ../gpu-worker
    
    cd ..
    print_success "GPU Worker built successfully"
}

# Build Host Daemon (Rust)
build_hostd() {
    print_status "Building Host Daemon..."
    
    cd hostd
    cargo build --release
    cp target/release/hostd ../hostd
    
    cd ..
    print_success "Host Daemon built successfully"
}

# Build Docker images
build_docker_images() {
    if ! command -v docker &> /dev/null; then
        print_warning "Docker not available, skipping container builds"
        return
    fi
    
    print_status "Building Docker images..."
    
    # Build GPU Worker image
    docker build -t guardian-gpu-worker ./gpu-worker
    
    # Build Host Daemon image
    docker build -t guardian-hostd ./hostd
    
    # Build Minecraft Server image
    docker build -t guardian-minecraft ./docker
    
    print_success "Docker images built successfully"
}

# Create distribution package
create_distribution() {
    print_status "Creating distribution package..."
    
    DIST_DIR="guardian-dist"
    rm -rf $DIST_DIR
    mkdir -p $DIST_DIR
    
    # Copy binaries
    cp guardian-agent.jar $DIST_DIR/
    cp gpu-worker $DIST_DIR/
    cp hostd $DIST_DIR/
    cp libgpu_worker.so $DIST_DIR/
    
    # Copy configuration files
    cp -r configs $DIST_DIR/
    cp docker-compose.yml $DIST_DIR/
    cp README.md $DIST_DIR/
    
    # Copy scripts
    mkdir -p $DIST_DIR/scripts
    cp scripts/*.sh $DIST_DIR/scripts/
    chmod +x $DIST_DIR/scripts/*.sh
    
    # Create startup script
    cat > $DIST_DIR/start.sh << 'EOF'
#!/bin/bash
echo "🛡️ Starting Guardian Platform..."
docker-compose up -d
echo "✅ Guardian Platform started successfully!"
echo "📊 Dashboard available at: http://localhost:8080"
echo "📈 Metrics available at: http://localhost:9090"
EOF
    chmod +x $DIST_DIR/start.sh
    
    # Create stop script
    cat > $DIST_DIR/stop.sh << 'EOF'
#!/bin/bash
echo "🛡️ Stopping Guardian Platform..."
docker-compose down
echo "✅ Guardian Platform stopped successfully!"
EOF
    chmod +x $DIST_DIR/stop.sh
    
    print_success "Distribution package created: $DIST_DIR"
}

# Main build process
main() {
    print_status "Starting Guardian Platform build process..."
    
    check_dependencies
    build_guardian_agent
    build_gpu_worker
    build_hostd
    build_docker_images
    create_distribution
    
    print_success "🎉 Guardian Platform build completed successfully!"
    print_status "Distribution package available in: guardian-dist/"
    print_status "To start the platform: cd guardian-dist && ./start.sh"
}

# Run main function
main "$@"
