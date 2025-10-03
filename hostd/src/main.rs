use axum::{routing::{get, post, patch, delete}, Router, Json, extract::Path, http::StatusCode};
use tracing_subscriber::{fmt, EnvFilter};
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
mod routes;
mod boot;

#[tokio::main]
async fn main() {
    println!("Starting hostd...");
    let _ = fmt().with_env_filter(EnvFilter::from_default_env()).try_init();
    println!("Tracing initialized");

    // Try to reuse existing healthy hostd
    println!("Checking for existing hostd...");
    if let Some(h) = boot::try_attach_existing().await {
        println!("hostd already running on port {}", h.port);
        return;
    }
    println!("No existing hostd found, starting new instance");

    // Try new port discovery first, fallback to original range
    println!("Choosing port...");
    let port = match boot::choose_port().await {
        0 => 8080, // fallback to original default if port discovery fails
        p => p,
    };
    println!("Selected port: {}", port);

    // state: metrics hub etc.
    println!("Creating metrics hub...");
    let hub = routes::metrics::MetricsHub::new();
    {
        let hub2 = hub.clone();
        println!("Spawning metrics task...");
        tokio::spawn(async move {
            loop {
                hub2.push(routes::metrics::MetricsPoint { timestamp: chrono::Utc::now().timestamp_millis(), tps: 20.0, tick_p95_ms: 45.0, heap_mb: None, gpu_latency_ms: None }).await;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }
    println!("Metrics hub created");

    // core routes
    println!("Setting up routes...");
    let core = Router::new()
        // Basic server endpoints (minimal implementation)
        .route("/servers", get(get_servers_minimal))
        .route("/servers", post(create_server_minimal))
        .route("/servers/:id", get(get_server_minimal))
        .route("/servers/:id", patch(update_server_minimal))
        .route("/servers/:id", delete(delete_server_minimal))
        // Existing specialized routes
        .route("/servers/:id/world", get(routes::world::get_world))
        .route("/servers/:id/dimensions", get(routes::world::list_dimensions))
        .route("/servers/:id/pregen/status", get(routes::pregen::status))
        .route("/servers/:id/pregen/plan", post(routes::pregen::plan))
        .route("/servers/:id/pregen/start", post(routes::pregen::start))
        .route("/servers/:id/pregen/pause", post(routes::pregen::pause))
        .route("/servers/:id/pregen/resume", post(routes::pregen::resume))
        .route("/servers/:id/pregen/cancel", post(routes::pregen::cancel))
        .route("/servers/:id/import/scan", post(routes::import::scan))
        .route("/servers/:id/import/apply", post(routes::import::apply))
        .route("/servers/:id/metrics", get(routes::metrics::history))
        .route("/servers/:id/metrics/stream", get(routes::metrics::stream))
        .with_state(hub.clone());
    println!("Routes configured");

    let app = Router::new()
        .nest("/api", core.clone())
        .merge(core)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );
    println!("App router created with CORS support");

    // Bind with retry logic
    println!("Binding to port {}...", port);
    let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
        Ok(l) => {
            println!("Successfully bound to port {}", port);
            l
        },
        Err(e) => {
            println!("Failed to bind to port {}: {:?}", port, e);
            // Port unavailable, try again
            let fallback_port = boot::choose_port().await;
            let final_port = if fallback_port == 0 { 8080 } else { fallback_port };
            println!("Trying fallback port {}...", final_port);
            tokio::net::TcpListener::bind(("127.0.0.1", final_port)).await
                .unwrap_or_else(|_| panic!("Failed to bind to any port"))
        }
    };

    let actual_port = listener.local_addr().unwrap().port();
    println!("Writing port file for port {}...", actual_port);
    boot::write_port_file(actual_port).ok();
    boot::write_pid_lock(std::process::id(), actual_port).ok();

    // Add health endpoint with actual port
    println!("Adding health endpoint...");
    let health_router = boot::health_router(actual_port, std::process::id()).await;
    let app = app.merge(health_router);

    println!("hostd listening on http://127.0.0.1:{}", actual_port);
    tracing::info!("hostd listening on http://127.0.0.1:{}", actual_port);
    axum::serve(listener, app).await.unwrap();
}

// Minimal server data structures
#[derive(Serialize, Deserialize, Clone)]
struct ServerInfo {
    id: String,
    name: String,
    status: String,
    tps: f32,
    tick_p95: f32,
    heap_mb: u32,
    players_online: u32,
    gpu_queue_ms: f32,
    last_snapshot_at: Option<String>,
    blue_green: BlueGreenInfo,
    version: Option<String>,
    max_players: Option<u32>,
    uptime: Option<u64>,
    memory_usage: Option<u32>,
    cpu_usage: Option<f32>,
    world_size: Option<u64>,
    last_backup: Option<String>,
    auto_start: Option<bool>,
    config: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
struct BlueGreenInfo {
    active: String,
    candidate_healthy: bool,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Minimal server endpoints
async fn get_servers_minimal() -> Json<ApiResponse<Vec<ServerInfo>>> {
    // Return empty list for now - this is a minimal implementation
    Json(ApiResponse {
        success: true,
        data: Some(vec![]),
        error: None,
    })
}

async fn create_server_minimal(Json(_server_data): Json<serde_json::Value>) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // Create a dummy server for testing
    let server = ServerInfo {
        id: "test-server-1".to_string(),
        name: "Test Server".to_string(),
        status: "stopped".to_string(),
        tps: 20.0,
        tick_p95: 45.2,
        heap_mb: 2048,
        players_online: 0,
        gpu_queue_ms: 0.0,
        last_snapshot_at: None,
        blue_green: BlueGreenInfo {
            active: "blue".to_string(),
            candidate_healthy: false,
        },
        version: Some("1.20.1".to_string()),
        max_players: Some(20),
        uptime: None,
        memory_usage: Some(2048),
        cpu_usage: None,
        world_size: None,
        last_backup: None,
        auto_start: Some(false),
        config: None,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(server),
        error: None,
    }))
}

async fn get_server_minimal(Path(id): Path<String>) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // Return a dummy server for testing
    let server = ServerInfo {
        id: id.clone(),
        name: format!("Server {}", id),
        status: "stopped".to_string(),
        tps: 20.0,
        tick_p95: 45.2,
        heap_mb: 2048,
        players_online: 0,
        gpu_queue_ms: 0.0,
        last_snapshot_at: None,
        blue_green: BlueGreenInfo {
            active: "blue".to_string(),
            candidate_healthy: false,
        },
        version: Some("1.20.1".to_string()),
        max_players: Some(20),
        uptime: None,
        memory_usage: Some(2048),
        cpu_usage: None,
        world_size: None,
        last_backup: None,
        auto_start: Some(false),
        config: None,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(server),
        error: None,
    }))
}

async fn update_server_minimal(Path(_id): Path<String>, Json(_server_data): Json<serde_json::Value>) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // Return success for now
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
    }))
}

async fn delete_server_minimal(Path(_id): Path<String>) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // Return success for now
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
    }))
}
