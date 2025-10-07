# Guardian Server Manager - Security Notes

## Overview

This document outlines the security measures implemented in Guardian Server Manager and provides guidance for secure deployment and operation.

## Security Architecture

### Defense in Depth

Guardian implements multiple layers of security:

1. **Network Security**: Localhost-only binding by default
2. **Input Validation**: Comprehensive validation of all inputs
3. **Path Sanitization**: Protection against directory traversal
4. **Rate Limiting**: Protection against abuse and DoS attacks
5. **Error Handling**: Safe error responses without information leakage
6. **Resource Management**: Proper cleanup and resource limits

## Network Security

### Binding Configuration

**Default Behavior:**
- Guardian binds to `127.0.0.1` (localhost) by default
- No external network access without explicit configuration
- WebSocket connections limited to localhost

**Configuration:**
```yaml
# guardian.yaml
server:
  host: "127.0.0.1"  # Default: localhost only
  port: 52100        # Default port
```

**Security Considerations:**
- Never bind to `0.0.0.0` unless absolutely necessary
- Use firewall rules to restrict access
- Consider VPN or SSH tunneling for remote access
- Monitor network connections regularly

### WebSocket Security

**Implementation:**
- WebSocket connections limited to localhost
- No authentication required for localhost connections
- Rate limiting applied to WebSocket messages
- Input validation on all WebSocket messages

**Security Measures:**
- Message size limits
- Connection count limits
- Automatic cleanup of stale connections
- No sensitive data in WebSocket messages

## Input Validation

### Server Name Validation

**Rules:**
- Length: 1-50 characters
- Characters: Letters, numbers, hyphens only
- No spaces, special characters, or path separators
- No reserved names (CON, PRN, AUX, etc.)

**Implementation:**
```rust
pub fn validate_server_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 50 {
        return Err(ValidationError::InvalidLength);
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ValidationError::InvalidCharacters);
    }
    
    // Check for reserved names
    if RESERVED_NAMES.contains(&name.to_uppercase().as_str()) {
        return Err(ValidationError::ReservedName);
    }
    
    Ok(())
}
```

### Path Validation

**Rules:**
- No absolute paths
- No directory traversal (`..` segments)
- No null bytes or special characters
- Must be within allowed directories

**Implementation:**
```rust
pub fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() {
        return Err(ValidationError::EmptyPath);
    }
    
    if path.contains("..") || path.starts_with('/') || path.starts_with("C:\\") {
        return Err(ValidationError::InvalidPath);
    }
    
    if path.contains('\0') || path.contains('\n') || path.contains('\r') {
        return Err(ValidationError::InvalidCharacters);
    }
    
    Ok(())
}
```

### Port Validation

**Rules:**
- Valid port range: 1-65535
- No reserved ports (1-1023) without special privileges
- No duplicate ports across servers

**Implementation:**
```rust
pub fn validate_port(port: u16) -> Result<(), ValidationError> {
    if port == 0 {
        return Err(ValidationError::InvalidPort);
    }
    
    if port < 1024 && !has_privileges() {
        return Err(ValidationError::PrivilegedPort);
    }
    
    if is_port_in_use(port) {
        return Err(ValidationError::PortInUse);
    }
    
    Ok(())
}
```

### Memory Validation

**Rules:**
- Minimum: 512MB
- Maximum: 32GB
- Must be a multiple of 64MB
- Cannot exceed available system memory

**Implementation:**
```rust
pub fn validate_memory(memory_mb: u32) -> Result<(), ValidationError> {
    if memory_mb < 512 {
        return Err(ValidationError::InsufficientMemory);
    }
    
    if memory_mb > 32768 {
        return Err(ValidationError::ExcessiveMemory);
    }
    
    if memory_mb % 64 != 0 {
        return Err(ValidationError::InvalidMemoryAlignment);
    }
    
    if memory_mb > get_available_memory() {
        return Err(ValidationError::InsufficientSystemMemory);
    }
    
    Ok(())
}
```

## Path Sanitization

### Directory Traversal Protection

**Threat:** Malicious paths like `../../../etc/passwd` or `..\\..\\..\\windows\\system32`

**Protection:**
```rust
pub fn sanitize_path(path: &str) -> Result<String, SanitizationError> {
    let path = path.replace('\\', "/"); // Normalize separators
    
    if path.contains("..") || path.starts_with('/') {
        return Err(SanitizationError::DirectoryTraversal);
    }
    
    let components: Vec<&str> = path.split('/').collect();
    let mut sanitized = Vec::new();
    
    for component in components {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return Err(SanitizationError::DirectoryTraversal);
        }
        sanitized.push(component);
    }
    
    Ok(sanitized.join("/"))
}
```

### Archive Extraction Security

**Threat:** Malicious archive files with dangerous paths

