use crate::external_apis::{CurseForgeApiClient, ModrinthApiClient};
use crate::database::{DatabaseManager, ModDependency};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Version information for a mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub id: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    pub filename: String,
    pub file_size: u64,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub download_url: String,
    pub release_type: String, // "release", "beta", "alpha"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod metadata for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub provider: String, // "curseforge", "modrinth"
    pub project_id: String,
    pub slug: Option<String>,
    pub category: String,
    pub side: String, // "client", "server", "both"
    pub website_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Installed mod with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModWithMetadata {
    pub id: String,
    pub server_id: String,
    pub metadata: ModMetadata,
    pub version: ModVersion,
    pub file_path: String,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
}

/// Dependency resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolution {
    pub mod_id: String,
    pub version: String,
    pub dependencies: Vec<ResolvedDependency>,
    pub conflicts: Vec<DependencyConflict>,
}

/// Resolved dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub mod_id: String,
    pub version: String,
    pub required: bool,
    pub version_range: String,
}

/// Dependency conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConflict {
    pub mod_id: String,
    pub conflicting_mod_id: String,
    pub reason: String,
    pub severity: ConflictSeverity,
}

/// Conflict severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Warning,
    Error,
    Critical,
}

/// Version resolver for mods
#[derive(Clone)]
pub struct VersionResolver {
    curseforge_client: Option<CurseForgeApiClient>,
    modrinth_client: ModrinthApiClient,
    database: DatabaseManager,
}

impl VersionResolver {
    /// Create a new version resolver
    pub fn new(curseforge_client: Option<CurseForgeApiClient>, database: DatabaseManager) -> Self {
        Self {
            curseforge_client,
            modrinth_client: ModrinthApiClient::new(),
            database,
        }
    }

    /// Resolve "latest" version for a mod
    pub async fn resolve_latest_version(
        &self,
        provider: &str,
        project_id: &str,
        minecraft_version: &str,
        loader: &str,
    ) -> Result<ModVersion, Box<dyn Error>> {
        match provider {
            "curseforge" => {
                if let Some(ref client) = self.curseforge_client {
                    self.resolve_curseforge_latest(client, project_id, minecraft_version, loader).await
                } else {
                    Err("CurseForge API client not available".into())
                }
            }
            "modrinth" => {
                self.resolve_modrinth_latest(project_id, minecraft_version, loader).await
            }
            _ => Err(format!("Unknown provider: {}", provider).into())
        }
    }

    /// Resolve latest version from CurseForge
    async fn resolve_curseforge_latest(
        &self,
        client: &CurseForgeApiClient,
        project_id: &str,
        minecraft_version: &str,
        loader: &str,
    ) -> Result<ModVersion, Box<dyn Error>> {
        let project_id = project_id.parse::<u32>()?;
        
        // Get project files
        let files = client.get_project_files(project_id, Some(minecraft_version), None, None, None, None).await?;
        
        // Filter by Minecraft version and loader
        let compatible_files: Vec<_> = files.into_iter()
            .filter(|file| {
                file.game_versions.contains(&minecraft_version.to_string()) &&
                file.sortable_game_versions.iter().any(|v| v.game_version == minecraft_version)
            })
            .collect();
        
        if compatible_files.is_empty() {
            return Err(format!("No compatible files found for MC {} with loader {}", minecraft_version, loader).into());
        }
        
        // Sort by release type and date (prefer release, then beta, then alpha)
        let mut sorted_files = compatible_files;
        sorted_files.sort_by(|a, b| {
            // Sort by release type first
            let type_order = |rt: u32| match rt {
                1 => 0, // Release
                2 => 1, // Beta
                3 => 2, // Alpha
                _ => 3,
            };
            
            let type_cmp = type_order(a.release_type).cmp(&type_order(b.release_type));
            if type_cmp == std::cmp::Ordering::Equal {
                // Then by date (newest first)
                b.file_date.cmp(&a.file_date)
            } else {
                type_cmp
            }
        });
        
        let latest_file = sorted_files.first().ok_or("No files found")?;
        
        // Get SHA1 hash
        let sha1 = latest_file.hashes.iter()
            .find(|h| h.algo == 1) // SHA1
            .map(|h| h.value.clone());
        
        // Get SHA512 hash
        let sha512 = latest_file.hashes.iter()
            .find(|h| h.algo == 2) // SHA512
            .map(|h| h.value.clone());
        
        Ok(ModVersion {
            id: latest_file.id.to_string(),
            version: latest_file.display_name.clone(),
            minecraft_version: minecraft_version.to_string(),
            loader: loader.to_string(),
            filename: latest_file.file_name.clone(),
            file_size: latest_file.file_length,
            sha1,
            sha512,
            download_url: latest_file.download_url.clone(),
            release_type: match latest_file.release_type {
                1 => "release".to_string(),
                2 => "beta".to_string(),
                3 => "alpha".to_string(),
                _ => "unknown".to_string(),
            },
            created_at: latest_file.file_date.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: Utc::now(),
        })
    }

