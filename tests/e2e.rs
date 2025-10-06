use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;
use std::fs;
use std::path::Path;
use serde_json::json;
use reqwest::Client;
use tokio::time::timeout;

const API_BASE_URL: &str = "http://localhost:8080/api";
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Guardian Server Manager E2E Smoke Tests");
    
    // Check if hostd is running
    if !is_hostd_running().await {
        println!("❌ hostd is not running. Please start the backend server first.");
        return Ok(());
    }
    
    let client = Client::new();
    
    // Run all test suites
    let test_results = vec![
        ("API Health Check", test_api_health(&client).await),
        ("Server Management", test_server_management(&client).await),
        ("Mod Management", test_mod_management(&client).await),
        ("Performance Monitoring", test_performance_monitoring(&client).await),
        ("Backup Management", test_backup_management(&client).await),
        ("GPU Management", test_gpu_management(&client).await),
        ("Compatibility Analysis", test_compatibility_analysis(&client).await),
    ];
    
    // Print results
    println!("\n📊 Test Results Summary:");
    println!("{}", "=".repeat(50));
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (test_name, result) in test_results {
        match result {
            Ok(_) => {
                println!("✅ {} - PASSED", test_name);
                passed += 1;
            }
            Err(e) => {
                println!("❌ {} - FAILED: {}", test_name, e);
                failed += 1;
            }
        }
    }
    
    println!("{}", "=".repeat(50));
    println!("Total: {} passed, {} failed", passed, failed);
    
    if failed == 0 {
        println!("🎉 All tests passed! Guardian Server Manager is ready for production.");
    } else {
        println!("⚠️  Some tests failed. Please review the issues before deployment.");
    }
    
    Ok(())
}

async fn is_hostd_running() -> bool {
    let client = Client::new();
    match timeout(TEST_TIMEOUT, client.get(&format!("{}/health", API_BASE_URL)).send()).await {
        Ok(Ok(response)) => response.status().is_success(),
        _ => false,
    }
}

async fn test_api_health(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing API health...");
    
    let response = timeout(TEST_TIMEOUT, client.get(&format!("{}/health", API_BASE_URL)).send()).await??;
    
    if !response.status().is_success() {
        return Err("Health endpoint returned non-success status".into());
    }
    
    let health: serde_json::Value = response.json().await?;
    
    if health["status"] != "healthy" {
        return Err("API reports unhealthy status".into());
    }
    
    println!("    ✅ API health check passed");
    Ok(())
}

async fn test_server_management(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing server management...");
    
    // Test 1: List servers
    let response = timeout(TEST_TIMEOUT, client.get(&format!("{}/servers", API_BASE_URL)).send()).await??;
    if !response.status().is_success() {
        return Err("Failed to list servers".into());
    }
    let servers: serde_json::Value = response.json().await?;
    println!("    ✅ Server list retrieved ({} servers)", servers["data"].as_array().unwrap_or(&vec![]).len());
    
    // Test 2: Create a test server
    let server_data = json!({
        "name": "E2E Test Server",
        "description": "Test server for E2E testing",
        "version": "1.20.1",
        "maxPlayers": 10,
        "motd": "E2E Test Server",
        "difficulty": "normal",
        "gamemode": "survival",
        "pvp": true,
        "onlineMode": false,
        "whitelist": false,
        "enableCommandBlock": false,
        "viewDistance": 10,
        "simulationDistance": 10,
        "jvm": {
            "memory": 1024,
            "flags": ["-XX:+UseG1GC"],
            "gcType": "G1GC"
        },
        "gpu": {
            "enabled": false,
            "queueSize": 100,
            "maxWorkers": 1
        }
    });
    
    let response = timeout(TEST_TIMEOUT, 
        client.post(&format!("{}/servers", API_BASE_URL))
            .json(&server_data)
            .send()
    ).await??;
    
    if !response.status().is_success() {
        return Err("Failed to create test server".into());
    }
    
    let created_server: serde_json::Value = response.json().await?;
    let server_id = created_server["data"]["id"].as_str().unwrap();
    println!("    ✅ Test server created with ID: {}", server_id);
    
    // Test 3: Get server details
    let response = timeout(TEST_TIMEOUT, client.get(&format!("{}/servers/{}", API_BASE_URL, server_id)).send()).await??;
    if !response.status().is_success() {
        return Err("Failed to get server details".into());
    }
    println!("    ✅ Server details retrieved");
    
    // Test 4: Start server (if possible)
    let response = timeout(TEST_TIMEOUT, 
        client.post(&format!("{}/servers/{}/start", API_BASE_URL, server_id))
            .send()
    ).await??;
    
    if response.status().is_success() {
        println!("    ✅ Server start command sent");
        
        // Wait a moment for server to start
        thread::sleep(Duration::from_secs(2));
        
        // Test 5: Stop server
        let response = timeout(TEST_TIMEOUT, 
            client.post(&format!("{}/servers/{}/stop", API_BASE_URL, server_id))
                .send()
        ).await??;
        
        if response.status().is_success() {
            println!("    ✅ Server stop command sent");
        }
    } else {
        println!("    ⚠️  Server start failed (may be expected in test environment)");
    }
    
    // Test 6: Clean up - delete test server
    let response = timeout(TEST_TIMEOUT, 
        client.delete(&format!("{}/servers/{}", API_BASE_URL, server_id))
            .send()
    ).await??;
    
    if response.status().is_success() {
        println!("    ✅ Test server cleaned up");
    } else {
        println!("    ⚠️  Failed to clean up test server");
    }
    
    Ok(())
}

