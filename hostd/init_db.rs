use sqlx::{SqlitePool, migrate::Migrator};
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("./db/migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:guardian.db".to_string());
    
    println!("Connecting to database: {}", database_url);
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    println!("Running migrations...");
    MIGRATOR.run(&pool).await?;
    
    println!("Database initialized successfully!");
    
    pool.close().await;
    Ok(())
}
