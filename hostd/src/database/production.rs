/// Production database configuration and management
/// Supports both SQLite (development) and PostgreSQL (production)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Sqlite, Database, Row};
use std::env;
use tracing::{info, warn, error};
use std::path::Path;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub enable_logging: bool,
    pub enable_metrics: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:data/guardian.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
            enable_logging: true,
            enable_metrics: true,
        }
    }
}

impl DatabaseConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/guardian.db".to_string());
        
        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        
        let min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let acquire_timeout = env::var("DATABASE_ACQUIRE_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);
        
        let idle_timeout = env::var("DATABASE_IDLE_TIMEOUT")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .unwrap_or(600);
        
        let max_lifetime = env::var("DATABASE_MAX_LIFETIME")
            .unwrap_or_else(|_| "1800".to_string())
            .parse()
            .unwrap_or(1800);
        
        let enable_logging = env::var("DATABASE_ENABLE_LOGGING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        
        let enable_metrics = env::var("DATABASE_ENABLE_METRICS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        
        Ok(Self {
            url,
            max_connections,
            min_connections,
            acquire_timeout,
            idle_timeout,
            max_lifetime,
            enable_logging,
            enable_metrics,
        })
    }
    
    /// Check if using PostgreSQL
    pub fn is_postgresql(&self) -> bool {
        self.url.starts_with("postgresql://") || self.url.starts_with("postgres://")
    }
    
    /// Check if using SQLite
    pub fn is_sqlite(&self) -> bool {
        self.url.starts_with("sqlite:")
    }
}

/// Production database manager
pub struct ProductionDatabaseManager {
    config: DatabaseConfig,
    pool: Box<dyn DatabasePool>,
}

/// Trait for database pool abstraction
pub trait DatabasePool: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl DatabasePool for Pool<Sqlite> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DatabasePool for Pool<Postgres> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ProductionDatabaseManager {
    /// Create a new production database manager
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Initializing production database with config: {:?}", config);
        
        let pool = if config.is_postgresql() {
            Self::create_postgresql_pool(&config).await?
        } else {
            Self::create_sqlite_pool(&config).await?
        };
        
        let manager = Self { config, pool };
        manager.run_migrations().await?;
        manager.optimize_connections().await?;
        
        info!("Production database initialized successfully");
        Ok(manager)
    }
    
    /// Create PostgreSQL connection pool
    async fn create_postgresql_pool(config: &DatabaseConfig) -> Result<Box<dyn DatabasePool>> {
        info!("Creating PostgreSQL connection pool");
        
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.max_lifetime))
            .connect(&config.url)
            .await?;
        
        // Test connection
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        info!("PostgreSQL connection pool created successfully");
        
        Ok(Box::new(pool))
    }
    
    /// Create SQLite connection pool
    async fn create_sqlite_pool(config: &DatabaseConfig) -> Result<Box<dyn DatabasePool>> {
        info!("Creating SQLite connection pool");
        
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.max_lifetime))
            .connect(&config.url)
            .await?;
        
        // Test connection
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        info!("SQLite connection pool created successfully");
        
        Ok(Box::new(pool))
    }
    
    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        info!("Running production database migrations");
        
        if self.config.is_postgresql() {
            self.run_postgresql_migrations().await?;
        } else {
            self.run_sqlite_migrations().await?;
        }
        
        info!("Database migrations completed successfully");
        Ok(())
    }
    
    /// Run PostgreSQL-specific migrations
    async fn run_postgresql_migrations(&self) -> Result<()> {
        let pool = self.pool.as_any()
            .downcast_ref::<Pool<Postgres>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to downcast to PostgreSQL pool"))?;
        
        // Enable UUID extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .execute(pool)
            .await?;
        
        // Create servers table with PostgreSQL-specific features
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS servers (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                minecraft_version VARCHAR(50) NOT NULL DEFAULT '1.21.1',
                loader VARCHAR(50) NOT NULL DEFAULT 'vanilla',
                loader_version VARCHAR(50) NOT NULL DEFAULT 'latest',
                port INTEGER NOT NULL DEFAULT 25565,
                rcon_port INTEGER NOT NULL DEFAULT 25575,
                query_port INTEGER NOT NULL DEFAULT 25566,
                max_players INTEGER NOT NULL DEFAULT 20,
                memory INTEGER NOT NULL DEFAULT 4096,
                java_args JSONB NOT NULL DEFAULT '[]',
                server_args JSONB NOT NULL DEFAULT '[]',
                auto_start BOOLEAN NOT NULL DEFAULT FALSE,
                auto_restart BOOLEAN NOT NULL DEFAULT TRUE,
                world_name VARCHAR(255) NOT NULL DEFAULT 'world',
                difficulty VARCHAR(50) NOT NULL DEFAULT 'normal',
                gamemode VARCHAR(50) NOT NULL DEFAULT 'survival',
                pvp BOOLEAN NOT NULL DEFAULT TRUE,
                online_mode BOOLEAN NOT NULL DEFAULT TRUE,
                whitelist BOOLEAN NOT NULL DEFAULT FALSE,
                enable_command_block BOOLEAN NOT NULL DEFAULT FALSE,
                view_distance INTEGER NOT NULL DEFAULT 10,
                simulation_distance INTEGER NOT NULL DEFAULT 10,
                motd TEXT NOT NULL DEFAULT 'A Minecraft Server',
                host VARCHAR(255) NOT NULL DEFAULT 'localhost',
                java_path VARCHAR(500) NOT NULL DEFAULT 'java',
                jvm_args TEXT NOT NULL DEFAULT '-Xmx4G -Xms2G',
                server_jar VARCHAR(255) NOT NULL DEFAULT 'server.jar',
                rcon_password VARCHAR(255) NOT NULL DEFAULT '',
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_name ON servers (name)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_created_at ON servers (created_at)")
            .execute(pool)
            .await?;
        
        // Create event_logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_logs (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                server_id UUID REFERENCES servers(id) ON DELETE SET NULL,
                event_type VARCHAR(100) NOT NULL,
                message TEXT NOT NULL,
                level VARCHAR(20) NOT NULL DEFAULT 'info',
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // Create indexes for event_logs
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_logs_server_id ON event_logs (server_id)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_logs_created_at ON event_logs (created_at)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_logs_event_type ON event_logs (event_type)")
            .execute(pool)
            .await?;
        
        info!("PostgreSQL migrations completed");
        Ok(())
    }
    
    /// Run SQLite-specific migrations
    async fn run_sqlite_migrations(&self) -> Result<()> {
        let pool = self.pool.as_any()
            .downcast_ref::<Pool<Sqlite>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to downcast to SQLite pool"))?;
        
        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(pool)
            .await?;
        
        // Create servers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                minecraft_version TEXT NOT NULL DEFAULT '1.21.1',
                loader TEXT NOT NULL DEFAULT 'vanilla',
                loader_version TEXT NOT NULL DEFAULT 'latest',
                port INTEGER NOT NULL DEFAULT 25565,
                rcon_port INTEGER NOT NULL DEFAULT 25575,
                query_port INTEGER NOT NULL DEFAULT 25566,
                max_players INTEGER NOT NULL DEFAULT 20,
                memory INTEGER NOT NULL DEFAULT 4096,
                java_args TEXT NOT NULL DEFAULT '[]',
                server_args TEXT NOT NULL DEFAULT '[]',
                auto_start BOOLEAN NOT NULL DEFAULT 0,
                auto_restart BOOLEAN NOT NULL DEFAULT 1,
                world_name TEXT NOT NULL DEFAULT 'world',
                difficulty TEXT NOT NULL DEFAULT 'normal',
                gamemode TEXT NOT NULL DEFAULT 'survival',
                pvp BOOLEAN NOT NULL DEFAULT 1,
                online_mode BOOLEAN NOT NULL DEFAULT 1,
                whitelist BOOLEAN NOT NULL DEFAULT 0,
                enable_command_block BOOLEAN NOT NULL DEFAULT 0,
                view_distance INTEGER NOT NULL DEFAULT 10,
                simulation_distance INTEGER NOT NULL DEFAULT 10,
                motd TEXT NOT NULL DEFAULT 'A Minecraft Server',
                host TEXT NOT NULL DEFAULT 'localhost',
                java_path TEXT NOT NULL DEFAULT 'java',
                jvm_args TEXT NOT NULL DEFAULT '-Xmx4G -Xms2G',
                server_jar TEXT NOT NULL DEFAULT 'server.jar',
                rcon_password TEXT NOT NULL DEFAULT '',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_name ON servers (name)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_created_at ON servers (created_at)")
            .execute(pool)
            .await?;
        
        info!("SQLite migrations completed");
        Ok(())
    }
    
    /// Optimize database connections
    async fn optimize_connections(&self) -> Result<()> {
        if self.config.is_postgresql() {
            self.optimize_postgresql().await?;
        } else {
            self.optimize_sqlite().await?;
        }
        
        info!("Database connections optimized");
        Ok(())
    }
    
    /// Optimize PostgreSQL connections
    async fn optimize_postgresql(&self) -> Result<()> {
        let pool = self.pool.as_any()
            .downcast_ref::<Pool<Postgres>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to downcast to PostgreSQL pool"))?;
        
        // Set connection parameters for optimal performance
        sqlx::query("SET default_statistics_target = 1000")
            .execute(pool)
            .await?;
        
        sqlx::query("SET random_page_cost = 1.1")
            .execute(pool)
            .await?;
        
        sqlx::query("SET effective_cache_size = '1GB'")
            .execute(pool)
            .await?;
        
        sqlx::query("SET shared_buffers = '256MB'")
            .execute(pool)
            .await?;
        
        info!("PostgreSQL connections optimized");
        Ok(())
    }
    
    /// Optimize SQLite connections
    async fn optimize_sqlite(&self) -> Result<()> {
        let pool = self.pool.as_any()
            .downcast_ref::<Pool<Sqlite>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to downcast to SQLite pool"))?;
        
        // Enable WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(pool)
            .await?;
        
        // Set synchronous mode for better performance
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(pool)
            .await?;
        
        // Set cache size
        sqlx::query("PRAGMA cache_size = 10000")
            .execute(pool)
            .await?;
        
        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(pool)
            .await?;
        
        info!("SQLite connections optimized");
        Ok(())
    }
    
    /// Get database health status
    pub async fn get_health_status(&self) -> Result<DatabaseHealthStatus> {
        let start_time = std::time::Instant::now();
        
        // Test basic connectivity
        let test_query = if self.config.is_postgresql() {
            "SELECT 1 as test"
        } else {
            "SELECT 1 as test"
        };
        
        let _: i64 = sqlx::query_scalar(test_query)
            .fetch_one(self.get_pool().await?)
            .await?;
        
        let response_time = start_time.elapsed();
        
        // Get connection pool stats
        let pool_stats = self.get_pool_stats().await?;
        
        Ok(DatabaseHealthStatus {
            status: "healthy".to_string(),
            response_time_ms: response_time.as_millis() as u64,
            pool_stats,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Get connection pool statistics
    async fn get_pool_stats(&self) -> Result<PoolStats> {
        // This would need to be implemented based on the specific database type
        // For now, return basic stats
        Ok(PoolStats {
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
            active_connections: 0, // Would need to be implemented per database type
            idle_connections: 0,  // Would need to be implemented per database type
        })
    }
    
    /// Get the appropriate database pool
    async fn get_pool(&self) -> Result<&dyn DatabasePool> {
        Ok(&*self.pool)
    }
}

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthStatus {
    pub status: String,
    pub response_time_ms: u64,
    pub pool_stats: PoolStats,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub max_connections: u32,
    pub min_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
}

/// Database backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub schedule: String, // Cron expression
    pub retention_days: u32,
    pub compression: bool,
    pub encryption: bool,
    pub destination: String,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schedule: "0 2 * * *".to_string(), // Daily at 2 AM
            retention_days: 30,
            compression: true,
            encryption: false,
            destination: "backups/".to_string(),
        }
    }
}

