use axum::{routing::{get, post}, Router};
use tracing_subscriber::{fmt, EnvFilter};
mod routes;
mod boot;

#[tokio::main]
async fn main() {
    let _ = fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // Try to reuse existing healthy hostd
    if let Some(h) = boot::try_attach_existing().await {
        println!("hostd already running on port {}", h.port);
        return;
    }

    // Try new port discovery first, fallback to original range
    let port = match boot::choose_port().await {
        0 => 8080, // fallback to original default if port discovery fails
        p => p,
    };

    // state: metrics hub etc.
    let hub = routes::metrics::MetricsHub::new();
    {
        let hub2 = hub.clone();
        tokio::spawn(async move {
            loop {
                hub2.push(routes::metrics::MetricsPoint { timestamp: chrono::Utc::now().timestamp_millis(), tps: 20.0, tick_p95_ms: 45.0, heap_mb: None, gpu_latency_ms: None }).await;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    // core routes
    let core = Router::new()
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

    let app = Router::new()
        .nest("/api", core.clone())
        .merge(core);

    // Bind with retry logic
    let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
        Ok(l) => l,
        Err(_) => {
            // Port unavailable, try again
            let fallback_port = boot::choose_port().await;
            let final_port = if fallback_port == 0 { 8080 } else { fallback_port };
            tokio::net::TcpListener::bind(("127.0.0.1", final_port)).await
                .unwrap_or_else(|_| panic!("Failed to bind to any port"))
        }
    };

    let actual_port = listener.local_addr().unwrap().port();
    boot::write_port_file(actual_port).ok();
    boot::write_pid_lock(std::process::id(), actual_port).ok();

    // Add health endpoint with actual port
    let health_router = boot::health_router(actual_port, std::process::id()).await;
    let app = app.merge(health_router);

    tracing::info!("hostd listening on http://127.0.0.1:{}", actual_port);
    axum::serve(listener, app).await.unwrap();
}
