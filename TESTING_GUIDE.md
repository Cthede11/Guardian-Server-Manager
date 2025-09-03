# Guardian Platform Testing Guide

## 🚀 Quick Start Testing

### Prerequisites
- Docker and Docker Compose installed
- Git (to clone the repository)
- Web browser (Chrome, Firefox, Safari, or Edge)
- At least 4GB RAM available for Docker containers

### 1. Build and Start the Platform

```bash
# Clone the repository (if not already done)
git clone <your-repo-url>
cd modded-manager

# Make build script executable (Linux/Mac)
chmod +x scripts/build.sh

# Build all components
./scripts/build.sh

# Start the platform
docker-compose up -d
```

### 2. Access the Web Interface

Open your web browser and navigate to:
- **Main Dashboard**: http://localhost:8080
- **Server Management**: http://localhost:8080/server_management.html
- **Performance Monitoring**: http://localhost:8080/performance.html
- **Backup Management**: http://localhost:8080/backup.html
- **Deployment Management**: http://localhost:8080/deployment.html
- **Plugin Management**: http://localhost:8080/plugins.html
- **User Management**: http://localhost:8080/users.html
- **Settings**: http://localhost:8080/settings.html

### 3. Default Credentials
- **Username**: admin
- **Password**: admin123
- **API Key**: guardian-api-key-2024

## 🧪 Testing Scenarios

### Basic Functionality Tests

#### 1. Dashboard Testing
- [ ] Verify all metrics display correctly
- [ ] Check real-time updates (refresh every 30 seconds)
- [ ] Test navigation between different sections
- [ ] Verify responsive design on different screen sizes

#### 2. Server Management Testing
- [ ] Create a new server with different configurations
- [ ] Test start/stop/restart functionality
- [ ] Verify server status updates in real-time
- [ ] Test server deletion with confirmation
- [ ] Check resource allocation controls

#### 3. Performance Monitoring Testing
- [ ] Verify charts load and display data
- [ ] Test time range selection (1h, 6h, 24h, 7d, 30d)
- [ ] Check server-specific filtering
- [ ] Test real-time metric updates
- [ ] Verify performance recommendations

#### 4. Backup Management Testing
- [ ] Create different types of backups (Full, Incremental, Differential, Snapshot)
- [ ] Schedule automated backups
- [ ] Test backup restoration
- [ ] Verify backup progress tracking
- [ ] Check backup filtering and search

#### 5. Deployment Testing
- [ ] Test different deployment strategies
- [ ] Verify deployment progress tracking
- [ ] Test rollback functionality
- [ ] Check deployment logs
- [ ] Test deployment cancellation

#### 6. Plugin Management Testing
- [ ] Install plugins via different methods
- [ ] Enable/disable plugins
- [ ] Test plugin configuration
- [ ] Verify plugin status updates
- [ ] Test plugin uninstallation

#### 7. User Management Testing
- [ ] Create users with different roles
- [ ] Test user status changes
- [ ] Verify tenant management
- [ ] Check role-based permissions
- [ ] Test user search and filtering

#### 8. Settings Testing
- [ ] Test all configuration sections
- [ ] Verify form validation
- [ ] Test settings persistence
- [ ] Check settings reset functionality
- [ ] Verify real-time validation feedback

## 🔧 Development Testing

### Local Development Setup

```bash
# Start only the web server for UI testing
cd hostd
cargo run

# Or use Docker for the web server only
docker-compose up hostd
```

### API Testing

```bash
# Test API endpoints using curl
curl -X GET http://localhost:8080/api/health
curl -X GET http://localhost:8080/api/servers
curl -X POST http://localhost:8080/api/servers -H "Content-Type: application/json" -d '{"name":"Test Server","port":25565}'
```

### Unit Testing

```bash
# Run Rust unit tests
cd hostd
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with coverage
cargo test --coverage
```

### Java Agent Testing

```bash
# Build the Java agent
cd guardian-agent
./gradlew build

# Test with a Minecraft server
java -javaagent:build/libs/guardian-agent.jar -jar minecraft-server.jar
```

## 🐛 Troubleshooting

### Common Issues

