use axum::{extract::Path, Json};
use serde::Serialize;

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
pub async fn get_world(Path(server_id): Path<String>) -> Json<WorldInfo> {
    // TODO: read actual level.dat & server.properties
    let suggested = super::util::suggested_radius_for(&server_id).await.unwrap_or(5000);
    Json(WorldInfo {
        name: "world".into(),
        seed: 0,
        default_dimension: "minecraft:overworld".into(),
        dimensions: vec![
            "minecraft:overworld".into(),
            "minecraft:the_nether".into(),
            "minecraft:the_end".into(),
        ],
        world_border: WorldBorder { center: (0.0, 0.0), radius: suggested },
        pregen: PregenSummary { suggested_radius: suggested, state: "idle".into() },
    })
}

#[axum::debug_handler]
pub async fn list_dimensions(Path(_server_id): Path<String>) -> Json<Dimensions> {
    Json(Dimensions {
        items: vec![
            "minecraft:overworld".into(),
            "minecraft:the_nether".into(),
            "minecraft:the_end".into(),
        ],
    })
}