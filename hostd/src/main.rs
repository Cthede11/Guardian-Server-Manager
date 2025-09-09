use axum::{routing::{get, post}, Router};
use tracing_subscriber::{fmt, EnvFilter};
use chrono;

mod routes;

#[tokio::main]
async fn main() {
    // logging
    let _ = fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // metrics hub & fake sampler (replace with real supervisor sampling)
    let hub = routes::metrics::MetricsHub::new();
    let hub_clone = hub.clone();
    tokio::spawn(async move {
        loop {
            let now = chrono::Utc::now().timestamp_millis();
            let _ = hub_clone.push(routes::metrics::MetricsPoint {
                timestamp: now,
                tps: 20.0,
                tick_p95_ms: 45.0,
                heap_mb: None,
                gpu_latency_ms: None,
            }).await;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let app = Router::new()
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
        .with_state(hub) // state for metrics routes
        ;

    // Try to find an available port starting from 8080
    let mut port = 8080;
    let mut listener = loop {
        let addr = std::net::SocketAddr::from(([127,0,0,1], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("🚀 Guardian Host Daemon starting on {}", addr);
                tracing::info!("hostd listening on {}", addr);
                
                // Write the port to a file for the frontend to read
                if let Err(e) = std::fs::write("backend_port.txt", port.to_string()) {
                    eprintln!("⚠️  Warning: Could not write port file: {}", e);
                }
                
                break listener;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                println!("⚠️  Port {} is in use, trying port {}...", port, port + 1);
                port += 1;
                if port > 8090 {
                    eprintln!("❌ Could not find an available port between 8080-8090");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to bind to port {}: {}", port, e);
                std::process::exit(1);
            }
        }
    };

    println!("📡 Available endpoints:");
    println!("  GET  /servers/:id/world");
    println!("  GET  /servers/:id/dimensions");
    println!("  GET  /servers/:id/pregen/status");
    println!("  POST /servers/:id/pregen/start");
    println!("  GET  /servers/:id/metrics");
    println!("  GET  /servers/:id/metrics/stream");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
