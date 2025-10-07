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

use crate::api::ApiResponse;
use crate::database::{DatabaseManager, ModMetadata, ModVersion, ModDependency};
use crate::version_resolver::{VersionResolver, DependencyResolution};

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
    pub database: DatabaseManager,
    pub version_resolver: VersionResolver,
}

// Note: ModpackState cannot implement Default because it requires database and version_resolver

impl ModpackState {
    pub fn new(database: DatabaseManager, version_resolver: VersionResolver) -> Self {
        Self {
            database,
            version_resolver,
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
        
        // Dependency resolution
        .route("/api/mods/:id/dependencies", get(resolve_mod_dependencies))
        .route("/api/modpacks/:id/dependencies", get(resolve_modpack_dependencies))
        .route("/api/mods/resolve-dependencies", post(auto_resolve_dependencies))
        
        // Server mod management
        .route("/api/servers/:id/mods", get(get_server_mods))
        .route("/api/servers/:id/mods", post(add_mod_to_server))
        .route("/api/servers/:id/mods/:mod_id", delete(remove_mod_from_server))
        .route("/api/servers/:id/modpacks/:modpack_id", post(apply_modpack_to_server))
        .with_state(state)
}

// Minecraft Version Management

pub async fn get_minecraft_versions(State(state): State<ModpackState>) -> Json<ApiResponse<Vec<MinecraftVersion>>> {
    match state.database.get_minecraft_versions().await {
        Ok(db_versions) => {
        let versions: Vec<MinecraftVersion> = db_versions.into_iter().map(|v| MinecraftVersion {
            id: v.id.clone(),
            version: v.id.clone(), // Use id as version since there's no version field
            release_type: v.release_type,
            release_date: v.release_date.format("%Y-%m-%d").to_string(),
            supported_loaders: vec!["forge".to_string(), "fabric".to_string(), "quilt".to_string()],
        }).collect();
            
            Json(ApiResponse::success(versions))
        }
        Err(e) => {
            eprintln!("Error fetching Minecraft versions: {}", e);
            Json(ApiResponse::error("Failed to fetch Minecraft versions"))
        }
    }
}

pub async fn get_minecraft_version(
    State(state): State<ModpackState>,
    Path(version): Path<String>
) -> Result<Json<ApiResponse<MinecraftVersion>>, StatusCode> {
    // TODO: Implement proper version lookup from database
    // For now, return a mock response
    let mock_version = MinecraftVersion {
        id: version.clone(),
        version: version.clone(),
        release_type: "release".to_string(),
        release_date: chrono::Utc::now().to_rfc3339(),
        supported_loaders: vec!["forge".to_string(), "fabric".to_string()],
    };
    Ok(Json(ApiResponse::success(mock_version)))
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
    match state.database.search_mod_metadata(
        filters.search_query.as_deref(),
        filters.category.as_deref(),
        None, // provider filter not implemented yet
        filters.side.as_deref(),
        filters.limit,
    ).await {
        Ok(metadata_list) => {
            let mods: Vec<ModInfo> = metadata_list.into_iter().map(|m| ModInfo {
                id: m.id,
                name: m.name,
                description: m.description,
                author: m.author,
                version: "latest".to_string(), // TODO: Get actual latest version
                minecraft_version: "1.21.1".to_string(), // TODO: Get from context
                loader: "fabric".to_string(), // TODO: Get from context
                category: m.category,
                side: m.side,
                download_url: None, // TODO: Get from version resolver
                file_size: None,
                sha1: None,
                dependencies: vec![], // TODO: Get from database
                created_at: m.created_at,
                updated_at: m.updated_at,
            }).collect();
            
            let page = filters.page.unwrap_or(1);
            let per_page = filters.limit.unwrap_or(50);
            let total = mods.len() as u32;
            let start = ((page - 1) * per_page) as usize;
            let end = (start + per_page as usize).min(mods.len());
            
            let paginated_mods = if start < mods.len() {
                mods[start..end].to_vec()
            } else {
                Vec::new()
            };
            
            let result = ModSearchResult {
                mods: paginated_mods,
                total,
                page,
                per_page,
                has_more: end < mods.len(),
            };
            
            Json(ApiResponse::success(result))
        }
        Err(e) => {
            eprintln!("Error searching mods: {}", e);
            Json(ApiResponse::error("Failed to search mods"))
        }
    }
}

pub async fn get_mod(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<ModInfo>>, StatusCode> {
    match state.database.get_mod_metadata(&id).await {
        Ok(Some(metadata)) => {
            let mod_info = ModInfo {
                id: metadata.id,
                name: metadata.name,
                description: metadata.description,
                author: metadata.author,
                version: "latest".to_string(), // TODO: Get actual latest version
                minecraft_version: "1.21.1".to_string(), // TODO: Get from context
                loader: "fabric".to_string(), // TODO: Get from context
                category: metadata.category,
                side: metadata.side,
                download_url: None, // TODO: Get from version resolver
                file_size: None,
                sha1: None,
                dependencies: vec![], // TODO: Get from database
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
            };
            
            Ok(Json(ApiResponse::success(mod_info)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error fetching mod: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_mod_versions(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Json<ApiResponse<Vec<ModInfo>>> {
    match state.database.get_mod_versions_by_metadata_id(&id).await {
        Ok(versions) => {
            let mod_infos: Vec<ModInfo> = versions.into_iter().map(|v| ModInfo {
                id: v.id,
                name: "Mod Version".to_string(), // TODO: Get from metadata
                description: "Mod version".to_string(), // TODO: Get from metadata
                author: "Unknown".to_string(), // TODO: Get from metadata
                version: v.version,
                minecraft_version: v.minecraft_version,
                loader: v.loader,
                category: "unknown".to_string(), // TODO: Get from metadata
                side: "both".to_string(), // TODO: Get from metadata
                download_url: Some(v.download_url),
                file_size: Some(v.file_size),
                sha1: v.sha1,
                dependencies: vec![], // TODO: Get from database
                created_at: v.created_at,
                updated_at: v.updated_at,
            }).collect();
            
            Json(ApiResponse::success(mod_infos))
        }
        Err(e) => {
            eprintln!("Error fetching mod versions: {}", e);
            Json(ApiResponse::error("Failed to fetch mod versions"))
        }
    }
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
    match state.database.get_modpacks().await {
        Ok(db_modpacks) => {
            let modpacks: Vec<Modpack> = db_modpacks.into_iter().map(|mp| Modpack {
                id: mp.id,
                name: mp.name,
                description: mp.description,
                minecraft_version: mp.minecraft_version,
                loader: mp.loader,
                client_mods: serde_json::from_str(&mp.client_mods).unwrap_or_default(),
                server_mods: serde_json::from_str(&mp.server_mods).unwrap_or_default(),
                config: mp.config.and_then(|c| serde_json::from_str(&c).ok()),
                created_at: mp.created_at,
                updated_at: mp.updated_at,
            }).collect();
            
            Json(ApiResponse::success(modpacks))
        }
        Err(e) => {
            eprintln!("Error fetching modpacks: {}", e);
            Json(ApiResponse::error("Failed to fetch modpacks"))
        }
    }
}

pub async fn get_modpack(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    match state.database.get_modpack(&id).await {
        Ok(Some(db_modpack)) => {
            let modpack = Modpack {
                id: db_modpack.id,
                name: db_modpack.name,
                description: db_modpack.description,
                minecraft_version: db_modpack.minecraft_version,
                loader: db_modpack.loader,
                client_mods: serde_json::from_str(&db_modpack.client_mods).unwrap_or_default(),
                server_mods: serde_json::from_str(&db_modpack.server_mods).unwrap_or_default(),
                config: db_modpack.config.and_then(|c| serde_json::from_str(&c).ok()),
                created_at: db_modpack.created_at,
                updated_at: db_modpack.updated_at,
            };
            
            Ok(Json(ApiResponse::success(modpack)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error fetching modpack: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_modpack(
    State(state): State<ModpackState>,
    Json(request): Json<CreateModpackRequest>
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    let modpack_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let db_modpack = crate::database::Modpack {
        id: modpack_id.clone(),
        name: request.name.clone(),
        description: request.description.clone(),
        minecraft_version: request.minecraft_version.clone(),
        loader: request.loader.clone(),
        client_mods: serde_json::to_string(&request.client_mods).unwrap_or_default(),
        server_mods: serde_json::to_string(&request.server_mods).unwrap_or_default(),
        config: request.config.as_ref().and_then(|c| serde_json::to_string(c).ok()),
        created_at: now,
        updated_at: now,
    };
    
    match state.database.create_modpack(&db_modpack).await {
        Ok(_) => {
            let modpack = Modpack {
                id: modpack_id,
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
            
            Ok(Json(ApiResponse::success(modpack)))
        }
        Err(e) => {
            eprintln!("Error creating modpack: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_modpack(
    State(state): State<ModpackState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateModpackRequest>
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    // TODO: Implement proper modpack update using database
    // For now, return a mock response
    let mock_modpack = Modpack {
        id: id.clone(),
        name: request.name.unwrap_or_else(|| "Updated Modpack".to_string()),
        description: request.description,
        minecraft_version: "1.20.1".to_string(),
        loader: "forge".to_string(),
        client_mods: request.client_mods.unwrap_or_default(),
        server_mods: request.server_mods.unwrap_or_default(),
        config: request.config,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    Ok(Json(ApiResponse::success(mock_modpack)))
}

pub async fn delete_modpack(
    State(state): State<ModpackState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // TODO: Implement proper modpack deletion using database
    // For now, return success
    Ok(Json(ApiResponse::success(())))
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
    Path(id): Path<String>
) -> Json<ApiResponse<Vec<ModInfo>>> {
    // Placeholder implementation - in real implementation, this would read from server directory
    Json(ApiResponse::success(vec![]))
}

pub async fn add_mod_to_server(
    Path((id, mod_id)): Path<(String, String)>,
    Json(request): Json<serde_json::Value>
) -> Json<ApiResponse<()>> {
    // Placeholder implementation - in real implementation, this would download and install the mod
    println!("Adding mod {} to server {}", mod_id, id);
    Json(ApiResponse::success(()))
}

pub async fn remove_mod_from_server(
    Path((id, mod_id)): Path<(String, String)>
) -> Json<ApiResponse<()>> {
    // Placeholder implementation - in real implementation, this would remove the mod from server
    println!("Removing mod {} from server {}", mod_id, id);
    Json(ApiResponse::success(()))
}

pub async fn apply_modpack_to_server(
    Path((id, modpack_id)): Path<(String, String)>
) -> Json<ApiResponse<()>> {
    // Placeholder implementation - in real implementation, this would apply all mods from modpack
    println!("Applying modpack {} to server {}", modpack_id, id);
    Json(ApiResponse::success(()))
}

// Dependency Resolution

pub async fn resolve_mod_dependencies(
    State(state): State<ModpackState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>
) -> Json<ApiResponse<DependencyResolution>> {
    let default_mc_version = "1.21.1".to_string();
    let default_loader = "fabric".to_string();
    let minecraft_version = params.get("minecraft_version").unwrap_or(&default_mc_version);
    let loader = params.get("loader").unwrap_or(&default_loader);
    
    match state.version_resolver.resolve_dependencies(&id, "latest", minecraft_version, loader).await {
        Ok(resolution) => Json(ApiResponse::success(resolution)),
        Err(e) => {
            eprintln!("Error resolving mod dependencies: {}", e);
            Json(ApiResponse::error("Failed to resolve mod dependencies"))
        }
    }
}

pub async fn resolve_modpack_dependencies(
    State(state): State<ModpackState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>
) -> Json<ApiResponse<Vec<DependencyResolution>>> {
    let default_mc_version = "1.21.1".to_string();
    let default_loader = "fabric".to_string();
    let minecraft_version = params.get("minecraft_version").unwrap_or(&default_mc_version);
    let loader = params.get("loader").unwrap_or(&default_loader);
    
    // Get modpack mods
    match state.database.get_modpack(&id).await {
        Ok(Some(modpack)) => {
            let client_mods: Vec<String> = serde_json::from_str(&modpack.client_mods).unwrap_or_default();
            let server_mods: Vec<String> = serde_json::from_str(&modpack.server_mods).unwrap_or_default();
            let all_mods = [client_mods, server_mods].concat();
            
            match state.version_resolver.auto_resolve_dependencies(all_mods, minecraft_version, loader).await {
                Ok(resolutions) => Json(ApiResponse::success(resolutions)),
                Err(e) => {
                    eprintln!("Error resolving modpack dependencies: {}", e);
                    Json(ApiResponse::error("Failed to resolve modpack dependencies"))
                }
            }
        }
        Ok(None) => Json(ApiResponse::error("Modpack not found")),
        Err(e) => {
            eprintln!("Error fetching modpack: {}", e);
            Json(ApiResponse::error("Failed to fetch modpack"))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AutoResolveDependenciesRequest {
    pub mod_ids: Vec<String>,
    pub minecraft_version: String,
    pub loader: String,
}

pub async fn auto_resolve_dependencies(
    State(state): State<ModpackState>,
    Json(request): Json<AutoResolveDependenciesRequest>
) -> Json<ApiResponse<Vec<DependencyResolution>>> {
    match state.version_resolver.auto_resolve_dependencies(
        request.mod_ids,
        &request.minecraft_version,
        &request.loader
    ).await {
        Ok(resolutions) => Json(ApiResponse::success(resolutions)),
        Err(e) => {
            eprintln!("Error auto-resolving dependencies: {}", e);
            Json(ApiResponse::error("Failed to auto-resolve dependencies"))
        }
    }
}