#### 1. Port Conflicts
```bash
# Check if ports are in use
netstat -tulpn | grep :8080
netstat -tulpn | grep :25565

# Kill processes using ports
sudo kill -9 $(lsof -t -i:8080)
```

#### 2. Docker Issues
```bash
# Check Docker status
docker ps
docker-compose ps

# View logs
docker-compose logs hostd
docker-compose logs gpu-worker

# Restart services
docker-compose restart
```

#### 3. Permission Issues (Linux/Mac)
```bash
# Fix script permissions
chmod +x scripts/build.sh
chmod +x scripts/test.sh

# Fix Docker permissions
sudo usermod -aG docker $USER
```

#### 4. Memory Issues
```bash
# Check available memory
free -h

# Increase Docker memory limit
# In Docker Desktop: Settings > Resources > Memory
```

### Log Locations

- **Host Daemon**: `hostd/logs/hostd.log`
- **GPU Worker**: `gpu-worker/logs/gpu-worker.log`
- **Minecraft Server**: `minecraft/logs/latest.log`
- **Docker Logs**: `docker-compose logs`

## 📊 Performance Testing

### Load Testing

```bash
# Install k6 for load testing
curl https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz -L | tar xvz --strip-components 1

# Run load tests
k6 run tests/load-test.js
```

### Stress Testing

```bash
# Test with multiple concurrent users
ab -n 1000 -c 10 http://localhost:8080/api/servers

# Test API rate limiting
for i in {1..100}; do curl http://localhost:8080/api/health; done
```

## 🔒 Security Testing

### Authentication Testing
- [ ] Test login with valid credentials
- [ ] Test login with invalid credentials
- [ ] Test session timeout
- [ ] Test API key authentication
- [ ] Test role-based access control

### Input Validation Testing
- [ ] Test SQL injection attempts
- [ ] Test XSS attempts
- [ ] Test file upload security
- [ ] Test API parameter validation

## 📱 Mobile Testing

### Responsive Design Testing
- [ ] Test on mobile devices (iOS/Android)
- [ ] Test on tablets
- [ ] Test on different screen orientations
- [ ] Test touch interactions
- [ ] Test mobile-specific features

## 🚀 Production Testing

### Deployment Testing
```bash
# Test production deployment
docker-compose -f docker-compose.prod.yml up -d

# Test with SSL certificates
docker-compose -f docker-compose.ssl.yml up -d

# Test with load balancer
docker-compose -f docker-compose.lb.yml up -d
```

### Monitoring Testing
- [ ] Test Prometheus metrics collection
- [ ] Test Grafana dashboards
- [ ] Test alerting rules
- [ ] Test log aggregation
- [ ] Test health checks

## 📝 Test Data

### Sample Server Configurations
```yaml
# Test server configurations
test_servers:
  - name: "Test Vanilla Server"
    version: "1.20.1"
    mod_loader: "vanilla"
    memory: 2
    max_players: 10
  
  - name: "Test Modded Server"
    version: "1.19.4"
    mod_loader: "forge"
    memory: 8
    max_players: 50
```

### Sample User Accounts
```yaml
# Test user accounts
test_users:
  - username: "admin"
    role: "admin"
    email: "admin@test.com"
  
  - username: "operator"
    role: "operator"
    email: "operator@test.com"
  
  - username: "user"
    role: "user"
    email: "user@test.com"
```

## 🎯 Automated Testing

### CI/CD Pipeline Testing
```bash
# Run full test suite
./scripts/test.sh

# Run specific test categories
./scripts/test.sh --unit
./scripts/test.sh --integration
./scripts/test.sh --e2e
```

### Test Reports
- **Unit Test Results**: `test-results/unit-tests.xml`
- **Integration Test Results**: `test-results/integration-tests.xml`
- **Coverage Reports**: `test-results/coverage/`
- **Performance Reports**: `test-results/performance/`

## 📞 Support

If you encounter issues during testing:

1. Check the troubleshooting section above
2. Review the logs for error messages
3. Verify all prerequisites are met
4. Check the GitHub issues for known problems
5. Create a new issue with detailed information

### Useful Commands
```bash
# Quick health check
curl http://localhost:8080/api/health

# View all running containers
docker ps

# Check resource usage
docker stats

# View system logs
journalctl -u docker
```

Happy testing! 🎉
