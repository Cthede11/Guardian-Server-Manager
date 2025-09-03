use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use reqwest::Client;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub headers: HashMap<String, String>,
    pub retry_config: RetryConfig,
    pub status: WebhookStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Retry configuration for failed webhook deliveries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

/// Webhook status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebhookStatus {
    Active,
    Inactive,
    Suspended,
    Error,
}

/// Webhook delivery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event_id: String,
    pub attempt: u32,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
    Expired,
}

/// Webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

/// Webhook manager for external integrations
pub struct WebhookManager {
    webhooks: Arc<RwLock<HashMap<String, Webhook>>>,
    deliveries: Arc<RwLock<HashMap<String, Vec<WebhookDelivery>>>>,
    events: Arc<RwLock<Vec<WebhookEvent>>>,
    client: Client,
    delivery_queue: Arc<RwLock<Vec<DeliveryTask>>>,
}

/// Delivery task for async processing
#[derive(Debug, Clone)]
pub struct DeliveryTask {
    pub webhook_id: String,
    pub event: WebhookEvent,
    pub attempt: u32,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            deliveries: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            client: Client::new(),
            delivery_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize webhook manager
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing webhook manager...");
        
        // Start delivery worker
        self.start_delivery_worker().await;
        
        // Start retry worker
        self.start_retry_worker().await;
        
