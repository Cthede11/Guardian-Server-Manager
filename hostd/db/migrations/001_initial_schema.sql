-- Initial database schema for Guardian Server Manager
-- This migration creates the core tables for server management

-- Servers table
CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    minecraft_version TEXT NOT NULL,
    loader TEXT NOT NULL,
    loader_version TEXT NOT NULL,
    port INTEGER NOT NULL,
    rcon_port INTEGER NOT NULL,
    query_port INTEGER NOT NULL,
    max_players INTEGER NOT NULL,
    memory INTEGER NOT NULL,
    java_args TEXT NOT NULL,
    server_args TEXT NOT NULL,
    auto_start BOOLEAN NOT NULL DEFAULT 0,
    auto_restart BOOLEAN NOT NULL DEFAULT 0,
    world_name TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    gamemode TEXT NOT NULL,
    pvp BOOLEAN NOT NULL DEFAULT 1,
    online_mode BOOLEAN NOT NULL DEFAULT 1,
    whitelist BOOLEAN NOT NULL DEFAULT 0,
    enable_command_block BOOLEAN NOT NULL DEFAULT 0,
    view_distance INTEGER NOT NULL,
    simulation_distance INTEGER NOT NULL,
    motd TEXT NOT NULL,
    host TEXT NOT NULL,
    java_path TEXT NOT NULL,
    jvm_args TEXT NOT NULL,
    server_jar TEXT NOT NULL,
    rcon_password TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Server logs table
CREATE TABLE IF NOT EXISTS server_logs (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    source TEXT,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- Server metrics table
CREATE TABLE IF NOT EXISTS server_metrics (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tps REAL NOT NULL,
    tick_p95 REAL NOT NULL,
    heap_mb INTEGER NOT NULL,
    players_online INTEGER NOT NULL,
    gpu_queue_ms REAL NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_usage INTEGER NOT NULL,
    disk_usage INTEGER NOT NULL,
    network_in INTEGER NOT NULL,
    network_out INTEGER NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role_id TEXT,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_login DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    permissions TEXT NOT NULL, -- JSON string
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Backup targets table
CREATE TABLE IF NOT EXISTS backup_targets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL, -- 'local', 's3', 'ftp', etc.
    config TEXT NOT NULL, -- JSON string
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Server backups table
CREATE TABLE IF NOT EXISTS server_backups (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    name TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    status TEXT NOT NULL, -- 'pending', 'in_progress', 'completed', 'failed'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES backup_targets(id) ON DELETE CASCADE
);

-- Modpacks table
CREATE TABLE IF NOT EXISTS modpacks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL,
    minecraft_version TEXT NOT NULL,
    loader TEXT NOT NULL,
    loader_version TEXT NOT NULL,
    mods TEXT NOT NULL, -- JSON string
    is_public BOOLEAN NOT NULL DEFAULT 0,
    created_by TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Server modpacks table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS server_modpacks (
    server_id TEXT NOT NULL,
    modpack_id TEXT NOT NULL,
    installed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (server_id, modpack_id),
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (modpack_id) REFERENCES modpacks(id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_server_logs_server_id ON server_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_server_logs_timestamp ON server_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_server_metrics_server_id ON server_metrics(server_id);
CREATE INDEX IF NOT EXISTS idx_server_metrics_timestamp ON server_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_server_backups_server_id ON server_backups(server_id);
CREATE INDEX IF NOT EXISTS idx_server_backups_status ON server_backups(status);
CREATE INDEX IF NOT EXISTS idx_modpacks_minecraft_version ON modpacks(minecraft_version);
CREATE INDEX IF NOT EXISTS idx_modpacks_loader ON modpacks(loader);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
