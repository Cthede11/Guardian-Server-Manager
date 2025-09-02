use hostd::*;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "hostd")]
#[command(about = "Guardian Host Daemon - Process supervision and high availability")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/configs/hostd.yaml")]
    config: String,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Daemon mode
    #[arg(short, long)]
    daemon: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = args.log_level.parse::<tracing::Level>()?;
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    
    info!("Starting Guardian Host Daemon...");
    
    // Load configuration
    let config = Config::load(&args.config)?;
    info!("Configuration loaded from: {}", args.config);
    
    // Create host daemon
    let mut hostd = HostDaemon::new(config).await?;
    
    // Start the daemon
    if args.daemon {
        info!("Running in daemon mode");
        hostd.run_daemon().await?;
    } else {
        info!("Running in foreground mode");
        hostd.run().await?;
    }
    
    info!("Guardian Host Daemon stopped");
    Ok(())
}
