# Guardian Platform - Production Deployment Guide

This guide provides comprehensive instructions for deploying Guardian Platform in a production environment as a headless backend service.

## Table of Contents

1. [Overview](#overview)
2. [System Requirements](#system-requirements)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Security](#security)
6. [Monitoring](#monitoring)
7. [Backup & Recovery](#backup--recovery)
8. [Scaling](#scaling)
9. [Maintenance](#maintenance)
10. [Troubleshooting](#troubleshooting)

## Overview

Guardian Platform is designed for production deployment with enterprise-grade features:

- **High Availability**: Blue/green deployments, automatic failover
- **Scalability**: Multi-tenant architecture, horizontal scaling
- **Security**: JWT authentication, role-based access control
- **Monitoring**: Prometheus metrics collection
- **Backup**: Automated backups with point-in-time recovery

## System Requirements

### Minimum Requirements

- **OS**: Windows Server 2019+, Ubuntu 20.04+, CentOS 8+
- **CPU**: 8 cores (16+ recommended)
- **RAM**: 32GB (64GB+ recommended)
- **Storage**: 500GB SSD (1TB+ recommended)
- **Network**: 1Gbps connection
- **Java**: OpenJDK 21+
- **Docker**: 20.10+ (for containerized deployment)

### Recommended Production Setup

- **CPU**: 16+ cores (Intel Xeon or AMD EPYC)
- **RAM**: 64GB+ ECC memory
- **Storage**: NVMe SSD with RAID 1/10
- **Network**: 10Gbps connection with redundancy
- **GPU**: NVIDIA RTX 3060+ or AMD RX 6600+ (for acceleration)

## Installation

### Option 1: Docker Compose (Recommended)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-org/guardian-platform.git
   cd guardian-platform
   ```

2. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your production settings
   ```

3. **Deploy with Docker Compose**:
   ```bash
   docker-compose -f docker-compose.prod.yml up -d
   ```

### Option 2: Manual Installation

1. **Install dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install openjdk-21-jdk rustc cargo docker.io docker-compose

   # CentOS/RHEL
   sudo yum install java-21-openjdk rust cargo docker docker-compose
   ```

2. **Build components**:
   ```bash
   ./scripts/build.sh --release
   ```

3. **Install systemd services**:
   ```bash
   sudo ./scripts/install-systemd.sh
   ```

## Configuration

### Core Configuration

Edit `configs/server.yaml`:

```yaml
# Production configuration
minecraft:
  loader: neoforge
  version: 1.20.1

java:
  heap_gb: 16
  flags: 
    - "-XX:+UseG1GC"
    - "-XX:+UnlockExperimentalVMOptions"
    - "-XX:MaxGCPauseMillis=100"
    - "-XX:+DisableExplicitGC"

paths:
  mods_dir: /opt/guardian/servers/mods
  config_dir: /opt/guardian/servers/config
  world_dir: /opt/guardian/servers/worlds
  backup_dir: /opt/guardian/backups

# High Availability
ha:
  autosave_minutes: 5
  snapshot_keep: 168  # 1 week
  blue_green: true
  health_check_interval: 30

# GPU Acceleration
gpu:
  enabled: true
  batch_size_chunks: 128
  memory_limit_gb: 8

# Monitoring
monitoring:
  metrics_port: 9090
  log_level: info
  prometheus_enabled: true
```

### Security Configuration

1. **Generate JWT secret**:
   ```bash
   openssl rand -base64 32 > /opt/guardian/secrets/jwt_secret
   chmod 600 /opt/guardian/secrets/jwt_secret
   ```

2. **Configure SSL/TLS** (for API endpoints):
   ```yaml
   api:
     ssl:
       enabled: true
       cert_file: /opt/guardian/ssl/cert.pem
       key_file: /opt/guardian/ssl/key.pem
   ```

3. **Set up firewall**:
   ```bash
   # UFW (Ubuntu)
   sudo ufw allow 22/tcp    # SSH
   sudo ufw allow 9090/tcp  # Prometheus metrics
   sudo ufw allow 25565/tcp # Minecraft
   sudo ufw enable

   # firewalld (CentOS)
   sudo firewall-cmd --permanent --add-port=9090/tcp
   sudo firewall-cmd --permanent --add-port=25565/tcp
   sudo firewall-cmd --reload
   ```

## Security

### Authentication & Authorization

1. **Configure admin user**:
   ```bash
   guardian-cli user create admin --role admin --password "secure-password"
   ```

2. **Set up multi-factor authentication**:
   ```bash
   guardian-cli user enable-mfa admin
   ```

3. **Configure API keys**:
   ```bash
   guardian-cli api-key create --name "monitoring" --permissions "read:metrics"
   ```

### Network Security

1. **Use reverse proxy** (Nginx example for API endpoints):
   ```nginx
   server {
       listen 443 ssl;
       server_name your-domain.com;
       
       ssl_certificate /path/to/cert.pem;
       ssl_certificate_key /path/to/key.pem;
       
       location /api/ {
           proxy_pass http://localhost:9090;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   ```

2. **Enable rate limiting**:
   ```yaml
   api:
     rate_limit:
       enabled: true
       requests_per_minute: 60
       burst_size: 10
   ```

## Monitoring

### Prometheus Configuration

Edit `monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'guardian-platform'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'minecraft-servers'
    static_configs:
      - targets: ['localhost:25565']
    metrics_path: /metrics
    scrape_interval: 10s
```

### Alerting Configuration

Configure alerts for critical metrics:
   ```yaml
   alerts:
     - name: "High Memory Usage"
       condition: "memory_usage_percent > 90"
       duration: "5m"
       action: "restart_server"
     
     - name: "Low TPS"
       condition: "tps < 15"
       duration: "2m"
       action: "alert_admin"
   ```

## Backup & Recovery

### Automated Backups

1. **Configure backup schedule**:
   ```yaml
   backup:
     schedule: "0 */6 * * *"  # Every 6 hours
     retention_days: 30
     compression: true
     encryption: true
     destinations:
       - type: local
         path: /opt/guardian/backups
       - type: s3
         bucket: guardian-backups
         region: us-west-2
   ```

2. **Test backup restoration**:
   ```bash
   guardian-cli backup restore --backup-id "2024-01-15-12-00-00" --dry-run
   ```

### Disaster Recovery

1. **Create recovery plan**:
   ```bash
   # Document recovery procedures
   cat > /opt/guardian/recovery-plan.md << EOF
   # Disaster Recovery Plan
   
   ## Full System Recovery
   1. Restore from latest backup
   2. Verify configuration
   3. Start services in order
   4. Run health checks
   
   ## Partial Recovery
   1. Identify affected components
   2. Restore specific backups
   3. Validate data integrity
   EOF
   ```

## Scaling

### Horizontal Scaling

1. **Load balancer configuration**:
   ```yaml
   load_balancer:
     type: nginx
     upstreams:
       - server1:9090
       - server2:9090
       - server3:9090
     health_check: /health
   ```

2. **Database clustering**:
   ```yaml
   database:
     type: postgresql
     cluster:
       primary: "db1.example.com:5432"
       replicas:
         - "db2.example.com:5432"
         - "db3.example.com:5432"
   ```

### Vertical Scaling

1. **Resource allocation**:
   ```yaml
   resources:
     cpu_limit: "8"
     memory_limit: "32Gi"
     storage_limit: "1Ti"
   ```

## Maintenance

### Regular Maintenance Tasks

1. **Daily**:
   - Check system health
   - Review logs for errors
   - Verify backup completion

2. **Weekly**:
   - Update mod compatibility rules
   - Clean up old logs
   - Performance analysis

3. **Monthly**:
   - Security updates
   - Capacity planning
   - Disaster recovery testing

### Update Procedures

1. **Blue/Green Deployment**:
   ```bash
   # Deploy to green environment
   guardian-cli deploy --environment green --version 1.1.0
   
   # Run smoke tests
   guardian-cli test --environment green
   
   # Switch traffic
   guardian-cli switch --from blue --to green
   
   # Cleanup old environment
   guardian-cli cleanup --environment blue
   ```

2. **Rollback Procedure**:
   ```bash
   # Switch back to blue
   guardian-cli switch --from green --to blue
   
   # Investigate issues
   guardian-cli logs --environment green --level error
   ```

## Troubleshooting

### Common Issues

1. **High Memory Usage**:
   ```bash
   # Check memory usage
   guardian-cli metrics --metric memory_usage
   
   # Restart with more memory
   guardian-cli server restart --memory 16g
   ```

2. **GPU Acceleration Issues**:
   ```bash
   # Check GPU status
   guardian-cli gpu status
   
   # Disable GPU acceleration
   guardian-cli config set gpu.enabled false
   ```

3. **Network Connectivity**:
   ```bash
   # Test connectivity
   guardian-cli network test --target minecraft-server
   
   # Check firewall rules
   guardian-cli firewall status
   ```

### Log Analysis

1. **Centralized logging**:
   ```yaml
   logging:
     level: info
     format: json
     destination: elasticsearch
     elasticsearch:
       url: "http://elasticsearch:9200"
       index: "guardian-logs"
   ```

2. **Log queries**:
   ```bash
   # Search for errors
   guardian-cli logs search --query "level:error" --time-range "1h"
   
   # Performance analysis
   guardian-cli logs search --query "tps < 15" --time-range "24h"
   ```

### Performance Optimization

1. **JVM Tuning**:
   ```yaml
   java:
     flags:
       - "-XX:+UseG1GC"
       - "-XX:MaxGCPauseMillis=100"
       - "-XX:G1HeapRegionSize=16m"
       - "-XX:+UseStringDeduplication"
   ```

2. **GPU Optimization**:
   ```yaml
   gpu:
     batch_size_chunks: 256
     memory_limit_gb: 12
     async_processing: true
   ```

## Support

For production support:

- **Documentation**: [docs.guardian-platform.com](https://docs.guardian-platform.com)
- **Community**: [Discord Server](https://discord.gg/guardian-platform)
- **Enterprise Support**: [support@guardian-platform.com](mailto:support@guardian-platform.com)
- **Emergency Hotline**: +1-800-GUARDIAN

## License

Guardian Platform is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**Note**: This guide is for production deployments. For development and testing, see [DEVELOPMENT.md](DEVELOPMENT.md).