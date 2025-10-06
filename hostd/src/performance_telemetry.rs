use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::time::sleep;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Performance metrics for a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub server_id: String,
    pub timestamp: u64,
    pub tps: f32,                    // Ticks per second
    pub tick_ms: f32,               // Average tick time in milliseconds
    pub memory_used: u64,           // Memory used in bytes
    pub memory_max: u64,            // Maximum memory in bytes
    pub memory_percent: f32,        // Memory usage percentage
    pub io_read: u64,               // IO read bytes
    pub io_write: u64,              // IO write bytes
    pub cpu_usage: f32,             // CPU usage percentage
    pub disk_usage: u64,            // Disk usage in bytes
    pub network_in: u64,            // Network input bytes
    pub network_out: u64,           // Network output bytes
    pub player_count: u32,          // Number of players online
    pub chunk_count: u32,           // Number of loaded chunks
    pub entity_count: u32,          // Number of entities
}

/// Performance telemetry collector
pub struct PerformanceTelemetry {
    metrics_history: Arc<Mutex<HashMap<String, Vec<PerformanceMetrics>>>>,
    collection_interval: Duration,
    is_running: Arc<Mutex<bool>>,
}

impl PerformanceTelemetry {
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(HashMap::new())),
            collection_interval,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start collecting performance metrics for all servers
    pub async fn start_collection(&self, servers_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(()); // Already running
        }
        *is_running = true;
        drop(is_running);

        let metrics_history = self.metrics_history.clone();
        let collection_interval = self.collection_interval;
        let is_running = self.is_running.clone();
        let servers_path = servers_path.to_path_buf();

        tokio::spawn(async move {
            while *is_running.lock().await {
                if let Err(e) = Self::collect_metrics_for_all_servers(&metrics_history, &servers_path).await {
                    eprintln!("Error collecting performance metrics: {}", e);
                }
                sleep(collection_interval).await;
            }
        });

        Ok(())
    }

    /// Stop collecting performance metrics
    pub async fn stop_collection(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
    }

    /// Collect metrics for all servers
    async fn collect_metrics_for_all_servers(
        metrics_history: &Arc<Mutex<HashMap<String, Vec<PerformanceMetrics>>>>,
        servers_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !servers_path.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(servers_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let server_path = entry.path();
            if server_path.is_dir() {
                if let Some(server_id) = server_path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(metrics) = Self::collect_server_metrics(server_id, &server_path).await {
                        let mut history = metrics_history.lock().await;
                        let server_metrics = history.entry(server_id.to_string()).or_insert_with(Vec::new);
                        server_metrics.push(metrics);
                        
                        // Keep only last 1000 metrics per server
                        if server_metrics.len() > 1000 {
                            server_metrics.drain(0..server_metrics.len() - 1000);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Collect performance metrics for a specific server
    async fn collect_server_metrics(server_id: &str, server_path: &Path) -> Result<PerformanceMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Try to read server logs for TPS and tick information
        let (tps, tick_ms) = Self::extract_tps_from_logs(server_path).await.unwrap_or((20.0, 50.0));

        // Get memory usage from server process
        let (memory_used, memory_max, memory_percent) = Self::get_memory_usage(server_id).await.unwrap_or((0, 0, 0.0));

        // Get IO statistics
        let (io_read, io_write) = Self::get_io_stats(server_id).await.unwrap_or((0, 0));

        // Get CPU usage
        let cpu_usage = Self::get_cpu_usage(server_id).await.unwrap_or(0.0);

        // Get disk usage
        let disk_usage = Self::get_disk_usage(server_path).await.unwrap_or(0);

        // Get network statistics
        let (network_in, network_out) = Self::get_network_stats(server_id).await.unwrap_or((0, 0));

        // Get server-specific metrics
        let (player_count, chunk_count, entity_count) = Self::get_server_specific_metrics(server_path).await.unwrap_or((0, 0, 0));

        Ok(PerformanceMetrics {
            server_id: server_id.to_string(),
            timestamp,
            tps,
            tick_ms,
            memory_used,
            memory_max,
            memory_percent,
            io_read,
            io_write,
            cpu_usage,
            disk_usage,
            network_in,
            network_out,
            player_count,
            chunk_count,
            entity_count,
        })
    }

    /// Extract TPS and tick time from server logs
    async fn extract_tps_from_logs(server_path: &Path) -> Result<(f32, f32), Box<dyn std::error::Error + Send + Sync>> {
        let logs_dir = server_path.join("logs");
        if !logs_dir.exists() {
            return Ok((20.0, 50.0)); // Default values
        }

        let mut entries = fs::read_dir(&logs_dir).await?;
        let mut latest_log = None;
        let mut latest_time = 0u64;

        // Find the latest log file
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("latest.log") || file_name.ends_with(".log") {
                        if let Ok(metadata) = entry.metadata().await {
                            if let Ok(modified) = metadata.modified() {
                                let modified_time = modified.duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                if modified_time > latest_time {
                                    latest_time = modified_time;
                                    latest_log = Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(log_path) = latest_log {
            let content = fs::read_to_string(&log_path).await?;
            return Self::parse_tps_from_log_content(&content);
        }

        Ok((20.0, 50.0)) // Default values
    }

    /// Parse TPS and tick time from log content
    fn parse_tps_from_log_content(content: &str) -> Result<(f32, f32), Box<dyn std::error::Error + Send + Sync>> {
        let mut tps = 20.0;
        let mut tick_ms = 50.0;

        // Look for TPS patterns in the log
        for line in content.lines().rev().take(100) { // Check last 100 lines
            if line.contains("TPS:") {
                if let Some(tps_str) = line.split("TPS:").nth(1) {
                    if let Some(tps_value) = tps_str.split_whitespace().next() {
                        if let Ok(parsed_tps) = tps_value.parse::<f32>() {
                            tps = parsed_tps;
                        }
                    }
                }
            }
            
            if line.contains("Tick:") && line.contains("ms") {
                if let Some(tick_str) = line.split("Tick:").nth(1) {
                    if let Some(tick_value) = tick_str.split_whitespace().next() {
                        if let Ok(parsed_tick) = tick_value.parse::<f32>() {
                            tick_ms = parsed_tick;
                        }
                    }
                }
            }
        }

        Ok((tps, tick_ms))
    }

    /// Get memory usage for a server process
    async fn get_memory_usage(server_id: &str) -> Result<(u64, u64, f32), Box<dyn std::error::Error + Send + Sync>> {
        // This would typically involve finding the server process and reading its memory usage
        // For now, return placeholder values
        Ok((1024 * 1024 * 1024, 2 * 1024 * 1024 * 1024, 50.0)) // 1GB used, 2GB max, 50% usage
    }

    /// Get IO statistics for a server process
    async fn get_io_stats(server_id: &str) -> Result<(u64, u64), Box<dyn std::error::Error + Send + Sync>> {
        // This would typically involve reading /proc/[pid]/io on Linux or similar on other systems
        // For now, return placeholder values
        Ok((1024 * 1024, 512 * 1024)) // 1MB read, 512KB write
    }

    /// Get CPU usage for a server process
    async fn get_cpu_usage(server_id: &str) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
        // This would typically involve monitoring the server process CPU usage
        // For now, return placeholder value
        Ok(25.0) // 25% CPU usage
    }

    /// Get disk usage for a server directory
    async fn get_disk_usage(server_path: &Path) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut total_size = 0u64;
        let mut dirs_to_process = vec![server_path.to_path_buf()];
        
        while let Some(current_dir) = dirs_to_process.pop() {
            if current_dir.exists() {
                let mut entries = fs::read_dir(&current_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = entry.metadata().await {
                            total_size += metadata.len();
                        }
                    } else if path.is_dir() {
                        dirs_to_process.push(path);
                    }
                }
            }
        }
        
        Ok(total_size)
    }

    /// Get network statistics for a server process
    async fn get_network_stats(server_id: &str) -> Result<(u64, u64), Box<dyn std::error::Error + Send + Sync>> {
        // This would typically involve reading network statistics for the server process
        // For now, return placeholder values
        Ok((1024 * 1024, 2 * 1024 * 1024)) // 1MB in, 2MB out
    }

    /// Get server-specific metrics (players, chunks, entities)
    async fn get_server_specific_metrics(server_path: &Path) -> Result<(u32, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
        // Try to read from server stats files or logs
        let stats_file = server_path.join("stats.json");
        if stats_file.exists() {
            if let Ok(content) = fs::read_to_string(&stats_file).await {
                if let Ok(stats) = serde_json::from_str::<serde_json::Value>(&content) {
                    let player_count = stats.get("players").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let chunk_count = stats.get("chunks").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let entity_count = stats.get("entities").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    return Ok((player_count, chunk_count, entity_count));
                }
            }
        }

        // Fallback to parsing from logs
        Ok((0, 0, 0)) // Placeholder values
    }

    /// Get performance metrics for a specific server
    pub async fn get_server_metrics(&self, server_id: &str) -> Option<Vec<PerformanceMetrics>> {
        let history = self.metrics_history.lock().await;
        history.get(server_id).cloned()
    }

    /// Get latest performance metrics for a specific server
    pub async fn get_latest_metrics(&self, server_id: &str) -> Option<PerformanceMetrics> {
        let history = self.metrics_history.lock().await;
        history.get(server_id)?.last().cloned()
    }

    /// Get performance metrics for all servers
    pub async fn get_all_metrics(&self) -> HashMap<String, Vec<PerformanceMetrics>> {
        let history = self.metrics_history.lock().await;
        history.clone()
    }

    /// Get performance summary for a server
    pub async fn get_performance_summary(&self, server_id: &str) -> Option<PerformanceSummary> {
        let history = self.metrics_history.lock().await;
        let metrics = history.get(server_id)?;
        
        if metrics.is_empty() {
            return None;
        }

        let avg_tps = metrics.iter().map(|m| m.tps).sum::<f32>() / metrics.len() as f32;
        let avg_tick_ms = metrics.iter().map(|m| m.tick_ms).sum::<f32>() / metrics.len() as f32;
        let avg_memory_percent = metrics.iter().map(|m| m.memory_percent).sum::<f32>() / metrics.len() as f32;
        let avg_cpu_usage = metrics.iter().map(|m| m.cpu_usage).sum::<f32>() / metrics.len() as f32;

        let latest = metrics.last().unwrap();
        
        Some(PerformanceSummary {
            server_id: server_id.to_string(),
            current_tps: latest.tps,
            average_tps: avg_tps,
            current_tick_ms: latest.tick_ms,
            average_tick_ms: avg_tick_ms,
            memory_usage_percent: latest.memory_percent,
            average_memory_percent: avg_memory_percent,
            cpu_usage_percent: latest.cpu_usage,
            average_cpu_usage_percent: avg_cpu_usage,
            player_count: latest.player_count,
            chunk_count: latest.chunk_count,
            entity_count: latest.entity_count,
            data_points: metrics.len(),
        })
    }
}

/// Performance summary for a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub server_id: String,
    pub current_tps: f32,
    pub average_tps: f32,
    pub current_tick_ms: f32,
    pub average_tick_ms: f32,
    pub memory_usage_percent: f32,
    pub average_memory_percent: f32,
    pub cpu_usage_percent: f32,
    pub average_cpu_usage_percent: f32,
    pub player_count: u32,
    pub chunk_count: u32,
    pub entity_count: u32,
    pub data_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_performance_telemetry() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = tempdir()?;
        let telemetry = PerformanceTelemetry::new(Duration::from_secs(1));
        
        // Test starting collection
        telemetry.start_collection(temp_dir.path()).await?;
        
        // Wait a bit for collection
        sleep(Duration::from_millis(100)).await;
        
        // Stop collection
        telemetry.stop_collection().await;
        
        Ok(())
    }

    #[test]
    fn test_parse_tps_from_log_content() {
        let log_content = r#"
[12:34:56] [Server thread/INFO]: TPS: 19.5
[12:34:57] [Server thread/INFO]: Tick: 45.2ms
[12:34:58] [Server thread/INFO]: TPS: 20.0
"#;
        
        let (tps, tick_ms) = PerformanceTelemetry::parse_tps_from_log_content(log_content).unwrap();
        assert_eq!(tps, 20.0);
        assert_eq!(tick_ms, 45.2);
    }
}
