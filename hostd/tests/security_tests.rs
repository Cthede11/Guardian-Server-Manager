use hostd::security::path_sanitizer::{PathSanitizer, PathSanitizationError};
use hostd::security::validation::ValidationService;
use hostd::security::rate_limiting::{RateLimiter, RateLimitConfig};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_path_sanitizer_blocks_absolute_paths() {
    let temp_dir = TempDir::new().unwrap();
    let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

    // Test absolute paths
    assert!(sanitizer.sanitize_path("/absolute/path").is_err());
    assert!(sanitizer.sanitize_path("C:\\absolute\\path").is_err());
    assert!(sanitizer.sanitize_path("C:/absolute/path").is_err());
}

#[tokio::test]
async fn test_path_sanitizer_blocks_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

    // Test parent directory traversal
    assert!(sanitizer.sanitize_path("../parent").is_err());
    assert!(sanitizer.sanitize_path("..\\parent").is_err());
    assert!(sanitizer.sanitize_path("mods/../../../etc/passwd").is_err());
    assert!(sanitizer.sanitize_path("config/..\\..\\windows\\system32").is_err());
}

#[tokio::test]
async fn test_path_sanitizer_allows_valid_paths() {
    let temp_dir = TempDir::new().unwrap();
    let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

    // Test valid paths
    assert!(sanitizer.sanitize_path("mods/test.jar").is_ok());
    assert!(sanitizer.sanitize_path("config/test.json").is_ok());
    assert!(sanitizer.sanitize_path("world/region/r.0.0.mca").is_ok());
    assert!(sanitizer.sanitize_path("logs/latest.log").is_ok());
    assert!(sanitizer.sanitize_path("scripts/start.sh").is_ok());
}

#[tokio::test]
async fn test_path_sanitizer_blocks_disallowed_prefixes() {
    let temp_dir = TempDir::new().unwrap();
    let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

    // Test disallowed prefixes
    assert!(sanitizer.sanitize_path("random/file.txt").is_err());
    assert!(sanitizer.sanitize_path("system/config.ini").is_err());
    assert!(sanitizer.sanitize_path("etc/passwd").is_err());
}

#[tokio::test]
async fn test_path_sanitizer_canonicalization() {
    let temp_dir = TempDir::new().unwrap();
    let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

    // Create a symlink to test canonicalization
    let mods_dir = temp_dir.path().join("mods");
    std::fs::create_dir_all(&mods_dir).unwrap();
    
    // Test that canonicalization prevents escaping
    let result = sanitizer.sanitize_path("mods/../../../etc/passwd");
    assert!(result.is_err());
}

#[test]
fn test_server_name_validation() {
    // Valid names
    assert!(ValidationService::validate_server_name("My Server").is_ok());
    assert!(ValidationService::validate_server_name("Test-Server_123").is_ok());
    assert!(ValidationService::validate_server_name("A").is_ok());

    // Invalid names
    assert!(ValidationService::validate_server_name("").is_err());
    assert!(ValidationService::validate_server_name(&"a".repeat(51)).is_err());
    assert!(ValidationService::validate_server_name("Server@#$").is_err());
    assert!(ValidationService::validate_server_name("Server<script>").is_err());
}

#[test]
fn test_port_validation() {
    // Valid ports
    assert!(ValidationService::validate_port(25565).is_ok());
    assert!(ValidationService::validate_port(30000).is_ok());
    assert!(ValidationService::validate_port(65535).is_ok());

    // Invalid ports
    assert!(ValidationService::validate_port(1023).is_err());
    assert!(ValidationService::validate_port(0).is_err());
}

#[test]
fn test_memory_validation() {
    // Valid memory
    assert!(ValidationService::validate_memory(512).is_ok());
    assert!(ValidationService::validate_memory(2048).is_ok());
    assert!(ValidationService::validate_memory(32768).is_ok());

    // Invalid memory
    assert!(ValidationService::validate_memory(256).is_err());
    assert!(ValidationService::validate_memory(65536).is_err());
}

#[test]
fn test_provider_validation() {
    // Valid providers
    assert!(ValidationService::validate_provider("curseforge").is_ok());
    assert!(ValidationService::validate_provider("modrinth").is_ok());
    assert!(ValidationService::validate_provider("vanilla").is_ok());
    assert!(ValidationService::validate_provider("fabric").is_ok());
    assert!(ValidationService::validate_provider("quilt").is_ok());
    assert!(ValidationService::validate_provider("forge").is_ok());

    // Invalid providers
    assert!(ValidationService::validate_provider("").is_err());
    assert!(ValidationService::validate_provider("invalid").is_err());
    assert!(ValidationService::validate_provider("CURSEFORGE").is_err()); // Case sensitive
}

#[test]
fn test_api_key_validation() {
    // Valid API keys
    assert!(ValidationService::validate_api_key("valid_key_123").is_ok());
    assert!(ValidationService::validate_api_key("key-with-dashes").is_ok());
    assert!(ValidationService::validate_api_key("key_with_underscores").is_ok());
    assert!(ValidationService::validate_api_key("key.with.dots").is_ok());

    // Invalid API keys
    assert!(ValidationService::validate_api_key("").is_err());
    assert!(ValidationService::validate_api_key("short").is_err());
    assert!(ValidationService::validate_api_key(&"a".repeat(201)).is_err());
    assert!(ValidationService::validate_api_key("key@with#invalid").is_err());
}

