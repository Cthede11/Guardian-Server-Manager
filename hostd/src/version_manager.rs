use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

/// Minecraft release types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReleaseType {
    Release,
    Snapshot,
    Beta,
    Alpha,
    Classic,
    Indev,
    Infdev,
}

impl std::fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseType::Release => write!(f, "release"),
            ReleaseType::Snapshot => write!(f, "snapshot"),
            ReleaseType::Beta => write!(f, "beta"),
            ReleaseType::Alpha => write!(f, "alpha"),
            ReleaseType::Classic => write!(f, "classic"),
            ReleaseType::Indev => write!(f, "indev"),
            ReleaseType::Infdev => write!(f, "infdev"),
        }
    }
}

/// Mod loader types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum ModLoader {
    Vanilla,
    Forge { version: String },
    Fabric { version: String },
    Quilt { version: String },
    NeoForge { version: String },
    Rift { version: String },
    Liteloader { version: String },
    Custom { name: String, version: String },
}

impl std::fmt::Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModLoader::Vanilla => write!(f, "vanilla"),
            ModLoader::Forge { version } => write!(f, "forge-{}", version),
            ModLoader::Fabric { version } => write!(f, "fabric-{}", version),
            ModLoader::Quilt { version } => write!(f, "quilt-{}", version),
            ModLoader::NeoForge { version } => write!(f, "neoforge-{}", version),
            ModLoader::Rift { version } => write!(f, "rift-{}", version),
            ModLoader::Liteloader { version } => write!(f, "liteloader-{}", version),
            ModLoader::Custom { name, version } => write!(f, "{}-{}", name, version),
        }
    }
}

/// Enhanced Minecraft version with loader support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub release_type: ReleaseType,
    pub release_date: chrono::DateTime<chrono::Utc>,
    pub protocol_version: i32,
    pub data_version: i32,
    pub is_supported: bool,
    pub supported_loaders: Vec<ModLoader>,
}

/// Version manager for handling Minecraft and mod loader versions
#[derive(Debug, Clone)]
pub struct VersionManager {
    minecraft_versions: HashMap<String, MinecraftVersionInfo>,
    loader_versions: HashMap<ModLoader, Vec<String>>,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new() -> Self {
        let mut manager = Self {
            minecraft_versions: HashMap::new(),
            loader_versions: HashMap::new(),
        };
        
        // Initialize with known versions
        manager.initialize_versions();
        manager
    }

