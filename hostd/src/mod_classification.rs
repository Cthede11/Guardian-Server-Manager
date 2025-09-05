use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

/// Mod side classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModSide {
    Client,           // Only client-side
    Server,           // Only server-side
    Both,             // Both client and server
    Universal,        // Works on both but different behavior
    Optional,         // Optional for either side
}

impl std::fmt::Display for ModSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModSide::Client => write!(f, "client"),
            ModSide::Server => write!(f, "server"),
            ModSide::Both => write!(f, "both"),
            ModSide::Universal => write!(f, "universal"),
            ModSide::Optional => write!(f, "optional"),
        }
    }
}

/// Mod category classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModCategory {
    // Core Categories
    Core,
    Library,
    API,
    
    // Gameplay Categories
    Adventure,
    Building,
    Combat,
    Economy,
    Exploration,
    Farming,
    Magic,
    Technology,
    Transportation,
    Utility,
    
    // Technical Categories
    Performance,
    Optimization,
    Debugging,
    Development,
    Testing,
    
    // Content Categories
    Biomes,
    Dimensions,
    Mobs,
    Items,
    Blocks,
    WorldGeneration,
    
    // Integration Categories
    Integration,
    Compatibility,
    Translation,
    
    // Other
    Miscellaneous,
    Unknown,
}

impl std::fmt::Display for ModCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModCategory::Core => write!(f, "core"),
            ModCategory::Library => write!(f, "library"),
            ModCategory::API => write!(f, "api"),
            ModCategory::Adventure => write!(f, "adventure"),
            ModCategory::Building => write!(f, "building"),
            ModCategory::Combat => write!(f, "combat"),
            ModCategory::Economy => write!(f, "economy"),
            ModCategory::Exploration => write!(f, "exploration"),
            ModCategory::Farming => write!(f, "farming"),
            ModCategory::Magic => write!(f, "magic"),
            ModCategory::Technology => write!(f, "technology"),
            ModCategory::Transportation => write!(f, "transportation"),
            ModCategory::Utility => write!(f, "utility"),
            ModCategory::Performance => write!(f, "performance"),
            ModCategory::Optimization => write!(f, "optimization"),
            ModCategory::Debugging => write!(f, "debugging"),
            ModCategory::Development => write!(f, "development"),
            ModCategory::Testing => write!(f, "testing"),
            ModCategory::Biomes => write!(f, "biomes"),
            ModCategory::Dimensions => write!(f, "dimensions"),
            ModCategory::Mobs => write!(f, "mobs"),
            ModCategory::Items => write!(f, "items"),
            ModCategory::Blocks => write!(f, "blocks"),
            ModCategory::WorldGeneration => write!(f, "world_generation"),
            ModCategory::Integration => write!(f, "integration"),
            ModCategory::Compatibility => write!(f, "compatibility"),
            ModCategory::Translation => write!(f, "translation"),
            ModCategory::Miscellaneous => write!(f, "miscellaneous"),
            ModCategory::Unknown => write!(f, "unknown"),
        }
    }
}

/// Mod source classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModSource {
    CurseForge,
    Modrinth,
    GitHub,
    Local,
    Custom,
}

impl std::fmt::Display for ModSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModSource::CurseForge => write!(f, "curseforge"),
            ModSource::Modrinth => write!(f, "modrinth"),
            ModSource::GitHub => write!(f, "github"),
            ModSource::Local => write!(f, "local"),
            ModSource::Custom => write!(f, "custom"),
        }
    }
}

/// Conflict severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictSeverity {
    Critical,  // Cannot run together
    Warning,   // May cause issues
    Info,      // Minor compatibility issues
}

impl std::fmt::Display for ConflictSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictSeverity::Critical => write!(f, "critical"),
            ConflictSeverity::Warning => write!(f, "warning"),
            ConflictSeverity::Info => write!(f, "info"),
        }
    }
}

/// Mod dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_range: String,  // ">=1.0.0,<2.0.0"
    pub side: ModSide,
    pub required: bool,
}

/// Mod conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConflict {
    pub mod_id: String,
    pub reason: String,
    pub severity: ConflictSeverity,
}

/// Enhanced mod information with comprehensive classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub minecraft_versions: Vec<String>,  // All supported MC versions
    pub loader_versions: HashMap<String, Vec<String>>, // Loader type -> versions
    pub side: ModSide,
    pub category: ModCategory,
    pub dependencies: Vec<ModDependency>,
    pub conflicts: Vec<ModConflict>,
    pub source: ModSource,
    pub download_url: String,
    pub file_size: u64,
    pub sha256: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub download_count: Option<u64>,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
}

