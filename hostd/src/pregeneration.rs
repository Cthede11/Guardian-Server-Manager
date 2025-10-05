// Pregeneration module placeholder
// This will be implemented later

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenerationJob {
    pub id: Uuid,
    pub server_id: String,
    pub status: String,
    pub progress: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}