    /// Initialize with known Minecraft versions and loaders
    fn initialize_versions(&mut self) {
        info!("Initializing version manager with known versions...");
        
        // Add some key Minecraft versions
        let versions = vec![
            // Latest versions
            ("1.21.1", ReleaseType::Release, 766, 3939),
            ("1.21", ReleaseType::Release, 765, 3938),
            ("1.20.6", ReleaseType::Release, 764, 3937),
            ("1.20.5", ReleaseType::Release, 763, 3936),
            ("1.20.4", ReleaseType::Release, 762, 3935),
            ("1.20.3", ReleaseType::Release, 761, 3934),
            ("1.20.2", ReleaseType::Release, 760, 3933),
            ("1.20.1", ReleaseType::Release, 759, 3932),
            ("1.20", ReleaseType::Release, 758, 3931),
            
            // 1.19.x series
            ("1.19.4", ReleaseType::Release, 762, 3935),
            ("1.19.3", ReleaseType::Release, 761, 3934),
            ("1.19.2", ReleaseType::Release, 760, 3933),
            ("1.19.1", ReleaseType::Release, 759, 3932),
            ("1.19", ReleaseType::Release, 758, 3931),
            
            // 1.18.x series
            ("1.18.2", ReleaseType::Release, 758, 2975),
            ("1.18.1", ReleaseType::Release, 757, 2975),
            ("1.18", ReleaseType::Release, 757, 2975),
            
            // 1.17.x series
            ("1.17.1", ReleaseType::Release, 756, 2730),
            ("1.17", ReleaseType::Release, 755, 2724),
            
            // 1.16.x series
            ("1.16.5", ReleaseType::Release, 754, 2586),
            ("1.16.4", ReleaseType::Release, 754, 2586),
            ("1.16.3", ReleaseType::Release, 753, 2586),
            ("1.16.2", ReleaseType::Release, 752, 2586),
            ("1.16.1", ReleaseType::Release, 751, 2586),
            ("1.16", ReleaseType::Release, 751, 2586),
            
            // 1.15.x series
            ("1.15.2", ReleaseType::Release, 750, 2230),
            ("1.15.1", ReleaseType::Release, 750, 2230),
            ("1.15", ReleaseType::Release, 750, 2230),
            
            // 1.14.x series
            ("1.14.4", ReleaseType::Release, 498, 1976),
            ("1.14.3", ReleaseType::Release, 498, 1976),
            ("1.14.2", ReleaseType::Release, 498, 1976),
            ("1.14.1", ReleaseType::Release, 498, 1976),
            ("1.14", ReleaseType::Release, 498, 1976),
            
            // 1.13.x series
            ("1.13.2", ReleaseType::Release, 404, 1631),
            ("1.13.1", ReleaseType::Release, 404, 1631),
            ("1.13", ReleaseType::Release, 404, 1631),
            
            // 1.12.x series
            ("1.12.2", ReleaseType::Release, 340, 1343),
            ("1.12.1", ReleaseType::Release, 340, 1343),
            ("1.12", ReleaseType::Release, 340, 1343),
            
            // 1.11.x series
            ("1.11.2", ReleaseType::Release, 316, 922),
            ("1.11.1", ReleaseType::Release, 316, 922),
            ("1.11", ReleaseType::Release, 316, 922),
            
            // 1.10.x series
            ("1.10.2", ReleaseType::Release, 210, 512),
            ("1.10.1", ReleaseType::Release, 210, 512),
            ("1.10", ReleaseType::Release, 210, 512),
            
            // 1.9.x series
            ("1.9.4", ReleaseType::Release, 110, 184),
            ("1.9.3", ReleaseType::Release, 110, 184),
            ("1.9.2", ReleaseType::Release, 110, 184),
            ("1.9.1", ReleaseType::Release, 110, 184),
            ("1.9", ReleaseType::Release, 110, 184),
            
            // 1.8.x series
            ("1.8.9", ReleaseType::Release, 47, 1343),
            ("1.8.8", ReleaseType::Release, 47, 1343),
            ("1.8.7", ReleaseType::Release, 47, 1343),
            ("1.8.6", ReleaseType::Release, 47, 1343),
            ("1.8.5", ReleaseType::Release, 47, 1343),
            ("1.8.4", ReleaseType::Release, 47, 1343),
            ("1.8.3", ReleaseType::Release, 47, 1343),
            ("1.8.2", ReleaseType::Release, 47, 1343),
            ("1.8.1", ReleaseType::Release, 47, 1343),
            ("1.8", ReleaseType::Release, 47, 1343),
            
            // 1.7.x series
            ("1.7.10", ReleaseType::Release, 5, 1343),
            ("1.7.9", ReleaseType::Release, 5, 1343),
            ("1.7.8", ReleaseType::Release, 5, 1343),
            ("1.7.7", ReleaseType::Release, 5, 1343),
            ("1.7.6", ReleaseType::Release, 5, 1343),
            ("1.7.5", ReleaseType::Release, 5, 1343),
            ("1.7.4", ReleaseType::Release, 5, 1343),
            ("1.7.3", ReleaseType::Release, 5, 1343),
            ("1.7.2", ReleaseType::Release, 5, 1343),
            ("1.7.1", ReleaseType::Release, 5, 1343),
            ("1.7", ReleaseType::Release, 5, 1343),
            
            // 1.6.x series
            ("1.6.4", ReleaseType::Release, 74, 1343),
            ("1.6.3", ReleaseType::Release, 74, 1343),
            ("1.6.2", ReleaseType::Release, 74, 1343),
            ("1.6.1", ReleaseType::Release, 74, 1343),
            ("1.6", ReleaseType::Release, 74, 1343),
            
            // 1.5.x series
            ("1.5.2", ReleaseType::Release, 61, 1343),
            ("1.5.1", ReleaseType::Release, 61, 1343),
            ("1.5", ReleaseType::Release, 61, 1343),
            
            // 1.4.x series
            ("1.4.7", ReleaseType::Release, 51, 1343),
            ("1.4.6", ReleaseType::Release, 51, 1343),
            ("1.4.5", ReleaseType::Release, 51, 1343),
            ("1.4.4", ReleaseType::Release, 51, 1343),
            ("1.4.3", ReleaseType::Release, 51, 1343),
            ("1.4.2", ReleaseType::Release, 51, 1343),
            ("1.4.1", ReleaseType::Release, 51, 1343),
            ("1.4", ReleaseType::Release, 51, 1343),
            
            // 1.3.x series
            ("1.3.2", ReleaseType::Release, 39, 1343),
            ("1.3.1", ReleaseType::Release, 39, 1343),
            ("1.3", ReleaseType::Release, 39, 1343),
            
            // 1.2.x series
            ("1.2.5", ReleaseType::Release, 29, 1343),
            ("1.2.4", ReleaseType::Release, 29, 1343),
            ("1.2.3", ReleaseType::Release, 29, 1343),
            ("1.2.2", ReleaseType::Release, 29, 1343),
            ("1.2.1", ReleaseType::Release, 29, 1343),
            ("1.2", ReleaseType::Release, 29, 1343),
            
            // 1.1.x series
            ("1.1", ReleaseType::Release, 23, 1343),
            
            // 1.0.x series
            ("1.0.1", ReleaseType::Release, 22, 1343),
            ("1.0", ReleaseType::Release, 22, 1343),
            
            // Beta versions
            ("1.0.0", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.8.1", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.8", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.7.3", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.7.2", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.7.1", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.7", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6.6", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6.5", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6.4", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6.3", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6.2", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6.1", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.6", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.5.1", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.5", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.4.01", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.4", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.3.01", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.3", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.2.02", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.2.01", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.2", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.1.02", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.1.01", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.1", ReleaseType::Beta, 22, 1343),
            ("1.0.0-beta-1.0", ReleaseType::Beta, 22, 1343),
            
            // Alpha versions
            ("1.0.0-alpha-2.1.0", ReleaseType::Alpha, 22, 1343),
            ("1.0.0-alpha-2.0.0", ReleaseType::Alpha, 22, 1343),
            ("1.0.0-alpha-1.2.0", ReleaseType::Alpha, 22, 1343),
            ("1.0.0-alpha-1.1.0", ReleaseType::Alpha, 22, 1343),
            ("1.0.0-alpha-1.0.0", ReleaseType::Alpha, 22, 1343),
        ];

        for (version, release_type, protocol, data_version) in versions {
            let supported_loaders = Self::get_supported_loaders_for_version(version);
            
            self.minecraft_versions.insert(version.to_string(), MinecraftVersionInfo {
                id: version.to_string(),
                release_type: release_type.clone(),
                release_date: chrono::Utc::now() - chrono::Duration::days((rand::random::<u64>() % 3650) as i64), // Random date within last 10 years
                protocol_version: protocol,
                data_version,
                is_supported: true,
                supported_loaders,
            });
        }

        // Initialize loader versions
        self.initialize_loader_versions();
        
        info!("Version manager initialized with {} Minecraft versions", self.minecraft_versions.len());
    }

    /// Get supported loaders for a specific Minecraft version
    fn get_supported_loaders_for_version(version: &str) -> Vec<ModLoader> {
        let mut loaders = vec![ModLoader::Vanilla]; // Vanilla is always supported
        
        // Parse version to determine loader support
        if let Some((major, minor, patch)) = Self::parse_version(version) {
            // Forge support (1.6.4+)
            if (major == 1 && minor >= 6) || major > 1 {
                loaders.push(ModLoader::Forge { version: "latest".to_string() });
            }
            
            // Fabric support (1.14+)
            if (major == 1 && minor >= 14) || major > 1 {
                loaders.push(ModLoader::Fabric { version: "latest".to_string() });
                loaders.push(ModLoader::Quilt { version: "latest".to_string() });
            }
            
            // NeoForge support (1.20.1+)
            if (major == 1 && minor >= 20) || major > 1 {
                loaders.push(ModLoader::NeoForge { version: "latest".to_string() });
            }
            
            // Rift support (1.13.2 only)
            if major == 1 && minor == 13 && patch == 2 {
                loaders.push(ModLoader::Rift { version: "latest".to_string() });
            }
            
            // Liteloader support (1.7.10 - 1.12.2)
            if (major == 1 && minor >= 7 && minor <= 12) {
                loaders.push(ModLoader::Liteloader { version: "latest".to_string() });
            }
        }
        
        loaders
    }

    /// Parse version string into major.minor.patch
    fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            let patch = if parts.len() >= 3 { parts[2].parse().ok()? } else { 0 };
            Some((major, minor, patch))
        } else {
            None
        }
    }

    /// Initialize loader versions
    fn initialize_loader_versions(&mut self) {
        // This would typically fetch from loader APIs
        // For now, we'll add some example versions
        
        self.loader_versions.insert(
            ModLoader::Forge { version: "latest".to_string() },
            vec![
                "47.2.0".to_string(), // 1.21.1
                "49.0.0".to_string(), // 1.20.4
                "48.0.0".to_string(), // 1.20.1
                "47.1.0".to_string(), // 1.20
                "46.0.0".to_string(), // 1.19.4
                "45.0.0".to_string(), // 1.19.2
                "44.0.0".to_string(), // 1.19
                "43.0.0".to_string(), // 1.18.2
                "42.0.0".to_string(), // 1.18.1
                "41.0.0".to_string(), // 1.18
                "40.0.0".to_string(), // 1.17.1
                "39.0.0".to_string(), // 1.17
                "38.0.0".to_string(), // 1.16.5
                "37.0.0".to_string(), // 1.16.4
                "36.0.0".to_string(), // 1.16.3
                "35.0.0".to_string(), // 1.16.2
                "34.0.0".to_string(), // 1.16.1
                "33.0.0".to_string(), // 1.16
                "32.0.0".to_string(), // 1.15.2
                "31.0.0".to_string(), // 1.15.1
                "30.0.0".to_string(), // 1.15
                "29.0.0".to_string(), // 1.14.4
                "28.0.0".to_string(), // 1.14.3
                "27.0.0".to_string(), // 1.14.2
                "26.0.0".to_string(), // 1.14.1
                "25.0.0".to_string(), // 1.14
                "24.0.0".to_string(), // 1.13.2
                "23.0.0".to_string(), // 1.13.1
                "22.0.0".to_string(), // 1.13
                "21.0.0".to_string(), // 1.12.2
                "20.0.0".to_string(), // 1.12.1
                "19.0.0".to_string(), // 1.12
                "18.0.0".to_string(), // 1.11.2
                "17.0.0".to_string(), // 1.11.1
                "16.0.0".to_string(), // 1.11
                "15.0.0".to_string(), // 1.10.2
                "14.0.0".to_string(), // 1.10.1
                "13.0.0".to_string(), // 1.10
                "12.0.0".to_string(), // 1.9.4
                "11.0.0".to_string(), // 1.9.3
                "10.0.0".to_string(), // 1.9.2
                "9.0.0".to_string(),  // 1.9.1
                "8.0.0".to_string(),  // 1.9
                "7.0.0".to_string(),  // 1.8.9
                "6.0.0".to_string(),  // 1.8.8
                "5.0.0".to_string(),  // 1.8.7
                "4.0.0".to_string(),  // 1.8.6
                "3.0.0".to_string(),  // 1.8.5
                "2.0.0".to_string(),  // 1.8.4
                "1.0.0".to_string(),  // 1.8.3
            ]
        );

        self.loader_versions.insert(
            ModLoader::Fabric { version: "latest".to_string() },
            vec![
                "0.15.0".to_string(), // 1.21.1
                "0.14.0".to_string(), // 1.20.4
                "0.13.0".to_string(), // 1.20.1
                "0.12.0".to_string(), // 1.20
                "0.11.0".to_string(), // 1.19.4
                "0.10.0".to_string(), // 1.19.2
                "0.9.0".to_string(),  // 1.19
                "0.8.0".to_string(),  // 1.18.2
                "0.7.0".to_string(),  // 1.18.1
                "0.6.0".to_string(),  // 1.18
                "0.5.0".to_string(),  // 1.17.1
                "0.4.0".to_string(),  // 1.17
                "0.3.0".to_string(),  // 1.16.5
                "0.2.0".to_string(),  // 1.16.4
                "0.1.0".to_string(),  // 1.16.3
            ]
        );

        self.loader_versions.insert(
            ModLoader::Quilt { version: "latest".to_string() },
            vec![
                "0.8.0".to_string(), // 1.21.1
                "0.7.0".to_string(), // 1.20.4
                "0.6.0".to_string(), // 1.20.1
                "0.5.0".to_string(), // 1.20
                "0.4.0".to_string(), // 1.19.4
                "0.3.0".to_string(), // 1.19.2
                "0.2.0".to_string(), // 1.19
                "0.1.0".to_string(), // 1.18.2
            ]
        );

        self.loader_versions.insert(
            ModLoader::NeoForge { version: "latest".to_string() },
            vec![
                "20.1.0".to_string(), // 1.20.1
                "20.0.0".to_string(), // 1.20
            ]
        );

        self.loader_versions.insert(
            ModLoader::Rift { version: "latest".to_string() },
            vec![
                "1.0.0".to_string(), // 1.13.2
            ]
        );

        self.loader_versions.insert(
            ModLoader::Liteloader { version: "latest".to_string() },
            vec![
                "1.12.2".to_string(),
                "1.12.1".to_string(),
                "1.12".to_string(),
                "1.11.2".to_string(),
                "1.11.1".to_string(),
                "1.11".to_string(),
                "1.10.2".to_string(),
                "1.10.1".to_string(),
                "1.10".to_string(),
                "1.9.4".to_string(),
                "1.9.3".to_string(),
                "1.9.2".to_string(),
                "1.9.1".to_string(),
                "1.9".to_string(),
                "1.8.9".to_string(),
                "1.8.8".to_string(),
                "1.8.7".to_string(),
                "1.8.6".to_string(),
                "1.8.5".to_string(),
                "1.8.4".to_string(),
                "1.8.3".to_string(),
                "1.8.2".to_string(),
                "1.8.1".to_string(),
                "1.8".to_string(),
                "1.7.10".to_string(),
                "1.7.9".to_string(),
                "1.7.8".to_string(),
                "1.7.7".to_string(),
                "1.7.6".to_string(),
                "1.7.5".to_string(),
                "1.7.4".to_string(),
                "1.7.3".to_string(),
                "1.7.2".to_string(),
                "1.7.1".to_string(),
                "1.7".to_string(),
            ]
        );
    }

    /// Get all Minecraft versions
    pub fn get_minecraft_versions(&self) -> Vec<MinecraftVersionInfo> {
        self.minecraft_versions.values().cloned().collect()
    }

    /// Get versions by release type
    pub fn get_versions_by_type(&self, release_type: &ReleaseType) -> Vec<MinecraftVersionInfo> {
        self.minecraft_versions
            .values()
            .filter(|v| v.release_type == *release_type)
            .cloned()
            .collect()
    }

    /// Get supported loaders for a specific Minecraft version
    pub fn get_supported_loaders(&self, minecraft_version: &str) -> Result<Vec<ModLoader>> {
        if let Some(version_info) = self.minecraft_versions.get(minecraft_version) {
            Ok(version_info.supported_loaders.clone())
        } else {
            Err(anyhow::anyhow!("Minecraft version {} not found", minecraft_version))
        }
    }

    /// Get loader versions for a specific loader
    pub fn get_loader_versions(&self, loader: &ModLoader) -> Result<Vec<String>> {
        self.loader_versions
            .get(loader)
            .map(|v| v.clone())
            .ok_or_else(|| anyhow::anyhow!("Loader {:?} not found", loader))
    }

    /// Check if a loader is supported for a specific Minecraft version
    pub fn is_loader_supported(&self, minecraft_version: &str, loader: &ModLoader) -> bool {
        if let Some(version_info) = self.minecraft_versions.get(minecraft_version) {
            version_info.supported_loaders.contains(loader)
        } else {
            false
        }
    }

    /// Get the latest version of a specific release type
    pub fn get_latest_version(&self, release_type: &ReleaseType) -> Option<MinecraftVersionInfo> {
        self.minecraft_versions
            .values()
            .filter(|v| v.release_type == *release_type)
            .max_by_key(|v| &v.release_date)
            .cloned()
    }

    /// Get version timeline (sorted by release date)
    pub fn get_version_timeline(&self) -> Vec<MinecraftVersionInfo> {
        let mut versions: Vec<MinecraftVersionInfo> = self.minecraft_versions.values().cloned().collect();
        versions.sort_by_key(|v| v.release_date);
        versions
    }

    /// Search versions by query
    pub fn search_versions(&self, query: &str) -> Vec<MinecraftVersionInfo> {
        let query_lower = query.to_lowercase();
        self.minecraft_versions
            .values()
            .filter(|v| {
                v.id.to_lowercase().contains(&query_lower) ||
                v.release_type.to_string().to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_manager_initialization() {
        let manager = VersionManager::new();
        assert!(!manager.minecraft_versions.is_empty());
        assert!(!manager.loader_versions.is_empty());
    }

    #[test]
    fn test_get_supported_loaders() {
        let manager = VersionManager::new();
        
        // Test modern version (should have Forge, Fabric, Quilt, NeoForge)
        let loaders = manager.get_supported_loaders("1.21.1").unwrap();
        assert!(loaders.contains(&ModLoader::Vanilla));
        assert!(loaders.iter().any(|l| matches!(l, ModLoader::Forge { .. })));
        assert!(loaders.iter().any(|l| matches!(l, ModLoader::Fabric { .. })));
        assert!(loaders.iter().any(|l| matches!(l, ModLoader::Quilt { .. })));
        assert!(loaders.iter().any(|l| matches!(l, ModLoader::NeoForge { .. })));
        
        // Test older version (should have Forge, Liteloader)
        let loaders = manager.get_supported_loaders("1.12.2").unwrap();
        assert!(loaders.contains(&ModLoader::Vanilla));
        assert!(loaders.iter().any(|l| matches!(l, ModLoader::Forge { .. })));
        assert!(loaders.iter().any(|l| matches!(l, ModLoader::Liteloader { .. })));
    }

    #[test]
    fn test_version_parsing() {
        assert_eq!(VersionManager::parse_version("1.21.1"), Some((1, 21, 1)));
        assert_eq!(VersionManager::parse_version("1.20"), Some((1, 20, 0)));
        assert_eq!(VersionManager::parse_version("1.0"), Some((1, 0, 0)));
        assert_eq!(VersionManager::parse_version("invalid"), None);
    }

    #[test]
    fn test_search_versions() {
        let manager = VersionManager::new();
        
        let results = manager.search_versions("1.21");
        assert!(!results.is_empty());
        assert!(results.iter().any(|v| v.id.starts_with("1.21")));
        
        let results = manager.search_versions("release");
        assert!(!results.is_empty());
        assert!(results.iter().all(|v| v.release_type == ReleaseType::Release));
    }
}
