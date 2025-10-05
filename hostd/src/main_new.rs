use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{fmt, EnvFilter};

use hostd::core::{
    app_state::AppState,
    config::Config,
    api::create_api_router,
    websocket::WebSocketManager,
    errors::Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Guardian Server Manager...");

    // Load configuration
    let config = Config::load()
        .map_err(|e| hostd::core::errors::AppError::Internal(format!("Failed to load config: {}", e)))?;

    // Create application state
    let app_state = Arc::new(AppState::new(config).await?);

    // Start the application
    app_state.start().await?;

    // Create the main router
    let app = Router::new()
        .route("/", get(|| async { "Guardian Server Manager API" }))
        .nest("/api", create_api_router())
        .route("/ws", get(WebSocketManager::handle_websocket))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(app_state.clone());

    // Get the server address
    let addr = app_state.config.server_addr();
    
    tracing::info!("Guardian Server Manager listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| hostd::core::errors::AppError::Network(format!("Failed to bind to address: {}", e)))?;

    axum::serve(listener, app).await
        .map_err(|e| hostd::core::errors::AppError::Network(format!("Server error: {}", e)))?;

    // Cleanup on shutdown
    app_state.stop().await?;

    Ok(())
}
