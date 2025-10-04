use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// World information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub name: String,
    pub seed: i64,
    pub default_dimension: String,
    pub dimensions: Vec<String>,
    pub world_border: WorldBorder,
    pub pregen: PregenSummary,
    pub size_mb: u64,
    pub last_modified: DateTime<Utc>,
    pub player_count: u32,
    pub difficulty: String,
    pub gamemode: String,
    pub hardcore: bool,
    pub spawn_protection: u32,
    pub view_distance: u32,
    pub simulation_distance: u32,
}

/// World border information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBorder {
    pub center: (f64, f64),
    pub radius: u32,
    pub warning_distance: u32,
    pub warning_time: u32,
}

/// Pregen summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenSummary {
    pub suggested_radius: u32,
    pub state: String,
    pub progress: f64,
    pub eta_seconds: Option<u64>,
}

/// World dimension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionInfo {
    pub name: String,
    pub dimension_type: String,
    pub generator: String,
    pub seed: i64,
    pub size_mb: u64,
    pub chunk_count: u32,
    pub last_accessed: DateTime<Utc>,
}

/// World freeze information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFreeze {
    pub id: String,
    pub server_id: String,
    pub x: i32,
    pub z: i32,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub cause: String,
    pub resolved: bool,
}

/// World heatmap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldHeatmap {
    pub server_id: String,
    pub dimension: String,
    pub data: Vec<HeatmapPoint>,
    pub resolution: u32,
    pub bounds: (i32, i32, i32, i32), // min_x, min_z, max_x, max_z
    pub generated_at: DateTime<Utc>,
}

/// Heatmap point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapPoint {
    pub x: i32,
    pub z: i32,
    pub intensity: f64,
    pub data_type: String, // "chunks", "entities", "tile_entities", "freezes"
}

/// World manager for handling world data and operations
#[derive(Clone)]
pub struct WorldManager {
    /// Cache of world information by server ID
    world_cache: Arc<RwLock<HashMap<String, WorldInfo>>>,
    /// Cache of dimension information by server ID
    dimension_cache: Arc<RwLock<HashMap<String, Vec<DimensionInfo>>>>,
    /// World freeze tracking
    freeze_tracker: Arc<RwLock<Vec<WorldFreeze>>>,
    /// Base directory for server worlds
    worlds_base_dir: PathBuf,
}

impl WorldManager {
    pub fn new(worlds_base_dir: PathBuf) -> Self {
        Self {
            world_cache: Arc::new(RwLock::new(HashMap::new())),
            dimension_cache: Arc::new(RwLock::new(HashMap::new())),
            freeze_tracker: Arc::new(RwLock::new(Vec::new())),
            worlds_base_dir,
        }
    }