#[test]
fn test_input_sanitization() {
    // Test control character removal
    let input = "Hello\x00World\x1F";
    let sanitized = ValidationService::sanitize_input(input);
    assert_eq!(sanitized, "HelloWorld");

    // Test trimming
    let input = "  Hello World  ";
    let sanitized = ValidationService::sanitize_input(input);
    assert_eq!(sanitized, "Hello World");
}

#[tokio::test]
async fn test_rate_limiter_basic() {
    let config = RateLimitConfig {
        requests_per_minute: 2,
        burst_limit: 2,
        window_size: Duration::from_secs(60),
    };
    
    let rate_limiter = RateLimiter::new(config);
    
    // First two requests should be allowed
    assert!(rate_limiter.is_allowed("test_key").await);
    assert!(rate_limiter.is_allowed("test_key").await);
    
    // Third request should be blocked
    assert!(!rate_limiter.is_allowed("test_key").await);
}

#[tokio::test]
async fn test_rate_limiter_different_keys() {
    let config = RateLimitConfig {
        requests_per_minute: 1,
        burst_limit: 1,
        window_size: Duration::from_secs(60),
    };
    
    let rate_limiter = RateLimiter::new(config);
    
    // Different keys should have separate limits
    assert!(rate_limiter.is_allowed("key1").await);
    assert!(rate_limiter.is_allowed("key2").await);
    assert!(!rate_limiter.is_allowed("key1").await);
    assert!(!rate_limiter.is_allowed("key2").await);
}

#[tokio::test]
async fn test_rate_limiter_reset() {
    let config = RateLimitConfig {
        requests_per_minute: 1,
        burst_limit: 1,
        window_size: Duration::from_secs(1),
    };
    
    let rate_limiter = RateLimiter::new(config);
    
    // First request should be allowed
    assert!(rate_limiter.is_allowed("test_key").await);
    
    // Second request should be blocked
    assert!(!rate_limiter.is_allowed("test_key").await);
    
    // Reset the rate limit
    rate_limiter.reset("test_key").await;
    
    // Request should be allowed again
    assert!(rate_limiter.is_allowed("test_key").await);
}

#[tokio::test]
async fn test_binding_verification() {
    // Test that the default binding is 127.0.0.1
    let config = hostd::core::guardian_config::GuardianConfig::default();
    assert_eq!(config.guardian_host, "127.0.0.1");
    
    // Test that port is reasonable
    assert!(config.guardian_port > 1024);
    assert!(config.guardian_port <= 65535);
}

#[test]
fn test_java_path_validation() {
    // Test that Java path validation works
    let valid_paths = ["java", "C:\\Program Files\\Java\\bin\\java.exe", "/usr/bin/java"];
    for path in valid_paths {
        assert!(ValidationService::validate_file_path(path).is_ok());
    }
    
    // Test that dangerous paths are rejected
    let invalid_paths = ["../../../etc/passwd", "C:\\Windows\\System32\\cmd.exe", ""];
    for path in invalid_paths {
        assert!(ValidationService::validate_file_path(path).is_err());
    }
}

#[test]
fn test_minecraft_version_validation() {
    // Valid versions
    assert!(ValidationService::validate_minecraft_version("1.20.1").is_ok());
    assert!(ValidationService::validate_minecraft_version("1.19.4").is_ok());
    assert!(ValidationService::validate_minecraft_version("1.18.2").is_ok());

    // Invalid versions
    assert!(ValidationService::validate_minecraft_version("").is_err());
    assert!(ValidationService::validate_minecraft_version("invalid").is_err());
    assert!(ValidationService::validate_minecraft_version("1.20").is_ok()); // This should be valid
}

#[test]
fn test_loader_validation() {
    // Valid loaders
    assert!(ValidationService::validate_loader("vanilla").is_ok());
    assert!(ValidationService::validate_loader("fabric").is_ok());
    assert!(ValidationService::validate_loader("quilt").is_ok());
    assert!(ValidationService::validate_loader("forge").is_ok());

    // Invalid loaders
    assert!(ValidationService::validate_loader("").is_err());
    assert!(ValidationService::validate_loader("invalid").is_err());
    assert!(ValidationService::validate_loader("VANILLA").is_err()); // Case sensitive
}

#[test]
fn test_password_strength_validation() {
    // Valid passwords
    assert!(ValidationService::validate_password_strength("Password123!").is_ok());
    assert!(ValidationService::validate_password_strength("MyStr0ng!Pass").is_ok());

    // Invalid passwords
    assert!(ValidationService::validate_password_strength("").is_err());
    assert!(ValidationService::validate_password_strength("weak").is_err());
    assert!(ValidationService::validate_password_strength("nouppercase123!").is_err());
    assert!(ValidationService::validate_password_strength("NOLOWERCASE123!").is_err());
    assert!(ValidationService::validate_password_strength("NoNumbers!").is_err());
    assert!(ValidationService::validate_password_strength("NoSpecial123").is_err());
}
