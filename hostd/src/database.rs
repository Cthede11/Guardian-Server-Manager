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

/// Health status for database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: Option<String>,
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

/// Minecraft version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub release_type: String,
    pub release_date: chrono::DateTime<chrono::Utc>,
    pub protocol_version: i32,
    pub data_version: i32,
    pub is_supported: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Mod loader version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub id: i64,
    pub loader_type: String,
    pub version: String,
    pub minecraft_version: String,
    pub download_url: String,
    pub file_size: u64,
    pub sha256: String,
    pub is_stable: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Mod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub side: String,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Mod version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub id: i64,
    pub mod_id: String,
    pub version: String,
    pub minecraft_versions: String, // JSON array
    pub loader_versions: String,    // JSON object
    pub download_url: String,
    pub file_size: u64,
    pub sha256: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Mod dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub id: i64,
    pub mod_version_id: i64,
    pub dependency_mod_id: String,
    pub version_range: String,
    pub side: String,
    pub required: bool,
}

/// Mod conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConflict {
    pub id: i64,
    pub mod_version_id: i64,
    pub conflicting_mod_id: String,
    pub reason: String,
    pub severity: String,
}

/// Modpack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modpack {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub loader: String,
    pub client_mods: String, // JSON array
    pub server_mods: String, // JSON array
    pub config: Option<String>, // JSON object
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Server mod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMod {
    pub id: i64,
    pub server_id: String,
    pub mod_id: String,
    pub mod_version_id: i64,
    pub enabled: bool,
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

        // Create minecraft_versions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS minecraft_versions (
                id TEXT PRIMARY KEY,
                release_type TEXT NOT NULL,
                release_date DATETIME NOT NULL,
                protocol_version INTEGER NOT NULL,
                data_version INTEGER NOT NULL,
                is_supported BOOLEAN DEFAULT TRUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create loader_versions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loader_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                loader_type TEXT NOT NULL,
                version TEXT NOT NULL,
                minecraft_version TEXT NOT NULL,
                download_url TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                sha256 TEXT NOT NULL,
                is_stable BOOLEAN DEFAULT TRUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (minecraft_version) REFERENCES minecraft_versions(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mods table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mods (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                category TEXT NOT NULL,
                side TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mod_versions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mod_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_id TEXT NOT NULL,
                version TEXT NOT NULL,
                minecraft_versions TEXT NOT NULL,
                loader_versions TEXT NOT NULL,
                download_url TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                sha256 TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (mod_id) REFERENCES mods(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mod_dependencies table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mod_dependencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_version_id INTEGER NOT NULL,
                dependency_mod_id TEXT NOT NULL,
                version_range TEXT NOT NULL,
                side TEXT NOT NULL,
                required BOOLEAN DEFAULT TRUE,
                FOREIGN KEY (mod_version_id) REFERENCES mod_versions(id),
                FOREIGN KEY (dependency_mod_id) REFERENCES mods(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mod_conflicts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mod_conflicts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_version_id INTEGER NOT NULL,
                conflicting_mod_id TEXT NOT NULL,
                reason TEXT NOT NULL,
                severity TEXT NOT NULL,
                FOREIGN KEY (mod_version_id) REFERENCES mod_versions(id),
                FOREIGN KEY (conflicting_mod_id) REFERENCES mods(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create modpacks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS modpacks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                minecraft_version TEXT NOT NULL,
                loader TEXT NOT NULL,
                client_mods TEXT NOT NULL,
                server_mods TEXT NOT NULL,
                config TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create server_mods table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS server_mods (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                mod_id TEXT NOT NULL,
                mod_version_id INTEGER NOT NULL,
                enabled BOOLEAN DEFAULT TRUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
                FOREIGN KEY (mod_id) REFERENCES mods(id),
                FOREIGN KEY (mod_version_id) REFERENCES mod_versions(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create additional indexes for modpack system
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mods_category ON mods (category)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mods_side ON mods (side)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mods_source ON mods (source)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mod_versions_mod_id ON mod_versions (mod_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_modpacks_minecraft_version ON modpacks (minecraft_version)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_modpacks_loader ON modpacks (loader)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_server_mods_server_id ON server_mods (server_id)")
            .execute(&self.pool)
            .await?;

        // Populate default Minecraft versions
        self.populate_default_minecraft_versions().await?;
        
        // Populate default loader versions
        self.populate_default_loader_versions().await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    async fn populate_default_minecraft_versions(&self) -> Result<()> {
        // Insert some common Minecraft versions
        let versions = vec![
            ("1.21.1", "release", "2024-08-20T00:00:00Z", 767, 15, true),
            ("1.21", "release", "2024-06-13T00:00:00Z", 766, 15, true),
            ("1.20.6", "release", "2024-05-14T00:00:00Z", 765, 15, true),
            ("1.20.4", "release", "2024-01-15T00:00:00Z", 764, 15, true),
            ("1.20.1", "release", "2023-06-12T00:00:00Z", 763, 15, true),
            ("1.19.4", "release", "2023-03-14T00:00:00Z", 762, 15, true),
            ("1.18.2", "release", "2022-02-28T00:00:00Z", 758, 15, true),
            ("1.17.1", "release", "2021-07-06T00:00:00Z", 756, 15, true),
        ];

        for (version, release_type, release_date, protocol_version, data_version, is_supported) in versions {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO minecraft_versions 
                (id, release_type, release_date, protocol_version, data_version, is_supported)
                VALUES (?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(version)
            .bind(release_type)
            .bind(release_date)
            .bind(protocol_version)
            .bind(data_version)
            .bind(is_supported)
            .execute(&self.pool)
            .await?;
        }

        info!("Populated default Minecraft versions");
        Ok(())
    }

    async fn populate_default_loader_versions(&self) -> Result<()> {
        // Insert some common loader versions for Forge and Fabric
        let loaders = vec![
            ("forge", "47.4.0", "1.21.1", "https://maven.minecraftforge.net/net/minecraftforge/forge/1.21.1-47.4.0/forge-1.21.1-47.4.0-installer.jar", 12345678, "sha256hash1", true),
            ("forge", "47.3.0", "1.21", "https://maven.minecraftforge.net/net/minecraftforge/forge/1.21-47.3.0/forge-1.21-47.3.0-installer.jar", 12345678, "sha256hash2", true),
            ("fabric", "0.15.11", "1.21.1", "https://maven.fabricmc.net/net/fabricmc/fabric-installer/0.15.11/fabric-installer-0.15.11.jar", 8765432, "sha256hash3", true),
            ("fabric", "0.15.10", "1.21", "https://maven.fabricmc.net/net/fabricmc/fabric-installer/0.15.10/fabric-installer-0.15.10.jar", 8765432, "sha256hash4", true),
            ("quilt", "0.8.0", "1.21.1", "https://maven.quiltmc.org/repository/release/org/quiltmc/quilt-installer/0.8.0/quilt-installer-0.8.0.jar", 5432109, "sha256hash5", true),
        ];

        for (loader_type, version, minecraft_version, download_url, file_size, sha256, is_stable) in loaders {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO loader_versions 
                (loader_type, version, minecraft_version, download_url, file_size, sha256, is_stable)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(loader_type)
            .bind(version)
            .bind(minecraft_version)
            .bind(download_url)
            .bind(file_size)
            .bind(sha256)
            .bind(is_stable)
            .execute(&self.pool)
            .await?;
        }

        info!("Populated default loader versions");
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

    // Modpack database methods
    pub async fn get_minecraft_versions(&self) -> Result<Vec<MinecraftVersion>> {
        let rows = sqlx::query(
            r#"
            SELECT id, release_type, release_date, protocol_version, data_version, is_supported, created_at
            FROM minecraft_versions
            ORDER BY release_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let versions = rows
            .into_iter()
            .map(|row| MinecraftVersion {
                id: row.get("id"),
                release_type: row.get("release_type"),
                release_date: row.get("release_date"),
                protocol_version: row.get("protocol_version"),
                data_version: row.get("data_version"),
                is_supported: row.get("is_supported"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(versions)
    }

    pub async fn get_loader_versions(
        &self,
        minecraft_version: Option<&String>,
        loader_type: Option<&String>,
    ) -> Result<Vec<LoaderVersion>> {
        let mut query = sqlx::query(
            r#"
            SELECT id, loader_type, version, minecraft_version, download_url, file_size, sha256, is_stable, created_at
            FROM loader_versions
            WHERE 1=1
            "#
        );

        if let Some(mc_version) = minecraft_version {
            query = query.bind(mc_version);
        }
        if let Some(loader) = loader_type {
            query = query.bind(loader);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await?;

        let versions = rows
            .into_iter()
            .map(|row| LoaderVersion {
                id: row.get("id"),
                loader_type: row.get("loader_type"),
                version: row.get("version"),
                minecraft_version: row.get("minecraft_version"),
                download_url: row.get("download_url"),
                file_size: row.get::<i64, _>("file_size") as u64,
                sha256: row.get("sha256"),
                is_stable: row.get("is_stable"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(versions)
    }

    pub async fn search_mods(&self, params: &crate::api::ModSearchQuery) -> Result<Vec<ModInfo>> {
        let mut query = sqlx::query(
            r#"
            SELECT id, name, description, category, side, source, created_at, updated_at
            FROM mods
            WHERE 1=1
            "#
        );

        if let Some(search_query) = &params.search_query {
            query = query.bind(format!("%{}%", search_query));
        }
        if let Some(category) = &params.category {
            query = query.bind(category);
        }
        if let Some(side) = &params.side {
            query = query.bind(side);
        }
        if let Some(source) = &params.source {
            query = query.bind(source);
        }

        let limit = params.limit.unwrap_or(50);
        let offset = (params.page.unwrap_or(1) - 1) * limit;

        let rows = query
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mods = rows
            .into_iter()
            .map(|row| ModInfo {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                category: row.get("category"),
                side: row.get("side"),
                source: row.get("source"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(mods)
    }

    pub async fn get_mod(&self, id: &str) -> Result<Option<ModInfo>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, category, side, source, created_at, updated_at
            FROM mods WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ModInfo {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                category: row.get("category"),
                side: row.get("side"),
                source: row.get("source"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_mod_versions(&self, mod_id: &str) -> Result<Vec<ModVersion>> {
        let rows = sqlx::query(
            r#"
            SELECT id, mod_id, version, minecraft_versions, loader_versions, download_url, file_size, sha256, created_at
            FROM mod_versions
            WHERE mod_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(mod_id)
        .fetch_all(&self.pool)
        .await?;

        let versions = rows
            .into_iter()
            .map(|row| ModVersion {
                id: row.get("id"),
                mod_id: row.get("mod_id"),
                version: row.get("version"),
                minecraft_versions: row.get("minecraft_versions"),
                loader_versions: row.get("loader_versions"),
                download_url: row.get("download_url"),
                file_size: row.get::<i64, _>("file_size") as u64,
                sha256: row.get("sha256"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(versions)
    }

    pub async fn get_modpacks(&self) -> Result<Vec<Modpack>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, minecraft_version, loader, client_mods, server_mods, config, created_at, updated_at
            FROM modpacks
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let modpacks = rows
            .into_iter()
            .map(|row| Modpack {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                minecraft_version: row.get("minecraft_version"),
                loader: row.get("loader"),
                client_mods: row.get("client_mods"),
                server_mods: row.get("server_mods"),
                config: row.get("config"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(modpacks)
    }

    pub async fn create_modpack(&self, modpack: &Modpack) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO modpacks (
                id, name, description, minecraft_version, loader,
                client_mods, server_mods, config, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&modpack.id)
        .bind(&modpack.name)
        .bind(&modpack.description)
        .bind(&modpack.minecraft_version)
        .bind(&modpack.loader)
        .bind(&modpack.client_mods)
        .bind(&modpack.server_mods)
        .bind(&modpack.config)
        .bind(modpack.created_at)
        .bind(modpack.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Created modpack: {}", modpack.id);
        Ok(())
    }

    pub async fn get_modpack(&self, id: &str) -> Result<Option<Modpack>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, minecraft_version, loader, client_mods, server_mods, config, created_at, updated_at
            FROM modpacks WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Modpack {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                minecraft_version: row.get("minecraft_version"),
                loader: row.get("loader"),
                client_mods: row.get("client_mods"),
                server_mods: row.get("server_mods"),
                config: row.get("config"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_modpack(&self, modpack: &Modpack) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE modpacks SET
                name = ?, description = ?, minecraft_version = ?, loader = ?,
                client_mods = ?, server_mods = ?, config = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&modpack.name)
        .bind(&modpack.description)
        .bind(&modpack.minecraft_version)
        .bind(&modpack.loader)
        .bind(&modpack.client_mods)
        .bind(&modpack.server_mods)
        .bind(&modpack.config)
        .bind(modpack.updated_at)
        .bind(&modpack.id)
        .execute(&self.pool)
        .await?;

        info!("Updated modpack: {}", modpack.id);
        Ok(())
    }

    pub async fn delete_modpack(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM modpacks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        info!("Deleted modpack: {}", id);
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
    
    /// Get database health status
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        // Check database connectivity
        let _: i64 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(HealthStatus {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            details: Some("Database connection successful".to_string()),
        })
    }
}
