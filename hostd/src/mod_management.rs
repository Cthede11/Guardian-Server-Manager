use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::database::{DatabaseManager, Task, Mod};
use crate::websocket::WebSocketManager;
use crate::mod_manager::ModManager;

/// Mod search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub provider: String, // curseforge, modrinth
    pub project_id: String,
    pub version_id: String,
    pub download_url: String,
    pub file_size: u64,
    pub dependencies: Vec<ModDependency>,
    pub compatibility: ModCompatibility,
    pub downloads: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_range: String,
    pub required: bool,
    pub provider: String,
}

/// Mod compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCompatibility {
    pub mc_version: String,
    pub loader: String,
    pub compatible: bool,
    pub conflicts: Vec<String>,
    pub warnings: Vec<String>,
}

/// Mod installation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstallationPlan {
    pub id: String,
    pub server_id: String,
    pub operations: Vec<ModOperation>,
    pub conflicts: Vec<ModConflict>,
    pub warnings: Vec<String>,
    pub total_size: u64,
    pub created_at: DateTime<Utc>,
}

/// Mod operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModOperation {
    Install {
        mod_id: String,
        version: String,
        provider: String,
        file_path: PathBuf,
    },
    Update {
        mod_id: String,
        from_version: String,
        to_version: String,
        provider: String,
        file_path: PathBuf,
    },
    Remove {
        mod_id: String,
        file_path: PathBuf,
    },
    Enable {
        mod_id: String,
    },
    Disable {
        mod_id: String,
    },
}

/// Mod conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConflict {
    pub mod_id: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub resolution: Option<ConflictResolution>,
}

/// Conflict type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    VersionConflict,
    DependencyMissing,
    DependencyConflict,
    LoaderMismatch,
    DuplicateMod,
    ABIIncompatibility,
}

/// Conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UpdateMod { to_version: String },
    RemoveMod,
    AddDependency { mod_id: String, version: String },
    ReplaceMod { with_mod: String, version: String },
}

/// Mod management transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModTransaction {
    pub id: String,
    pub server_id: String,
    pub status: TransactionStatus,
    pub plan: ModInstallationPlan,
    pub progress: f64,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub rollback_data: Option<Vec<Mod>>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

/// Mod management manager
pub struct ModManagementManager {
    transactions: Arc<RwLock<HashMap<String, ModTransaction>>>,
    db: DatabaseManager,
    websocket_manager: Option<Arc<WebSocketManager>>,
    mod_cache: Arc<RwLock<HashMap<String, ModSearchResult>>>,
}

impl ModManagementManager {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            db,
            websocket_manager: None,
            mod_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the WebSocket manager for real-time updates
    pub fn set_websocket_manager(&mut self, websocket_manager: Arc<WebSocketManager>) {
        self.websocket_manager = Some(websocket_manager);
    }

