# Guardian Server Manager - Production Deployment Guide

## Overview

Guardian Server Manager is a comprehensive Minecraft server management platform built with Rust, featuring real-time monitoring, automated backups, user management, and WebSocket communication.

## Features

### ✅ Core Features
- **Server Management**: Start, stop, restart, and monitor Minecraft servers
- **Real-time Monitoring**: WebSocket-based live updates and metrics
- **User Authentication**: JWT-based authentication with role-based access control
- **Backup System**: Automated backups with compression and retention policies
- **RCON Integration**: Remote console access for server management
- **Mod Management**: Install and manage mods and modpacks
- **Database Management**: SQLite-based data persistence with migrations
- **API**: RESTful API for all operations
- **WebSocket**: Real-time communication for live updates

### ✅ Security Features
- **Input Validation**: Comprehensive validation and sanitization
- **Rate Limiting**: API rate limiting and DDoS protection
- **SQL Injection Prevention**: Parameterized queries and input sanitization
- **XSS Prevention**: HTML and JavaScript sanitization
- **Path Traversal Prevention**: Secure file path handling
- **Command Injection Prevention**: Safe command execution
- **CORS Configuration**: Configurable cross-origin resource sharing
- **Security Headers**: Comprehensive security headers

### ✅ Monitoring & Observability
- **Health Checks**: System and service health monitoring
- **Metrics Collection**: Performance and usage metrics
- **Alerting**: Automated alerting for critical issues
- **Logging**: Structured logging with multiple levels
- **Prometheus Integration**: Metrics export for monitoring
- **Grafana Dashboards**: Visual monitoring and alerting

### ✅ Production Features
- **Docker Support**: Containerized deployment
- **Docker Compose**: Multi-service orchestration
- **CI/CD Pipeline**: Automated testing and deployment
- **Performance Optimization**: Caching and connection pooling
- **Error Handling**: Comprehensive error handling and recovery
- **Testing**: Unit, integration, and end-to-end tests

## Quick Start

### Prerequisites

- Docker and Docker Compose
- 10GB+ available disk space
- 4GB+ RAM
- Linux/macOS/Windows with WSL2

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/guardian-server-manager.git
cd guardian-server-manager
```

### 2. Deploy with Docker Compose

```bash
# Make deployment script executable
chmod +x scripts/deploy-production.sh

# Deploy to production
./scripts/deploy-production.sh latest
```

### 3. Access the Application

- **Guardian API**: http://localhost:8080
- **Guardian WebSocket**: ws://localhost:8081
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000

### 4. Default Credentials

- **Guardian Admin**: `admin` / `admin123`
- **Grafana Admin**: `admin` / `[generated-password]`

## Configuration

### Environment Variables

```bash
# Required
JWT_SECRET=your-secret-key-here
GRAFANA_PASSWORD=your-grafana-password

