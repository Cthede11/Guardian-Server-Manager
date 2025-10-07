use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Console message levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsoleLevel {
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl std::fmt::Display for ConsoleLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleLevel::Info => write!(f, "INFO"),
            ConsoleLevel::Warn => write!(f, "WARN"),
            ConsoleLevel::Error => write!(f, "ERROR"),
            ConsoleLevel::Debug => write!(f, "DEBUG"),
            ConsoleLevel::Trace => write!(f, "TRACE"),
        }
    }
}

/// Console message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub id: String,
    pub server_id: String,
    pub timestamp: DateTime<Utc>,
    pub level: ConsoleLevel,
    pub message: String,
    pub source: Option<String>, // e.g., "minecraft", "plugin", "system"
    pub tags: Vec<String>,
}

/// Console stream filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleFilter {
    pub levels: Option<Vec<ConsoleLevel>>,
    pub sources: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Console streamer for managing real-time console output
pub struct ConsoleStreamer {
    /// Server-specific console channels
    server_channels: Arc<RwLock<HashMap<String, broadcast::Sender<ConsoleMessage>>>>,
    /// Global console channel
    global_channel: broadcast::Sender<ConsoleMessage>,
    /// Message history for each server
    message_history: Arc<RwLock<HashMap<String, Vec<ConsoleMessage>>>>,
    /// Maximum history size per server
    max_history: usize,
}

impl ConsoleStreamer {
    pub fn new(max_history: usize) -> Self {
        let (global_channel, _) = broadcast::channel(1000);
        
        Self {
            server_channels: Arc::new(RwLock::new(HashMap::new())),
            global_channel,
            message_history: Arc::new(RwLock::new(HashMap::new())),
            max_history,
        }
    }

    /// Add a console message for a server
    pub async fn add_message(&self, server_id: &str, level: ConsoleLevel, message: String, source: Option<String>) {
        let console_message = ConsoleMessage {
            id: Uuid::new_v4().to_string(),
            server_id: server_id.to_string(),
            timestamp: Utc::now(),
            level,
            message: message.clone(),
            source,
            tags: vec![],
        };

        // Add to history
        {
            let mut history = self.message_history.write().await;
            let server_history = history.entry(server_id.to_string()).or_insert_with(Vec::new);
            server_history.push(console_message.clone());
            
            // Trim history if it exceeds max size
            if server_history.len() > self.max_history {
                server_history.drain(0..server_history.len() - self.max_history);
            }
        }

        // Broadcast to server-specific channel
        if let Some(server_tx) = self.get_server_channel(server_id).await {
            let _ = server_tx.send(console_message.clone());
        }

        // Broadcast to global channel
        let _ = self.global_channel.send(console_message);
    }

    /// Get or create server-specific channel
    async fn get_server_channel(&self, server_id: &str) -> Option<broadcast::Sender<ConsoleMessage>> {
        let mut channels = self.server_channels.write().await;
        
        if let Some(tx) = channels.get(server_id) {
            Some(tx.clone())
        } else {
            let (tx, _) = broadcast::channel(100);
            channels.insert(server_id.to_string(), tx.clone());
            Some(tx)
        }
    }