    /// Search for mods
    pub async fn search_mods(
        &self,
        query: &str,
        provider: Option<&str>,
        mc_version: Option<&str>,
        loader: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<ModSearchResult>> {
        info!("Searching for mods: {}", query);

        // This would integrate with CurseForge and Modrinth APIs
        // For now, return mock results
        let mut results = Vec::new();

        // Mock search results
        for i in 0..(limit.unwrap_or(10) as usize) {
            results.push(ModSearchResult {
                id: format!("mod_{}", i),
                name: format!("{} Mod {}", query, i),
                description: format!("A mod that does something related to {}", query),
                version: "1.0.0".to_string(),
                author: "Mod Author".to_string(),
                provider: provider.unwrap_or("curseforge").to_string(),
                project_id: format!("project_{}", i),
                version_id: format!("version_{}", i),
                download_url: format!("https://example.com/mod_{}.jar", i),
                file_size: 1024 * 1024, // 1MB
                dependencies: Vec::new(),
                compatibility: ModCompatibility {
                    mc_version: mc_version.unwrap_or("1.20.1").to_string(),
                    loader: loader.unwrap_or("fabric").to_string(),
                    compatible: true,
                    conflicts: Vec::new(),
                    warnings: Vec::new(),
                },
                downloads: 1000 + i as u64,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(results)
    }

    /// Create a mod installation plan
    pub async fn create_installation_plan(
        &self,
        server_id: &str,
        mod_ids: Vec<String>,
        operations: Vec<ModOperation>,
    ) -> Result<String> {
        let plan_id = Uuid::new_v4().to_string();

        // Analyze conflicts and dependencies
        let conflicts = self.analyze_conflicts(&mod_ids, server_id).await?;
        let warnings = self.analyze_warnings(&mod_ids, server_id).await?;

        // Calculate total size
        let total_size = self.calculate_total_size(&operations).await?;

        let plan = ModInstallationPlan {
            id: plan_id.clone(),
            server_id: server_id.to_string(),
            operations,
            conflicts,
            warnings,
            total_size,
            created_at: Utc::now(),
        };

        // Store plan in memory
        {
            let mut transactions = self.transactions.write().await;
            let transaction = ModTransaction {
                id: plan_id.clone(),
                server_id: server_id.to_string(),
                status: TransactionStatus::Pending,
                plan: plan.clone(),
                progress: 0.0,
                created_at: Utc::now(),
                started_at: None,
                finished_at: None,
                error: None,
                rollback_data: None,
            };
            transactions.insert(plan_id.clone(), transaction);
        }

        info!("Created mod installation plan: {} for server: {}", plan_id, server_id);
        Ok(plan_id)
    }

    /// Apply a mod installation plan
    pub async fn apply_plan(&self, plan_id: &str) -> Result<()> {
        let mut transaction = {
            let mut transactions = self.transactions.write().await;
            transactions.get_mut(plan_id)
                .ok_or_else(|| anyhow!("Plan not found: {}", plan_id))?
                .clone()
        };

        if transaction.status != TransactionStatus::Pending {
            return Err(anyhow!("Plan cannot be applied in status: {:?}", transaction.status));
        }

        transaction.status = TransactionStatus::Running;
        transaction.started_at = Some(Utc::now());

        // Create rollback data
        transaction.rollback_data = Some(self.get_current_mods(&transaction.server_id).await?);

        // Update in memory
        {
            let mut transactions = self.transactions.write().await;
            transactions.insert(plan_id.to_string(), transaction.clone());
        }

        // Start the application process
        self.start_application_process(plan_id).await?;

        info!("Started applying mod installation plan: {}", plan_id);
        Ok(())
    }

    /// Rollback a mod installation plan
    pub async fn rollback_plan(&self, plan_id: &str) -> Result<()> {
        let mut transaction = {
            let mut transactions = self.transactions.write().await;
            transactions.get_mut(plan_id)
                .ok_or_else(|| anyhow!("Plan not found: {}", plan_id))?
                .clone()
        };

        if transaction.status != TransactionStatus::Running {
            return Err(anyhow!("Plan cannot be rolled back in status: {:?}", transaction.status));
        }

        // Restore previous mod state
        if let Some(rollback_data) = &transaction.rollback_data {
            self.restore_mods(&transaction.server_id, rollback_data).await?;
        }

        transaction.status = TransactionStatus::RolledBack;
        transaction.finished_at = Some(Utc::now());

        // Update in memory
        {
            let mut transactions = self.transactions.write().await;
            transactions.insert(plan_id.to_string(), transaction);
        }

        info!("Rolled back mod installation plan: {}", plan_id);
        Ok(())
    }

    /// Get a mod installation plan
    pub async fn get_plan(&self, plan_id: &str) -> Option<ModInstallationPlan> {
        let transactions = self.transactions.read().await;
        transactions.get(plan_id).map(|t| t.plan.clone())
    }

    /// Get all plans for a server
    pub async fn get_server_plans(&self, server_id: &str) -> Vec<ModInstallationPlan> {
        let transactions = self.transactions.read().await;
        transactions.values()
            .filter(|t| t.server_id == server_id)
            .map(|t| t.plan.clone())
            .collect()
    }

    /// Get current mods for a server
    async fn get_current_mods(&self, server_id: &str) -> Result<Vec<Mod>> {
        self.db.get_mods_by_server(server_id).await
    }

    /// Analyze conflicts between mods
    async fn analyze_conflicts(&self, mod_ids: &[String], server_id: &str) -> Result<Vec<ModConflict>> {
        let mut conflicts = Vec::new();

        // Get current mods
        let current_mods = self.get_current_mods(server_id).await?;
        let current_mod_ids: HashSet<String> = current_mods.iter().map(|m| m.id.clone()).collect();

        // Check for duplicates
        let mut seen_mods = HashSet::new();
        for mod_id in mod_ids {
            if !seen_mods.insert(mod_id) {
                conflicts.push(ModConflict {
                    mod_id: mod_id.clone(),
                    conflict_type: ConflictType::DuplicateMod,
                    description: format!("Mod {} is specified multiple times", mod_id),
                    resolution: Some(ConflictResolution::RemoveMod),
                });
            }
        }

        // Check for version conflicts
        for mod_id in mod_ids {
            if current_mod_ids.contains(mod_id) {
                conflicts.push(ModConflict {
                    mod_id: mod_id.clone(),
                    conflict_type: ConflictType::VersionConflict,
                    description: format!("Mod {} is already installed", mod_id),
                    resolution: Some(ConflictResolution::UpdateMod { to_version: "latest".to_string() }),
                });
            }
        }

        Ok(conflicts)
    }

    /// Analyze warnings for mods
    async fn analyze_warnings(&self, mod_ids: &[String], _server_id: &str) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for known problematic mods
        for mod_id in mod_ids {
            if mod_id.contains("optifine") {
                warnings.push(format!("Mod {} may cause compatibility issues with other mods", mod_id));
            }
        }

        Ok(warnings)
    }

    /// Calculate total size of operations
    async fn calculate_total_size(&self, operations: &[ModOperation]) -> Result<u64> {
        let mut total_size = 0u64;

        for operation in operations {
            match operation {
                ModOperation::Install { file_path, .. } |
                ModOperation::Update { file_path, .. } => {
                    if file_path.exists() {
                        total_size += std::fs::metadata(file_path)?.len();
                    }
                }
                _ => {}
            }
        }

        Ok(total_size)
    }

    /// Start the application process
    async fn start_application_process(&self, plan_id: &str) -> Result<()> {
        let transaction = {
            let transactions = self.transactions.read().await;
            transactions.get(plan_id).cloned()
                .ok_or_else(|| anyhow!("Plan not found: {}", plan_id))?
        };

        // Spawn application task
        let transactions = self.transactions.clone();
        let db = self.db.clone();
        let websocket_manager = self.websocket_manager.clone();
        let mod_manager = ModManager::new(PathBuf::from("downloads"));

        let plan_id_clone = plan_id.to_string();
        let transactions_clone = transactions.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_application_process(
                &plan_id_clone,
                &transaction,
                transactions,
                db,
                websocket_manager,
                mod_manager,
            ).await {
                error!("Application process failed for plan {}: {}", plan_id_clone, e);
                
                // Update transaction status to failed
                if let Some(mut transaction) = transactions_clone.write().await.get_mut(&plan_id_clone) {
                    transaction.status = TransactionStatus::Failed;
                    transaction.error = Some(e.to_string());
                    transaction.finished_at = Some(Utc::now());
                }
            }
        });

        Ok(())
    }

    /// Run the application process
    async fn run_application_process(
        plan_id: &str,
        transaction: &ModTransaction,
        transactions: Arc<RwLock<HashMap<String, ModTransaction>>>,
        db: DatabaseManager,
        websocket_manager: Option<Arc<WebSocketManager>>,
        mod_manager: ModManager,
    ) -> Result<()> {
        info!("Starting application process for plan: {}", plan_id);

        let operations = &transaction.plan.operations;
        let total_operations = operations.len();

        for (i, operation) in operations.iter().enumerate() {
            // Check if transaction was cancelled
            {
                let transactions = transactions.read().await;
                if let Some(t) = transactions.get(plan_id) {
                    if t.status == TransactionStatus::RolledBack {
                        info!("Plan {} was rolled back, stopping application", plan_id);
                        return Ok(());
                    }
                }
            }

            // Apply operation
            match operation {
                ModOperation::Install { mod_id, version, provider, file_path } => {
                    Self::apply_install_operation(&mod_manager, mod_id, version, provider, &file_path.to_string_lossy(), &transaction.server_id).await?;
                }
                ModOperation::Update { mod_id, from_version, to_version, provider, file_path } => {
                    Self::apply_update_operation(&mod_manager, mod_id, from_version, to_version, provider, &file_path.to_string_lossy(), &transaction.server_id).await?;
                }
                ModOperation::Remove { mod_id, file_path } => {
                    Self::apply_remove_operation(&mod_manager, mod_id, &file_path.to_string_lossy(), &transaction.server_id).await?;
                }
                ModOperation::Enable { mod_id } => {
                    Self::apply_enable_operation(&mod_manager, mod_id, &transaction.server_id).await?;
                }
                ModOperation::Disable { mod_id } => {
                    Self::apply_disable_operation(&mod_manager, mod_id, &transaction.server_id).await?;
                }
            }

            // Update progress
            let progress = (i as f64 + 1.0) / total_operations as f64;
            {
                let mut transactions = transactions.write().await;
                if let Some(t) = transactions.get_mut(plan_id) {
                    t.progress = progress;
                }
            }

            // Update database
            if let Err(e) = db.update_task(&Task {
                id: plan_id.to_string(),
                server_id: transaction.server_id.clone().into(),
                kind: "mod_management".to_string(),
                status: "running".to_string(),
                progress,
                log: Some(format!("Applied operation {}/{}", i + 1, total_operations)),
                metadata: Some(serde_json::to_value(operation)?),
                started_at: transaction.started_at,
                finished_at: None,
                created_at: transaction.created_at,
                updated_at: Utc::now(),
            }).await {
                error!("Failed to update task progress: {}", e);
            }

            // Send WebSocket update
            if let Some(ws_manager) = &websocket_manager {
                let task = Task {
                    id: plan_id.to_string(),
                    server_id: transaction.server_id.clone().into(),
                    kind: "mod_management".to_string(),
                    status: "running".to_string(),
                    progress,
                    log: Some(format!("Applied operation: {:?}", operation)),
                    metadata: Some(serde_json::to_value(operation)?),
                    started_at: transaction.started_at,
                    finished_at: None,
                    created_at: transaction.created_at,
                    updated_at: Utc::now(),
                };

                ws_manager.send_task_update(Some(&transaction.server_id), task).await;
            }
        }

        // Mark transaction as completed
        {
            let mut transactions = transactions.write().await;
            if let Some(t) = transactions.get_mut(plan_id) {
                t.status = TransactionStatus::Completed;
                t.progress = 1.0;
                t.finished_at = Some(Utc::now());
            }
        }

        info!("Completed application process for plan: {}", plan_id);
        Ok(())
    }

    /// Apply install operation
    async fn apply_install_operation(
        mod_manager: &ModManager,
        mod_id: &str,
        version: &str,
        provider: &str,
        file_path: &str,
        server_id: &str,
    ) -> Result<()> {
        // TODO: Implement actual install operation
        info!("Installed mod: {} version: {}", mod_id, version);
        Ok(())
    }

    /// Apply update operation
    async fn apply_update_operation(
        mod_manager: &ModManager,
        mod_id: &str,
        from_version: &str,
        to_version: &str,
        provider: &str,
        file_path: &str,
        server_id: &str,
    ) -> Result<()> {
        // TODO: Implement actual update operation
        info!("Updated mod: {} from {} to {}", mod_id, from_version, to_version);
        Ok(())
    }

    /// Apply remove operation
    async fn apply_remove_operation(
        mod_manager: &ModManager,
        mod_id: &str,
        file_path: &str,
        server_id: &str,
    ) -> Result<()> {
        // TODO: Implement actual remove operation
        info!("Removed mod: {}", mod_id);
        Ok(())
    }

    /// Apply enable operation
    async fn apply_enable_operation(mod_manager: &ModManager, mod_id: &str, server_id: &str) -> Result<()> {
        // TODO: Implement actual enable operation
        info!("Enabled mod: {}", mod_id);
        Ok(())
    }

    /// Apply disable operation
    async fn apply_disable_operation(mod_manager: &ModManager, mod_id: &str, server_id: &str) -> Result<()> {
        // TODO: Implement actual disable operation
        info!("Disabled mod: {}", mod_id);
        Ok(())
    }

    /// Restore mods to previous state
    async fn restore_mods(&self, server_id: &str, rollback_data: &[Mod]) -> Result<()> {
        // Remove all current mods
        let current_mods = self.get_current_mods(server_id).await?;
        for mod_record in current_mods {
            self.db.delete_mod(&mod_record.id).await?;
        }

        // Restore previous mods
        for mod_record in rollback_data {
            self.db.create_mod(mod_record).await?;
        }

        info!("Restored mods for server: {}", server_id);
        Ok(())
    }
}
