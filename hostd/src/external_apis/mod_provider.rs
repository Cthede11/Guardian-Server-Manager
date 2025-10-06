use crate::mod_manager::{ModInfo, ModDependency};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Trait for unified mod provider interface
#[async_trait::async_trait]
pub trait ModProvider: Send + Sync {
    /// Search for mods
    async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ModInfo>, Box<dyn Error>>;

    /// Get mod details by ID
    async fn get_mod(&self, mod_id: &str) -> Result<ModInfo, Box<dyn Error>>;

    /// Get mod version details
    async fn get_mod_version(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<ModInfo, Box<dyn Error>>;

    /// Download mod file
    async fn download_mod(
        &self,
        mod_id: &str,
        version: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn Error>>;

    /// Get mod dependencies
    async fn get_mod_dependencies(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<Vec<ModDependency>, Box<dyn Error>>;

    /// Check for mod updates
    async fn check_for_updates(
        &self,
        mod_id: &str,
        current_version: &str,
    ) -> Result<Option<String>, Box<dyn Error>>;
}


/// Provider type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    CurseForge,
    Modrinth,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::CurseForge => write!(f, "CurseForge"),
            ProviderType::Modrinth => write!(f, "Modrinth"),
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub api_key: Option<String>,
    pub base_url: String,
    pub rate_limit: Option<u32>, // requests per minute
}

impl ProviderConfig {
    pub fn curseforge(api_key: String) -> Self {
        Self {
            provider_type: ProviderType::CurseForge,
            api_key: Some(api_key),
            base_url: "https://api.curseforge.com/v1".to_string(),
            rate_limit: Some(100), // CurseForge rate limit
        }
    }

    pub fn modrinth() -> Self {
        Self {
            provider_type: ProviderType::Modrinth,
            api_key: None,
            base_url: "https://api.modrinth.com/v2".to_string(),
            rate_limit: Some(300), // Modrinth rate limit
        }
    }
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn ModProvider>, Box<dyn Error>> {
        match config.provider_type {
            ProviderType::CurseForge => {
                let api_key = config.api_key.ok_or("CurseForge API key required")?;
                Ok(Box::new(crate::external_apis::curseforge::CurseForgeApiClient::new(api_key)))
            }
            ProviderType::Modrinth => {
                Ok(Box::new(crate::external_apis::modrinth::ModrinthApiClient::new()))
            }
        }
    }
}

/// Multi-provider mod manager
pub struct MultiProviderModManager {
    providers: Vec<Box<dyn ModProvider>>,
}

impl MultiProviderModManager {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn ModProvider>) {
        self.providers.push(provider);
    }

    /// Search across all providers
    pub async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ModInfo>, Box<dyn Error>> {
        let mut all_results = Vec::new();
        
        for provider in &self.providers {
            match provider.search_mods(query, minecraft_version, loader, limit).await {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    tracing::warn!("Provider search failed: {}", e);
                    continue;
                }
            }
        }

        // Sort by relevance (name similarity to query)
        all_results.sort_by(|a, b| {
            let a_score = self.calculate_relevance_score(&a.name, query);
            let b_score = self.calculate_relevance_score(&b.name, query);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit if specified
        if let Some(limit) = limit {
            all_results.truncate(limit);
        }

        Ok(all_results)
    }

    /// Get mod from any provider
    pub async fn get_mod(&self, mod_id: &str) -> Result<ModInfo, Box<dyn Error>> {
        for provider in &self.providers {
            match provider.get_mod(mod_id).await {
                Ok(mod_info) => return Ok(mod_info),
                Err(_) => continue,
            }
        }
        Err("Mod not found in any provider".into())
    }

    /// Download mod from any provider
    pub async fn download_mod(
        &self,
        mod_id: &str,
        version: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        for provider in &self.providers {
            match provider.download_mod(mod_id, version, file_path).await {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
        Err("Failed to download mod from any provider".into())
    }

    /// Check for updates across all providers
    pub async fn check_for_updates(
        &self,
        mod_id: &str,
        current_version: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        for provider in &self.providers {
            match provider.check_for_updates(mod_id, current_version).await {
                Ok(Some(version)) => return Ok(Some(version)),
                Ok(None) => continue,
                Err(_) => continue,
            }
        }
        Ok(None)
    }

    fn calculate_relevance_score(&self, name: &str, query: &str) -> f32 {
        let name_lower = name.to_lowercase();
        let query_lower = query.to_lowercase();
        
        if name_lower == query_lower {
            return 1.0;
        }
        
        if name_lower.contains(&query_lower) {
            return 0.8;
        }
        
        // Simple word matching
        let name_words: Vec<&str> = name_lower.split_whitespace().collect();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        let mut matches = 0;
        for query_word in &query_words {
            for name_word in &name_words {
                if name_word.contains(query_word) {
                    matches += 1;
                    break;
                }
            }
        }
        
        if !query_words.is_empty() {
            matches as f32 / query_words.len() as f32
        } else {
            0.0
        }
    }
}
