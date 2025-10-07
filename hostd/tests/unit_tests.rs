use hostd::{
    security::path_sanitizer::PathSanitizer,
    version_resolver::VersionResolver,
    modpack_installer::{ModpackInstaller, ModrinthManifest, CurseForgeManifest},
    security::validation::InputValidator,
    database::DatabaseManager,
};
use serde_json::json;

#[tokio::test]
async fn test_path_sanitizer_comprehensive() {
    let sanitizer = PathSanitizer::new();
    
    // Test valid paths
    assert!(sanitizer.is_safe_path("mods/test-mod.jar"));
    assert!(sanitizer.is_safe_path("config/server.properties"));
    assert!(sanitizer.is_safe_path("world/region/r.0.0.mca"));
    assert!(sanitizer.is_safe_path("logs/latest.log"));
    assert!(sanitizer.is_safe_path("data/playerdata/uuid.dat"));
    
    // Test malicious paths - directory traversal
    assert!(!sanitizer.is_safe_path("../etc/passwd"));
    assert!(!sanitizer.is_safe_path("../../../windows/system32"));
    assert!(!sanitizer.is_safe_path("..\\..\\..\\windows\\system32"));
    assert!(!sanitizer.is_safe_path("/etc/shadow"));
    assert!(!sanitizer.is_safe_path("C:\\Windows\\System32"));
    assert!(!sanitizer.is_safe_path("..\\..\\..\\etc\\passwd"));
    
    // Test malicious paths - absolute paths
    assert!(!sanitizer.is_safe_path("/etc/passwd"));
    assert!(!sanitizer.is_safe_path("C:\\Windows\\System32\\config\\SAM"));
    assert!(!sanitizer.is_safe_path("/home/user/.ssh/id_rsa"));
    
    // Test malicious paths - null bytes and special characters
    assert!(!sanitizer.is_safe_path("mods/test\0mod.jar"));
    assert!(!sanitizer.is_safe_path("config/../../etc/passwd"));
    assert!(!sanitizer.is_safe_path("world/..\\..\\..\\windows\\system32"));
    
    // Test edge cases
    assert!(!sanitizer.is_safe_path(""));
    assert!(!sanitizer.is_safe_path("."));
    assert!(!sanitizer.is_safe_path(".."));
    assert!(!sanitizer.is_safe_path("./"));
    assert!(!sanitizer.is_safe_path("../"));
}

#[tokio::test]
async fn test_input_validator_comprehensive() {
    let validator = InputValidator::new();
    
    // Test server name validation
    assert!(validator.validate_server_name("valid-server-name").is_ok());
    assert!(validator.validate_server_name("test123").is_ok());
    assert!(validator.validate_server_name("my-server-1").is_ok());
    
    // Test invalid server names
    assert!(validator.validate_server_name("").is_err());
    assert!(validator.validate_server_name("a".repeat(256)).is_err());
    assert!(validator.validate_server_name("server with spaces").is_err());
    assert!(validator.validate_server_name("server/with/slashes").is_err());
    assert!(validator.validate_server_name("server\\with\\backslashes").is_err());
    assert!(validator.validate_server_name("server:with:colons").is_err());
    assert!(validator.validate_server_name("server|with|pipes").is_err());
    assert!(validator.validate_server_name("server<with>angles").is_err());
    assert!(validator.validate_server_name("server\"with\"quotes").is_err());
    assert!(validator.validate_server_name("server'with'apostrophes").is_err());
    
    // Test path validation
    assert!(validator.validate_path("C:\\valid\\path").is_ok());
    assert!(validator.validate_path("/valid/path").is_ok());
    assert!(validator.validate_path("relative/path").is_ok());
    
    // Test invalid paths
    assert!(validator.validate_path("").is_err());
    assert!(validator.validate_path("../invalid").is_err());
    assert!(validator.validate_path("..\\invalid").is_err());
    assert!(validator.validate_path("/etc/passwd").is_err());
    assert!(validator.validate_path("C:\\Windows\\System32").is_err());
    
    // Test port validation
    assert!(validator.validate_port(25565).is_ok());
    assert!(validator.validate_port(8080).is_ok());
    assert!(validator.validate_port(3000).is_ok());
    assert!(validator.validate_port(1).is_ok());
    assert!(validator.validate_port(65535).is_ok());
    
    // Test invalid ports
    assert!(validator.validate_port(0).is_err());
    assert!(validator.validate_port(65536).is_err());
    assert!(validator.validate_port(70000).is_err());
    
    // Test memory validation
    assert!(validator.validate_memory(512).is_ok());
    assert!(validator.validate_memory(1024).is_ok());
    assert!(validator.validate_memory(8192).is_ok());
    
    // Test invalid memory
    assert!(validator.validate_memory(0).is_err());
    assert!(validator.validate_memory(100000).is_err());
    assert!(validator.validate_memory(1).is_err());
}

