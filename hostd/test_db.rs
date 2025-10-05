use sqlx::{SqlitePool, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite:guardian.db").await?;
    
    // Test a simple query
    let result = sqlx::query("SELECT 1 as test")
        .fetch_one(&pool)
        .await?;
    
    println!("Database connection successful: {}", result.get::<i32, _>("test"));
    
    pool.close().await;
    Ok(())
}
