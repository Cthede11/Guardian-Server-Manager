use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use crate::external_apis::mod_provider::ModProvider;
use crate::mod_manager::ModDependency;
use crate::mod_manager::ModInfo;
use chrono::Utc;

/// Modrinth API client for fetching mod data
#[derive(Clone)]
pub struct ModrinthApiClient {
    client: Client,
    base_url: String,
}

/// Modrinth project response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProject {
    pub id: String,
    pub slug: String,
    pub project_type: String,
    pub team: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: String,
    pub updated: String,
    pub approved: Option<String>,
    pub status: String,
    pub moderator_message: Option<String>,
    pub license: ModrinthLicense,
    pub client_side: String,
    pub server_side: String,
    pub downloads: u64,
    pub followers: u64,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub versions: Vec<String>,
    pub icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Vec<ModrinthDonationUrl>,
    pub gallery: Vec<ModrinthGalleryImage>,
    pub color: Option<u32>,
    pub thread_id: Option<String>,
    pub monetization_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthLicense {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthDonationUrl {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthGalleryImage {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: String,
    pub ordering: i32,
}

/// Modrinth version response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub changelog_url: Option<String>,
    pub date_published: String,
    pub downloads: u64,
    pub version_type: String,
    pub status: String,
    pub requested_status: Option<String>,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

/// Modrinth search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthSearchResponse {
    pub hits: Vec<ModrinthProject>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

impl ModrinthApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }

    /// Search for mods
    pub async fn search_mods(
        &self,
        query: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
        category: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<ModrinthSearchResponse> {
        let mut params = vec![
            ("query", query.to_string()),
        ];
        
        let facets = self.build_facets(game_version, loader, category)?;
        if !facets.is_empty() && facets != "[]" {
            params.push(("facets", facets));
        }

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            params.push(("offset", offset.to_string()));
        }

        let url = format!("{}/search", self.base_url);
        let response = self.client
            .get(&url)
            .query(&params)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status() == 400 {
                warn!("Modrinth API bad request, returning empty results");
                return Ok(ModrinthSearchResponse {
                    hits: vec![],
                    offset: 0,
                    limit: 0,
                    total_hits: 0,
                });
            }
            error!("Modrinth API error: {}", response.status());
            return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
        }

        let search_response: ModrinthSearchResponse = response.json().await?;
        info!("Found {} mods on Modrinth", search_response.total_hits);
        Ok(search_response)
    }

    /// Get project details
    pub async fn get_project(&self, project_id: &str) -> Result<ModrinthProject> {
        let url = format!("{}/project/{}", self.base_url, project_id);
        let response = self.client
            .get(&url)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Modrinth API error for project {}: {}", project_id, response.status());
            return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
        }

        let project: ModrinthProject = response.json().await?;
        Ok(project)
    }

    /// Get project versions
    pub async fn get_project_versions(
        &self,
        project_id: &str,
        game_versions: Option<Vec<&str>>,
        loaders: Option<Vec<&str>>,
    ) -> Result<Vec<ModrinthVersion>> {
        let mut params = vec![];
        
        if let Some(versions) = game_versions {
            for version in versions {
                params.push(("game_versions", version));
            }
        }
        
        if let Some(loaders) = loaders {
            for loader in loaders {
                params.push(("loaders", loader));
            }
        }

        let url = format!("{}/project/{}/version", self.base_url, project_id);
        let response = self.client
            .get(&url)
            .query(&params)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Modrinth API error for project versions {}: {}", project_id, response.status());
            return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
        }

        let versions: Vec<ModrinthVersion> = response.json().await?;
        Ok(versions)
    }

    /// Get specific version
    pub async fn get_version(&self, version_id: &str) -> Result<ModrinthVersion> {
        let url = format!("{}/version/{}", self.base_url, version_id);
        let response = self.client
            .get(&url)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Modrinth API error for version {}: {}", version_id, response.status());
            return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
        }

        let version: ModrinthVersion = response.json().await?;
        Ok(version)
    }

    /// Get game versions
    pub async fn get_game_versions(&self) -> Result<Vec<String>> {
        let url = format!("{}/tag/game_version", self.base_url);
        let response = self.client
            .get(&url)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Modrinth API error for game versions: {}", response.status());
            return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
        }

        let versions: Vec<serde_json::Value> = response.json().await?;
        let version_strings: Vec<String> = versions
            .into_iter()
            .filter_map(|v| v.get("version").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .collect();

        Ok(version_strings)
    }

    /// Get loaders
    pub async fn get_loaders(&self) -> Result<Vec<String>> {
        let url = format!("{}/tag/loader", self.base_url);
        let response = self.client
            .get(&url)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Modrinth API error for loaders: {}", response.status());
            return Err(anyhow::anyhow!("Modrinth API error: {}", response.status()));
        }

        let loaders: Vec<serde_json::Value> = response.json().await?;
        let loader_strings: Vec<String> = loaders
            .into_iter()
            .filter_map(|v| v.get("name").and_then(|s| s.as_str()).map(|s| s.to_string()))
            .collect();

        Ok(loader_strings)
    }

    /// Build facets for search
    fn build_facets(
        &self,
        game_version: Option<&str>,
        loader: Option<&str>,
        category: Option<&str>,
    ) -> Result<String> {
        let mut facets = Vec::new();

        if let Some(version) = game_version {
            facets.push(format!(r#"["versions:{}"]"#, version));
        }

        if let Some(loader) = loader {
            facets.push(format!(r#"["categories:{}"]"#, loader));
        }

        if let Some(category) = category {
            facets.push(format!(r#"["categories:{}"]"#, category));
        }

        if facets.is_empty() {
            Ok("[]".to_string())
        } else {
            Ok(format!("[{}]", facets.join(",")))
        }
    }

    /// Convert Modrinth project to our ModInfo format
    pub fn convert_to_mod_info(&self, project: &ModrinthProject) -> crate::database::ModInfo {
        crate::database::ModInfo {
            id: project.id.clone(),
            name: project.title.clone(),
            description: Some(project.description.clone()),
            category: self.map_category(&project.categories),
            side: self.map_side(&project.client_side, &project.server_side),
            source: "modrinth".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Map Modrinth categories to our category system
    fn map_category(&self, categories: &[String]) -> String {
        // Map Modrinth categories to our category system
        for category in categories {
            match category.as_str() {
                "adventure" => return "adventure".to_string(),
                "cursed" => return "miscellaneous".to_string(),
                "decoration" => return "building".to_string(),
                "economy" => return "economy".to_string(),
                "equipment" => return "utility".to_string(),
                "food" => return "utility".to_string(),
                "game-mechanics" => return "core".to_string(),
                "library" => return "library".to_string(),
                "magic" => return "magic".to_string(),
                "management" => return "utility".to_string(),
                "minigame" => return "miscellaneous".to_string(),
                "mobs" => return "mobs".to_string(),
                "optimization" => return "optimization".to_string(),
                "social" => return "utility".to_string(),
                "storage" => return "utility".to_string(),
                "technology" => return "technology".to_string(),
                "transportation" => return "transportation".to_string(),
                "utility" => return "utility".to_string(),
                "worldgen" => return "world_generation".to_string(),
                _ => continue,
            }
        }
        "miscellaneous".to_string()
    }

    /// Map Modrinth side to our side system
    fn map_side(&self, client_side: &str, server_side: &str) -> String {
        match (client_side, server_side) {
            ("required", "required") => "both".to_string(),
            ("required", "optional") => "client".to_string(),
            ("optional", "required") => "server".to_string(),
            ("optional", "optional") => "universal".to_string(),
            ("unsupported", "required") => "server".to_string(),
            ("required", "unsupported") => "client".to_string(),
            _ => "universal".to_string(),
        }
    }
}

impl Default for ModrinthApiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ModProvider for ModrinthApiClient {
    async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ModInfo>, Box<dyn std::error::Error>> {
        let search_results = self.search_mods(
            query,
            minecraft_version,
            loader,
            None, // category
            limit.map(|l| l as u32),
            None, // offset
        ).await?;
        
        let mut mod_infos = Vec::new();
        
        for result in search_results.hits {
            if let Ok(mod_info) = self.get_mod(&result.id).await {
                mod_infos.push(mod_info);
            }
        }
        
        Ok(mod_infos)
    }

    async fn get_mod(&self, mod_id: &str) -> Result<ModInfo, Box<dyn std::error::Error>> {
        let project = self.get_project(mod_id).await?;
        let versions = self.get_project_versions(mod_id, None, None).await?;
        
        if let Some(version_info) = versions.first() {
            Ok(ModInfo {
                id: project.id,
                name: project.title,
                description: project.description,
                author: "Unknown".to_string(), // Modrinth doesn't provide author in project response
                version: version_info.version_number.clone(),
                minecraft_version: version_info.game_versions.first().cloned().unwrap_or_else(|| "1.20.1".to_string()),
                loader: version_info.loaders.first().cloned().unwrap_or_else(|| "fabric".to_string()),
                category: project.categories.first().cloned().unwrap_or_else(|| "misc".to_string()),
                side: if project.client_side == "required" && project.server_side == "required" {
                    "both".to_string()
                } else if project.client_side == "required" {
                    "client".to_string()
                } else if project.server_side == "required" {
                    "server".to_string()
                } else {
                    "both".to_string()
                },
                download_url: version_info.files.first().map(|f| f.url.clone()),
                file_size: version_info.files.first().map(|f| f.size),
                sha1: version_info.files.first().and_then(|f| f.hashes.get("sha1").cloned()),
                dependencies: version_info.dependencies.iter().map(|d| ModDependency {
                    mod_id: d.project_id.clone().unwrap_or_else(|| "unknown".to_string()),
                    version_range: "any".to_string(),
                    required: d.dependency_type == "required",
                }).collect(),
                created_at: project.published.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: project.updated.parse().unwrap_or_else(|_| Utc::now()),
            })
        } else {
            Err("No versions found for the project".into())
        }
    }

    async fn get_mod_version(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        let project = self.get_project(mod_id).await?;
        let versions = self.get_project_versions(mod_id, Some(vec![version]), None).await?;
        
        if let Some(version_info) = versions.first() {
            Ok(ModInfo {
                id: project.id,
                name: project.title,
                description: project.description,
                author: "Unknown".to_string(),
                version: version_info.version_number.clone(),
                minecraft_version: version_info.game_versions.first().cloned().unwrap_or_else(|| "1.20.1".to_string()),
                loader: version_info.loaders.first().cloned().unwrap_or_else(|| "fabric".to_string()),
                category: project.categories.first().cloned().unwrap_or_else(|| "misc".to_string()),
                side: if project.client_side == "required" && project.server_side == "required" {
                    "both".to_string()
                } else if project.client_side == "required" {
                    "client".to_string()
                } else if project.server_side == "required" {
                    "server".to_string()
                } else {
                    "both".to_string()
                },
                download_url: version_info.files.first().map(|f| f.url.clone()),
                file_size: version_info.files.first().map(|f| f.size),
                sha1: version_info.files.first().and_then(|f| f.hashes.get("sha1").cloned()),
                dependencies: version_info.dependencies.iter().map(|d| ModDependency {
                    mod_id: d.project_id.clone().unwrap_or_else(|| "unknown".to_string()),
                    version_range: "any".to_string(),
                    required: d.dependency_type == "required",
                }).collect(),
                created_at: project.published.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: project.updated.parse().unwrap_or_else(|_| Utc::now()),
            })
        } else {
            Err("No versions found for the specified version".into())
        }
    }

    async fn download_mod(
        &self,
        mod_id: &str,
        version: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mod_info = self.get_mod_version(mod_id, version).await?;
        
        if let Some(download_url) = mod_info.download_url {
            let response = self.client.get(&download_url).send().await?;
            let content = response.bytes().await?;
            
            std::fs::write(file_path, content)?;
            Ok(())
        } else {
            Err("No download URL available".into())
        }
    }

    async fn get_mod_dependencies(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<Vec<ModDependency>, Box<dyn std::error::Error>> {
        let mod_info = self.get_mod_version(mod_id, version).await?;
        Ok(mod_info.dependencies)
    }

    async fn check_for_updates(
        &self,
        mod_id: &str,
        current_version: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let versions = self.get_project_versions(mod_id, None, None).await?;
        
        if let Some(latest_version) = versions.first() {
            if latest_version.version_number != current_version {
                return Ok(Some(latest_version.version_number.clone()));
            }
        }
        
        Ok(None)
    }
}
