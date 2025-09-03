# Guardian Platform Development Guide

This guide provides comprehensive instructions for setting up, building, and developing the Guardian Platform for modded Minecraft server hosting.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Project Structure](#project-structure)
4. [Development Setup](#development-setup)
5. [Building the Platform](#building-the-platform)
6. [Running the Platform](#running-the-platform)
7. [Development Workflow](#development-workflow)
8. [Testing](#testing)
9. [Contributing](#contributing)

## Overview

Guardian is a high-performance, self-healing hosting platform designed specifically for modded Minecraft servers. It provides:

- **Non-destructive crash prevention** through entity/block entity freezing
- **GPU-accelerated world generation** using WGSL shaders
- **Intelligent mod compatibility management** via runtime patching
- **High availability** with automatic restarts and snapshots
- **Comprehensive monitoring** with Prometheus and Grafana

## Prerequisites

### Required Software

- **Java 21+** - For Minecraft server and Guardian Agent
- **Rust 1.75+** - For GPU Worker and Host Daemon
- **Docker & Docker Compose** - For containerized deployment
- **Git** - For version control

### Optional Software

- **Gradle** - For Java builds (or use included wrapper)
- **Node.js** - For web dashboard development
- **Vulkan SDK** - For GPU development and testing

### Hardware Requirements

- **CPU**: Multi-core processor (8+ cores recommended)
- **RAM**: 16GB+ (32GB+ for large modpacks)
- **GPU**: Vulkan-compatible GPU for acceleration
- **Storage**: SSD recommended for world data

## Project Structure

```
guardian/
├── guardian-agent/          # Java/Kotlin agent with NeoForge/Forge integration
│   ├── src/main/java/       # Java source code
│   ├── src/main/resources/  # Mixin configs and mod metadata
│   └── build.gradle.kts     # Gradle build configuration
├── gpu-worker/              # Rust sidecar using wgpu for GPU acceleration
│   ├── src/                 # Rust source code
│   ├── src/kernels/         # WGSL shaders and GPU kernels
│   └── Cargo.toml           # Rust dependencies
├── hostd/                   # Rust watchdog daemon for process supervision
│   ├── src/                 # Rust source code
│   └── Cargo.toml           # Rust dependencies
├── configs/                 # Configuration templates and example rules
│   ├── server.yaml          # Server configuration
│   └── rules.yaml           # Compatibility rules
├── monitoring/              # Prometheus and Grafana configurations
│   ├── prometheus.yml       # Prometheus configuration
│   └── grafana/             # Grafana dashboards and datasources
├── scripts/                 # Build and deployment scripts
│   └── build.sh             # Main build script
├── docker-compose.yml       # Container orchestration
└── README.md                # Project documentation
```

## Development Setup

### 1. Clone the Repository

```bash
git clone <repository-url>
cd guardian
```

### 2. Install Dependencies

#### Java Development
```bash
# Install Java 21+ (Ubuntu/Debian)
sudo apt update
sudo apt install openjdk-21-jdk

# Install Java 21+ (macOS)
brew install openjdk@21

# Install Java 21+ (Windows)
# Download from https://adoptium.net/
```

#### Rust Development
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install additional Rust components
rustup component add rustfmt clippy
```

#### Docker
```bash
# Install Docker (Ubuntu/Debian)
sudo apt update
sudo apt install docker.io docker-compose

# Install Docker (macOS)
brew install docker docker-compose

# Install Docker (Windows)
# Download Docker Desktop from https://www.docker.com/
```

### 3. Verify Installation

```bash
# Check Java version
java -version

# Check Rust version
rustc --version

# Check Docker version
docker --version
docker-compose --version
```

## Building the Platform

### Quick Build

Use the provided build script for a complete build:

```bash
# Make script executable (Linux/macOS)
chmod +x scripts/build.sh

# Run build script
./scripts/build.sh
```

### Manual Build

#### 1. Build Guardian Agent (Java)

```bash
cd guardian-agent

# Using Gradle wrapper
./gradlew build

# Or using system Gradle
gradle build

# Copy JAR to project root
cp build/libs/guardian-agent-*.jar ../guardian-agent.jar
```

#### 2. Build GPU Worker (Rust)

```bash
cd gpu-worker

# Build release version
cargo build --release

# Copy binaries to project root
cp target/release/libgpu_worker.so ../libgpu_worker.so
cp target/release/gpu-worker ../gpu-worker
```

#### 3. Build Host Daemon (Rust)

```bash
cd hostd

# Build release version
cargo build --release

# Copy binary to project root
cp target/release/hostd ../hostd
```

#### 4. Build Docker Images

```bash
# Build all images
docker build -t guardian-gpu-worker ./gpu-worker
docker build -t guardian-hostd ./hostd
docker build -t guardian-minecraft ./docker
```

## Running the Platform

### Using Docker Compose (Recommended)

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

### Manual Deployment

#### 1. Start GPU Worker

```bash
# Set environment variables
export RUST_LOG=info
export GPU_WORKER_IPC=shm

# Run GPU worker
./gpu-worker
```

#### 2. Start Host Daemon

```bash
# Set environment variables
export RUST_LOG=info
export HOSTD_CONFIG=configs/hostd.yaml

# Run host daemon
./hostd --config configs/hostd.yaml --daemon
```

#### 3. Start Minecraft Server

```bash
# Set environment variables
export GUARDIAN_CONFIG=configs/server.yaml
export GUARDIAN_RULES=configs/rules.yaml

# Run Minecraft server with Guardian Agent
java -javaagent:guardian-agent.jar \
     -Dguardian.config.file=configs/server.yaml \
     -Dguardian.rules.file=configs/rules.yaml \
     -jar server.jar nogui
```

### Accessing Services

- **Web Dashboard**: http://localhost:8080
- **Prometheus Metrics**: http://localhost:9090
- **Grafana Dashboard**: http://localhost:3000 (admin/admin)
- **Minecraft Server**: localhost:25565

## Development Workflow

### 1. Setting Up Development Environment

```bash
# Install development dependencies
cd guardian-agent
./gradlew build

cd ../gpu-worker
cargo build

cd ../hostd
cargo build
```

### 2. Running Tests

#### Java Tests
```bash
cd guardian-agent
./gradlew test
```

#### Rust Tests
```bash
cd gpu-worker
cargo test

cd ../hostd
cargo test
```

### 3. Code Quality

#### Java Code Quality
```bash
cd guardian-agent
./gradlew checkstyleMain
./gradlew spotbugsMain
```

#### Rust Code Quality
```bash
cd gpu-worker
cargo clippy
cargo fmt

cd ../hostd
cargo clippy
cargo fmt
```

### 4. Debugging

#### Java Debugging
```bash
# Enable debug logging
export GUARDIAN_LOG_LEVEL=DEBUG

# Attach debugger
java -agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=5005 \
     -javaagent:guardian-agent.jar \
     -jar server.jar nogui
```

#### Rust Debugging
```bash
# Enable debug logging
export RUST_LOG=debug

# Run with debug symbols
cargo run --bin gpu-worker
cargo run --bin hostd
```

## Testing

### Unit Tests

```bash
# Run all tests
./scripts/test.sh
```

### Integration Tests

```bash
# Start test environment
docker-compose -f docker-compose.test.yml up -d

# Run integration tests
./scripts/integration-test.sh

# Cleanup
docker-compose -f docker-compose.test.yml down
```

### Performance Tests

```bash
# Run performance benchmarks
./scripts/benchmark.sh
```

## Contributing

### 1. Fork and Clone

```bash
# Fork the repository on GitHub
git clone https://github.com/your-username/guardian.git
cd guardian
```

### 2. Create Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 4. Test Changes

```bash
# Run all tests
./scripts/test.sh

# Run specific component tests
cd guardian-agent && ./gradlew test
cd gpu-worker && cargo test
cd hostd && cargo test
```

### 5. Submit Pull Request

```bash
# Commit changes
git add .
git commit -m "Add your feature description"

# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

### Code Style Guidelines

#### Java
- Follow Google Java Style Guide
- Use meaningful variable and method names
- Add Javadoc for public APIs
- Keep methods under 50 lines when possible

#### Rust
- Follow Rust API Guidelines
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add documentation comments for public items

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build process or auxiliary tool changes

## Troubleshooting

### Common Issues

#### Java Issues
```bash
# Check Java version
java -version

# Clear Gradle cache
cd guardian-agent
./gradlew clean
```

#### Rust Issues
```bash
# Update Rust
rustup update

# Clean build cache
cargo clean
```

#### Docker Issues
```bash
# Check Docker status
docker info

# Restart Docker service
sudo systemctl restart docker
```

#### GPU Issues
```bash
# Check Vulkan support
vulkaninfo

# Check GPU drivers
nvidia-smi  # For NVIDIA
lspci | grep VGA  # For general GPU info
```

### Getting Help

- Check the [Issues](https://github.com/your-org/guardian/issues) page
- Join our [Discord](https://discord.gg/guardian) server
- Read the [Wiki](https://github.com/your-org/guardian/wiki)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
