use axum::{
    extract::{Path, State, Query},
    response::IntoResponse,
    Json,
};
use axum::response::sse::{Sse, Event, KeepAlive};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Serialize, Clone)]
pub struct MetricsPoint {
    pub timestamp: i64,              // ms epoch
    pub tps: f32,
    pub tick_p95_ms: f32,
    pub heap_mb: Option<f32>,        // optional (agent-dependent)
    pub gpu_latency_ms: Option<f32>, // optional
}

#[derive(Clone)]
pub struct MetricsHub {
    pub tx: broadcast::Sender<MetricsPoint>,
    pub ring: Arc<tokio::sync::RwLock<VecDeque<MetricsPoint>>>,
}

impl MetricsHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(256);
        Self { tx, ring: Arc::new(tokio::sync::RwLock::new(VecDeque::with_capacity(600))) }
    }
    pub async fn push(&self, p: MetricsPoint) {
        let _ = self.tx.send(p.clone());
        let mut r = self.ring.write().await;
        if r.len() == 600 { r.pop_front(); }
        r.push_back(p);
    }
}

#[derive(Deserialize)]
pub struct Range { pub from: Option<i64>, pub to: Option<i64> }

// GET /servers/:id/metrics
pub async fn history(State(hub): State<MetricsHub>, _id: Path<String>, _q: Query<Range>) -> Json<Vec<MetricsPoint>> {
    let r = hub.ring.read().await.clone();
    Json(r.into())
}

// GET /servers/:id/metrics/stream
pub async fn stream(State(hub): State<MetricsHub>, _id: Path<String>) -> impl IntoResponse {
    let rx = hub.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|p| async move { p.ok() })
        .map(|p| Event::default().json_data(p));
    Sse::new(stream).keep_alive(KeepAlive::new())
}