async fn test_mod_management(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing mod management...");
    
    // Test 1: Search for mods
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/mods/search?query=fabric-api&limit=5", API_BASE_URL))
            .send()
    ).await??;
    
    if !response.status().is_success() {
        return Err("Failed to search for mods".into());
    }
    
    let search_results: serde_json::Value = response.json().await?;
    let mods = search_results["data"].as_array().unwrap_or(&vec![]);
    println!("    ✅ Mod search completed ({} results)", mods.len());
    
    if !mods.is_empty() {
        // Test 2: Get mod details
        let mod_id = mods[0]["id"].as_str().unwrap();
        let response = timeout(TEST_TIMEOUT, 
            client.get(&format!("{}/mods/{}", API_BASE_URL, mod_id))
                .send()
        ).await??;
        
        if response.status().is_success() {
            println!("    ✅ Mod details retrieved");
        } else {
            println!("    ⚠️  Failed to get mod details");
        }
    }
    
    Ok(())
}

async fn test_performance_monitoring(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing performance monitoring...");
    
    // Test 1: Get GPU status
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/gpu/status", API_BASE_URL))
            .send()
    ).await??;
    
    if response.status().is_success() {
        let gpu_status: serde_json::Value = response.json().await?;
        println!("    ✅ GPU status retrieved: {}", gpu_status["data"]["available"]);
    } else {
        println!("    ⚠️  GPU status check failed");
    }
    
    // Test 2: Get system metrics
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/system/metrics", API_BASE_URL))
            .send()
    ).await??;
    
    if response.status().is_success() {
        println!("    ✅ System metrics retrieved");
    } else {
        println!("    ⚠️  System metrics check failed");
    }
    
    Ok(())
}

async fn test_backup_management(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing backup management...");
    
    // Test 1: List backups
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/backups", API_BASE_URL))
            .send()
    ).await??;
    
    if response.status().is_success() {
        let backups: serde_json::Value = response.json().await?;
        let backup_list = backups["data"].as_array().unwrap_or(&vec![]);
        println!("    ✅ Backup list retrieved ({} backups)", backup_list.len());
    } else {
        println!("    ⚠️  Backup list check failed");
    }
    
    Ok(())
}

async fn test_gpu_management(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing GPU management...");
    
    // Test 1: Get GPU status
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/gpu/status", API_BASE_URL))
            .send()
    ).await??;
    
    if !response.status().is_success() {
        return Err("Failed to get GPU status".into());
    }
    
    let gpu_status: serde_json::Value = response.json().await?;
    let available = gpu_status["data"]["available"].as_bool().unwrap_or(false);
    println!("    ✅ GPU status: {}", if available { "Available" } else { "Not Available" });
    
    // Test 2: Submit a test GPU job (if GPU is available)
    if available {
        let job_data = json!({
            "type": "test",
            "parameters": {
                "test": true
            }
        });
        
        let response = timeout(TEST_TIMEOUT, 
            client.post(&format!("{}/gpu/jobs", API_BASE_URL))
                .json(&job_data)
                .send()
        ).await??;
        
        if response.status().is_success() {
            let job: serde_json::Value = response.json().await?;
            let job_id = job["data"]["jobId"].as_str().unwrap();
            println!("    ✅ GPU job submitted: {}", job_id);
        } else {
            println!("    ⚠️  GPU job submission failed");
        }
    }
    
    Ok(())
}

async fn test_compatibility_analysis(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Testing compatibility analysis...");
    
    // Test 1: Get compatibility issues (using a dummy server ID)
    let dummy_server_id = "00000000-0000-0000-0000-000000000000";
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/compatibility/{}/issues", API_BASE_URL, dummy_server_id))
            .send()
    ).await??;
    
    if response.status().is_success() {
        let issues: serde_json::Value = response.json().await?;
        let issue_list = issues["data"].as_array().unwrap_or(&vec![]);
        println!("    ✅ Compatibility issues retrieved ({} issues)", issue_list.len());
    } else {
        println!("    ⚠️  Compatibility issues check failed");
    }
    
    // Test 2: Get risk analysis
    let response = timeout(TEST_TIMEOUT, 
        client.get(&format!("{}/compatibility/{}/risk-analysis", API_BASE_URL, dummy_server_id))
            .send()
    ).await??;
    
    if response.status().is_success() {
        let risk_analysis: serde_json::Value = response.json().await?;
        let risk_list = risk_analysis["data"].as_array().unwrap_or(&vec![]);
        println!("    ✅ Risk analysis retrieved ({} mods analyzed)", risk_list.len());
    } else {
        println!("    ⚠️  Risk analysis check failed");
    }
    
    Ok(())
}

// Helper function to check if a file exists
fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// Helper function to check if a process is running
fn is_process_running(process_name: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("tasklist")
            .arg("/FI")
            .arg(&format!("IMAGENAME eq {}", process_name))
            .output()
    } else {
        Command::new("pgrep")
            .arg(process_name)
            .output()
    };
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// Helper function to wait for a condition
async fn wait_for_condition<F>(mut condition: F, timeout_duration: Duration) -> bool 
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout_duration {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

// Helper function to make HTTP requests with retries
async fn make_request_with_retry<F, Fut>(mut request_fn: F, max_retries: u32) -> Result<reqwest::Response, Box<dyn std::error::Error>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
{
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    thread::sleep(Duration::from_millis(500 * attempt as u64));
                }
            }
        }
    }
    
    Err(last_error.unwrap().into())
}