**Protection:**
```rust
pub fn is_safe_archive_path(path: &str) -> bool {
    let path = path.replace('\\', "/");
    
    // Check for directory traversal
    if path.contains("..") || path.starts_with('/') {
        return false;
    }
    
    // Check for absolute paths
    if path.starts_with("C:") || path.starts_with("/") {
        return false;
    }
    
    // Check for reserved names
    let filename = path.split('/').last().unwrap_or("");
    if RESERVED_FILENAMES.contains(&filename.to_uppercase().as_str()) {
        return false;
    }
    
    true
}
```

## Rate Limiting

### Implementation

**Per-IP Rate Limits:**
- Search endpoints: 60 requests/minute
- Download endpoints: 30 requests/minute
- Server management: 10 requests/minute
- Authentication: 5 requests/minute

**Implementation:**
```rust
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    limits: HashMap<String, (u32, Duration)>,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, client_id: &str, endpoint: &str) -> Result<(), RateLimitError> {
        let (limit, window) = self.limits.get(endpoint).unwrap_or(&(10, Duration::from_secs(60)));
        let now = Instant::now();
        
        let client_requests = self.requests.entry(client_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        client_requests.retain(|&time| now.duration_since(time) < *window);
        
        if client_requests.len() >= *limit as usize {
            return Err(RateLimitError::Exceeded);
        }
        
        client_requests.push(now);
        Ok(())
    }
}
```

### Rate Limit Headers

**Response Headers:**
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1640995200
```

## Error Handling

### Safe Error Responses

**Principle:** Never expose sensitive information in error responses

**Implementation:**
```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found"),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error"),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error"),
            // Never expose internal details
        };
        
        let response = ApiResponse {
            success: false,
            data: None,
            error: Some(message.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        (status, Json(response)).into_response()
    }
}
```

### Logging Security

**Safe Logging Practices:**
- Never log API keys or passwords
- Sanitize user inputs before logging
- Use structured logging with appropriate levels
- Rotate logs regularly
- Secure log file permissions

**Implementation:**
```rust
// Good: Sanitized logging
info!("User {} created server {}", user_id, server_name);

// Bad: Potential information leakage
info!("User {} created server {} with password {}", user_id, server_name, password);

// Good: Sanitized API key logging
info!("API key validation {} for provider {}", 
    if api_key.is_empty() { "failed" } else { "succeeded" }, 
    provider
);
```

## Secret Management

### API Key Storage

**Storage Method:**
- Encrypted storage using system keyring when available
- Fallback to obfuscated file storage
- No plaintext storage in configuration files

**Implementation:**
```rust
pub struct SecretStorage {
    keyring: Option<Keyring>,
    fallback_path: PathBuf,
}

impl SecretStorage {
    pub async fn store_api_key(&self, provider: &str, key: &str) -> Result<(), SecretError> {
        if let Some(keyring) = &self.keyring {
            keyring.store(&format!("guardian_{}", provider), key)?;
        } else {
            self.store_encrypted_file(provider, key).await?;
        }
        Ok(())
    }
    
    pub async fn get_api_key(&self, provider: &str) -> Result<Option<String>, SecretError> {
        if let Some(keyring) = &self.keyring {
            Ok(keyring.get(&format!("guardian_{}", provider))?)
        } else {
            self.get_encrypted_file(provider).await
        }
    }
}
```

### Environment Variables

**Secure Environment Variable Usage:**
```bash
# Good: Use environment variables for secrets
export CURSEFORGE_API_KEY="your_key_here"
export MODRINTH_API_KEY="your_key_here"

# Bad: Never hardcode secrets in code
const API_KEY = "your_key_here"; // DON'T DO THIS
```

## Resource Management

### Memory Limits

**Server Memory Limits:**
- Maximum per server: 32GB
- System memory check before allocation
- Graceful handling of memory pressure

**Implementation:**
```rust
pub fn validate_memory_allocation(requested: u32) -> Result<(), MemoryError> {
    let available = get_available_memory();
    let reserved = get_reserved_memory();
    let usable = available - reserved;
    
    if requested > usable {
        return Err(MemoryError::InsufficientMemory);
    }
    
    if requested > MAX_SERVER_MEMORY {
        return Err(MemoryError::ExcessiveMemory);
    }
    
    Ok(())
}
```

### File System Limits

**Archive Extraction Limits:**
- Maximum file size: 100MB
- Maximum total extraction size: 1GB
- Maximum file count: 10,000 files
- Timeout: 5 minutes

**Implementation:**
```rust
pub struct ExtractionLimits {
    max_file_size: u64,
    max_total_size: u64,
    max_file_count: usize,
    timeout: Duration,
}

impl ExtractionLimits {
    pub fn new() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_total_size: 1024 * 1024 * 1024, // 1GB
            max_file_count: 10000,
            timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}
```

## Security Headers

### HTTP Security Headers

**Implementation:**
```rust
pub fn add_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'self'".parse().unwrap());
    
    response
}
```

## Deployment Security

### Production Deployment

**Security Checklist:**
- [ ] Run on dedicated server with minimal attack surface
- [ ] Use firewall to restrict network access
- [ ] Enable system logging and monitoring
- [ ] Regular security updates
- [ ] Backup and disaster recovery plan
- [ ] Access control and authentication
- [ ] Network segmentation
- [ ] Intrusion detection system

### Docker Deployment

**Security Considerations:**
```dockerfile
# Use non-root user
RUN adduser --disabled-password --gecos '' guardian
USER guardian