    /// Resolve latest version from Modrinth
    async fn resolve_modrinth_latest(
        &self,
        project_id: &str,
        minecraft_version: &str,
        loader: &str,
    ) -> Result<ModVersion, Box<dyn Error>> {
        // Get project versions
        let versions = self.modrinth_client.get_project_versions(project_id, Some(vec![minecraft_version]), Some(vec![loader])).await?;
        
        // Filter by Minecraft version and loader
        let compatible_versions: Vec<_> = versions.into_iter()
            .filter(|version| {
                version.game_versions.contains(&minecraft_version.to_string()) &&
                version.loaders.contains(&loader.to_string())
            })
            .collect();
        
        if compatible_versions.is_empty() {
            return Err(format!("No compatible versions found for MC {} with loader {}", minecraft_version, loader).into());
        }
        
        // Sort by version number and date
        let mut sorted_versions = compatible_versions;
        sorted_versions.sort_by(|a, b| {
            // Sort by version number (semantic versioning)
            version_compare(&a.version_number, &b.version_number)
                .unwrap_or_else(|| b.date_published.cmp(&a.date_published))
        });
        
        let latest_version = sorted_versions.first().ok_or("No versions found")?;
        
        // Get the primary file
        let primary_file = latest_version.files.first().ok_or("No files in version")?;
        
        Ok(ModVersion {
            id: latest_version.id.clone(),
            version: latest_version.version_number.clone(),
            minecraft_version: minecraft_version.to_string(),
            loader: loader.to_string(),
            filename: primary_file.filename.clone(),
            file_size: primary_file.size,
            sha1: primary_file.hashes.get("sha1").cloned(),
            sha512: primary_file.hashes.get("sha512").cloned(),
            download_url: primary_file.url.clone(),
            release_type: latest_version.version_type.clone(),
            created_at: latest_version.date_published.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| Utc::now()),
            updated_at: Utc::now(),
        })
    }

    /// Get mod metadata
    pub async fn get_mod_metadata(
        &self,
        provider: &str,
        project_id: &str,
    ) -> Result<ModMetadata, Box<dyn Error>> {
        match provider {
            "curseforge" => {
                if let Some(ref client) = self.curseforge_client {
                    self.get_curseforge_metadata(client, project_id).await
                } else {
                    Err("CurseForge API client not available".into())
                }
            }
            "modrinth" => {
                self.get_modrinth_metadata(project_id).await
            }
            _ => Err(format!("Unknown provider: {}", provider).into())
        }
    }

    /// Get CurseForge mod metadata
    async fn get_curseforge_metadata(
        &self,
        client: &CurseForgeApiClient,
        project_id: &str,
    ) -> Result<ModMetadata, Box<dyn Error>> {
        let project_id = project_id.parse::<u32>()?;
        let project = client.get_project(project_id).await?;
        
        Ok(ModMetadata {
            id: project.id.to_string(),
            name: project.name.clone(),
            description: project.summary.clone(),
            author: project.authors.first().map(|a| a.name.clone()).unwrap_or_default(),
            provider: "curseforge".to_string(),
            project_id: project.id.to_string(),
            slug: Some(project.slug.clone()),
            category: "mod".to_string(), // CurseForge doesn't have a simple category field
            side: "both".to_string(), // Default to both, would need to check mod metadata
            website_url: project.links.website_url.clone(),
            source_url: None,
            issues_url: None,
            created_at: project.date_created.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: project.date_modified.parse().unwrap_or_else(|_| Utc::now()),
        })
    }

    /// Get Modrinth mod metadata
    async fn get_modrinth_metadata(
        &self,
        project_id: &str,
    ) -> Result<ModMetadata, Box<dyn Error>> {
        let project = self.modrinth_client.get_project(project_id).await?;
        
        Ok(ModMetadata {
            id: project.id.clone(),
            name: project.title.clone(),
            description: project.description.clone(),
            author: project.team.clone(),
            provider: "modrinth".to_string(),
            project_id: project.id.clone(),
            slug: Some(project.slug.clone()),
            category: project.categories.first().cloned().unwrap_or_default(),
            side: project.server_side.to_string(),
            website_url: None,
            source_url: Some(project.source_url.clone().unwrap_or_default()),
            issues_url: Some(project.issues_url.clone().unwrap_or_default()),
            created_at: project.published.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| Utc::now()),
            updated_at: project.updated.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| Utc::now()),
        })
    }

    /// Resolve dependencies for a mod
    pub async fn resolve_dependencies(
        &self,
        mod_id: &str,
        version: &str,
        minecraft_version: &str,
        loader: &str,
    ) -> Result<DependencyResolution, Box<dyn Error>> {
        // Get dependencies from database
        let dependencies = self.database.get_mod_dependencies(mod_id).await?;

        // Resolve each dependency
        let mut resolved_dependencies = Vec::new();
        let mut conflicts = Vec::new();

        for dependency in dependencies {
            match self.resolve_single_dependency(
                &dependency,
                minecraft_version,
                loader,
            ).await {
                Ok(resolved) => {
                    resolved_dependencies.push(resolved);
                }
                Err(e) => {
                    conflicts.push(DependencyConflict {
                        mod_id: dependency.dependency_mod_id.clone(),
                        conflicting_mod_id: mod_id.to_string(),
                        reason: e.to_string(),
                        severity: if dependency.required {
                            ConflictSeverity::Error
                        } else {
                            ConflictSeverity::Warning
                        },
                    });
                }
            }
        }

        Ok(DependencyResolution {
            mod_id: mod_id.to_string(),
            version: version.to_string(),
            dependencies: resolved_dependencies,
            conflicts,
        })
    }

    /// Auto-resolve all dependencies for a modpack
    pub async fn auto_resolve_dependencies(
        &self,
        mod_ids: Vec<String>,
        minecraft_version: &str,
        loader: &str,
    ) -> Result<Vec<DependencyResolution>, Box<dyn Error>> {
        let mut resolved_mods = HashMap::new();
        let mut to_resolve = mod_ids;

        while !to_resolve.is_empty() {
            let mut new_dependencies = Vec::new();

            for mod_id in to_resolve {
                if resolved_mods.contains_key(&mod_id) {
                    continue;
                }

                // Get mod metadata
                let mod_metadata = self.database.get_mod_metadata(&mod_id).await?;
                if let Some(metadata) = mod_metadata {
                    let latest_version = self.resolve_latest_version(
                        &metadata.provider,
                        &metadata.project_id,
                        minecraft_version,
                        loader,
                    ).await?;

                    // Resolve dependencies
                    let resolution = self.resolve_dependencies(
                        &mod_id,
                        &latest_version.version,
                        minecraft_version,
                        loader,
                    ).await?;

                    // Add new dependencies to resolve list
                    for dep in &resolution.dependencies {
                        if !resolved_mods.contains_key(&dep.mod_id) {
                            new_dependencies.push(dep.mod_id.clone());
                        }
                    }

                    resolved_mods.insert(mod_id, resolution);
                }
            }

            to_resolve = new_dependencies;
        }

        Ok(resolved_mods.into_values().collect())
    }

    /// Resolve a single dependency
    async fn resolve_single_dependency(
        &self,
        dependency: &ModDependency,
        minecraft_version: &str,
        loader: &str,
    ) -> Result<ResolvedDependency, Box<dyn Error>> {
        // Get dependency mod metadata
        let dep_metadata = self.database.get_mod_metadata(&dependency.dependency_mod_id).await?;
        
        if let Some(metadata) = dep_metadata {
            // Resolve latest version
            let latest_version = self.resolve_latest_version(
                &metadata.provider,
                &metadata.project_id,
                minecraft_version,
                loader,
            ).await?;

            // Check version compatibility
            if self.is_version_compatible(&latest_version.version, &dependency.version_range)? {
                Ok(ResolvedDependency {
                    mod_id: dependency.dependency_mod_id.clone(),
                    version: latest_version.version,
                    required: dependency.required,
                    version_range: dependency.version_range.clone(),
                })
            } else {
                Err(format!(
                    "Version {} does not satisfy range {}",
                    latest_version.version,
                    dependency.version_range
                ).into())
            }
        } else {
            Err(format!("Dependency mod not found: {}", dependency.dependency_mod_id).into())
        }
    }

    /// Check if a version is compatible with a version range
    fn is_version_compatible(&self, version: &str, range: &str) -> Result<bool, Box<dyn Error>> {
        // Simple version range checking - can be enhanced with proper semver parsing
        if range == "*" || range == "latest" {
            return Ok(true);
        }

        if range.starts_with(">=") {
            let min_version = &range[2..];
            Ok(version >= min_version)
        } else if range.starts_with(">") {
            let min_version = &range[1..];
            Ok(version > min_version)
        } else if range.starts_with("<=") {
            let max_version = &range[2..];
            Ok(version <= max_version)
        } else if range.starts_with("<") {
            let max_version = &range[1..];
            Ok(version < max_version)
        } else if range.contains("..") {
            let parts: Vec<&str> = range.split("..").collect();
            if parts.len() == 2 {
                let min_version = parts[0];
                let max_version = parts[1];
                Ok(version >= min_version && version <= max_version)
            } else {
                Ok(false)
            }
        } else {
            // Exact version match
            Ok(version == range)
        }
    }

    /// Store resolved dependencies in database
    pub async fn store_resolved_dependencies(
        &self,
        resolution: &DependencyResolution,
    ) -> Result<(), Box<dyn Error>> {
        for dependency in &resolution.dependencies {
            // Create or update mod metadata if needed
            let mod_metadata = self.database.get_mod_metadata(&dependency.mod_id).await?;
            if mod_metadata.is_none() {
                // Get metadata from API
                let metadata = self.get_mod_metadata("modrinth", &dependency.mod_id).await?;
                let db_metadata = crate::database::ModMetadata {
                    id: metadata.id,
                    name: metadata.name,
                    description: metadata.description,
                    author: metadata.author,
                    provider: metadata.provider,
                    project_id: metadata.project_id,
                    slug: metadata.slug,
                    category: metadata.category,
                    side: metadata.side,
                    website_url: metadata.website_url,
                    source_url: metadata.source_url,
                    issues_url: metadata.issues_url,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                };
                self.database.create_mod_metadata(&db_metadata).await?;
            }

            // Create mod version
            let mod_version = ModVersion {
                id: Uuid::new_v4().to_string(),
                version: dependency.version.clone(),
                minecraft_version: "1.21.1".to_string(), // TODO: Get from context
                loader: "fabric".to_string(), // TODO: Get from context
                filename: format!("{}-{}.jar", dependency.mod_id, dependency.version),
                file_size: 0, // TODO: Get from API
                sha1: None,
                sha512: None,
                download_url: String::new(), // TODO: Get from API
                release_type: "release".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Store in database
            let db_mod_version = crate::database::ModVersion {
                id: mod_version.id,
                mod_metadata_id: dependency.mod_id.clone(),
                version: mod_version.version,
                minecraft_version: mod_version.minecraft_version,
                loader: mod_version.loader,
                filename: mod_version.filename,
                file_size: mod_version.file_size,
                sha1: mod_version.sha1,
                sha256: None,
                sha512: mod_version.sha512,
                download_url: mod_version.download_url,
                release_type: mod_version.release_type,
                created_at: mod_version.created_at,
                updated_at: mod_version.updated_at,
            };
            self.database.create_mod_version(&db_mod_version).await?;
        }

        Ok(())
    }
}

/// Compare semantic versions
fn version_compare(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    
    let max_len = a_parts.len().max(b_parts.len());
    
    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        
        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return Some(other),
        }
    }
    
    Some(std::cmp::Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert_eq!(version_compare("1.2.3", "1.2.4"), Some(std::cmp::Ordering::Less));
        assert_eq!(version_compare("1.2.4", "1.2.3"), Some(std::cmp::Ordering::Greater));
        assert_eq!(version_compare("1.2.3", "1.2.3"), Some(std::cmp::Ordering::Equal));
        assert_eq!(version_compare("1.2.3", "1.2"), Some(std::cmp::Ordering::Greater));
        assert_eq!(version_compare("1.2", "1.2.3"), Some(std::cmp::Ordering::Less));
    }
}
