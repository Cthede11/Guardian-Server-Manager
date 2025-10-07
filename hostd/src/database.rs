use anyhow::Result;
use chrono;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use tracing::{info, warn, debug};
use uuid::Uuid;

/// Database manager for Guardian
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

/// Server configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub port: u16,
    pub rcon_port: u16,
    pub query_port: u16,
    pub max_players: u32,
    pub memory: u32,
    pub java_args: String, // JSON string
    pub server_args: String, // JSON string
    pub auto_start: bool,
    pub auto_restart: bool,
    pub world_name: String,
    pub difficulty: String,
    pub gamemode: String,
    pub pvp: bool,
    pub online_mode: bool,
    pub whitelist: bool,
    pub enable_command_block: bool,
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub motd: String,
    // Additional fields needed for production
    pub host: String,
    pub java_path: String,
    pub jvm_args: String,
    pub server_jar: String,
    pub server_directory: String,
    pub rcon_password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Server log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerLog {
    pub id: String,
    pub server_id: String,
    pub level: String,
    pub message: String,
    pub component: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Server metrics entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerMetric {
    pub id: String,
    pub server_id: String,
    pub tps: f64,
    pub tick_p95: f64,
    pub heap_mb: u32,
    pub players_online: u32,
    pub gpu_queue_ms: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Global settings stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub id: String,
    pub cf_api_key: Option<String>,
    pub modrinth_token: Option<String>,
    pub java_path: String,
    pub default_ram_mb: u32,
    pub data_dir: String,
    pub telemetry_opt_in: bool,
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

/// Task information for tracking long-running operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub server_id: Option<String>,
    pub kind: String, // download, install, backup, worldgen, lighting, import, compat_scan
    pub status: String, // pending, running, done, failed
    pub progress: f64,
    pub log: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Mod information with enhanced fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: String,
    pub provider: String, // curseforge, modrinth
    pub project_id: String,
    pub version_id: String,
    pub filename: String,
    pub sha1: String,
    pub server_id: Option<String>,
    pub enabled: bool,
    pub category: String,
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
                minecraft_version TEXT NOT NULL DEFAULT '1.20.1',
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
                server_directory TEXT NOT NULL DEFAULT 'data/servers',
                rcon_password TEXT NOT NULL DEFAULT '',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                id TEXT PRIMARY KEY,
                cf_api_key TEXT,
                modrinth_token TEXT,
                java_path TEXT NOT NULL DEFAULT 'java',
                default_ram_mb INTEGER NOT NULL DEFAULT 4096,
                data_dir TEXT NOT NULL DEFAULT 'data',
                telemetry_opt_in BOOLEAN NOT NULL DEFAULT 0,
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

        // Create tasks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                server_id TEXT,
                kind TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                progress REAL NOT NULL DEFAULT 0.0,
                log TEXT,
                metadata TEXT,
                started_at DATETIME,
                finished_at DATETIME,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mods table (enhanced)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mods (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                project_id TEXT NOT NULL,
                version_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                sha1 TEXT NOT NULL,
                server_id TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                category TEXT NOT NULL DEFAULT 'unknown',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers (id) ON DELETE CASCADE
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
        // Add missing columns if they don't exist (migration for existing databases)
        info!("Adding missing columns to existing tables...");
        
        // Add max_players column to servers table
        if let Err(e) = sqlx::query("ALTER TABLE servers ADD COLUMN max_players INTEGER DEFAULT 20")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add max_players column to servers table: {}", e);
            }
        }
        
        // Add minecraft_version column to servers table
        if let Err(e) = sqlx::query("ALTER TABLE servers ADD COLUMN minecraft_version TEXT DEFAULT '1.20.1'")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add minecraft_version column to servers table: {}", e);
            }
        }
        
        // Add server_id column to tasks table
        if let Err(e) = sqlx::query("ALTER TABLE tasks ADD COLUMN server_id TEXT")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add server_id column to tasks table: {}", e);
            }
        }
        
        // Add server_id column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN server_id TEXT")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add server_id column to mods table: {}", e);
            }
        }
        
        // Add category column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN category TEXT DEFAULT 'unknown'")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add category column to mods table: {}", e);
            }
        }
        
        // Add side column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN side TEXT DEFAULT 'both'")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add side column to mods table: {}", e);
            }
        }
        
        // Add source column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN source TEXT DEFAULT 'unknown'")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add source column to mods table: {}", e);
            }
        }
        
        // Add name column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN name TEXT DEFAULT 'Unknown'")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add name column to mods table: {}", e);
            }
        }
        
        // Add description column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN description TEXT")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add description column to mods table: {}", e);
            }
        }
        
        // Add server_id column to backup_configs table
        if let Err(e) = sqlx::query("ALTER TABLE backup_configs ADD COLUMN server_id TEXT")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add server_id column to backup_configs table: {}", e);
            }
        }
        
        // Add server_id column to backup_records table
        if let Err(e) = sqlx::query("ALTER TABLE backup_records ADD COLUMN server_id TEXT")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add server_id column to backup_records table: {}", e);
            }
        }
        
        // Add server_id column to event_logs table
        if let Err(e) = sqlx::query("ALTER TABLE event_logs ADD COLUMN server_id TEXT")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add server_id column to event_logs table: {}", e);
            }
        }
        
        // Add missing columns for ServerConfig fields
        let missing_columns = [
            ("loader", "TEXT DEFAULT 'vanilla'"),
            ("loader_version", "TEXT DEFAULT 'latest'"),
            ("query_port", "INTEGER DEFAULT 25566"),
            ("memory", "INTEGER DEFAULT 4096"),
            ("java_args", "TEXT DEFAULT '[]'"),
            ("server_args", "TEXT DEFAULT '[]'"),
            ("world_name", "TEXT DEFAULT 'world'"),
            ("difficulty", "TEXT DEFAULT 'normal'"),
            ("gamemode", "TEXT DEFAULT 'survival'"),
            ("pvp", "BOOLEAN DEFAULT 1"),
            ("online_mode", "BOOLEAN DEFAULT 1"),
            ("whitelist", "BOOLEAN DEFAULT 0"),
            ("enable_command_block", "BOOLEAN DEFAULT 0"),
            ("view_distance", "INTEGER DEFAULT 10"),
            ("simulation_distance", "INTEGER DEFAULT 10"),
            ("motd", "TEXT DEFAULT 'A Minecraft Server'"),
            ("host", "TEXT DEFAULT 'localhost'"),
            ("java_path", "TEXT DEFAULT 'java'"),
            ("jvm_args", "TEXT DEFAULT '-Xmx4G -Xms2G'"),
            ("server_jar", "TEXT DEFAULT 'server.jar'"),
            ("server_directory", "TEXT DEFAULT 'data/servers'"),
            ("rcon_password", "TEXT DEFAULT ''"),
        ];
        
        for (column_name, column_def) in &missing_columns {
            if let Err(e) = sqlx::query(&format!("ALTER TABLE servers ADD COLUMN {} {}", column_name, column_def))
                .execute(&self.pool)
                .await
            {
                if !e.to_string().contains("duplicate column name") {
                    warn!("Failed to add {} column to servers table: {}", column_name, e);
                }
            }
        }
        
        // Add provider column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN provider TEXT DEFAULT 'unknown'")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add provider column to mods table: {}", e);
            }
        }
        
        // Add project_id column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN project_id TEXT DEFAULT ''")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add project_id column to mods table: {}", e);
            }
        }
        
        // Add version_id column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN version_id TEXT DEFAULT ''")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add version_id column to mods table: {}", e);
            }
        }
        
        // Add filename column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN filename TEXT DEFAULT ''")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add filename column to mods table: {}", e);
            }
        }
        
        // Add sha1 column to mods table
        if let Err(e) = sqlx::query("ALTER TABLE mods ADD COLUMN sha1 TEXT DEFAULT ''")
            .execute(&self.pool)
            .await
        {
            if !e.to_string().contains("duplicate column name") {
                warn!("Failed to add sha1 column to mods table: {}", e);
            }
        }
        
        info!("Database migration completed");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_name ON servers (name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_max_players ON servers (max_players)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_server_id ON tasks (server_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_kind ON tasks (kind)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks (created_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mods_server_id ON mods (server_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mods_provider ON mods (provider)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mods_enabled ON mods (enabled)")
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

        // Note: mods table already created above with enhanced schema

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

        // Create server_logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS server_logs (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                component TEXT,
                timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers (id) ON DELETE CASCADE
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
                id, name, minecraft_version, loader, loader_version, port, rcon_port, query_port,
                max_players, memory, java_args, server_args, auto_start, auto_restart,
                world_name, difficulty, gamemode, pvp, online_mode, whitelist,
                enable_command_block, view_distance, simulation_distance, motd,
                host, java_path, jvm_args, server_jar, server_directory, rcon_password,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.minecraft_version)
        .bind(&config.loader)
        .bind(&config.loader_version)
        .bind(config.port)
        .bind(config.rcon_port)
        .bind(config.query_port)
        .bind(config.max_players)
        .bind(config.memory)
        .bind(&config.java_args)
        .bind(&config.server_args)
        .bind(config.auto_start)
        .bind(config.auto_restart)
        .bind(&config.world_name)
        .bind(&config.difficulty)
        .bind(&config.gamemode)
        .bind(config.pvp)
        .bind(config.online_mode)
        .bind(config.whitelist)
        .bind(config.enable_command_block)
        .bind(config.view_distance)
        .bind(config.simulation_distance)
        .bind(&config.motd)
        .bind(&config.host)
        .bind(&config.java_path)
        .bind(&config.jvm_args)
        .bind(&config.server_jar)
        .bind(&config.server_directory)
        .bind(&config.rcon_password)
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
            SELECT id, name, minecraft_version, loader, loader_version, port, rcon_port, query_port,
                   max_players, memory, java_args, server_args, auto_start, auto_restart,
                   world_name, difficulty, gamemode, pvp, online_mode, whitelist,
                   enable_command_block, view_distance, simulation_distance, motd,
                   host, java_path, jvm_args, server_jar, rcon_password,
                   created_at, updated_at
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
                minecraft_version: row.get("minecraft_version"),
                loader: row.get("loader"),
                loader_version: row.get("loader_version"),
                port: row.get("port"),
                rcon_port: row.get("rcon_port"),
                query_port: row.get("query_port"),
                max_players: row.get("max_players"),
                memory: row.get("memory"),
                java_args: row.get("java_args"),
                server_args: row.get("server_args"),
                auto_start: row.get("auto_start"),
                auto_restart: row.get("auto_restart"),
                world_name: row.get("world_name"),
                difficulty: row.get("difficulty"),
                gamemode: row.get("gamemode"),
                pvp: row.get("pvp"),
                online_mode: row.get("online_mode"),
                whitelist: row.get("whitelist"),
                enable_command_block: row.get("enable_command_block"),
                view_distance: row.get("view_distance"),
                simulation_distance: row.get("simulation_distance"),
                motd: row.get("motd"),
                host: row.get("host"),
                java_path: row.get("java_path"),
                jvm_args: row.get("jvm_args"),
                server_jar: row.get("server_jar"),
                server_directory: row.get("server_directory"),
                rcon_password: row.get("rcon_password"),
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
            SELECT id, name, minecraft_version, loader, loader_version, port, rcon_port, query_port,
                   max_players, memory, java_args, server_args, auto_start, auto_restart,
                   world_name, difficulty, gamemode, pvp, online_mode, whitelist,
                   enable_command_block, view_distance, simulation_distance, motd,
                   host, java_path, jvm_args, server_jar, rcon_password,
                   created_at, updated_at
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
                minecraft_version: row.get("minecraft_version"),
                loader: row.get("loader"),
                loader_version: row.get("loader_version"),
                port: row.get("port"),
                rcon_port: row.get("rcon_port"),
                query_port: row.get("query_port"),
                max_players: row.get("max_players"),
                memory: row.get("memory"),
                java_args: row.get("java_args"),
                server_args: row.get("server_args"),
                auto_start: row.get("auto_start"),
                auto_restart: row.get("auto_restart"),
                world_name: row.get("world_name"),
                difficulty: row.get("difficulty"),
                gamemode: row.get("gamemode"),
                pvp: row.get("pvp"),
                online_mode: row.get("online_mode"),
                whitelist: row.get("whitelist"),
                enable_command_block: row.get("enable_command_block"),
                view_distance: row.get("view_distance"),
                simulation_distance: row.get("simulation_distance"),
                motd: row.get("motd"),
                host: row.get("host"),
                java_path: row.get("java_path"),
                jvm_args: row.get("jvm_args"),
                server_jar: row.get("server_jar"),
                server_directory: row.get("server_directory"),
                rcon_password: row.get("rcon_password"),
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
                auto_start = ?, auto_restart = ?, max_players = ?,
                updated_at = ?
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
        .bind(config.max_players)
        // pregeneration_policy field removed
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

    // Settings methods
    pub async fn get_settings(&self) -> Result<Option<Settings>> {
        let row = sqlx::query(
            r#"
            SELECT id, cf_api_key, modrinth_token, java_path, default_ram_mb,
                   data_dir, telemetry_opt_in, created_at, updated_at
            FROM settings LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Settings {
                id: row.get("id"),
                cf_api_key: row.get("cf_api_key"),
                modrinth_token: row.get("modrinth_token"),
                java_path: row.get("java_path"),
                default_ram_mb: row.get("default_ram_mb"),
                data_dir: row.get("data_dir"),
                telemetry_opt_in: row.get("telemetry_opt_in"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_settings(&self, settings: &Settings) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO settings (
                id, cf_api_key, modrinth_token, java_path, default_ram_mb,
                data_dir, telemetry_opt_in, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&settings.id)
        .bind(&settings.cf_api_key)
        .bind(&settings.modrinth_token)
        .bind(&settings.java_path)
        .bind(settings.default_ram_mb)
        .bind(&settings.data_dir)
        .bind(settings.telemetry_opt_in)
        .bind(settings.created_at)
        .bind(settings.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Updated settings: {}", settings.id);
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

    // Task methods
    pub async fn create_task(&self, task: &Task) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, server_id, kind, status, progress, log, metadata,
                started_at, finished_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&task.id)
        .bind(&task.server_id)
        .bind(&task.kind)
        .bind(&task.status)
        .bind(task.progress)
        .bind(&task.log)
        .bind(&task.metadata)
        .bind(task.started_at)
        .bind(task.finished_at)
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Created task: {}", task.id);
        Ok(())
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT id, server_id, kind, status, progress, log, metadata,
                   started_at, finished_at, created_at, updated_at
            FROM tasks WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Task {
                id: row.get("id"),
                server_id: row.get("server_id"),
                kind: row.get("kind"),
                status: row.get("status"),
                progress: row.get("progress"),
                log: row.get("log"),
                metadata: row.get("metadata"),
                started_at: row.get("started_at"),
                finished_at: row.get("finished_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_tasks_by_server(&self, server_id: &str) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            r#"
            SELECT id, server_id, kind, status, progress, log, metadata,
                   started_at, finished_at, created_at, updated_at
            FROM tasks WHERE server_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await?;

        let tasks = rows
            .into_iter()
            .map(|row| Task {
                id: row.get("id"),
                server_id: row.get("server_id"),
                kind: row.get("kind"),
                status: row.get("status"),
                progress: row.get("progress"),
                log: row.get("log"),
                metadata: row.get("metadata"),
                started_at: row.get("started_at"),
                finished_at: row.get("finished_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(tasks)
    }

    pub async fn update_task(&self, task: &Task) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tasks SET
                status = ?, progress = ?, log = ?, metadata = ?,
                started_at = ?, finished_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&task.status)
        .bind(task.progress)
        .bind(&task.log)
        .bind(&task.metadata)
        .bind(task.started_at)
        .bind(task.finished_at)
        .bind(task.updated_at)
        .bind(&task.id)
        .execute(&self.pool)
        .await?;

        info!("Updated task: {}", task.id);
        Ok(())
    }

    pub async fn delete_task(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        info!("Deleted task: {}", id);
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
        .bind(record.completed_at)
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
        .bind(record.completed_at)
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

    // Enhanced mod methods
    pub async fn create_mod(&self, mod_info: &Mod) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO mods (
                id, provider, project_id, version_id, filename, sha1,
                server_id, enabled, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&mod_info.id)
        .bind(&mod_info.provider)
        .bind(&mod_info.project_id)
        .bind(&mod_info.version_id)
        .bind(&mod_info.filename)
        .bind(&mod_info.sha1)
        .bind(&mod_info.server_id)
        .bind(mod_info.enabled)
        .bind(mod_info.created_at)
        .bind(mod_info.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Created mod: {}", mod_info.id);
        Ok(())
    }

    pub async fn get_mod(&self, id: &str) -> Result<Option<Mod>> {
        let row = sqlx::query(
            r#"
            SELECT id, provider, project_id, version_id, filename, sha1,
                   server_id, enabled, created_at, updated_at
            FROM mods WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Mod {
                id: row.get("id"),
                provider: row.get("provider"),
                project_id: row.get("project_id"),
                version_id: row.get("version_id"),
                filename: row.get("filename"),
                sha1: row.get("sha1"),
                server_id: row.get("server_id"),
                enabled: row.get("enabled"),
                category: row.get("category"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_mods_by_server(&self, server_id: &str) -> Result<Vec<Mod>> {
        let rows = sqlx::query(
            r#"
            SELECT id, provider, project_id, version_id, filename, sha1,
                   server_id, enabled, created_at, updated_at
            FROM mods WHERE server_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await?;

        let mods = rows
            .into_iter()
            .map(|row| Mod {
                id: row.get("id"),
                provider: row.get("provider"),
                project_id: row.get("project_id"),
                version_id: row.get("version_id"),
                filename: row.get("filename"),
                sha1: row.get("sha1"),
                server_id: row.get("server_id"),
                enabled: row.get("enabled"),
                category: row.get("category"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(mods)
    }

    pub async fn update_mod(&self, mod_info: &Mod) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE mods SET
                provider = ?, project_id = ?, version_id = ?, filename = ?,
                sha1 = ?, server_id = ?, enabled = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&mod_info.provider)
        .bind(&mod_info.project_id)
        .bind(&mod_info.version_id)
        .bind(&mod_info.filename)
        .bind(&mod_info.sha1)
        .bind(&mod_info.server_id)
        .bind(mod_info.enabled)
        .bind(mod_info.updated_at)
        .bind(&mod_info.id)
        .execute(&self.pool)
        .await?;

        info!("Updated mod: {}", mod_info.id);
        Ok(())
    }

    pub async fn delete_mod(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM mods WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        info!("Deleted mod: {}", id);
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

    pub async fn search_mods(&self, params: &crate::api::ModSearchQuery) -> Result<Vec<Mod>> {
        let mut query = sqlx::query(
            r#"
            SELECT id, provider, project_id, version_id, filename, sha1, server_id, enabled, created_at, updated_at
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
            .map(|row| Mod {
                id: row.get("id"),
                provider: row.get("provider"),
                project_id: row.get("project_id"),
                version_id: row.get("version_id"),
                filename: row.get("filename"),
                sha1: row.get("sha1"),
                server_id: row.get("server_id"),
                enabled: row.get("enabled"),
                category: row.get("category"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(mods)
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

    /// Log a server message
    pub async fn log_server_message(
        &self,
        server_id: &str,
        level: &str,
        message: &str,
        component: Option<&str>,
    ) -> Result<()> {
        let log = ServerLog {
            id: Uuid::new_v4().to_string(),
            server_id: server_id.to_string(),
            level: level.to_string(),
            message: message.to_string(),
            component: component.map(|s| s.to_string()),
            timestamp: chrono::Utc::now(),
        };
        
        sqlx::query(
            "INSERT INTO server_logs (id, server_id, level, message, component, timestamp) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&log.id)
        .bind(&log.server_id)
        .bind(&log.level)
        .bind(&log.message)
        .bind(&log.component)
        .bind(log.timestamp)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    /// Get database health status
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        // Check database connectivity
        let _: i64 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        
        // Get basic stats
        let server_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM servers")
            .fetch_one(&self.pool)
            .await?;
        
        let log_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM server_logs")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(HealthStatus {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            details: Some(format!("Servers: {}, Logs: {}", server_count, log_count)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let db = DatabaseManager::new(&database_url).await.expect("Failed to create test database");
        
        // Test creating a server
        let server = ServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            minecraft_version: "1.21.1".to_string(),
            loader: "vanilla".to_string(),
            loader_version: "1.21.1".to_string(),
            host: "localhost".to_string(),
            port: 25565,
            rcon_port: 25575,
            query_port: 25566,
            max_players: 20,
            memory: 4096,
            java_args: "[]".to_string(),
            server_args: "[]".to_string(),
            auto_start: true,
            auto_restart: true,
            world_name: "world".to_string(),
            difficulty: "normal".to_string(),
            gamemode: "survival".to_string(),
            pvp: true,
            online_mode: true,
            whitelist: false,
            enable_command_block: false,
            view_distance: 10,
            simulation_distance: 10,
            motd: "A Minecraft Server".to_string(),
            java_path: "/usr/bin/java".to_string(),
            jvm_args: "-Xmx4G".to_string(),
            server_jar: "server.jar".to_string(),
            rcon_password: "password".to_string(),
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
