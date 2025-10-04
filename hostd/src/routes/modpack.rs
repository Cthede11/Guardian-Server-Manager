use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::ApiResponse;

/// Minecraft version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub version: String,
    pub release_type: String,
    pub release_date: String,
    pub supported_loaders: Vec<String>,
}

/// Mod loader information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoader {
    pub loader_type: String,
    pub version: String,
    pub minecraft_version: String,
    pub stable: bool,
}

/// Mod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    pub category: String,
    pub side: String, // client, server, both
    pub download_url: Option<String>,
    pub file_size: Option<u64>,
    pub sha1: Option<String>,
    pub dependencies: Vec<ModDependency>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_range: String,
    pub required: bool,
}

/// Mod search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub mods: Vec<ModInfo>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
}

/// Modpack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modpack {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub loader: String,
    pub client_mods: Vec<String>,
    pub server_mods: Vec<String>,
    pub config: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Modpack compatibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackCompatibility {
    pub compatible: bool,
    pub issues: Vec<CompatibilityIssue>,
    pub warnings: Vec<String>,
}

/// Compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    pub mod_id: String,
    pub issue_type: String,
    pub description: String,
    pub severity: String, // error, warning, info
}

/// Mod search filters
#[derive(Debug, Deserialize)]
pub struct ModFilters {
    pub search_query: Option<String>,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub category: Option<String>,
    pub side: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Modpack creation request
#[derive(Debug, Deserialize)]
pub struct CreateModpackRequest {
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub loader: String,
    pub client_mods: Vec<String>,
    pub server_mods: Vec<String>,
    pub config: Option<serde_json::Value>,
}

/// Modpack update request
#[derive(Debug, Deserialize)]
pub struct UpdateModpackRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub client_mods: Option<Vec<String>>,
    pub server_mods: Option<Vec<String>>,
    pub config: Option<serde_json::Value>,
}

/// Modpack statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackStats {
    pub total_modpacks: u32,
    pub total_mods: u32,
    pub most_popular_mods: Vec<ModInfo>,
    pub compatibility_stats: CompatibilityStats,
}

/// Compatibility statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityStats {
    pub total_checks: u32,
    pub successful_checks: u32,
    pub failed_checks: u32,
}

/// Mod statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModStats {
    pub download_count: u32,
    pub rating: f32,
    pub compatibility_score: f32,
    pub used_in_modpacks: u32,
}

/// In-memory storage for modpacks and mods
pub type ModpackStore = std::sync::Arc<std::sync::Mutex<HashMap<String, Modpack>>>;
pub type ModStore = std::sync::Arc<std::sync::Mutex<HashMap<String, ModInfo>>>;
pub type VersionStore = std::sync::Arc<std::sync::Mutex<Vec<MinecraftVersion>>>;

/// Application state for modpack routes
#[derive(Clone)]
pub struct ModpackState {
    pub modpacks: ModpackStore,
    pub mods: ModStore,
    pub versions: VersionStore,
}

