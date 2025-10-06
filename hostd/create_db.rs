use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "sqlite:data/guardian.db";
    let pool = SqlitePool::connect(database_url).await?;
    
    // Create the servers table with the correct schema
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            rcon_port INTEGER NOT NULL,
            rcon_password TEXT NOT NULL,
            java_path TEXT NOT NULL,
            server_jar TEXT NOT NULL,
            jvm_args TEXT NOT NULL,
            server_args TEXT NOT NULL,
            auto_start BOOLEAN NOT NULL DEFAULT 0,
            auto_restart BOOLEAN NOT NULL DEFAULT 0,
            max_players INTEGER NOT NULL,
            minecraft_version TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#)
    .execute(&pool)
    .await?;
    
    println!("Database created successfully!");
    Ok(())
}
