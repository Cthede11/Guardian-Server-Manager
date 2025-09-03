use hostd::*;
use tokio;
use std::path::PathBuf;

#[tokio::test]
async fn test_host_daemon_initialization() {
    let config = Config::load("configs/server.yaml").await.unwrap();
    let daemon = HostDaemon::new(config).await;
    assert!(daemon.is_ok());
}

#[tokio::test]
async fn test_auth_manager() {
    let config = Config::load("configs/server.yaml").await.unwrap();
    let auth_manager = AuthManager::new(config);
    auth_manager.initialize().await.unwrap();
    
    // Test authentication
    let token = auth_manager.authenticate("admin", "admin123", "127.0.0.1").await;
    assert!(token.is_ok());
    
    // Test token validation
    let claims = auth_manager.validate_token(&token.unwrap());
    assert!(claims.is_ok());
}

#[tokio::test]
async fn test_tenant_manager() {
    let tenant_manager = TenantManager::new();
    
    // Create a tenant
    let config = TenantConfig {
        minecraft_version: "1.20.1".to_string(),
        modpack_id: Some("test-pack".to_string()),
        server_properties: std::collections::HashMap::new(),
        guardian_config: Config::default(),
        custom_rules: None,
        plugins: Vec::new(),
    };
    
    let tenant = tenant_manager.create_tenant("Test Tenant".to_string(), "user1".to_string(), config).await;
    assert!(tenant.is_ok());
    
    let tenant_id = tenant.unwrap().id;
    
    // Create an instance
    let instance_config = InstanceConfig {
        port: 25565,
        world_name: "test-world".to_string(),
        difficulty: "normal".to_string(),
        gamemode: "survival".to_string(),
        max_players: 20,
        motd: "Test Server".to_string(),
        whitelist_enabled: false,
        whitelist: Vec::new(),
        ops: Vec::new(),
    };
    
    let instance = tenant_manager.create_instance(&tenant_id, "Test Instance".to_string(), instance_config).await;
    assert!(instance.is_ok());
}

#[tokio::test]
async fn test_plugin_manager() {
    let plugin_dir = PathBuf::from("./test-plugins");
    let plugin_manager = PluginManager::new(plugin_dir);
    plugin_manager.initialize().await.unwrap();
    
    // Test plugin listing
    let plugins = plugin_manager.list_plugins().await;
    assert!(plugins.is_empty()); // Should be empty initially
}

#[tokio::test]
async fn test_webhook_manager() {
    let webhook_manager = WebhookManager::new();
    webhook_manager.initialize().await.unwrap();
    
    // Create a webhook
    let webhook = webhook_manager.create_webhook(
        "Test Webhook".to_string(),
        "https://example.com/webhook".to_string(),
        vec!["server.start".to_string(), "server.stop".to_string()]
    ).await;
    assert!(webhook.is_ok());
    
    // Test event publishing
    let result = webhook_manager.publish_event(
        "server.start",
        serde_json::json!({"server_id": "test-server"}),
        "test"
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_compliance_manager() {
    let compliance_manager = ComplianceManager::new();
    compliance_manager.initialize().await.unwrap();
    
    // Test GDPR compliance report
    let report = compliance_manager.get_compliance_report(ComplianceFramework::GDPR).await;
    assert!(report.is_ok());
    
    let report = report.unwrap();
    assert!(report.compliance_score >= 0.0 && report.compliance_score <= 100.0);
}

#[tokio::test]
async fn test_community_manager() {
    let community_manager = CommunityManager::new();
    community_manager.initialize().await.unwrap();
    
    // Test mod search
    let filters = ModSearchFilters::default();
    let results = community_manager.search_mods("create", filters).await;
    assert!(!results.is_empty()); // Should have sample mods
}

#[tokio::test]
async fn test_ai_manager() {
    let ai_manager = AIManager::new();
    ai_manager.initialize().await.unwrap();
    
    // Test crash prediction
    let metrics = PerformanceMetrics::default();
    let prediction = ai_manager.predict_crash("test-server", metrics).await;
    assert!(prediction.is_ok());
    
    let prediction = prediction.unwrap();
    assert!(prediction.crash_probability >= 0.0 && prediction.crash_probability <= 1.0);
    
    // Test anomaly detection
    let anomalies = ai_manager.detect_anomalies("test-server", metrics).await;
    assert!(anomalies.is_ok());
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Initialize all managers
    let config = Config::load("configs/server.yaml").await.unwrap();
    let auth_manager = AuthManager::new(config.clone());
    auth_manager.initialize().await.unwrap();
    
    let tenant_manager = TenantManager::new();
    let plugin_manager = PluginManager::new(PathBuf::from("./test-plugins"));
    plugin_manager.initialize().await.unwrap();
    
    let webhook_manager = WebhookManager::new();
    webhook_manager.initialize().await.unwrap();
    
    let compliance_manager = ComplianceManager::new();
    compliance_manager.initialize().await.unwrap();
    
    let community_manager = CommunityManager::new();
    community_manager.initialize().await.unwrap();
    
    let ai_manager = AIManager::new();
    ai_manager.initialize().await.unwrap();
    
    // Simulate a complete workflow
    // 1. Authenticate user
    let token = auth_manager.authenticate("admin", "admin123", "127.0.0.1").await.unwrap();
    let claims = auth_manager.validate_token(&token).unwrap();
    
    // 2. Create tenant
    let tenant_config = TenantConfig {
        minecraft_version: "1.20.1".to_string(),
        modpack_id: Some("test-pack".to_string()),
        server_properties: std::collections::HashMap::new(),
        guardian_config: config,
        custom_rules: None,
        plugins: Vec::new(),
    };
    
    let tenant = tenant_manager.create_tenant("Test Tenant".to_string(), claims.sub, tenant_config).await.unwrap();
    
    // 3. Create webhook for notifications
    let webhook = webhook_manager.create_webhook(
        "Tenant Notifications".to_string(),
        "https://example.com/tenant-webhook".to_string(),
        vec!["tenant.created".to_string(), "instance.started".to_string()]
    ).await.unwrap();
    
    // 4. Publish tenant creation event
    webhook_manager.publish_event(
        "tenant.created",
        serde_json::json!({
            "tenant_id": tenant.id,
            "name": tenant.name
        }),
        "tenant_manager"
    ).await.unwrap();
    
    // 5. Get AI insights for the tenant
    let insights = ai_manager.get_ai_insights(&tenant.id).await.unwrap();
    assert_eq!(insights.server_id, tenant.id);
    
    // 6. Check compliance
    let compliance_report = compliance_manager.get_compliance_report(ComplianceFramework::GDPR).await.unwrap();
    assert!(compliance_report.compliance_score > 0.0);
    
    println!("End-to-end workflow completed successfully!");
}
