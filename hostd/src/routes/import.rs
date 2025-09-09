use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ScanReq { pub staging_dir: String }

#[derive(Serialize)]
pub struct ImportCandidate { pub region_file: String, pub chunks: u32 }

#[derive(Serialize)]
pub struct ScanResp { pub candidates: Vec<ImportCandidate> }

#[derive(Deserialize)]
pub struct ApplyReq { pub candidates: Vec<String>, pub tps_floor: f32 }

#[derive(Serialize)]
pub struct ApplyResp { pub applied: Vec<String>, pub skipped: Vec<String> }

pub async fn scan(Path(_id): Path<String>, Json(_req): Json<ScanReq>) -> Json<ScanResp> {
    // TODO: compute diff of staged vs live; skip loaded chunks
    Json(ScanResp { candidates: vec![] })
}

pub async fn apply(Path(_id): Path<String>, Json(_req): Json<ApplyReq>) -> Json<ApplyResp> {
    // TODO: TPS-aware pacing; fsync + atomic rename per region
    Json(ApplyResp { applied: vec![], skipped: vec![] })
}