        info!("Webhook manager initialized");
        Ok(())
    }

    /// Create webhook
    pub async fn create_webhook(&self, name: String, url: String, events: Vec<String>) -> Result<Webhook> {
        let webhook_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let webhook = Webhook {
            id: webhook_id.clone(),
            name: name.clone(),
            url: url.clone(),
            events: events.clone(),
            secret: None,
            headers: HashMap::new(),
            retry_config: RetryConfig::default(),
            status: WebhookStatus::Active,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        };

        let mut webhooks = self.webhooks.write().await;
        webhooks.insert(webhook_id.clone(), webhook.clone());
        
        info!("Created webhook: {} -> {}", name, url);
        Ok(webhook)
    }

    /// Get webhook by ID
    pub async fn get_webhook(&self, webhook_id: &str) -> Option<Webhook> {
        let webhooks = self.webhooks.read().await;
        webhooks.get(webhook_id).cloned()
    }

    /// List webhooks
    pub async fn list_webhooks(&self) -> Vec<Webhook> {
        let webhooks = self.webhooks.read().await;
        webhooks.values().cloned().collect()
    }

    /// Update webhook
    pub async fn update_webhook(&self, webhook_id: &str, updates: WebhookUpdate) -> Result<()> {
        let mut webhooks = self.webhooks.write().await;
        if let Some(webhook) = webhooks.get_mut(webhook_id) {
            if let Some(name) = updates.name {
                webhook.name = name;
            }
            if let Some(url) = updates.url {
                webhook.url = url;
            }
            if let Some(events) = updates.events {
                webhook.events = events;
            }
            if let Some(secret) = updates.secret {
                webhook.secret = Some(secret);
            }
            if let Some(headers) = updates.headers {
                webhook.headers = headers;
            }
            if let Some(retry_config) = updates.retry_config {
                webhook.retry_config = retry_config;
            }
            if let Some(status) = updates.status {
                webhook.status = status;
            }
            webhook.updated_at = Utc::now();
            
            info!("Updated webhook: {}", webhook_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Webhook not found: {}", webhook_id))
        }
    }

    /// Delete webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let mut webhooks = self.webhooks.write().await;
        webhooks.remove(webhook_id);
        
        let mut deliveries = self.deliveries.write().await;
        deliveries.remove(webhook_id);
        
        info!("Deleted webhook: {}", webhook_id);
        Ok(())
    }

    /// Publish event to webhooks
    pub async fn publish_event(&self, event_type: &str, data: serde_json::Value, source: &str) -> Result<()> {
        let event = WebhookEvent {
            id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            data: data.clone(),
            timestamp: Utc::now(),
            source: source.to_string(),
            metadata: HashMap::new(),
        };

        // Store event
        {
            let mut events = self.events.write().await;
            events.push(event.clone());
            
            // Keep only recent events (last 10000)
            if events.len() > 10000 {
                events.drain(0..events.len() - 10000);
            }
        }

        // Find webhooks that subscribe to this event
        let webhooks = self.webhooks.read().await;
        for webhook in webhooks.values() {
            if webhook.status == WebhookStatus::Active && 
               webhook.events.contains(&event_type.to_string()) {
                
                // Queue delivery
                let task = DeliveryTask {
                    webhook_id: webhook.id.clone(),
                    event: event.clone(),
                    attempt: 1,
                };
                
                let mut queue = self.delivery_queue.write().await;
                queue.push(task);
            }
        }

        info!("Published event: {} to {} webhooks", event_type, 
              webhooks.values().filter(|w| w.events.contains(&event_type.to_string())).count());
        Ok(())
    }

    /// Start delivery worker
    async fn start_delivery_worker(&self) {
        let webhooks = self.webhooks.clone();
        let deliveries = self.deliveries.clone();
        let client = self.client.clone();
        let queue = self.delivery_queue.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // Process delivery queue
                let tasks = {
                    let mut queue_guard = queue.write().await;
                    let tasks = queue_guard.drain(..).collect::<Vec<_>>();
                    tasks
                };
                
                for task in tasks {
                    Self::deliver_webhook(&webhooks, &deliveries, &client, task).await;
                }
            }
        });
    }

    /// Start retry worker
    async fn start_retry_worker(&self) {
        let deliveries = self.deliveries.clone();
        let queue = self.delivery_queue.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Check for retries
                let now = Utc::now();
                let mut retry_tasks = Vec::new();
                
                {
                    let deliveries_guard = deliveries.read().await;
                    for (webhook_id, webhook_deliveries) in deliveries_guard.iter() {
                        for delivery in webhook_deliveries {
                            if delivery.status == DeliveryStatus::Retrying &&
                               delivery.next_retry_at.map_or(false, |t| t <= now) {
                                retry_tasks.push(DeliveryTask {
                                    webhook_id: webhook_id.clone(),
                                    event: WebhookEvent {
                                        id: delivery.event_id.clone(),
                                        event_type: "retry".to_string(),
                                        data: serde_json::Value::Null,
                                        timestamp: now,
                                        source: "retry_worker".to_string(),
                                        metadata: HashMap::new(),
                                    },
                                    attempt: delivery.attempt + 1,
                                });
                            }
                        }
                    }
                }
                
                // Queue retry tasks
                if !retry_tasks.is_empty() {
                    let mut queue_guard = queue.write().await;
                    queue_guard.extend(retry_tasks);
                }
            }
        });
    }

    /// Deliver webhook
    async fn deliver_webhook(
        webhooks: &Arc<RwLock<HashMap<String, Webhook>>>,
        deliveries: &Arc<RwLock<HashMap<String, Vec<WebhookDelivery>>>>,
        client: &Client,
        task: DeliveryTask,
    ) {
        let webhook = {
            let webhooks_guard = webhooks.read().await;
            webhooks_guard.get(&task.webhook_id).cloned()
        };

        let webhook = match webhook {
            Some(w) => w,
            None => {
                error!("Webhook not found: {}", task.webhook_id);
                return;
            }
        };

        // Create delivery record
        let delivery_id = Uuid::new_v4().to_string();
        let mut delivery = WebhookDelivery {
            id: delivery_id.clone(),
            webhook_id: task.webhook_id.clone(),
            event_id: task.event.id.clone(),
            attempt: task.attempt,
            status: DeliveryStatus::Pending,
            response_code: None,
            response_body: None,
            error_message: None,
            delivered_at: None,
            next_retry_at: None,
        };

        // Prepare request
        let mut request = client.post(&webhook.url);
        
        // Add headers
        for (key, value) in &webhook.headers {
            request = request.header(key, value);
        }
        
        // Add default headers
        request = request
            .header("Content-Type", "application/json")
            .header("User-Agent", "Guardian-Webhook/1.0")
            .header("X-Webhook-Event", &task.event.event_type)
            .header("X-Webhook-ID", &task.event.id)
            .header("X-Webhook-Timestamp", task.event.timestamp.to_rfc3339());

        // Add signature if secret is configured
        if let Some(secret) = &webhook.secret {
            let payload = serde_json::to_string(&task.event).unwrap_or_default();
            let signature = Self::generate_signature(&payload, secret);
            request = request.header("X-Webhook-Signature", signature);
        }

        // Set request body
        request = request.json(&task.event);

        // Send request
        let start_time = std::time::Instant::now();
        let result = request.send().await;
        let duration = start_time.elapsed();

        // Process response
        match result {
            Ok(response) => {
                let status = response.status();
                let response_text = response.text().await.unwrap_or_default();
                
                delivery.status = if status.is_success() {
                    DeliveryStatus::Delivered
                } else {
                    DeliveryStatus::Failed
                };
                delivery.response_code = Some(status.as_u16());
                delivery.response_body = Some(response_text);
                delivery.delivered_at = Some(Utc::now());
                
                if !status.is_success() && task.attempt < webhook.retry_config.max_attempts {
                    delivery.status = DeliveryStatus::Retrying;
                    delivery.next_retry_at = Some(Utc::now() + 
                        chrono::Duration::milliseconds(Self::calculate_retry_delay(
                            task.attempt,
                            &webhook.retry_config
                        )));
                }
                
                info!("Webhook delivered: {} ({}ms, {})", 
                      webhook.name, duration.as_millis(), status);
            }
            Err(e) => {
                delivery.status = if task.attempt < webhook.retry_config.max_attempts {
                    DeliveryStatus::Retrying
                } else {
                    DeliveryStatus::Failed
                };
                delivery.error_message = Some(e.to_string());
                
                if delivery.status == DeliveryStatus::Retrying {
                    delivery.next_retry_at = Some(Utc::now() + 
                        chrono::Duration::milliseconds(Self::calculate_retry_delay(
                            task.attempt,
                            &webhook.retry_config
                        )));
                }
                
                error!("Webhook delivery failed: {} - {}", webhook.name, e);
            }
        }

        // Store delivery record
        {
            let mut deliveries_guard = deliveries.write().await;
            let webhook_deliveries = deliveries_guard.entry(task.webhook_id).or_insert_with(Vec::new);
            webhook_deliveries.push(delivery);
            
            // Keep only recent deliveries (last 1000 per webhook)
            if webhook_deliveries.len() > 1000 {
                webhook_deliveries.drain(0..webhook_deliveries.len() - 1000);
            }
        }
    }

    /// Generate webhook signature
    fn generate_signature(payload: &str, secret: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(attempt: u32, config: &RetryConfig) -> i64 {
        let delay = (config.initial_delay_ms as f64 * 
                    config.backoff_multiplier.powi(attempt as i32 - 1)) as i64;
        delay.min(config.max_delay_ms as i64)
    }

    /// Get webhook deliveries
    pub async fn get_webhook_deliveries(&self, webhook_id: &str, limit: usize) -> Vec<WebhookDelivery> {
        let deliveries = self.deliveries.read().await;
        if let Some(webhook_deliveries) = deliveries.get(webhook_id) {
            webhook_deliveries.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get webhook events
    pub async fn get_webhook_events(&self, limit: usize) -> Vec<WebhookEvent> {
        let events = self.events.read().await;
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Test webhook
    pub async fn test_webhook(&self, webhook_id: &str) -> Result<()> {
        let test_event = WebhookEvent {
            id: Uuid::new_v4().to_string(),
            event_type: "test".to_string(),
            data: serde_json::json!({
                "message": "This is a test webhook from Guardian",
                "timestamp": Utc::now().to_rfc3339(),
            }),
            timestamp: Utc::now(),
            source: "webhook_test".to_string(),
            metadata: HashMap::new(),
        };

        self.publish_event("test", test_event.data, "webhook_test").await?;
        info!("Test webhook sent: {}", webhook_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WebhookUpdate {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub secret: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_config: Option<RetryConfig>,
    pub status: Option<WebhookStatus>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}
