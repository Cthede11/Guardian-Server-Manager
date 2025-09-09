use std::{fs, io::Write, net::Ipv4Addr, path::PathBuf};
use axum::{routing::get, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::{info, warn};

const DEFAULT_MIN: u16 = 52100;
const DEFAULT_MAX: u16 = 52150;
const FORBIDDEN: &[u16] = &[80, 443, 3000, 5173, 8000, 8080, 9000];

#[derive(Clone, Copy, Serialize, serde::Deserialize, Debug)]
pub struct Health { pub ok: bool, pub port: u16, pub pid: u32 }

fn parse_range() -> (u16,u16) {
    if let Ok(s) = std::env::var("HOSTD_PORT_RANGE") {
        let parts: Vec<_> = s.split('-').collect();
        if parts.len()==2 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                if a < b { return (a,b); }
            }
        }
    }
    (DEFAULT_MIN, DEFAULT_MAX)
}

pub async fn choose_port() -> u16 {
    let (min,max) = parse_range();
    
    // Try preferred range first
    for port in min..=max {
        if FORBIDDEN.contains(&port) { continue; }
        if TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await.is_ok() {
            info!("Selected port {} from range {}-{}", port, min, max);
            return port;
        }
    }
    
    // Fallback to original range 8080-8090 if preferred range fails
    warn!("Preferred range {}-{} unavailable, trying fallback range 8080-8090", min, max);
    for port in 8080..=8090 {
        if TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await.is_ok() {
            info!("Selected fallback port {}", port);
            return port;
        }
    }
    
    warn!("All port ranges exhausted, using OS assignment");
    0 // OS will assign
}

pub fn write_port_file(port: u16) -> std::io::Result<()> {
    let mut f = fs::File::create(PathBuf::from("backend_port.txt"))?;
    write!(f, "{}", port)?;
    Ok(())
}

pub fn write_pid_lock(pid: u32, port: u16) -> std::io::Result<()> {
    let mut f = fs::File::create("hostd.lock")?;
    write!(f, "pid={}\nport={}\n", pid, port)?;
    Ok(())
}

pub async fn health_router(port: u16, pid: u32) -> Router {
    Router::new().route("/healthz", get(move || async move {
        axum::Json(Health { ok: true, port, pid })
    }))
}

// If backend_port.txt exists and /healthz is healthy, reuse it.
pub async fn try_attach_existing() -> Option<Health> {
    if let Ok(txt) = fs::read_to_string("backend_port.txt") {
        if let Ok(port) = txt.trim().parse::<u16>() {
            let url = format!("http://127.0.0.1:{}/healthz", port);
            if let Ok(resp) = reqwest::get(url).await {
                if resp.status().is_success() {
                    if let Ok(h) = resp.json::<Health>().await {
                        if h.ok { info!("Reusing hostd on port {}", h.port); return Some(h); }
                    }
                }
            } else {
                warn!("Stale backend_port.txt – will overwrite");
            }
        }
    }
    None
}