/// Mod classification engine
#[derive(Debug, Clone)]
pub struct ModClassificationEngine {
    category_keywords: HashMap<ModCategory, Vec<String>>,
    side_indicators: HashMap<ModSide, Vec<String>>,
    conflict_patterns: Vec<ConflictPattern>,
}

/// Pattern for detecting mod conflicts
#[derive(Debug, Clone)]
struct ConflictPattern {
    mod_patterns: Vec<String>,
    reason: String,
    severity: ConflictSeverity,
}

impl ModClassificationEngine {
    /// Create a new mod classification engine
    pub fn new() -> Self {
        let mut engine = Self {
            category_keywords: HashMap::new(),
            side_indicators: HashMap::new(),
            conflict_patterns: Vec::new(),
        };
        
        engine.initialize_classification_rules();
        engine
    }

    /// Initialize classification rules and patterns
    fn initialize_classification_rules(&mut self) {
        info!("Initializing mod classification rules...");
        
        // Category keywords
        self.category_keywords.insert(ModCategory::Core, vec![
            "core".to_string(), "base".to_string(), "foundation".to_string(), "essential".to_string(), "required".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Library, vec![
            "library".to_string(), "lib".to_string(), "common".to_string(), "shared".to_string(), "dependency".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::API, vec![
            "api".to_string(), "interface".to_string(), "hook".to_string(), "integration".to_string(), "bridge".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Adventure, vec![
            "adventure".to_string(), "exploration".to_string(), "dungeon".to_string(), "quest".to_string(), "rpg".to_string(), "story".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Building, vec![
            "building".to_string(), "construction".to_string(), "architecture".to_string(), "decorative".to_string(), "furniture".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Combat, vec![
            "combat".to_string(), "weapon".to_string(), "armor".to_string(), "battle".to_string(), "pvp".to_string(), "fighting".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Economy, vec![
            "economy".to_string(), "money".to_string(), "currency".to_string(), "trade".to_string(), "shop".to_string(), "market".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Exploration, vec![
            "exploration".to_string(), "world".to_string(), "biome".to_string(), "dimension".to_string(), "adventure".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Farming, vec![
            "farming".to_string(), "agriculture".to_string(), "crop".to_string(), "animal".to_string(), "food".to_string(), "harvest".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Magic, vec![
            "magic".to_string(), "spell".to_string(), "enchantment".to_string(), "mystical".to_string(), "arcane".to_string(), "potion".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Technology, vec![
            "technology".to_string(), "machine".to_string(), "automation".to_string(), "industrial".to_string(), "tech".to_string(), "engineering".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Transportation, vec![
            "transportation".to_string(), "vehicle".to_string(), "car".to_string(), "plane".to_string(), "boat".to_string(), "train".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Utility, vec![
            "utility".to_string(), "tool".to_string(), "helper".to_string(), "assistant".to_string(), "convenience".to_string(), "quality of life".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Performance, vec![
            "performance".to_string(), "optimization".to_string(), "fps".to_string(), "lag".to_string(), "smooth".to_string(), "fast".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Optimization, vec![
            "optimization".to_string(), "optimize".to_string(), "performance".to_string(), "efficiency".to_string(), "speed".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Debugging, vec![
            "debug".to_string(), "debugging".to_string(), "development".to_string(), "testing".to_string(), "troubleshooting".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Development, vec![
            "development".to_string(), "dev".to_string(), "programming".to_string(), "coding".to_string(), "api".to_string(), "sdk".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Testing, vec![
            "test".to_string(), "testing".to_string(), "debug".to_string(), "development".to_string(), "experimental".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Biomes, vec![
            "biome".to_string(), "biomes".to_string(), "world".to_string(), "terrain".to_string(), "environment".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Dimensions, vec![
            "dimension".to_string(), "dimensions".to_string(), "world".to_string(), "realm".to_string(), "space".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Mobs, vec![
            "mob".to_string(), "mobs".to_string(), "creature".to_string(), "animal".to_string(), "entity".to_string(), "npc".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Items, vec![
            "item".to_string(), "items".to_string(), "tool".to_string(), "weapon".to_string(), "armor".to_string(), "equipment".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Blocks, vec![
            "block".to_string(), "blocks".to_string(), "tile".to_string(), "structure".to_string(), "building".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::WorldGeneration, vec![
            "world".to_string(), "generation".to_string(), "terrain".to_string(), "biome".to_string(), "structure".to_string(), "ore".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Integration, vec![
            "integration".to_string(), "bridge".to_string(), "connect".to_string(), "link".to_string(), "compatibility".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Compatibility, vec![
            "compatibility".to_string(), "compat".to_string(), "patch".to_string(), "fix".to_string(), "support".to_string()
        ]);
        
        self.category_keywords.insert(ModCategory::Translation, vec![
            "translation".to_string(), "language".to_string(), "localization".to_string(), "locale".to_string(), "i18n".to_string()
        ]);
        
        // Side indicators
        self.side_indicators.insert(ModSide::Client, vec![
            "client".to_string(), "gui".to_string(), "interface".to_string(), "render".to_string(), "graphics".to_string(), "hud".to_string(), "ui".to_string()
        ]);
        
        self.side_indicators.insert(ModSide::Server, vec![
            "server".to_string(), "world".to_string(), "tick".to_string(), "entity".to_string(), "block".to_string(), "chunk".to_string()
        ]);
        
        self.side_indicators.insert(ModSide::Both, vec![
            "both".to_string(), "universal".to_string(), "shared".to_string(), "common".to_string()
        ]);
        
        self.side_indicators.insert(ModSide::Universal, vec![
            "universal".to_string(), "both".to_string(), "shared".to_string(), "common".to_string()
        ]);
        
        self.side_indicators.insert(ModSide::Optional, vec![
            "optional".to_string(), "optional".to_string(), "recommended".to_string(), "suggested".to_string()
        ]);
        
        // Conflict patterns
        self.conflict_patterns.push(ConflictPattern {
            mod_patterns: vec!["optifine".to_string(), "sodium".to_string()],
            reason: "Both are rendering optimization mods".to_string(),
            severity: ConflictSeverity::Critical,
        });
        
        self.conflict_patterns.push(ConflictPattern {
            mod_patterns: vec!["jei".to_string(), "nei".to_string()],
            reason: "Both are item viewing mods".to_string(),
            severity: ConflictSeverity::Critical,
        });
        
        self.conflict_patterns.push(ConflictPattern {
            mod_patterns: vec!["forge".to_string(), "fabric".to_string()],
            reason: "Different mod loaders".to_string(),
            severity: ConflictSeverity::Critical,
        });
        
        self.conflict_patterns.push(ConflictPattern {
            mod_patterns: vec!["worldedit".to_string(), "voxelsniper".to_string()],
            reason: "Both are world editing tools".to_string(),
            severity: ConflictSeverity::Warning,
        });
        
        info!("Mod classification rules initialized");
    }

    /// Classify a mod based on its information
    pub fn classify_mod(&self, mod_info: &ModInfo) -> Result<ModClassification> {
        let category = self.classify_category(mod_info);
        let side = self.classify_side(mod_info);
        let conflicts = self.detect_conflicts(mod_info);
        
        Ok(ModClassification {
            category: category.clone(),
            side: side.clone(),
            conflicts,
            confidence: self.calculate_confidence(mod_info, &category, &side),
        })
    }

    /// Classify mod category based on name, description, and tags
    fn classify_category(&self, mod_info: &ModInfo) -> ModCategory {
        let text = format!("{} {} {}",
            mod_info.name.to_lowercase(),
            mod_info.description.as_ref().unwrap_or(&"".to_string()).to_lowercase(),
            mod_info.tags.join(" ").to_lowercase()
        );
        
        let mut best_category = ModCategory::Unknown;
        let mut best_score = 0.0;
        
        for (category, keywords) in &self.category_keywords {
            let score = self.calculate_keyword_score(&text, keywords);
            if score > best_score {
                best_score = score;
                best_category = category.clone();
            }
        }
        
        best_category
    }

    /// Classify mod side based on name, description, and tags
    fn classify_side(&self, mod_info: &ModInfo) -> ModSide {
        let text = format!("{} {} {}",
            mod_info.name.to_lowercase(),
            mod_info.description.as_ref().unwrap_or(&"".to_string()).to_lowercase(),
            mod_info.tags.join(" ").to_lowercase()
        );
        
        let mut best_side = ModSide::Both;
        let mut best_score = 0.0;
        
        for (side, indicators) in &self.side_indicators {
            let score = self.calculate_keyword_score(&text, indicators);
            if score > best_score {
                best_score = score;
                best_side = side.clone();
            }
        }
        
        best_side
    }

    /// Detect potential conflicts with other mods
    fn detect_conflicts(&self, mod_info: &ModInfo) -> Vec<ModConflict> {
        let mut conflicts = Vec::new();
        
        for pattern in &self.conflict_patterns {
            for mod_pattern in &pattern.mod_patterns {
                if mod_info.name.to_lowercase().contains(mod_pattern) {
                    conflicts.push(ModConflict {
                        mod_id: mod_info.id.clone(),
                        reason: pattern.reason.clone(),
                        severity: pattern.severity.clone(),
                    });
                }
            }
        }
        
        conflicts
    }

    /// Calculate keyword score for classification
    fn calculate_keyword_score(&self, text: &str, keywords: &[String]) -> f32 {
        let mut score = 0.0;
        let word_count = text.split_whitespace().count() as f32;
        
        for keyword in keywords {
            let matches = text.matches(keyword).count() as f32;
            score += matches / word_count;
        }
        
        score
    }

    /// Calculate classification confidence
    fn calculate_confidence(&self, mod_info: &ModInfo, category: &ModCategory, side: &ModSide) -> f32 {
        let mut confidence = 0.5; // Base confidence
        
        // Increase confidence based on description length
        if let Some(desc) = &mod_info.description {
            confidence += (desc.len() as f32 / 1000.0).min(0.3);
        }
        
        // Increase confidence based on tag count
        confidence += (mod_info.tags.len() as f32 / 10.0).min(0.2);
        
        // Increase confidence if category matches keywords
        if let Some(keywords) = self.category_keywords.get(category) {
            let text = format!("{} {} {}",
                mod_info.name.to_lowercase(),
                mod_info.description.as_ref().unwrap_or(&"".to_string()).to_lowercase(),
                mod_info.tags.join(" ").to_lowercase()
            );
            let score = self.calculate_keyword_score(&text, keywords);
            confidence += score * 0.3;
        }
        
        confidence.min(1.0)
    }

    /// Get all available categories
    pub fn get_categories(&self) -> Vec<ModCategory> {
        self.category_keywords.keys().cloned().collect()
    }

    /// Get all available sides
    pub fn get_sides(&self) -> Vec<ModSide> {
        self.side_indicators.keys().cloned().collect()
    }

    /// Get category keywords
    pub fn get_category_keywords(&self, category: &ModCategory) -> Option<&Vec<String>> {
        self.category_keywords.get(category)
    }

    /// Get side indicators
    pub fn get_side_indicators(&self, side: &ModSide) -> Option<&Vec<String>> {
        self.side_indicators.get(side)
    }
}

/// Mod classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModClassification {
    pub category: ModCategory,
    pub side: ModSide,
    pub conflicts: Vec<ModConflict>,
    pub confidence: f32,
}

impl Default for ModClassificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_classification() {
        let engine = ModClassificationEngine::new();
        
        let mod_info = ModInfo {
            id: "test-mod".to_string(),
            name: "OptiFine".to_string(),
            description: Some("Performance optimization mod for Minecraft".to_string()),
            version: "1.0.0".to_string(),
            minecraft_versions: vec!["1.21.1".to_string()],
            loader_versions: HashMap::new(),
            side: ModSide::Client,
            category: ModCategory::Performance,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            source: ModSource::CurseForge,
            download_url: "https://example.com".to_string(),
            file_size: 1024,
            sha256: "abc123".to_string(),
            last_updated: chrono::Utc::now(),
            download_count: Some(1000000),
            rating: Some(4.5),
            tags: vec!["performance".to_string(), "optimization".to_string()],
        };
        
        let classification = engine.classify_mod(&mod_info).unwrap();
        assert_eq!(classification.category, ModCategory::Performance);
        assert_eq!(classification.side, ModSide::Client);
        assert!(!classification.conflicts.is_empty()); // Should detect OptiFine conflicts
    }

    #[test]
    fn test_category_classification() {
        let engine = ModClassificationEngine::new();
        
        let mod_info = ModInfo {
            id: "test-mod".to_string(),
            name: "JEI".to_string(),
            description: Some("Just Enough Items - Item viewing mod".to_string()),
            version: "1.0.0".to_string(),
            minecraft_versions: vec!["1.21.1".to_string()],
            loader_versions: HashMap::new(),
            side: ModSide::Client,
            category: ModCategory::Utility,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            source: ModSource::CurseForge,
            download_url: "https://example.com".to_string(),
            file_size: 1024,
            sha256: "abc123".to_string(),
            last_updated: chrono::Utc::now(),
            download_count: Some(1000000),
            rating: Some(4.5),
            tags: vec!["utility".to_string(), "items".to_string()],
        };
        
        let classification = engine.classify_mod(&mod_info).unwrap();
        assert_eq!(classification.category, ModCategory::Utility);
    }

    #[test]
    fn test_side_classification() {
        let engine = ModClassificationEngine::new();
        
        let mod_info = ModInfo {
            id: "test-mod".to_string(),
            name: "Server Mod".to_string(),
            description: Some("Server-side world generation mod".to_string()),
            version: "1.0.0".to_string(),
            minecraft_versions: vec!["1.21.1".to_string()],
            loader_versions: HashMap::new(),
            side: ModSide::Server,
            category: ModCategory::WorldGeneration,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            source: ModSource::CurseForge,
            download_url: "https://example.com".to_string(),
            file_size: 1024,
            sha256: "abc123".to_string(),
            last_updated: chrono::Utc::now(),
            download_count: Some(1000000),
            rating: Some(4.5),
            tags: vec!["server".to_string(), "world".to_string()],
        };
        
        let classification = engine.classify_mod(&mod_info).unwrap();
        assert_eq!(classification.side, ModSide::Server);
    }
}
