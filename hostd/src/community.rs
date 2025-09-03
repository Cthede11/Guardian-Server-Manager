use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Community features for mod compatibility database and sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityManager {
    pub mod_database: Arc<RwLock<ModDatabase>>,
    pub compatibility_reports: Arc<RwLock<HashMap<String, Vec<CompatibilityReport>>>>,
    pub user_contributions: Arc<RwLock<HashMap<String, Vec<UserContribution>>>>,
    pub community_rules: Arc<RwLock<Vec<CommunityRule>>>,
    pub mod_packs: Arc<RwLock<HashMap<String, ModPack>>>,
    pub sharing_platform: Arc<RwLock<SharingPlatform>>,
}

/// Mod database for community-driven compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDatabase {
    pub mods: HashMap<String, ModInfo>,
    pub compatibility_matrix: HashMap<String, HashMap<String, CompatibilityStatus>>,
    pub last_updated: DateTime<Utc>,
    pub version: String,
}

/// Mod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub dependencies: Vec<ModDependency>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub download_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub compatibility_score: f64,
    pub popularity_score: f64,
    pub last_updated: DateTime<Utc>,
    pub community_verified: bool,
    pub guardian_optimized: bool,
}

/// Mod loader types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModLoader {
    Forge,
    NeoForge,
    Fabric,
    Quilt,
    Bukkit,
    Spigot,
    Paper,
}

/// Mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_range: String,
    pub required: bool,
    pub side: DependencySide,
}

/// Dependency side
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencySide {
    Client,
    Server,
    Both,
}

/// Compatibility status between mods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompatibilityStatus {
    Compatible,
    Incompatible,
    Partial,
    Unknown,
    RequiresPatch,
    Tested,
}

/// Compatibility report from community
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub id: String,
    pub mod_a: String,
    pub mod_b: String,
    pub status: CompatibilityStatus,
    pub description: String,
    pub issues: Vec<String>,
    pub solutions: Vec<String>,
    pub test_environment: TestEnvironment,
    pub reporter: String,
    pub verified: bool,
    pub upvotes: u32,
    pub downvotes: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Test environment details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironment {
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub mod_versions: HashMap<String, String>,
    pub java_version: String,
    pub os: String,
    pub guardian_version: String,
}

/// User contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContribution {
    pub id: String,
    pub user_id: String,
    pub contribution_type: ContributionType,
    pub title: String,
    pub description: String,
    pub content: serde_json::Value,
    pub mod_ids: Vec<String>,
    pub status: ContributionStatus,
    pub upvotes: u32,
    pub downvotes: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contribution types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContributionType {
    CompatibilityReport,
    Patch,
    Configuration,
    Tutorial,
    BugReport,
    FeatureRequest,
    ModPack,
    Rule,
}

/// Contribution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContributionStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Merged,
    Archived,
}

/// Community rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_content: String,
    pub mod_ids: Vec<String>,
    pub author: String,
    pub verified: bool,
    pub usage_count: u32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub mods: Vec<ModPackEntry>,
    pub configs: HashMap<String, String>,
    pub custom_rules: Vec<String>,
    pub guardian_optimized: bool,
    pub performance_rating: f64,
    pub stability_rating: f64,
    pub download_count: u32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod pack entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPackEntry {
    pub mod_id: String,
    pub version: String,
    pub required: bool,
    pub config_overrides: Option<HashMap<String, serde_json::Value>>,
}

/// Sharing platform for community content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPlatform {
    pub published_content: HashMap<String, PublishedContent>,
    pub user_repositories: HashMap<String, UserRepository>,
    pub featured_content: Vec<String>,
    pub trending_content: Vec<String>,
}

/// Published content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedContent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content_type: ContentType,
    pub author: String,
    pub content: serde_json::Value,
    pub tags: Vec<String>,
    pub downloads: u32,
    pub views: u32,
    pub rating: f64,
    pub featured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Content types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    ModPack,
    Configuration,
    Rule,
    Tutorial,
    Patch,
    Script,
    ResourcePack,
    DataPack,
}