    /// Get world information for a server
    pub async fn get_world_info(&self, server_id: &str) -> Result<WorldInfo, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.world_cache.read().await;
            if let Some(info) = cache.get(server_id) {
                return Ok(info.clone());
            }
        }

        // Load from disk
        let world_info = self.load_world_info(server_id).await?;
        
        // Update cache
        {
            let mut cache = self.world_cache.write().await;
            cache.insert(server_id.to_string(), world_info.clone());
        }

        Ok(world_info)
    }

    /// Load world information from disk
    async fn load_world_info(&self, server_id: &str) -> Result<WorldInfo, Box<dyn std::error::Error>> {
        let world_path = self.worlds_base_dir.join(server_id).join("world");
        
        if !world_path.exists() {
            return Err("World directory does not exist".into());
        }

        // Read level.dat
        let level_dat_path = world_path.join("level.dat");
        let (seed, difficulty, gamemode, hardcore, spawn_protection, view_distance, simulation_distance) = 
            self.read_level_dat(&level_dat_path).await?;

        // Read server.properties
        let server_properties_path = self.worlds_base_dir.join(server_id).join("server.properties");
        let (world_name, world_border) = self.read_server_properties(&server_properties_path).await?;

        // Calculate world size
        let size_mb = self.calculate_world_size(&world_path).await?;

        // Get dimensions
        let dimensions = self.get_available_dimensions(&world_path).await?;

        // Get player count (simplified)
        let player_count = self.get_player_count(&world_path).await?;

        // Get last modified time
        let last_modified = self.get_last_modified(&world_path).await?;

        // Calculate suggested pregen radius
        let suggested_radius = self.calculate_suggested_radius(&world_path).await?;

        Ok(WorldInfo {
            name: world_name,
            seed,
            default_dimension: "minecraft:overworld".to_string(),
            dimensions,
            world_border,
            pregen: PregenSummary {
                suggested_radius,
                state: "idle".to_string(),
                progress: 0.0,
                eta_seconds: None,
            },
            size_mb,
            last_modified,
            player_count,
            difficulty,
            gamemode,
            hardcore,
            spawn_protection,
            view_distance,
            simulation_distance,
        })
    }

    /// Read level.dat file
    async fn read_level_dat(&self, path: &Path) -> Result<(i64, String, String, bool, u32, u32, u32), Box<dyn std::error::Error>> {
        // In a real implementation, this would parse the NBT format
        // For now, return placeholder values
        Ok((
            0, // seed
            "normal".to_string(), // difficulty
            "survival".to_string(), // gamemode
            false, // hardcore
            16, // spawn_protection
            10, // view_distance
            10, // simulation_distance
        ))
    }

    /// Read server.properties file
    async fn read_server_properties(&self, path: &Path) -> Result<(String, WorldBorder), Box<dyn std::error::Error>> {
        // In a real implementation, this would parse the properties file
        // For now, return placeholder values
        Ok((
            "world".to_string(),
            WorldBorder {
                center: (0.0, 0.0),
                radius: 5000,
                warning_distance: 5,
                warning_time: 15,
            }
        ))
    }

    /// Calculate world size in MB
    async fn calculate_world_size(&self, world_path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        let mut total_size = 0u64;
        
        if world_path.exists() {
            let mut entries = tokio::fs::read_dir(world_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += Box::pin(self.calculate_directory_size(&entry.path())).await?;
                }
            }
        }
        
        Ok(total_size / (1024 * 1024)) // Convert to MB
    }

    /// Calculate directory size recursively
    async fn calculate_directory_size(&self, dir_path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        let mut total_size = 0u64;
        
        if dir_path.exists() {
            let mut entries = tokio::fs::read_dir(dir_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += Box::pin(self.calculate_directory_size(&entry.path())).await?;
                }
            }
        }
        
        Ok(total_size)
    }

    /// Get available dimensions
    async fn get_available_dimensions(&self, world_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut dimensions = vec!["minecraft:overworld".to_string()];
        
        // Check for Nether
        if world_path.join("DIM-1").exists() {
            dimensions.push("minecraft:the_nether".to_string());
        }
        
        // Check for End
        if world_path.join("DIM1").exists() {
            dimensions.push("minecraft:the_end".to_string());
        }
        
        Ok(dimensions)
    }

    /// Get player count from world data
    async fn get_player_count(&self, world_path: &Path) -> Result<u32, Box<dyn std::error::Error>> {
        // In a real implementation, this would read player data files
        // For now, return 0
        Ok(0)
    }

    /// Get last modified time
    async fn get_last_modified(&self, world_path: &Path) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
        let metadata = tokio::fs::metadata(world_path).await?;
        let modified = metadata.modified()?;
        let datetime: DateTime<Utc> = modified.into();
        Ok(datetime)
    }

    /// Calculate suggested pregen radius
    async fn calculate_suggested_radius(&self, world_path: &Path) -> Result<u32, Box<dyn std::error::Error>> {
        // In a real implementation, this would analyze world size and performance
        // For now, return a default value
        Ok(5000)
    }

    /// Get dimensions for a server
    pub async fn get_dimensions(&self, server_id: &str) -> Result<Vec<DimensionInfo>, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.dimension_cache.read().await;
            if let Some(dimensions) = cache.get(server_id) {
                return Ok(dimensions.clone());
            }
        }

        // Load from disk
        let dimensions = self.load_dimensions(server_id).await?;
        
        // Update cache
        {
            let mut cache = self.dimension_cache.write().await;
            cache.insert(server_id.to_string(), dimensions.clone());
        }

        Ok(dimensions)
    }

    /// Load dimensions from disk
    async fn load_dimensions(&self, server_id: &str) -> Result<Vec<DimensionInfo>, Box<dyn std::error::Error>> {
        let world_path = self.worlds_base_dir.join(server_id).join("world");
        let mut dimensions = Vec::new();

        // Overworld
        dimensions.push(DimensionInfo {
            name: "minecraft:overworld".to_string(),
            dimension_type: "minecraft:overworld".to_string(),
            generator: "minecraft:overworld".to_string(),
            seed: 0,
            size_mb: self.calculate_world_size(&world_path).await?,
            chunk_count: 0, // Would calculate from region files
            last_accessed: Utc::now(),
        });

        // Nether
        let nether_path = world_path.join("DIM-1");
        if nether_path.exists() {
            dimensions.push(DimensionInfo {
                name: "minecraft:the_nether".to_string(),
                dimension_type: "minecraft:the_nether".to_string(),
                generator: "minecraft:the_nether".to_string(),
                seed: 0,
                size_mb: self.calculate_world_size(&nether_path).await?,
                chunk_count: 0,
                last_accessed: Utc::now(),
            });
        }

        // End
        let end_path = world_path.join("DIM1");
        if end_path.exists() {
            dimensions.push(DimensionInfo {
                name: "minecraft:the_end".to_string(),
                dimension_type: "minecraft:the_end".to_string(),
                generator: "minecraft:the_end".to_string(),
                seed: 0,
                size_mb: self.calculate_world_size(&end_path).await?,
                chunk_count: 0,
                last_accessed: Utc::now(),
            });
        }

        Ok(dimensions)
    }

    /// Get world freezes
    pub async fn get_world_freezes(&self, server_id: &str) -> Result<Vec<WorldFreeze>, Box<dyn std::error::Error>> {
        let freezes = self.freeze_tracker.read().await;
        Ok(freezes.iter()
            .filter(|f| f.server_id == server_id)
            .cloned()
            .collect())
    }

    /// Add world freeze
    pub async fn add_world_freeze(&self, server_id: &str, x: i32, z: i32, duration_ms: u64, cause: String) -> Result<(), Box<dyn std::error::Error>> {
        let freeze = WorldFreeze {
            id: Uuid::new_v4().to_string(),
            server_id: server_id.to_string(),
            x,
            z,
            duration_ms,
            timestamp: Utc::now(),
            cause,
            resolved: false,
        };

        let mut freezes = self.freeze_tracker.write().await;
        freezes.push(freeze);
        Ok(())
    }

    /// Resolve world freeze
    pub async fn resolve_world_freeze(&self, freeze_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut freezes = self.freeze_tracker.write().await;
        if let Some(freeze) = freezes.iter_mut().find(|f| f.id == freeze_id) {
            freeze.resolved = true;
        }
        Ok(())
    }

    /// Generate world heatmap
    pub async fn generate_heatmap(&self, server_id: &str, dimension: &str, data_type: &str, resolution: u32) -> Result<WorldHeatmap, Box<dyn std::error::Error>> {
        // In a real implementation, this would analyze world data
        // For now, return placeholder data
        let heatmap = WorldHeatmap {
            server_id: server_id.to_string(),
            dimension: dimension.to_string(),
            data: vec![], // Would be populated with actual heatmap data
            resolution,
            bounds: (-1000, -1000, 1000, 1000),
            generated_at: Utc::now(),
        };

        Ok(heatmap)
    }

    /// Clear world cache
    pub async fn clear_cache(&self, server_id: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(server_id) = server_id {
            let mut world_cache = self.world_cache.write().await;
            world_cache.remove(server_id);
            
            let mut dimension_cache = self.dimension_cache.write().await;
            dimension_cache.remove(server_id);
        } else {
            let mut world_cache = self.world_cache.write().await;
            world_cache.clear();
            
            let mut dimension_cache = self.dimension_cache.write().await;
            dimension_cache.clear();
        }
        Ok(())
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new(PathBuf::from("./worlds"))
    }
}
