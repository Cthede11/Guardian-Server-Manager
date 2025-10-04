use axum::{extract::{Path, State}, Json, http::StatusCode};
use serde::Serialize;
use crate::ApiResponse;

#[derive(Serialize, Clone)]
pub struct WorldBorder { pub center: (f64, f64), pub radius: u32 }

#[derive(Serialize, Clone)]
pub struct PregenSummary { pub suggested_radius: u32, pub state: String }

#[derive(Serialize, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub seed: i64,
    pub default_dimension: String,
    pub dimensions: Vec<String>,
    pub world_border: WorldBorder,
    pub pregen: PregenSummary,
}

#[derive(Serialize)]
pub struct Dimensions { pub items: Vec<String> }

#[axum::debug_handler]
pub async fn get_world(
    State(world_manager): State<crate::world_manager::WorldManager>,
    Path(server_id): Path<String>
) -> Result<Json<ApiResponse<WorldInfo>>, StatusCode> {
    match world_manager.get_world_info(&server_id).await {
        Ok(world_info) => Ok(Json(ApiResponse::success(WorldInfo {
            name: world_info.name,
            default_dimension: world_info.default_dimension,
            pregen: PregenSummary {
                suggested_radius: world_info.pregen.suggested_radius,
                state: world_info.pregen.state,
            },
            dimensions: world_info.dimensions,
            seed: world_info.seed,
            world_border: WorldBorder {
                center: world_info.world_border.center,
                radius: world_info.world_border.radius,
            },
        }))),
        Err(e) => {
            tracing::error!("Failed to get world info for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
pub async fn list_dimensions(
    State(world_manager): State<crate::world_manager::WorldManager>,
    Path(server_id): Path<String>
) -> Result<Json<ApiResponse<Dimensions>>, StatusCode> {
    match world_manager.get_dimensions(&server_id).await {
        Ok(dimensions) => {
            let items: Vec<String> = dimensions.into_iter().map(|d| d.name).collect();
            Ok(Json(ApiResponse {
                success: true,
                data: Some(Dimensions { items }),
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get dimensions for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}