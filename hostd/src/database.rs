use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Database manager for Guardian
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

/// Server configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub java_path: String,
    pub server_jar: String,
    pub jvm_args: String,
    pub server_args: String,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// User settings stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: String,
    pub theme: String,
    pub language: String,
    pub notifications: bool,
    pub auto_refresh: bool,
    pub refresh_interval: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub schedule: String,
    pub retention_days: u32,
    pub include_world: bool,
    pub include_logs: bool,
    pub include_configs: bool,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub config_id: String,
    pub server_id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub id: String,
    pub server_id: Option<String>,
    pub event_type: String,
    pub message: String,
    pub level: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        let manager = Self { pool };
        manager.run_migrations().await?;
        
        Ok(manager)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");

        // Create servers table
        sqlx::query(
            r#"
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
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create user_settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_settings (
                id TEXT PRIMARY KEY,
                theme TEXT NOT NULL DEFAULT 'dark',
                language TEXT NOT NULL DEFAULT 'en',
                notifications BOOLEAN NOT NULL DEFAULT 1,
                auto_refresh BOOLEAN NOT NULL DEFAULT 1,
                refresh_interval INTEGER NOT NULL DEFAULT 5,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create backup_configs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backup_configs (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                name TEXT NOT NULL,
                schedule TEXT NOT NULL,
                retention_days INTEGER NOT NULL DEFAULT 7,
                include_world BOOLEAN NOT NULL DEFAULT 1,
                include_logs BOOLEAN NOT NULL DEFAULT 1,
                include_configs BOOLEAN NOT NULL DEFAULT 1,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create backup_records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backup_records (
                id TEXT PRIMARY KEY,
                config_id TEXT NOT NULL,
                server_id TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at DATETIME,
                FOREIGN KEY (config_id) REFERENCES backup_configs (id) ON DELETE CASCADE,
                FOREIGN KEY (server_id) REFERENCES servers (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create event_logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_logs (
                id TEXT PRIMARY KEY,
                server_id TEXT,
                event_type TEXT NOT NULL,
                message TEXT NOT NULL,
                level TEXT NOT NULL DEFAULT 'info',
                metadata TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers (id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_name ON servers (name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_backup_configs_server_id ON backup_configs (server_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_backup_records_server_id ON backup_records (server_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_backup_records_created_at ON backup_records (created_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_logs_server_id ON event_logs (server_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_logs_created_at ON event_logs (created_at)")
            .execute(&self.pool)
            .await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    // Server configuration methods
    pub async fn create_server(&self, config: &ServerConfig) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO servers (
                id, name, host, port, rcon_port, rcon_password,
                java_path, server_jar, jvm_args, server_args,
                auto_start, auto_restart, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.host)
        .bind(config.port)
        .bind(config.rcon_port)
        .bind(&config.rcon_password)
        .bind(&config.java_path)
        .bind(&config.server_jar)
        .bind(&config.jvm_args)
        .bind(&config.server_args)
        .bind(config.auto_start)
        .bind(config.auto_restart)
        .bind(config.created_at)
        .bind(config.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Created server configuration: {}", config.id);
        Ok(())
    }

    pub async fn get_server(&self, id: &str) -> Result<Option<ServerConfig>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, host, port, rcon_port, rcon_password,
                   java_path, server_jar, jvm_args, server_args,
                   auto_start, auto_restart, created_at, updated_at
            FROM servers WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ServerConfig {
                id: row.get("id"),
                name: row.get("name"),
                host: row.get("host"),
                port: row.get("port"),
                rcon_port: row.get("rcon_port"),
                rcon_password: row.get("rcon_password"),
                java_path: row.get("java_path"),
                server_jar: row.get("server_jar"),
                jvm_args: row.get("jvm_args"),
                server_args: row.get("server_args"),
                auto_start: row.get("auto_start"),
                auto_restart: row.get("auto_restart"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_servers(&self) -> Result<Vec<ServerConfig>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, host, port, rcon_port, rcon_password,
                   java_path, server_jar, jvm_args, server_args,
                   auto_start, auto_restart, created_at, updated_at
            FROM servers ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let servers = rows
            .into_iter()
            .map(|row| ServerConfig {
                id: row.get("id"),
                name: row.get("name"),
                host: row.get("host"),
                port: row.get("port"),
                rcon_port: row.get("rcon_port"),
                rcon_password: row.get("rcon_password"),
                java_path: row.get("java_path"),
                server_jar: row.get("server_jar"),
                jvm_args: row.get("jvm_args"),
                server_args: row.get("server_args"),
                auto_start: row.get("auto_start"),
                auto_restart: row.get("auto_restart"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(servers)
    }

    pub async fn update_server(&self, config: &ServerConfig) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE servers SET
                name = ?, host = ?, port = ?, rcon_port = ?, rcon_password = ?,
                java_path = ?, server_jar = ?, jvm_args = ?, server_args = ?,
                auto_start = ?, auto_restart = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&config.name)
        .bind(&config.host)
        .bind(config.port)
        .bind(config.rcon_port)
        .bind(&config.rcon_password)
        .bind(&config.java_path)
        .bind(&config.server_jar)
        .bind(&config.jvm_args)
        .bind(&config.server_args)
        .bind(config.auto_start)
        .bind(config.auto_restart)
        .bind(config.updated_at)
        .bind(&config.id)
        .execute(&self.pool)
        .await?;

        info!("Updated server configuration: {}", config.id);
        Ok(())
    }

    pub async fn delete_server(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        info!("Deleted server configuration: {}", id);
        Ok(())
    }

    // User settings methods
    pub async fn get_user_settings(&self) -> Result<Option<UserSettings>> {
        let row = sqlx::query(
            r#"
            SELECT id, theme, language, notifications, auto_refresh,
                   refresh_interval, created_at, updated_at
            FROM user_settings LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(UserSettings {
                id: row.get("id"),
                theme: row.get("theme"),
                language: row.get("language"),
                notifications: row.get("notifications"),
                auto_refresh: row.get("auto_refresh"),
                refresh_interval: row.get("refresh_interval"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_user_settings(&self, settings: &UserSettings) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_settings (
                id, theme, language, notifications, auto_refresh,
                refresh_interval, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&settings.id)
        .bind(&settings.theme)
        .bind(&settings.language)
        .bind(settings.notifications)
        .bind(settings.auto_refresh)
        .bind(settings.refresh_interval)
        .bind(settings.created_at)
        .bind(settings.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Updated user settings: {}", settings.id);
        Ok(())
    }

    // Event logging methods
    pub async fn log_event(&self, event: &EventLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event_logs (
                id, server_id, event_type, message, level, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.id)
        .bind(&event.server_id)
        .bind(&event.event_type)
        .bind(&event.message)
        .bind(&event.level)
        .bind(&event.metadata)
        .bind(event.created_at)
        .execute(&self.pool)
        .await?;

        debug!("Logged event: {} - {}", event.event_type, event.message);
        Ok(())
    }

    pub async fn get_events(
        &self,
        server_id: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<EventLog>> {
        let limit = limit.unwrap_or(100);
        
        let query = if let Some(server_id) = server_id {
            sqlx::query(
                r#"
                SELECT id, server_id, event_type, message, level, metadata, created_at
                FROM event_logs
                WHERE server_id = ?
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(server_id)
            .bind(limit)
        } else {
            sqlx::query(
                r#"
                SELECT id, server_id, event_type, message, level, metadata, created_at
                FROM event_logs
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
        };

        let rows = query.fetch_all(&self.pool).await?;

        let events = rows
            .into_iter()
            .map(|row| EventLog {
                id: row.get("id"),
                server_id: row.get("server_id"),
                event_type: row.get("event_type"),
                message: row.get("message"),
                level: row.get("level"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(events)
    }

    // Backup configuration methods
    pub async fn create_backup_config(&self, config: &BackupConfig) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO backup_configs (
                id, server_id, name, schedule, retention_days,
                include_world, include_logs, include_configs,
                enabled, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&config.id)
        .bind(&config.server_id)
        .bind(&config.name)
        .bind(&config.schedule)
        .bind(config.retention_days)
        .bind(config.include_world)
        .bind(config.include_logs)
        .bind(config.include_configs)
        .bind(config.enabled)
        .bind(config.created_at)
        .bind(config.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Created backup configuration: {}", config.id);
        Ok(())
    }

    pub async fn get_backup_configs(&self, server_id: &str) -> Result<Vec<BackupConfig>> {
        let rows = sqlx::query(
            r#"
            SELECT id, server_id, name, schedule, retention_days,
                   include_world, include_logs, include_configs,
                   enabled, created_at, updated_at
            FROM backup_configs
            WHERE server_id = ?
            ORDER BY name
            "#,
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await?;

        let configs = rows
            .into_iter()
            .map(|row| BackupConfig {
                id: row.get("id"),
                server_id: row.get("server_id"),
                name: row.get("name"),
                schedule: row.get("schedule"),
                retention_days: row.get("retention_days"),
                include_world: row.get("include_world"),
                include_logs: row.get("include_logs"),
                include_configs: row.get("include_configs"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(configs)
    }

    // Backup record methods
    pub async fn create_backup_record(&self, record: &BackupRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO backup_records (
                id, config_id, server_id, name, path, size_bytes,
                status, created_at, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.id)
        .bind(&record.config_id)
        .bind(&record.server_id)
        .bind(&record.name)
        .bind(&record.path)
        .bind(record.size_bytes as i64)
        .bind(&record.status)
        .bind(record.created_at)
        .bind(&record.completed_at)
        .execute(&self.pool)
        .await?;

        info!("Created backup record: {}", record.id);
        Ok(())
    }

    pub async fn get_backup_records(&self, server_id: &str) -> Result<Vec<BackupRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_id, server_id, name, path, size_bytes,
                   status, created_at, completed_at
            FROM backup_records
            WHERE server_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await?;

        let records = rows
            .into_iter()
            .map(|row| BackupRecord {
                id: row.get("id"),
                config_id: row.get("config_id"),
                server_id: row.get("server_id"),
                name: row.get("name"),
                path: row.get("path"),
                size_bytes: row.get::<i64, _>("size_bytes") as u64,
                status: row.get("status"),
                created_at: row.get("created_at"),
                completed_at: row.get("completed_at"),
            })
            .collect();

        Ok(records)
    }

    pub async fn update_backup_record(&self, record: &BackupRecord) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE backup_records SET
                status = ?, completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&record.status)
        .bind(&record.completed_at)
        .bind(&record.id)
        .execute(&self.pool)
        .await?;

        info!("Updated backup record: {}", record.id);
        Ok(())
    }

    /// Clean up old backup records based on retention policy
    pub async fn cleanup_old_backups(&self) -> Result<()> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(30);
        
        let result = sqlx::query(
            "DELETE FROM backup_records WHERE created_at < ? AND status = 'completed'"
        )
        .bind(cutoff_date)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            info!("Cleaned up {} old backup records", result.rows_affected());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let db = DatabaseManager::new(&database_url).await.unwrap();
        
        // Test creating a server
        let server = ServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            host: "localhost".to_string(),
            port: 25565,
            rcon_port: 25575,
            rcon_password: "password".to_string(),
            java_path: "/usr/bin/java".to_string(),
            server_jar: "server.jar".to_string(),
            jvm_args: "-Xmx4G".to_string(),
            server_args: "nogui".to_string(),
            auto_start: true,
            auto_restart: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        db.create_server(&server).await.unwrap();
        
        // Test retrieving the server
        let retrieved = db.get_server("test-server").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Server");
    }

    #[tokio::test]
    async fn test_event_logging() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let db = DatabaseManager::new(&database_url).await.unwrap();
        
        let event = EventLog {
            id: Uuid::new_v4().to_string(),
            server_id: Some("test-server".to_string()),
            event_type: "server_start".to_string(),
            message: "Server started successfully".to_string(),
            level: "info".to_string(),
            metadata: Some(serde_json::json!({"port": 25565})),
            created_at: chrono::Utc::now(),
        };
        
        db.log_event(&event).await.unwrap();
        
        let events = db.get_events(Some("test-server"), Some(10)).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "server_start");
    }
}
