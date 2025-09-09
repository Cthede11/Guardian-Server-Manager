use hostd::boot;

#[tokio::main]
async fn main() {
    println!("Testing port selection...");
    
    // Test port selection
    let port = boot::choose_port().await;
    println!("Selected port: {}", port);
    
    // Test port file writing
    if let Err(e) = boot::write_port_file(port) {
        println!("Failed to write port file: {}", e);
    } else {
        println!("Port file written successfully");
    }
    
    // Test health check
    if let Some(health) = boot::try_attach_existing().await {
        println!("Found existing healthy instance: {:?}", health);
    } else {
        println!("No existing healthy instance found");
    }
    
    // Test basic server
    let health_router = boot::health_router(port, std::process::id()).await;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    let actual_port = listener.local_addr().unwrap().port();
    
    println!("Starting health server on port {}", actual_port);
    boot::write_port_file(actual_port).ok();
    
    println!("Server ready! Test with: curl http://127.0.0.1:{}/healthz", actual_port);
    println!("Press Ctrl+C to stop");
    
    axum::serve(listener, health_router).await.unwrap();
}