/// User repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRepository {
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub public: bool,
    pub content_ids: Vec<String>,
    pub followers: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CommunityManager {
    pub fn new() -> Self {
        Self {
            mod_database: Arc::new(RwLock::new(ModDatabase::new())),
            compatibility_reports: Arc::new(RwLock::new(HashMap::new())),
            user_contributions: Arc::new(RwLock::new(HashMap::new())),
            community_rules: Arc::new(RwLock::new(Vec::new())),
            mod_packs: Arc::new(RwLock::new(HashMap::new())),
            sharing_platform: Arc::new(RwLock::new(SharingPlatform::new())),
        }
    }

    /// Initialize community manager
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing community manager...");
        
        // Load mod database
        self.load_mod_database().await?;
        
        // Load community content
        self.load_community_content().await?;
        
        // Start background tasks
        self.start_background_tasks().await;
        
        info!("Community manager initialized");
        Ok(())
    }

    /// Load mod database from external sources
    async fn load_mod_database(&self) -> Result<()> {
        let mut database = self.mod_database.write().await;
        
        // TODO: Load from Modrinth, CurseForge, or other sources
        // For now, create some sample data
        
        let sample_mods = vec![
            ("create", "Create", "0.5.1", "Create is a mod about contraptions and automation", "simibubi"),
            ("flywheel", "Flywheel", "0.6.8", "Modern rendering engine for Minecraft mods", "jozufozu"),
            ("embeddium", "Embeddium", "0.1.27", "Sodium port for Forge", "embeddedt"),
            ("jei", "Just Enough Items", "12.4.0", "JEI is an item and recipe viewing mod", "mezz"),
            ("thermal", "Thermal Series", "10.3.0", "Thermal Expansion and related mods", "TeamCoFH"),
        ];

        for (id, name, version, description, author) in sample_mods {
            let mod_info = ModInfo {
                id: id.to_string(),
                name: name.to_string(),
                version: version.to_string(),
                description: description.to_string(),
                author: author.to_string(),
                license: "MIT".to_string(),
                minecraft_version: "1.20.1".to_string(),
                loader: ModLoader::NeoForge,
                dependencies: Vec::new(),
                categories: vec!["automation".to_string(), "technology".to_string()],
                tags: vec!["popular".to_string(), "optimized".to_string()],
                download_url: None,
                source_url: None,
                issues_url: None,
                compatibility_score: 0.9,
                popularity_score: 0.8,
                last_updated: Utc::now(),
                community_verified: true,
                guardian_optimized: true,
            };
            
            database.mods.insert(id.to_string(), mod_info);
        }
        
        database.last_updated = Utc::now();
        database.version = "1.0.0".to_string();
        
        info!("Loaded {} mods into database", database.mods.len());
        Ok(())
    }

    /// Load community content
    async fn load_community_content(&self) -> Result<()> {
        // TODO: Load from persistent storage
        info!("Loaded community content");
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        // Task to update mod database
        let mod_database = self.mod_database.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hour
            
            loop {
                interval.tick().await;
                
                // TODO: Update mod database from external sources
                // Updating mod database
            }
        });

        // Task to calculate trending content
        let sharing_platform = self.sharing_platform.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800)); // 30 minutes
            
            loop {
                interval.tick().await;
                
                // TODO: Calculate trending content based on downloads, views, ratings
                // Calculating trending content
            }
        });
    }

    /// Add mod to database
    pub async fn add_mod(&self, mod_info: ModInfo) -> Result<()> {
        let mut database = self.mod_database.write().await;
        database.mods.insert(mod_info.id.clone(), mod_info.clone());
        database.last_updated = Utc::now();
        
        info!("Added mod to database: {}", mod_info.name);
        Ok(())
    }

    /// Get mod information
    pub async fn get_mod(&self, mod_id: &str) -> Option<ModInfo> {
        let database = self.mod_database.read().await;
        database.mods.get(mod_id).cloned()
    }

    /// Search mods
    pub async fn search_mods(&self, query: &str, filters: ModSearchFilters) -> Vec<ModInfo> {
        let database = self.mod_database.read().await;
        let mut results = Vec::new();
        
        for mod_info in database.mods.values() {
            if self.matches_search(mod_info, query, &filters) {
                results.push(mod_info.clone());
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| {
            let a_score = self.calculate_relevance_score(a, query);
            let b_score = self.calculate_relevance_score(b, query);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }

    /// Check if mod matches search criteria
    fn matches_search(&self, mod_info: &ModInfo, query: &str, filters: &ModSearchFilters) -> bool {
        // Text search
        if !query.is_empty() {
            let query_lower = query.to_lowercase();
            if !mod_info.name.to_lowercase().contains(&query_lower) &&
               !mod_info.description.to_lowercase().contains(&query_lower) &&
               !mod_info.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                return false;
            }
        }
        
        // Filter by loader
        if let Some(loader) = &filters.loader {
            if &mod_info.loader != loader {
                return false;
            }
        }
        
        // Filter by minecraft version
        if let Some(version) = &filters.minecraft_version {
            if &mod_info.minecraft_version != version {
                return false;
            }
        }
        
        // Filter by categories
        if !filters.categories.is_empty() {
            if !filters.categories.iter().any(|cat| mod_info.categories.contains(cat)) {
                return false;
            }
        }
        
        // Filter by guardian optimized
        if let Some(optimized) = filters.guardian_optimized {
            if mod_info.guardian_optimized != optimized {
                return false;
            }
        }
        
        true
    }

    /// Calculate relevance score for search
    fn calculate_relevance_score(&self, mod_info: &ModInfo, query: &str) -> f64 {
        let mut score = 0.0;
        let query_lower = query.to_lowercase();
        
        // Name match (highest weight)
        if mod_info.name.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }
        
        // Tag match
        for tag in &mod_info.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 5.0;
            }
        }
        
        // Description match
        if mod_info.description.to_lowercase().contains(&query_lower) {
            score += 2.0;
        }
        
        // Popularity and compatibility scores
        score += mod_info.popularity_score * 2.0;
        score += mod_info.compatibility_score * 1.5;
        
        // Guardian optimized bonus
        if mod_info.guardian_optimized {
            score += 3.0;
        }
        
        score
    }

    /// Submit compatibility report
    pub async fn submit_compatibility_report(&self, report: CompatibilityReport) -> Result<()> {
        let report_id = report.id.clone();
        let mod_a = report.mod_a.clone();
        let mod_b = report.mod_b.clone();
        
        let mut reports = self.compatibility_reports.write().await;
        let key = format!("{}:{}", mod_a, mod_b);
        let mod_reports = reports.entry(key).or_insert_with(Vec::new);
        mod_reports.push(report);
        
        // Update compatibility matrix
        let mut database = self.mod_database.write().await;
        database.compatibility_matrix
            .entry(mod_a.clone())
            .or_insert_with(HashMap::new)
            .insert(mod_b.clone(), report.status.clone());
        
        database.compatibility_matrix
            .entry(mod_b)
            .or_insert_with(HashMap::new)
            .insert(mod_a, report.status);
        
        info!("Submitted compatibility report: {}", report_id);
        Ok(())
    }

    /// Get compatibility status between mods
    pub async fn get_compatibility_status(&self, mod_a: &str, mod_b: &str) -> CompatibilityStatus {
        let database = self.mod_database.read().await;
        database.compatibility_matrix
            .get(mod_a)
            .and_then(|matrix| matrix.get(mod_b))
            .cloned()
            .unwrap_or(CompatibilityStatus::Unknown)
    }

    /// Submit user contribution
    pub async fn submit_contribution(&self, contribution: UserContribution) -> Result<()> {
        let user_id = contribution.user_id.clone();
        let contribution_id = contribution.id.clone();
        
        let mut contributions = self.user_contributions.write().await;
        let user_contributions = contributions.entry(user_id).or_insert_with(Vec::new);
        user_contributions.push(contribution);
        
        info!("Submitted user contribution: {}", contribution_id);
        Ok(())
    }

    /// Create mod pack
    pub async fn create_mod_pack(&self, mod_pack: ModPack) -> Result<()> {
        let pack_id = mod_pack.id.clone();
        
        let mut mod_packs = self.mod_packs.write().await;
        mod_packs.insert(pack_id.clone(), mod_pack);
        
        info!("Created mod pack: {}", pack_id);
        Ok(())
    }

    /// Get mod pack
    pub async fn get_mod_pack(&self, pack_id: &str) -> Option<ModPack> {
        let mod_packs = self.mod_packs.read().await;
        mod_packs.get(pack_id).cloned()
    }

    /// Search mod packs
    pub async fn search_mod_packs(&self, query: &str) -> Vec<ModPack> {
        let mod_packs = self.mod_packs.read().await;
        let mut results = Vec::new();
        
        for pack in mod_packs.values() {
            if pack.name.to_lowercase().contains(&query.to_lowercase()) ||
               pack.description.to_lowercase().contains(&query.to_lowercase()) {
                results.push(pack.clone());
            }
        }
        
        // Sort by rating and download count
        results.sort_by(|a, b| {
            let a_score = a.rating * (a.download_count as f64 + 1.0).ln();
            let b_score = b.rating * (b.download_count as f64 + 1.0).ln();
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }

    /// Publish content to sharing platform
    pub async fn publish_content(&self, content: PublishedContent) -> Result<()> {
        let content_id = content.id.clone();
        
        let mut platform = self.sharing_platform.write().await;
        platform.published_content.insert(content_id.clone(), content);
        
        info!("Published content: {}", content_id);
        Ok(())
    }

    /// Get featured content
    pub async fn get_featured_content(&self) -> Vec<PublishedContent> {
        let platform = self.sharing_platform.read().await;
        platform.featured_content.iter()
            .filter_map(|id| platform.published_content.get(id).cloned())
            .collect()
    }

    /// Get trending content
    pub async fn get_trending_content(&self) -> Vec<PublishedContent> {
        let platform = self.sharing_platform.read().await;
        platform.trending_content.iter()
            .filter_map(|id| platform.published_content.get(id).cloned())
            .collect()
    }

    /// Rate content
    pub async fn rate_content(&self, content_id: &str, rating: f64) -> Result<()> {
        let mut platform = self.sharing_platform.write().await;
        if let Some(content) = platform.published_content.get_mut(content_id) {
            // Simple rating update - in production, use proper rating algorithm
            content.rating = (content.rating + rating) / 2.0;
            content.updated_at = Utc::now();
            
            info!("Rated content {}: {}", content_id, rating);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Content not found: {}", content_id))
        }
    }

    /// Get community statistics
    pub async fn get_community_stats(&self) -> CommunityStats {
        let database = self.mod_database.read().await;
        let reports = self.compatibility_reports.read().await;
        let contributions = self.user_contributions.read().await;
        let mod_packs = self.mod_packs.read().await;
        let platform = self.sharing_platform.read().await;
        
        CommunityStats {
            total_mods: database.mods.len(),
            total_compatibility_reports: reports.values().map(|v| v.len()).sum(),
            total_contributions: contributions.values().map(|v| v.len()).sum(),
            total_mod_packs: mod_packs.len(),
            total_published_content: platform.published_content.len(),
            guardian_optimized_mods: database.mods.values().filter(|m| m.guardian_optimized).count(),
            community_verified_mods: database.mods.values().filter(|m| m.community_verified).count(),
        }
    }
}

/// Mod search filters
#[derive(Debug, Clone, Default)]
pub struct ModSearchFilters {
    pub loader: Option<ModLoader>,
    pub minecraft_version: Option<String>,
    pub categories: Vec<String>,
    pub guardian_optimized: Option<bool>,
}

/// Community statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityStats {
    pub total_mods: usize,
    pub total_compatibility_reports: usize,
    pub total_contributions: usize,
    pub total_mod_packs: usize,
    pub total_published_content: usize,
    pub guardian_optimized_mods: usize,
    pub community_verified_mods: usize,
}

impl ModDatabase {
    pub fn new() -> Self {
        Self {
            mods: HashMap::new(),
            compatibility_matrix: HashMap::new(),
            last_updated: Utc::now(),
            version: "1.0.0".to_string(),
        }
    }
}

impl SharingPlatform {
    pub fn new() -> Self {
        Self {
            published_content: HashMap::new(),
            user_repositories: HashMap::new(),
            featured_content: Vec::new(),
            trending_content: Vec::new(),
        }
    }
}