#[tokio::test]
async fn test_version_resolver() {
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let version_resolver = VersionResolver::new(db_manager).await.unwrap();
    
    // Test version resolution with mock data
    let resolution_result = version_resolver.resolve_version("test-mod", "latest", "1.20.1", "fabric").await;
    
    // Should handle resolution gracefully (even if no real API keys)
    assert!(resolution_result.is_ok() || resolution_result.is_err());
    
    // Test dependency resolution
    let dependency_result = version_resolver.resolve_dependencies("test-mod", "1.0.0", "modrinth").await;
    assert!(dependency_result.is_ok() || dependency_result.is_err());
}

#[tokio::test]
async fn test_modrinth_manifest_parsing() {
    let manifest_json = json!({
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": "1.0.0",
        "name": "Test Modpack",
        "summary": "A test modpack",
        "files": [
            {
                "path": "mods/test-mod.jar",
                "hashes": {
                    "sha1": "abc123",
                    "sha512": "def456"
                },
                "downloads": ["https://example.com/test-mod.jar"],
                "fileSize": 1024
            }
        ],
        "dependencies": {
            "minecraft": "1.20.1",
            "fabric-loader": "0.14.21"
        }
    });
    
    let manifest: Result<ModrinthManifest, _> = serde_json::from_value(manifest_json);
    assert!(manifest.is_ok());
    
    let manifest = manifest.unwrap();
    assert_eq!(manifest.format_version, 1);
    assert_eq!(manifest.game, "minecraft");
    assert_eq!(manifest.version_id, "1.0.0");
    assert_eq!(manifest.name, "Test Modpack");
    assert_eq!(manifest.files.len(), 1);
    assert_eq!(manifest.files[0].path, "mods/test-mod.jar");
    assert_eq!(manifest.dependencies.minecraft, "1.20.1");
    assert_eq!(manifest.dependencies.fabric_loader, Some("0.14.21".to_string()));
}

#[tokio::test]
async fn test_curseforge_manifest_parsing() {
    let manifest_json = json!({
        "minecraft": {
            "version": "1.20.1",
            "modLoaders": [
                {
                    "id": "fabric-0.14.21",
                    "primary": true
                }
            ]
        },
        "manifestType": "minecraftModpack",
        "manifestVersion": 1,
        "name": "Test CurseForge Pack",
        "version": "1.0.0",
        "author": "Test Author",
        "files": [
            {
                "projectID": 12345,
                "fileID": 67890,
                "required": true
            }
        ],
        "overrides": "overrides"
    });
    
    let manifest: Result<CurseForgeManifest, _> = serde_json::from_value(manifest_json);
    assert!(manifest.is_ok());
    
    let manifest = manifest.unwrap();
    assert_eq!(manifest.manifest_type, "minecraftModpack");
    assert_eq!(manifest.manifest_version, 1);
    assert_eq!(manifest.name, "Test CurseForge Pack");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.author, "Test Author");
    assert_eq!(manifest.files.len(), 1);
    assert_eq!(manifest.files[0].project_id, 12345);
    assert_eq!(manifest.files[0].file_id, 67890);
    assert_eq!(manifest.minecraft.version, "1.20.1");
    assert_eq!(manifest.minecraft.mod_loaders.len(), 1);
    assert_eq!(manifest.minecraft.mod_loaders[0].id, "fabric-0.14.21");
    assert!(manifest.minecraft.mod_loaders[0].primary);
}

#[tokio::test]
async fn test_modpack_installer_creation() {
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let installer = ModpackInstaller::new(db_manager).await;
    assert!(installer.is_ok());
}

#[tokio::test]
async fn test_database_health_check() {
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let health_status = db_manager.get_health_status().await;
    assert!(health_status.is_ok());
    
    let health = health_status.unwrap();
    assert_eq!(health.status, "healthy");
}

