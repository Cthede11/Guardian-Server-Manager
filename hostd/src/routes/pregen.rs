use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Clone)]
pub struct Capabilities { pub gpu: bool, pub fallback: bool }

#[derive(Serialize, Clone)]
pub struct PregenStatus {
    pub state: String,            // idle|running|paused|failed|done
    pub progress: f32,            // 0..1
    pub eta_seconds: Option<u64>,
    pub capabilities: Capabilities,
    pub last_error: Option<String>,
}

#[derive(Deserialize)]
pub struct PlanReq { pub radius: u32, pub dimensions: Vec<String> }

#[derive(Serialize)]
pub struct PlanResp { pub ok: bool, pub plan_id: String }

static PREGEN_STATE: once_cell::sync::Lazy<Arc<RwLock<PregenStatus>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(RwLock::new(PregenStatus {
            state: "idle".into(),
            progress: 0.0,
            eta_seconds: None,
            capabilities: Capabilities { gpu: super::util::gpu_available(), fallback: true },
            last_error: None,
        }))
    });

pub async fn status(Path(_id): Path<String>) -> Json<PregenStatus> {
    Json(PREGEN_STATE.read().await.clone())
}

pub async fn plan(Path(_id): Path<String>, Json(_req): Json<PlanReq>) -> Json<PlanResp> {
    Json(PlanResp { ok: true, plan_id: uuid::Uuid::new_v4().to_string() })
}

pub async fn start(Path(_id): Path<String>) -> Json<PlanResp> {
    {
        let mut s = PREGEN_STATE.write().await;
        s.state = "running".into();
        s.progress = 0.0;
        s.last_error = None;
    }
    // TODO: kick GPU worker or fallback orchestration
    Json(PlanResp { ok: true, plan_id: "current".into() })
}

pub async fn pause(_: Path<String>) -> Json<PlanResp> { Json(PlanResp { ok: true, plan_id: "current".into() }) }
pub async fn resume(_: Path<String>) -> Json<PlanResp> { Json(PlanResp { ok: true, plan_id: "current".into() }) }
pub async fn cancel(_: Path<String>) -> Json<PlanResp> {
    let mut s = PREGEN_STATE.write().await;
    s.state = "idle".into();
    s.progress = 0.0;
    Json(PlanResp { ok: true, plan_id: "current".into() })
}