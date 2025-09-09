use std::{fs, path::PathBuf, io::Write, time::Duration};
use axum::{routing::get, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::{info, warn};

pub const PORT_MIN: u16 = 8080;
pub const PORT_MAX: u16 = 8090;

#[derive(Clone)]
pub struct BootInfo {
  pub port: u16,
  pub pid: u32,
}

#[derive(Serialize)]
struct Health { ok: bool, port: u16, pid: u32 }

pub async fn choose_port() -> u16 {
  for port in PORT_MIN..=PORT_MAX {
    if TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await.is_ok() {
      return port;
    }
  }
  // last resort
  PORT_MIN
}

pub fn write_port_file(port: u16) -> std::io::Result<()> {
  let mut p = PathBuf::from("backend_port.txt");
  let mut f = fs::File::create(&p)?;
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

// Poll an existing instance if port file exists and is healthy
pub async fn try_attach_existing() -> Option<BootInfo> {
  if let Ok(txt) = std::fs::read_to_string("backend_port.txt") {
    if let Ok(port) = txt.trim().parse::<u16>() {
      let url = format!("http://127.0.0.1:{}/healthz", port);
      if let Ok(resp) = reqwest::get(url).await {
        if resp.status().is_success() { // existing alive
          info!("Reusing existing hostd at {}", port);
          return Some(BootInfo { port, pid: std::process::id() });
        }
      } else {
        warn!("Stale backend_port.txt; will overwrite");
      }
    }
  }
  None
}