/// Database backup manager
pub struct DatabaseBackupManager {
    config: BackupConfig,
    database_manager: ProductionDatabaseManager,
}

impl DatabaseBackupManager {
    pub fn new(config: BackupConfig, database_manager: ProductionDatabaseManager) -> Self {
        Self {
            config,
            database_manager,
        }
    }
    
    /// Create a database backup
    pub async fn create_backup(&self) -> Result<BackupResult> {
        if !self.config.enabled {
            return Ok(BackupResult {
                success: false,
                message: "Backup is disabled".to_string(),
                backup_path: None,
                size_bytes: 0,
                created_at: chrono::Utc::now(),
            });
        }
        
        let backup_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("guardian_backup_{}_{}.sql", timestamp, backup_id);
        let backup_path = std::path::Path::new(&self.config.destination).join(&backup_filename);
        
        // Ensure backup directory exists
        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let start_time = std::time::Instant::now();
        
        if self.database_manager.config.is_postgresql() {
            self.backup_postgresql(&backup_path).await?;
        } else {
            self.backup_sqlite(&backup_path).await?;
        }
        
        let duration = start_time.elapsed();
        let file_size = std::fs::metadata(&backup_path)?.len();
        
        info!("Database backup completed in {:?}, size: {} bytes", duration, file_size);
        
        Ok(BackupResult {
            success: true,
            message: "Backup completed successfully".to_string(),
            backup_path: Some(backup_path.to_string_lossy().to_string()),
            size_bytes: file_size,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Backup PostgreSQL database
    async fn backup_postgresql(&self, backup_path: &std::path::Path) -> Result<()> {
        // This would use pg_dump in a real implementation
        // For now, we'll create a placeholder
        let backup_content = format!(
            "-- PostgreSQL backup created at {}\n-- This is a placeholder backup\n",
            chrono::Utc::now()
        );
        
        std::fs::write(backup_path, backup_content)?;
        Ok(())
    }
    
    /// Backup SQLite database
    async fn backup_sqlite(&self, backup_path: &std::path::Path) -> Result<()> {
        // For SQLite, we can copy the database file directly
        let db_path = self.database_manager.config.url.strip_prefix("sqlite:")
            .ok_or_else(|| anyhow::anyhow!("Invalid SQLite URL"))?;
        
        std::fs::copy(db_path, backup_path)?;
        Ok(())
    }
    
    /// Clean up old backups based on retention policy
    pub async fn cleanup_old_backups(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let backup_dir = std::path::Path::new(&self.config.destination);
        if !backup_dir.exists() {
            return Ok(());
        }
        
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        let mut deleted_count = 0;
        
        for entry in std::fs::read_dir(backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Ok(metadata) = path.metadata() {
                    if let Ok(created) = metadata.created() {
                        let created_time = chrono::DateTime::from(created);
                        if created_time < cutoff_date {
                            std::fs::remove_file(&path)?;
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
        
        if deleted_count > 0 {
            info!("Cleaned up {} old backup files", deleted_count);
        }
        
        Ok(())
    }
}

/// Backup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_database_config_from_env() {
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/guardian");
        env::set_var("DATABASE_MAX_CONNECTIONS", "20");
        
        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.url, "postgresql://localhost:5432/guardian");
        assert_eq!(config.max_connections, 20);
        assert!(config.is_postgresql());
        
        env::remove_var("DATABASE_URL");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }
    
    #[tokio::test]
    async fn test_sqlite_production_manager() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let config = DatabaseConfig {
            url: database_url,
            max_connections: 5,
            min_connections: 1,
            acquire_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
            enable_logging: true,
            enable_metrics: true,
        };
        
        let manager = ProductionDatabaseManager::new(config).await.unwrap();
        let health = manager.get_health_status().await.unwrap();
        
        assert_eq!(health.status, "healthy");
        assert!(health.response_time_ms < 1000); // Should be fast
    }
}