    /// Subscribe to console messages for a specific server
    pub async fn subscribe_to_server(&self, server_id: &str) -> broadcast::Receiver<ConsoleMessage> {
        let tx = self.get_server_channel(server_id).await.unwrap_or_else(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        tx.subscribe()
    }

    /// Subscribe to all console messages
    pub async fn subscribe_to_all(&self) -> broadcast::Receiver<ConsoleMessage> {
        self.global_channel.subscribe()
    }

    /// Get console message history for a server
    pub async fn get_history(&self, server_id: &str, filter: Option<ConsoleFilter>) -> Vec<ConsoleMessage> {
        let history = self.message_history.read().await;
        let mut messages = history.get(server_id).cloned()
            .unwrap_or_default();

        if let Some(filter) = filter {
            messages = self.apply_filter(messages, filter);
        }

        messages
    }

    /// Apply filter to console messages
    fn apply_filter(&self, mut messages: Vec<ConsoleMessage>, filter: ConsoleFilter) -> Vec<ConsoleMessage> {
        // Filter by levels
        if let Some(levels) = filter.levels {
            messages.retain(|msg| levels.contains(&msg.level));
        }

        // Filter by sources
        if let Some(sources) = filter.sources {
            messages.retain(|msg| {
                msg.source.as_ref()
                    .map(|s| sources.contains(s))
                    .unwrap_or(false)
            });
        }

        // Filter by tags
        if let Some(tags) = filter.tags {
            messages.retain(|msg| {
                tags.iter().any(|tag| msg.tags.contains(tag))
            });
        }

        // Filter by search term
        if let Some(search) = filter.search {
            let search_lower = search.to_lowercase();
            messages.retain(|msg| {
                msg.message.to_lowercase().contains(&search_lower)
            });
        }

        // Filter by timestamp
        if let Some(since) = filter.since {
            messages.retain(|msg| msg.timestamp >= since);
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            messages.truncate(limit);
        }

        messages
    }

    /// Clear history for a server
    pub async fn clear_history(&self, server_id: &str) {
        let mut history = self.message_history.write().await;
        history.remove(server_id);
    }

    /// Get server statistics
    pub async fn get_server_stats(&self, server_id: &str) -> ConsoleStats {
        let history = self.message_history.read().await;
        let messages = history.get(server_id).cloned().unwrap_or_default();

        let mut stats = ConsoleStats {
            total_messages: messages.len(),
            info_count: 0,
            warn_count: 0,
            error_count: 0,
            debug_count: 0,
            trace_count: 0,
            last_message: None,
        };

        for message in messages {
            match message.level {
                ConsoleLevel::Info => stats.info_count += 1,
                ConsoleLevel::Warn => stats.warn_count += 1,
                ConsoleLevel::Error => stats.error_count += 1,
                ConsoleLevel::Debug => stats.debug_count += 1,
                ConsoleLevel::Trace => stats.trace_count += 1,
            }

            if stats.last_message.is_none() || message.timestamp > stats.last_message.unwrap() {
                stats.last_message = Some(message.timestamp);
            }
        }

        stats
    }
}

/// Console statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleStats {
    pub total_messages: usize,
    pub info_count: usize,
    pub warn_count: usize,
    pub error_count: usize,
    pub debug_count: usize,
    pub trace_count: usize,
    pub last_message: Option<DateTime<Utc>>,
}

impl Default for ConsoleStreamer {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Console message parser for different server types
pub struct ConsoleParser;

impl ConsoleParser {
    /// Parse a raw console line and extract level, message, and source
    pub fn parse_line(line: &str) -> (ConsoleLevel, String, Option<String>) {
        // Common Minecraft server log patterns
        if line.contains("[ERROR]") || line.contains("ERROR:") {
            (ConsoleLevel::Error, line.to_string(), Some("minecraft".to_string()))
        } else if line.contains("[WARN]") || line.contains("WARN:") {
            (ConsoleLevel::Warn, line.to_string(), Some("minecraft".to_string()))
        } else if line.contains("[INFO]") || line.contains("INFO:") {
            (ConsoleLevel::Info, line.to_string(), Some("minecraft".to_string()))
        } else if line.contains("[DEBUG]") || line.contains("DEBUG:") {
            (ConsoleLevel::Debug, line.to_string(), Some("minecraft".to_string()))
        } else if line.contains("[TRACE]") || line.contains("TRACE:") {
            (ConsoleLevel::Trace, line.to_string(), Some("minecraft".to_string()))
        } else if line.starts_with("[") && line.contains("]") {
            // Generic bracketed format
            (ConsoleLevel::Info, line.to_string(), Some("system".to_string()))
        } else {
            // Default to info level
            (ConsoleLevel::Info, line.to_string(), None)
        }
    }

    /// Parse plugin-specific console messages
    pub fn parse_plugin_line(line: &str, plugin_name: &str) -> (ConsoleLevel, String, Option<String>) {
        let (level, message, _) = Self::parse_line(line);
        (level, message, Some(format!("plugin:{}", plugin_name)))
    }
}
