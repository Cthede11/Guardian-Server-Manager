-- Enhanced mod metadata schema
-- This migration adds comprehensive mod metadata support

-- Mod metadata table
CREATE TABLE IF NOT EXISTS mod_metadata (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    author TEXT NOT NULL,
    provider TEXT NOT NULL, -- 'curseforge', 'modrinth'
    project_id TEXT NOT NULL,
    slug TEXT,
    category TEXT NOT NULL,
    side TEXT NOT NULL, -- 'client', 'server', 'both'
    website_url TEXT,
    source_url TEXT,
    issues_url TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, project_id)
);

-- Mod versions table
CREATE TABLE IF NOT EXISTS mod_versions (
    id TEXT PRIMARY KEY,
    mod_metadata_id TEXT NOT NULL,
    version TEXT NOT NULL,
    minecraft_version TEXT NOT NULL,
    loader TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    sha1 TEXT,
    sha512 TEXT,
    download_url TEXT NOT NULL,
    release_type TEXT NOT NULL, -- 'release', 'beta', 'alpha'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mod_metadata_id) REFERENCES mod_metadata(id) ON DELETE CASCADE,
    UNIQUE(mod_metadata_id, version, minecraft_version, loader)
);

-- Installed mods table (replaces the old mods table)
CREATE TABLE IF NOT EXISTS installed_mods (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    mod_metadata_id TEXT NOT NULL,
    mod_version_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    installed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_metadata_id) REFERENCES mod_metadata(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_version_id) REFERENCES mod_versions(id) ON DELETE CASCADE,
    UNIQUE(server_id, mod_metadata_id)
);

-- Mod dependencies table
CREATE TABLE IF NOT EXISTS mod_dependencies (
    id TEXT PRIMARY KEY,
    mod_metadata_id TEXT NOT NULL,
    dependency_mod_id TEXT NOT NULL,
    version_range TEXT NOT NULL,
    required BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mod_metadata_id) REFERENCES mod_metadata(id) ON DELETE CASCADE,
    FOREIGN KEY (dependency_mod_id) REFERENCES mod_metadata(id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_mod_metadata_provider ON mod_metadata(provider);
CREATE INDEX IF NOT EXISTS idx_mod_metadata_project_id ON mod_metadata(project_id);
CREATE INDEX IF NOT EXISTS idx_mod_metadata_category ON mod_metadata(category);
CREATE INDEX IF NOT EXISTS idx_mod_metadata_side ON mod_metadata(side);

CREATE INDEX IF NOT EXISTS idx_mod_versions_mod_metadata_id ON mod_versions(mod_metadata_id);
CREATE INDEX IF NOT EXISTS idx_mod_versions_minecraft_version ON mod_versions(minecraft_version);
CREATE INDEX IF NOT EXISTS idx_mod_versions_loader ON mod_versions(loader);
CREATE INDEX IF NOT EXISTS idx_mod_versions_release_type ON mod_versions(release_type);

CREATE INDEX IF NOT EXISTS idx_installed_mods_server_id ON installed_mods(server_id);
CREATE INDEX IF NOT EXISTS idx_installed_mods_mod_metadata_id ON installed_mods(mod_metadata_id);
CREATE INDEX IF NOT EXISTS idx_installed_mods_enabled ON installed_mods(enabled);

CREATE INDEX IF NOT EXISTS idx_mod_dependencies_mod_metadata_id ON mod_dependencies(mod_metadata_id);
CREATE INDEX IF NOT EXISTS idx_mod_dependencies_dependency_mod_id ON mod_dependencies(dependency_mod_id);
CREATE INDEX IF NOT EXISTS idx_mod_dependencies_required ON mod_dependencies(required);