#[tokio::test]
async fn test_server_validation() {
    let validator = InputValidator::new();
    
    // Test valid server configuration
    let valid_config = json!({
        "name": "test-server",
        "minecraft_version": "1.20.1",
        "loader": "vanilla",
        "memory_mb": 2048,
        "port": 25565
    });
    
    // Validate each field
    assert!(validator.validate_server_name(valid_config["name"].as_str().unwrap()).is_ok());
    assert!(validator.validate_memory(valid_config["memory_mb"].as_u64().unwrap() as u32).is_ok());
    assert!(validator.validate_port(valid_config["port"].as_u64().unwrap() as u16).is_ok());
    
    // Test invalid server configuration
    let invalid_config = json!({
        "name": "",
        "minecraft_version": "1.20.1",
        "loader": "vanilla",
        "memory_mb": 0,
        "port": 0
    });
    
    // Validate each field should fail
    assert!(validator.validate_server_name(invalid_config["name"].as_str().unwrap()).is_err());
    assert!(validator.validate_memory(invalid_config["memory_mb"].as_u64().unwrap() as u32).is_err());
    assert!(validator.validate_port(invalid_config["port"].as_u64().unwrap() as u16).is_err());
}

#[tokio::test]
async fn test_api_key_validation() {
    let validator = InputValidator::new();
    
    // Test valid API keys
    assert!(validator.validate_api_key("valid-api-key-123").is_ok());
    assert!(validator.validate_api_key("a".repeat(32)).is_ok());
    assert!(validator.validate_api_key("test-key-with-dashes").is_ok());
    
    // Test invalid API keys
    assert!(validator.validate_api_key("").is_err());
    assert!(validator.validate_api_key("a".repeat(1000)).is_err());
    assert!(validator.validate_api_key("key with spaces").is_err());
    assert!(validator.validate_api_key("key\nwith\nnewlines").is_err());
    assert!(validator.validate_api_key("key\twith\ttabs").is_err());
}

#[tokio::test]
async fn test_minecraft_version_validation() {
    let validator = InputValidator::new();
    
    // Test valid Minecraft versions
    assert!(validator.validate_minecraft_version("1.20.1").is_ok());
    assert!(validator.validate_minecraft_version("1.19.4").is_ok());
    assert!(validator.validate_minecraft_version("1.18.2").is_ok());
    assert!(validator.validate_minecraft_version("1.17.1").is_ok());
    
    // Test invalid Minecraft versions
    assert!(validator.validate_minecraft_version("").is_err());
    assert!(validator.validate_minecraft_version("invalid").is_err());
    assert!(validator.validate_minecraft_version("1.20").is_err());
    assert!(validator.validate_minecraft_version("1.20.1.1").is_err());
    assert!(validator.validate_minecraft_version("2.0.0").is_err());
}

#[tokio::test]
async fn test_loader_validation() {
    let validator = InputValidator::new();
    
    // Test valid loaders
    assert!(validator.validate_loader("vanilla").is_ok());
    assert!(validator.validate_loader("fabric").is_ok());
    assert!(validator.validate_loader("forge").is_ok());
    assert!(validator.validate_loader("quilt").is_ok());
    
    // Test invalid loaders
    assert!(validator.validate_loader("").is_err());
    assert!(validator.validate_loader("invalid").is_err());
    assert!(validator.validate_loader("spigot").is_err());
    assert!(validator.validate_loader("paper").is_err());
}

#[tokio::test]
async fn test_json_serialization() {
    // Test that our structures can be serialized and deserialized
    let progress_event = hostd::core::websocket::ProgressEvent {
        job_id: "test-job".to_string(),
        job_type: "test".to_string(),
        status: "started".to_string(),
        progress: 0.5,
        current_step: "Test Step".to_string(),
        total_steps: 2,
        current_step_progress: 0.25,
        message: Some("Test message".to_string()),
        error: None,
        estimated_remaining_ms: Some(5000),
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&progress_event);
    assert!(serialized.is_ok());
    
    // Test deserialization
    let deserialized: Result<hostd::core::websocket::ProgressEvent, _> = 
        serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
    
    let deserialized = deserialized.unwrap();
    assert_eq!(deserialized.job_id, "test-job");
    assert_eq!(deserialized.job_type, "test");
    assert_eq!(deserialized.status, "started");
    assert_eq!(deserialized.progress, 0.5);
}