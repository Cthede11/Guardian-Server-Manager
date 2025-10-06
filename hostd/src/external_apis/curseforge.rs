use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use crate::external_apis::mod_provider::ModProvider;
use crate::mod_manager::ModDependency;
use crate::mod_manager::ModInfo;
use chrono::Utc;

/// CurseForge API client for fetching mod data
#[derive(Clone)]
pub struct CurseForgeApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

/// CurseForge project response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeProject {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub links: CurseForgeLinks,
    pub summary: String,
    pub status: u32,
    pub download_count: u64,
    pub is_featured: bool,
    pub primary_category_id: u32,
    pub categories: Vec<CurseForgeCategory>,
    pub class_id: u32,
    pub authors: Vec<CurseForgeAuthor>,
    pub logo: Option<CurseForgeLogo>,
    pub screenshots: Vec<CurseForgeScreenshot>,
    pub main_file_id: u32,
    pub latest_files: Vec<CurseForgeFile>,
    pub latest_files_indexes: Vec<CurseForgeFileIndex>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub allow_mod_distribution: Option<bool>,
    pub game_popularity_rank: u32,
    pub is_available: bool,
    pub thumbs_up: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeLinks {
    pub website_url: Option<String>,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeCategory {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub icon_url: String,
    pub date_modified: String,
    pub is_class: Option<bool>,
    pub class_id: Option<u32>,
    pub parent_category_id: Option<u32>,
    pub display_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeAuthor {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeLogo {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeScreenshot {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeFile {
    pub id: u32,
    pub game_id: u32,
    pub mod_id: u32,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: u32,
    pub file_status: u32,
    pub hashes: Vec<CurseForgeHash>,
    pub file_date: String,
    pub file_length: u64,
    pub download_count: u64,
    pub download_url: String,
    pub game_versions: Vec<String>,
    pub sortable_game_versions: Vec<CurseForgeSortableGameVersion>,
    pub dependencies: Vec<CurseForgeDependency>,
    pub expose_as_alternative: Option<bool>,
    pub parent_project_file_id: Option<u32>,
    pub alternate_file_id: Option<u32>,
    pub is_server_pack: Option<bool>,
    pub server_pack_file_id: Option<u32>,
    pub file_fingerprint: u64,
    pub modules: Vec<CurseForgeModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeFileIndex {
    pub game_version: String,
    pub file_id: u32,
    pub filename: String,
    pub release_type: u32,
    pub game_version_type_id: u32,
    pub mod_loader: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeHash {
    pub value: String,
    pub algo: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeSortableGameVersion {
    pub game_version_padded: String,
    pub game_version: String,
    pub game_version_clean: String,
    pub build: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeDependency {
    pub mod_id: u32,
    pub relation_type: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeModule {
    pub name: String,
    pub fingerprint: u64,
}

/// CurseForge search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeSearchResponse {
    pub data: Vec<CurseForgeProject>,
    pub pagination: CurseForgePagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgePagination {
    pub index: u32,
    pub page_size: u32,
    pub result_count: u32,
    pub total_count: u32,
}

/// CurseForge game version response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeGameVersionResponse {
    pub data: Vec<CurseForgeGameVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeGameVersion {
    pub id: u32,
    pub game_id: u32,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub files: Vec<CurseForgeFile>,
    pub sortable_game_versions: Vec<CurseForgeSortableGameVersion>,
    pub game_version_type_id: u32,
    pub game_version_status: u32,
    pub game_version_status_reason: Option<String>,
    pub game_version_date_status: Option<String>,
    pub game_version_length: u32,
    pub game_version_type_status: u32,
    pub game_version_parent: Option<String>,
    pub game_version_type_parent: Option<String>,
    pub game_version_type_parent_display_index: Option<u32>,
    pub game_version_type_parent_sort: Option<u32>,
    pub game_version_padded: String,
    pub game_version_clean: String,
    pub build: u32,
    pub java_version: Option<String>,
    pub use_files: Option<bool>,
    pub game_version_released: String,
    pub game_version_jvm: Option<String>,
    pub game_version_jvm_args: Option<String>,
    pub game_version_jvm_use_legacy_args: Option<bool>,
    pub game_version_jvm_legacy_args: Option<String>,
    pub game_version_jvm_mod_loader: Option<String>,
    pub game_version_jvm_mod_loader_version: Option<String>,
}

impl CurseForgeApiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key,
        }
    }

    /// Search for mods
    pub async fn search_mods(
        &self,
        game_id: u32,
        search_filter: Option<&str>,
        category_id: Option<u32>,
        game_version: Option<&str>,
        mod_loader_type: Option<u32>,
        sort_field: Option<u32>,
        sort_order: Option<&str>,
        page_size: Option<u32>,
        index: Option<u32>,
    ) -> Result<CurseForgeSearchResponse> {
        let mut params = vec![
            ("gameId", game_id.to_string()),
        ];

        if let Some(filter) = search_filter {
            params.push(("searchFilter", filter.to_string()));
        }
        if let Some(category) = category_id {
            params.push(("categoryId", category.to_string()));
        }
        if let Some(version) = game_version {
            params.push(("gameVersion", version.to_string()));
        }
        if let Some(loader) = mod_loader_type {
            params.push(("modLoaderType", loader.to_string()));
        }
        if let Some(sort) = sort_field {
            params.push(("sortField", sort.to_string()));
        }
        if let Some(order) = sort_order {
            params.push(("sortOrder", order.to_string()));
        }
        if let Some(size) = page_size {
            params.push(("pageSize", size.to_string()));
        }
        if let Some(idx) = index {
            params.push(("index", idx.to_string()));
        }

        let url = format!("{}/mods/search", self.base_url);
        let response = self.client
            .get(&url)
            .query(&params)
            .header("x-api-key", &self.api_key)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status() == 403 {
                warn!("CurseForge API key invalid or missing, returning empty results");
                return Ok(CurseForgeSearchResponse {
                    data: vec![],
                    pagination: CurseForgePagination {
                        index: 0,
                        page_size: 0,
                        result_count: 0,
                        total_count: 0,
                    },
                });
            }
            error!("CurseForge API error: {}", response.status());
            return Err(anyhow::anyhow!("CurseForge API error: {}", response.status()));
        }

        let search_response: CurseForgeSearchResponse = response.json().await?;
        info!("Found {} mods on CurseForge", search_response.pagination.total_count);
        Ok(search_response)
    }

    /// Get project details
    pub async fn get_project(&self, project_id: u32) -> Result<CurseForgeProject> {
        let url = format!("{}/mods/{}", self.base_url, project_id);
        let response = self.client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("CurseForge API error for project {}: {}", project_id, response.status());
            return Err(anyhow::anyhow!("CurseForge API error: {}", response.status()));
        }

        let project: CurseForgeProject = response.json().await?;
        Ok(project)
    }

    /// Get project files
    pub async fn get_project_files(
        &self,
        project_id: u32,
        game_version: Option<&str>,
        mod_loader_type: Option<u32>,
        game_version_type_id: Option<u32>,
        index: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<CurseForgeFile>> {
        let mut params = vec![];

        if let Some(version) = game_version {
            params.push(("gameVersion", version.to_string()));
        }
        if let Some(loader) = mod_loader_type {
            params.push(("modLoaderType", loader.to_string()));
        }
        if let Some(version_type) = game_version_type_id {
            params.push(("gameVersionTypeId", version_type.to_string()));
        }
        if let Some(idx) = index {
            params.push(("index", idx.to_string()));
        }
        if let Some(size) = page_size {
            params.push(("pageSize", size.to_string()));
        }

        let url = format!("{}/mods/{}/files", self.base_url, project_id);
        let response = self.client
            .get(&url)
            .query(&params)
            .header("x-api-key", &self.api_key)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("CurseForge API error for project files {}: {}", project_id, response.status());
            return Err(anyhow::anyhow!("CurseForge API error: {}", response.status()));
        }

        let files: Vec<CurseForgeFile> = response.json().await?;
        Ok(files)
    }

    /// Get game versions
    pub async fn get_game_versions(&self, game_id: u32) -> Result<Vec<CurseForgeGameVersion>> {
        let url = format!("{}/games/{}/versions", self.base_url, game_id);
        let response = self.client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("CurseForge API error for game versions: {}", response.status());
            return Err(anyhow::anyhow!("CurseForge API error: {}", response.status()));
        }

        let version_response: CurseForgeGameVersionResponse = response.json().await?;
        Ok(version_response.data)
    }

    /// Get categories
    pub async fn get_categories(&self, game_id: u32) -> Result<Vec<CurseForgeCategory>> {
        let url = format!("{}/categories", self.base_url);
        let response = self.client
            .get(&url)
            .query(&[("gameId", game_id.to_string())])
            .header("x-api-key", &self.api_key)
            .header("User-Agent", "Guardian-Minecraft-Server-Manager/1.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("CurseForge API error for categories: {}", response.status());
            return Err(anyhow::anyhow!("CurseForge API error: {}", response.status()));
        }

        let categories: Vec<CurseForgeCategory> = response.json().await?;
        Ok(categories)
    }

    /// Convert CurseForge project to our ModInfo format
    pub fn convert_to_mod_info(&self, project: &CurseForgeProject) -> crate::database::ModInfo {
        crate::database::ModInfo {
            id: project.id.to_string(),
            name: project.name.clone(),
            description: Some(project.summary.clone()),
            category: self.map_category(&project.categories),
            side: self.map_side(project),
            source: "curseforge".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Map CurseForge categories to our category system
    fn map_category(&self, categories: &[CurseForgeCategory]) -> String {
        for category in categories {
            match category.name.to_lowercase().as_str() {
                "adventure" => return "adventure".to_string(),
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

    /// Map CurseForge project to our side system
    fn map_side(&self, project: &CurseForgeProject) -> String {
        // CurseForge doesn't have explicit client/server side info in the basic project data
        // We'll need to check the files for more detailed information
        "universal".to_string()
    }
}

#[async_trait::async_trait]
impl ModProvider for CurseForgeApiClient {
    async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ModInfo>, Box<dyn std::error::Error>> {
        // CurseForge game ID for Minecraft is 432
        let search_results = self.search_mods(
            432, // Minecraft game ID
            Some(query),
            None, // category_id
            minecraft_version,
            None, // mod_loader_type
            None, // sort_field
            None, // sort_order
            limit.map(|l| l as u32),
            None, // index
        ).await?;
        
        let mut mod_infos = Vec::new();
        
        for result in search_results.data {
            if let Ok(mod_info) = self.get_mod(&result.id.to_string()).await {
                mod_infos.push(mod_info);
            }
        }
        
        Ok(mod_infos)
    }

    async fn get_mod(&self, mod_id: &str) -> Result<ModInfo, Box<dyn std::error::Error>> {
        let project_id = mod_id.parse::<u32>()
            .map_err(|_| "Invalid CurseForge project ID")?;
        
        let project = self.get_project(project_id).await?;
        let files = self.get_project_files(project_id, None, None, None, None, None).await?;
        
        if let Some(file) = files.first() {
            Ok(ModInfo {
                id: project.id.to_string(),
                name: project.name,
                description: project.summary,
                author: project.authors.first().map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                version: file.display_name.clone(),
                minecraft_version: file.sortable_game_versions.first().map(|v| v.game_version.clone()).unwrap_or_else(|| "1.20.1".to_string()),
                loader: "forge".to_string(), // CurseForge is primarily Forge
                category: "misc".to_string(),
                side: "both".to_string(),
                download_url: Some(file.download_url.clone()),
                file_size: Some(file.file_length),
                sha1: file.hashes.iter().find(|h| h.algo == 1).map(|h| h.value.clone()),
                dependencies: file.dependencies.iter().map(|d| ModDependency {
                    mod_id: d.mod_id.to_string(),
                    version_range: "any".to_string(),
                    required: d.relation_type == 1,
                }).collect(),
                created_at: project.date_created.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: project.date_modified.parse().unwrap_or_else(|_| Utc::now()),
            })
        } else {
            Err("No files found for the project".into())
        }
    }

    async fn get_mod_version(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        let project_id = mod_id.parse::<u32>()
            .map_err(|_| "Invalid CurseForge project ID")?;
        
        let project = self.get_project(project_id).await?;
        let files = self.get_project_files(project_id, Some(version), None, None, None, None).await?;
        
        if let Some(file) = files.first() {
            Ok(ModInfo {
                id: project.id.to_string(),
                name: project.name,
                description: project.summary,
                author: project.authors.first().map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                version: file.display_name.clone(),
                minecraft_version: file.sortable_game_versions.first().map(|v| v.game_version.clone()).unwrap_or_else(|| "1.20.1".to_string()),
                loader: "forge".to_string(),
                category: "misc".to_string(),
                side: "both".to_string(),
                download_url: Some(file.download_url.clone()),
                file_size: Some(file.file_length),
                sha1: file.hashes.iter().find(|h| h.algo == 1).map(|h| h.value.clone()),
                dependencies: file.dependencies.iter().map(|d| ModDependency {
                    mod_id: d.mod_id.to_string(),
                    version_range: "any".to_string(),
                    required: d.relation_type == 1,
                }).collect(),
                created_at: project.date_created.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: project.date_modified.parse().unwrap_or_else(|_| Utc::now()),
            })
        } else {
            Err("No files found for the specified version".into())
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
        let project_id = mod_id.parse::<u32>()
            .map_err(|_| "Invalid CurseForge project ID")?;
        
        let files = self.get_project_files(project_id, None, None, None, None, Some(1)).await?;
        
        if let Some(latest_file) = files.first() {
            if latest_file.display_name != current_version {
                return Ok(Some(latest_file.display_name.clone()));
            }
        }
        
        Ok(None)
    }
}