# Minimal base image
FROM rust:alpine

# Security updates
RUN apk update && apk upgrade

# Read-only filesystem where possible
VOLUME ["/data"]
```

### Network Security

**Firewall Rules:**
```bash
# Allow only localhost access
iptables -A INPUT -p tcp --dport 52100 -s 127.0.0.1 -j ACCEPT
iptables -A INPUT -p tcp --dport 52100 -j DROP

# Allow Minecraft server ports (if needed)
iptables -A INPUT -p tcp --dport 25565:25575 -j ACCEPT
```

## Monitoring and Auditing

### Security Monitoring

**Log Monitoring:**
- Failed authentication attempts
- Rate limit violations
- Input validation failures
- Path sanitization violations
- Resource limit violations

**Implementation:**
```rust
pub fn log_security_event(event: SecurityEvent) {
    match event {
        SecurityEvent::FailedAuth { user, ip } => {
            warn!("Failed authentication attempt for user {} from IP {}", user, ip);
        }
        SecurityEvent::RateLimitExceeded { ip, endpoint } => {
            warn!("Rate limit exceeded for IP {} on endpoint {}", ip, endpoint);
        }
        SecurityEvent::PathTraversalAttempt { path, ip } => {
            error!("Path traversal attempt with path '{}' from IP {}", path, ip);
        }
    }
}
```

### Audit Trail

**Audit Events:**
- Server creation, modification, deletion
- Mod installation and removal
- Backup creation and restoration
- Configuration changes
- User actions

**Implementation:**
```rust
pub struct AuditLogger {
    db: DatabaseManager,
}

impl AuditLogger {
    pub async fn log_action(&self, action: AuditAction) -> Result<(), AuditError> {
        let audit_entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            action: action.action,
            user: action.user,
            resource: action.resource,
            details: action.details,
            ip_address: action.ip_address,
        };
        
        self.db.create_audit_entry(audit_entry).await?;
        Ok(())
    }
}
```

## Incident Response

### Security Incident Response Plan

1. **Detection**: Monitor logs and alerts
2. **Assessment**: Determine scope and impact
3. **Containment**: Isolate affected systems
4. **Eradication**: Remove threats and vulnerabilities
5. **Recovery**: Restore normal operations
6. **Lessons Learned**: Update security measures

### Emergency Procedures

**In case of security breach:**
1. Immediately stop all servers
2. Disconnect from network
3. Preserve logs and evidence
4. Assess damage and scope
5. Notify relevant parties
6. Implement fixes
7. Restore from clean backups

## Compliance and Standards

### Security Standards

Guardian follows these security standards:
- **OWASP Top 10**: Protection against common web vulnerabilities
- **CWE/SANS Top 25**: Prevention of common programming errors
- **NIST Cybersecurity Framework**: Risk management and security controls

### Data Protection

**Data Handling:**
- No collection of personal data
- Local processing only
- No data transmission to third parties
- Secure deletion of temporary files

**Privacy Considerations:**
- Server logs may contain player information
- Backup files may contain world data
- Configuration files may contain sensitive settings
- Implement appropriate data retention policies

## Security Updates

### Update Process

1. **Monitoring**: Subscribe to security advisories
2. **Assessment**: Evaluate security updates
3. **Testing**: Test updates in development environment
4. **Deployment**: Deploy updates to production
5. **Verification**: Verify security improvements

### Vulnerability Reporting

**Reporting Security Issues:**
- Use GitHub Security Advisories
- Provide detailed reproduction steps
- Include affected versions
- Allow reasonable time for fixes

**Response Timeline:**
- Acknowledgment: 24 hours
- Initial assessment: 72 hours
- Fix development: 7-30 days
- Public disclosure: 90 days

## Best Practices

### Development Security

1. **Secure Coding**: Follow secure coding practices
2. **Code Review**: All code changes reviewed
3. **Testing**: Comprehensive security testing
4. **Dependencies**: Regular dependency updates
5. **Documentation**: Keep security documentation current

### Operational Security

1. **Access Control**: Principle of least privilege
2. **Monitoring**: Continuous security monitoring
3. **Backups**: Regular, secure backups
4. **Updates**: Timely security updates
5. **Training**: Security awareness training

### User Security

1. **Strong Passwords**: Use strong, unique passwords
2. **API Keys**: Secure API key management
3. **Network Security**: Secure network configuration
4. **Regular Updates**: Keep Guardian updated
5. **Monitoring**: Monitor for suspicious activity

## Conclusion

Guardian Server Manager implements comprehensive security measures to protect against common threats and vulnerabilities. However, security is an ongoing process that requires regular updates, monitoring, and user awareness.

For questions or concerns about security, please contact the development team or create a security advisory on GitHub.
