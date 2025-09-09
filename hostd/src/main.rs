use axum::{routing::{get, post}, Router};
use tracing_subscriber::{fmt, EnvFilter};
mod routes;
mod boot;

#[tokio::main]
async fn main() {
  let _ = fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

  // Reuse live instance if healthy
  if let Some(info) = boot::try_attach_existing().await {
    println!("hostd: already running on port {}", info.port);
    return;
  }

  // Choose free port
  let port = boot::choose_port().await;
  boot::write_port_file(port).ok();
  boot::write_pid_lock(std::process::id(), port).ok();

  // Metrics hub (state) + fake sampler until real agent wires in
  let hub = routes::metrics::MetricsHub::new();
  let hub_clone = hub.clone();
  tokio::spawn(async move {
    loop {
      let now = chrono::Utc::now().timestamp_millis();
      hub_clone.push(routes::metrics::MetricsPoint {
        timestamp: now, tps: 20.0, tick_p95_ms: 45.0, heap_mb: None, gpu_latency_ms: None
      }).await;
      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
  });

  // Core app (no /api)
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

  // Health router
  let health = boot::health_router(port, std::process::id()).await;

  // Final app: keep old and new paths; both served
  let app = Router::new()
    .nest("/api", core.clone())
    .merge(core)
    .merge(health);

  let addr = std::net::SocketAddr::from(([127,0,0,1], port));
  tracing::info!("hostd listening on {}", addr);
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
  axum::serve(listener, app).await.unwrap();
}