impl ModpackState {
    pub fn new() -> Self {
        Self {
            modpacks: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            mods: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            versions: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

/// Create modpack router
pub fn create_modpack_router(state: ModpackState) -> Router {
    Router::new()
        // Minecraft versions
        .route("/api/modpacks/versions", get(get_minecraft_versions))
        .route("/api/modpacks/versions/:version", get(get_minecraft_version))
        .route("/api/modpacks/loaders", get(get_supported_loaders))
        
        // Mod management
        .route("/api/modpacks/mods", get(search_mods))
        .route("/api/modpacks/mods/:id", get(get_mod))
        .route("/api/modpacks/mods/:id/versions", get(get_mod_versions))
        .route("/api/modpacks/mods/:id/stats", get(get_mod_stats))
        
        // Modpack management
        .route("/api/modpacks", get(get_modpacks))
        .route("/api/modpacks", post(create_modpack))
        .route("/api/modpacks/:id", get(get_modpack))
        .route("/api/modpacks/:id", put(update_modpack))
        .route("/api/modpacks/:id", delete(delete_modpack))
        .route("/api/modpacks/stats", get(get_modpack_stats))
        
        // Compatibility checking
        .route("/api/modpacks/compatibility", post(check_modpack_compatibility))
        .route("/api/mods/compatibility", post(check_mod_compatibility))
        
        // Server mod management
        .route("/api/servers/:server_id/mods", get(get_server_mods))
        .route("/api/servers/:server_id/mods", post(add_mod_to_server))
        .route("/api/servers/:server_id/mods/:mod_id", delete(remove_mod_from_server))
        .route("/api/servers/:server_id/modpacks/:modpack_id", post(apply_modpack_to_server))
        .with_state(state)
}

// Minecraft Version Management

pub async fn get_minecraft_versions(State(state): State<ModpackState>) -> Json<ApiResponse<Vec<MinecraftVersion>>> {
    let versions = state.versions.lock().unwrap();
    let version_list: Vec<MinecraftVersion> = versions.clone();
    
    Json(ApiResponse::success(version_list))
}

pub async fn get_minecraft_version(
    State(state): State<ModpackState>,
    Path(version): Path<String>
) -> Result<Json<ApiResponse<MinecraftVersion>>, StatusCode> {
    let versions = state.versions.lock().unwrap();
    
    match versions.iter().find(|v| v.version == version) {
        Some(version) => Ok(Json(ApiResponse::success(version.clone()))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_supported_loaders(
    Query(params): Query<HashMap<String, String>>
) -> Json<ApiResponse<Vec<ModLoader>>> {
    let default_version = "1.20.1".to_string();
    let minecraft_version = params.get("minecraft_version").unwrap_or(&default_version);
    
    let loaders = vec![
        ModLoader {
            loader_type: "forge".to_string(),
            version: "47.2.0".to_string(),
            minecraft_version: minecraft_version.clone(),
            stable: true,
        },
        ModLoader {
            loader_type: "fabric".to_string(),
            version: "0.14.21".to_string(),
            minecraft_version: minecraft_version.clone(),
            stable: true,
        },
        ModLoader {
            loader_type: "quilt".to_string(),
            version: "0.4.0".to_string(),
            minecraft_version: minecraft_version.clone(),
            stable: true,
        },
    ];
    
    Json(ApiResponse::success(loaders))
}

// Mod Management

pub async fn search_mods(
    State(state): State<ModpackState>,
    Query(filters): Query<ModFilters>
) -> Json<ApiResponse<ModSearchResult>> {
    let mods = state.mods.lock().unwrap();
    let mut filtered_mods: Vec<ModInfo> = mods.values().cloned().collect();
    
    // Apply filters
    if let Some(query) = &filters.search_query {
        filtered_mods.retain(|m| 
            m.name.to_lowercase().contains(&query.to_lowercase()) ||
            m.description.to_lowercase().contains(&query.to_lowercase())
        );
    }
    
    if let Some(mc_version) = &filters.minecraft_version {
        filtered_mods.retain(|m| m.minecraft_version == *mc_version);
    }
    
    if let Some(loader) = &filters.loader {
        filtered_mods.retain(|m| m.loader == *loader);
    }
    
    if let Some(category) = &filters.category {
        if category != "all" {
            filtered_mods.retain(|m| m.category == *category);
        }
    }
    
    if let Some(side) = &filters.side {
        if side != "all" {
            filtered_mods.retain(|m| m.side == *side);
        }
    }
    
    let page = filters.page.unwrap_or(1);
    let per_page = filters.limit.unwrap_or(50);
    let total = filtered_mods.len() as u32;
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(filtered_mods.len());
    
    let paginated_mods = if start < filtered_mods.len() {
        filtered_mods[start..end].to_vec()
    } else {
        Vec::new()
    };
    
    let result = ModSearchResult {
        mods: paginated_mods,
        total,
        page,
        per_page,
        has_more: end < filtered_mods.len(),
    };
    
    Json(ApiResponse::success(result))
}

pub async fn get_mod(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<ModInfo>>, StatusCode> {
    let mods = state.mods.lock().unwrap();
    
    match mods.get(&id) {
        Some(mod_info) => Ok(Json(ApiResponse::success(mod_info.clone()))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_mod_versions(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Json<ApiResponse<Vec<ModInfo>>> {
    let mods = state.mods.lock().unwrap();
    
    // For now, return the mod itself as the only version
    // In a real implementation, this would return all versions of the mod
    let versions = mods.get(&id)
        .map(|m| vec![m.clone()])
        .unwrap_or_default();
    
    Json(ApiResponse::success(versions))
}

pub async fn get_mod_stats(
    State(_state): State<ModpackState>,
    Path(_id): Path<String>
) -> Json<ApiResponse<ModStats>> {
    // Placeholder stats - in real implementation, this would come from analytics
    let stats = ModStats {
        download_count: 1000,
        rating: 4.5,
        compatibility_score: 0.95,
        used_in_modpacks: 25,
    };
    
    Json(ApiResponse::success(stats))
}

// Modpack Management

pub async fn get_modpacks(State(state): State<ModpackState>) -> Json<ApiResponse<Vec<Modpack>>> {
    let modpacks = state.modpacks.lock().unwrap();
    let modpack_list: Vec<Modpack> = modpacks.values().cloned().collect();
    
    Json(ApiResponse::success(modpack_list))
}

pub async fn get_modpack(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    let modpacks = state.modpacks.lock().unwrap();
    
    match modpacks.get(&id) {
        Some(modpack) => Ok(Json(ApiResponse::success(modpack.clone()))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_modpack(
    State(state): State<ModpackState>,
    Json(request): Json<CreateModpackRequest>
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    let modpack_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let modpack = Modpack {
        id: modpack_id.clone(),
        name: request.name,
        description: request.description,
        minecraft_version: request.minecraft_version,
        loader: request.loader,
        client_mods: request.client_mods,
        server_mods: request.server_mods,
        config: request.config,
        created_at: now,
        updated_at: now,
    };
    
    {
        let mut modpacks = state.modpacks.lock().unwrap();
        modpacks.insert(modpack_id.clone(), modpack.clone());
    }
    
    Ok(Json(ApiResponse::success(modpack)))
}

pub async fn update_modpack(
    State(state): State<ModpackState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateModpackRequest>
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    let mut modpacks = state.modpacks.lock().unwrap();
    
    match modpacks.get_mut(&id) {
        Some(modpack) => {
            if let Some(name) = request.name {
                modpack.name = name;
            }
            if let Some(description) = request.description {
                modpack.description = Some(description);
            }
            if let Some(client_mods) = request.client_mods {
                modpack.client_mods = client_mods;
            }
            if let Some(server_mods) = request.server_mods {
                modpack.server_mods = server_mods;
            }
            if let Some(config) = request.config {
                modpack.config = Some(config);
            }
            modpack.updated_at = Utc::now();
            
            Ok(Json(ApiResponse::success(modpack.clone())))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_modpack(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let mut modpacks = state.modpacks.lock().unwrap();
    
    if modpacks.remove(&id).is_some() {
        Ok(Json(ApiResponse::success(())))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_modpack_stats(State(_state): State<ModpackState>) -> Json<ApiResponse<ModpackStats>> {
    // Placeholder stats - in real implementation, this would come from analytics
    let stats = ModpackStats {
        total_modpacks: 50,
        total_mods: 1000,
        most_popular_mods: vec![], // Would be populated from actual data
        compatibility_stats: CompatibilityStats {
            total_checks: 500,
            successful_checks: 450,
            failed_checks: 50,
        },
    };
    
    Json(ApiResponse::success(stats))
}

// Compatibility Checking

pub async fn check_modpack_compatibility(
    Json(request): Json<ModpackCompatibility>
) -> Json<ApiResponse<ModpackCompatibility>> {
    // Placeholder implementation - in real implementation, this would check actual compatibility
    let result = ModpackCompatibility {
        compatible: true,
        issues: vec![],
        warnings: vec!["This is a placeholder compatibility check".to_string()],
    };
    
    Json(ApiResponse::success(result))
}

pub async fn check_mod_compatibility(
    Json(request): Json<serde_json::Value>
) -> Json<ApiResponse<serde_json::Value>> {
    // Placeholder implementation
    let result = serde_json::json!({
        "compatible": true,
        "issues": [],
        "warnings": ["This is a placeholder compatibility check"]
    });
    
    Json(ApiResponse::success(result))
}

// Server Mod Management

pub async fn get_server_mods(
    Path(server_id): Path<String>
) -> Json<ApiResponse<Vec<ModInfo>>> {
    // Placeholder implementation - in real implementation, this would read from server directory
    Json(ApiResponse::success(vec![]))
}

pub async fn add_mod_to_server(
    Path((server_id, mod_id)): Path<(String, String)>,
    Json(request): Json<serde_json::Value>
) -> Json<ApiResponse<()>> {
    // Placeholder implementation - in real implementation, this would download and install the mod
    println!("Adding mod {} to server {}", mod_id, server_id);
    Json(ApiResponse::success(()))
}

pub async fn remove_mod_from_server(
    Path((server_id, mod_id)): Path<(String, String)>
) -> Json<ApiResponse<()>> {
    // Placeholder implementation - in real implementation, this would remove the mod from server
    println!("Removing mod {} from server {}", mod_id, server_id);
    Json(ApiResponse::success(()))
}

pub async fn apply_modpack_to_server(
    Path((server_id, modpack_id)): Path<(String, String)>
) -> Json<ApiResponse<()>> {
    // Placeholder implementation - in real implementation, this would apply all mods from modpack
    println!("Applying modpack {} to server {}", modpack_id, server_id);
    Json(ApiResponse::success(()))
}