# Optional
REDIS_URL=redis://redis:6379
DATABASE_URL=sqlite:///app/data/guardian.db
RUST_LOG=info
```

### Production Configuration

The production configuration is located in `configs/production.yaml` and includes:

- Server settings (host, port, workers)
- Database configuration
- Redis configuration
- Authentication settings
- Security settings
- Monitoring configuration
- Backup settings
- Performance settings

## Architecture

### System Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Guardian API  │    │  Guardian WS    │    │   Prometheus    │
│   (Port 8080)   │    │  (Port 8081)    │    │   (Port 9090)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
         ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
         │     Redis       │    │   SQLite DB     │    │    Grafana      │
         │   (Port 6379)   │    │   (Data Dir)    │    │   (Port 3000)   │
         └─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Data Flow

1. **API Requests** → Guardian API → Database/Redis
2. **WebSocket Events** → Guardian WS → Real-time updates
3. **Metrics** → Prometheus → Grafana dashboards
4. **Alerts** → Monitoring system → Notifications

## API Documentation

### Authentication

All API endpoints require authentication except `/health` and `/api/auth/*`.

```bash
# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'

# Use token in subsequent requests
curl -H "Authorization: Bearer <token>" http://localhost:8080/api/servers
```

### Server Management

```bash
# List servers
GET /api/servers

# Create server
POST /api/servers
{
  "name": "My Server",
  "minecraft_version": "1.21.1",
  "loader": "vanilla",
  "port": 25565,
  "max_players": 20,
  "memory": 2048
}

# Start server
POST /api/servers/{id}/start

# Stop server
POST /api/servers/{id}/stop

# Restart server
POST /api/servers/{id}/restart
```

### Backup Management

```bash
# List backups
GET /api/servers/{id}/backups

# Create backup
POST /api/servers/{id}/backups
{
  "name": "Backup 1",
  "description": "Full server backup",
  "backup_type": "Full",
  "compression": "Zip"
}

# Restore backup
POST /api/servers/{id}/backups/{backup_id}/restore

# Delete backup
DELETE /api/servers/{id}/backups/{backup_id}
```

### WebSocket Events

Connect to `ws://localhost:8081` to receive real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:8081');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};

// Subscribe to server events
ws.send(JSON.stringify({
  type: 'subscribe',
  server_id: 'server-id'
}));
```

## Monitoring

### Health Checks

```bash
# API health
curl http://localhost:8080/health

# System metrics
curl http://localhost:8080/api/monitoring/metrics

# Alerts
curl http://localhost:8080/api/monitoring/alerts
```

### Prometheus Metrics

- `guardian_http_requests_total` - Total HTTP requests
- `guardian_http_request_duration_seconds` - Request duration
- `guardian_websocket_connections` - Active WebSocket connections
- `guardian_server_count` - Number of managed servers
- `guardian_backup_count` - Number of backups
- `guardian_database_connections` - Database connections
- `guardian_redis_connections` - Redis connections

### Grafana Dashboards

Pre-configured dashboards include:
- System Overview
- API Performance
- Server Management
- Backup Status
- User Activity
- Error Rates

## Security

### Authentication

- JWT-based authentication
- Password hashing with Argon2
- Session management
- Role-based access control

### Input Validation

- Server name validation
- Minecraft version validation
- Port number validation
- Memory allocation validation
- File path validation

### Security Headers

- Content Security Policy
- X-Frame-Options
- X-Content-Type-Options
- X-XSS-Protection
- Strict-Transport-Security

## Backup & Recovery

### Automated Backups

- Configurable retention policies
- Compression support
- Incremental backups
- Backup verification

### Manual Backup

```bash
# Create backup
curl -X POST http://localhost:8080/api/servers/{id}/backups \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "Manual Backup", "backup_type": "Full"}'
```

### Restore from Backup

```bash
# Restore backup
curl -X POST http://localhost:8080/api/servers/{id}/backups/{backup_id}/restore \
  -H "Authorization: Bearer <token>"
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports 8080, 8081, 9090, 3000, and 6379 are available
2. **Permission errors**: Check file permissions for data and log directories
3. **Memory issues**: Ensure sufficient RAM (4GB+ recommended)
4. **Disk space**: Monitor disk usage for backups and logs

### Logs

```bash
# View all logs
docker-compose -f docker-compose.production.yml logs -f

# View specific service logs
docker-compose -f docker-compose.production.yml logs -f guardian
docker-compose -f docker-compose.production.yml logs -f redis
```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
docker-compose -f docker-compose.production.yml up -d
```

## Performance Tuning

### Database Optimization

- Connection pooling
- Query optimization
- Index optimization
- Vacuum operations

### Caching

- Redis caching
- In-memory caching
- Response caching
- Static asset caching

### Resource Limits

```yaml
# docker-compose.production.yml
services:
  guardian:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 1G
          cpus: '0.5'
```

## Scaling

### Horizontal Scaling

- Load balancer configuration
- Multiple API instances
- Database clustering
- Redis clustering

### Vertical Scaling

- Increase memory allocation
- Add more CPU cores
- Optimize database settings
- Tune JVM parameters

## Maintenance

### Regular Tasks

1. **Database maintenance**: Weekly vacuum and analyze
2. **Log rotation**: Daily log cleanup
3. **Backup verification**: Weekly backup integrity checks
4. **Security updates**: Monthly dependency updates
5. **Performance monitoring**: Continuous monitoring

### Updates

```bash
# Update to latest version
git pull origin main
./scripts/deploy-production.sh latest
```

### Backup Before Updates

```bash
# Create system backup
./scripts/backup-system.sh
```

## Support

### Documentation

- API Documentation: `/docs` endpoint
- OpenAPI Specification: `/api/openapi.json`
- Health Check: `/health`

### Monitoring

- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000
- Logs: `docker-compose logs -f`

### Troubleshooting

1. Check logs for errors
2. Verify service health
3. Check resource usage
4. Review configuration
5. Test connectivity

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Changelog

See CHANGELOG.md for version history and updates.
