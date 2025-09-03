use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Compliance framework for GDPR, SOC2, and data privacy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceManager {
    pub policies: Arc<RwLock<HashMap<String, CompliancePolicy>>>,
    pub audits: Arc<RwLock<Vec<ComplianceAudit>>>,
    pub data_subjects: Arc<RwLock<HashMap<String, DataSubject>>>,
    pub consent_records: Arc<RwLock<Vec<ConsentRecord>>>,
    pub data_retention: Arc<RwLock<DataRetentionPolicy>>,
}

/// Compliance policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePolicy {
    pub id: String,
    pub name: String,
    pub framework: ComplianceFramework,
    pub version: String,
    pub description: String,
    pub rules: Vec<ComplianceRule>,
    pub effective_date: DateTime<Utc>,
    pub review_date: DateTime<Utc>,
    pub status: PolicyStatus,
}

/// Compliance frameworks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceFramework {
    GDPR,
    SOC2,
    CCPA,
    HIPAA,
    PCI_DSS,
    ISO27001,
    Custom(String),
}

/// Policy status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyStatus {
    Draft,
    Active,
    Suspended,
    Deprecated,
}

/// Compliance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub severity: RuleSeverity,
    pub enabled: bool,
}

/// Rule types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    DataRetention,
    DataAccess,
    DataDeletion,
    ConsentManagement,
    AuditLogging,
    Encryption,
    AccessControl,
    DataMinimization,
    PurposeLimitation,
}

/// Rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    NotContains,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

/// Rule actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    LogEvent,
    SendNotification,
    BlockAccess,
    EncryptData,
    DeleteData,
    AnonymizeData,
    RequireConsent,
    AuditTrail,
    AlertAdmin,
}

/// Rule severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Compliance audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAudit {
    pub id: String,
    pub audit_type: AuditType,
    pub framework: ComplianceFramework,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: AuditStatus,
    pub findings: Vec<AuditFinding>,
    pub recommendations: Vec<String>,
    pub auditor: String,
    pub scope: AuditScope,
}

/// Audit types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditType {
    Internal,
    External,
    SelfAssessment,
    Continuous,
}

/// Audit status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditStatus {
    Planned,
    InProgress,
    Completed,
    Failed,
}

/// Audit finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub category: String,
    pub evidence: Vec<String>,
    pub remediation: Option<String>,
    pub status: FindingStatus,
}

/// Finding status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FindingStatus {
    Open,
    InProgress,
    Resolved,
    Accepted,
}

/// Audit scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditScope {
    pub systems: Vec<String>,
    pub processes: Vec<String>,
    pub data_types: Vec<String>,
    pub time_period: TimePeriod,
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Data subject (GDPR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubject {
    pub id: String,
    pub identifier: String, // Email, username, etc.
    pub data_types: Vec<String>,
    pub consent_records: Vec<String>,
    pub data_retention_expiry: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: String,
    pub data_subject_id: String,
    pub purpose: String,
    pub data_types: Vec<String>,
    pub consent_given: bool,
    pub consent_date: DateTime<Utc>,
    pub withdrawal_date: Option<DateTime<Utc>>,
    pub legal_basis: LegalBasis,
    pub evidence: String, // IP address, user agent, etc.
}

/// Legal basis for processing (GDPR Article 6)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LegalBasis {
    Consent,
    Contract,
    LegalObligation,
    VitalInterests,
    PublicTask,
    LegitimateInterests,
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub rules: Vec<RetentionRule>,
    pub default_retention_days: u32,
    pub auto_deletion_enabled: bool,
    pub notification_days_before_deletion: u32,
}

/// Retention rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRule {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub retention_days: u32,
    pub conditions: Vec<RuleCondition>,
    pub legal_basis: Option<LegalBasis>,
}

impl ComplianceManager {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            audits: Arc::new(RwLock::new(Vec::new())),
            data_subjects: Arc::new(RwLock::new(HashMap::new())),
            consent_records: Arc::new(RwLock::new(Vec::new())),
            data_retention: Arc::new(RwLock::new(DataRetentionPolicy::default())),
        }
    }

    /// Initialize compliance manager with default policies
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing compliance manager...");
        
        // Create default GDPR policy
        self.create_default_gdpr_policy().await?;
        
        // Create default SOC2 policy
        self.create_default_soc2_policy().await?;
        
        // Initialize data retention policy
        self.initialize_data_retention().await?;
        
        info!("Compliance manager initialized");
        Ok(())
    }

    /// Create default GDPR policy
    async fn create_default_gdpr_policy(&self) -> Result<()> {
        let policy_id = "gdpr-default".to_string();
        let now = Utc::now();
        
        let policy = CompliancePolicy {
            id: policy_id.clone(),
            name: "GDPR Default Policy".to_string(),
            framework: ComplianceFramework::GDPR,
            version: "1.0".to_string(),
            description: "Default GDPR compliance policy for Guardian Platform".to_string(),
            rules: vec![
                // Data minimization rule
                ComplianceRule {
                    id: "gdpr-data-minimization".to_string(),
                    name: "Data Minimization".to_string(),
                    description: "Ensure only necessary data is collected and processed".to_string(),
                    rule_type: RuleType::DataMinimization,
                    conditions: vec![],
                    actions: vec![RuleAction {
                        action_type: ActionType::LogEvent,
                        parameters: HashMap::new(),
                    }],
                    severity: RuleSeverity::High,
                    enabled: true,
                },
                // Consent management rule
                ComplianceRule {
                    id: "gdpr-consent-management".to_string(),
                    name: "Consent Management".to_string(),
                    description: "Ensure proper consent is obtained before data processing".to_string(),
                    rule_type: RuleType::ConsentManagement,
                    conditions: vec![],
                    actions: vec![RuleAction {
                        action_type: ActionType::RequireConsent,
                        parameters: HashMap::new(),
                    }],
                    severity: RuleSeverity::Critical,
                    enabled: true,
                },
                // Right to erasure rule
                ComplianceRule {
                    id: "gdpr-right-to-erasure".to_string(),
                    name: "Right to Erasure".to_string(),
                    description: "Enable data subjects to request data deletion".to_string(),
                    rule_type: RuleType::DataDeletion,
                    conditions: vec![],
                    actions: vec![RuleAction {
                        action_type: ActionType::DeleteData,
                        parameters: HashMap::new(),
                    }],
                    severity: RuleSeverity::High,
                    enabled: true,
                },
            ],
            effective_date: now,
            review_date: now + chrono::Duration::days(365),
            status: PolicyStatus::Active,
        };

        let mut policies = self.policies.write().await;
        policies.insert(policy_id, policy);
        
        info!("Created default GDPR policy");
        Ok(())
    }

    /// Create default SOC2 policy
    async fn create_default_soc2_policy(&self) -> Result<()> {
        let policy_id = "soc2-default".to_string();
        let now = Utc::now();
        
        let policy = CompliancePolicy {
            id: policy_id.clone(),
            name: "SOC2 Default Policy".to_string(),
            framework: ComplianceFramework::SOC2,
            version: "1.0".to_string(),
            description: "Default SOC2 compliance policy for Guardian Platform".to_string(),
            rules: vec![
                // Access control rule
                ComplianceRule {
                    id: "soc2-access-control".to_string(),
                    name: "Access Control".to_string(),
                    description: "Implement proper access controls and authentication".to_string(),
                    rule_type: RuleType::AccessControl,
                    conditions: vec![],
                    actions: vec![RuleAction {
                        action_type: ActionType::BlockAccess,
                        parameters: HashMap::new(),
                    }],
                    severity: RuleSeverity::High,
                    enabled: true,
                },
                // Audit logging rule
                ComplianceRule {
                    id: "soc2-audit-logging".to_string(),
                    name: "Audit Logging".to_string(),
                    description: "Maintain comprehensive audit logs".to_string(),
                    rule_type: RuleType::AuditLogging,
                    conditions: vec![],
                    actions: vec![RuleAction {
                        action_type: ActionType::AuditTrail,
                        parameters: HashMap::new(),
                    }],
                    severity: RuleSeverity::High,
                    enabled: true,
                },
                // Encryption rule
                ComplianceRule {
                    id: "soc2-encryption".to_string(),
                    name: "Data Encryption".to_string(),
                    description: "Encrypt sensitive data at rest and in transit".to_string(),
                    rule_type: RuleType::Encryption,
                    conditions: vec![],
                    actions: vec![RuleAction {
                        action_type: ActionType::EncryptData,
                        parameters: HashMap::new(),
                    }],
                    severity: RuleSeverity::Critical,
                    enabled: true,
                },
            ],
            effective_date: now,
            review_date: now + chrono::Duration::days(365),
            status: PolicyStatus::Active,
        };

        let mut policies = self.policies.write().await;
        policies.insert(policy_id, policy);
        
        info!("Created default SOC2 policy");
        Ok(())
    }

    /// Initialize data retention policy
    async fn initialize_data_retention(&self) -> Result<()> {
        let retention_policy = DataRetentionPolicy {
            rules: vec![
                RetentionRule {
                    id: "user-data-retention".to_string(),
                    name: "User Data Retention".to_string(),
                    data_type: "user_data".to_string(),
                    retention_days: 2555, // 7 years
                    conditions: vec![],
                    legal_basis: Some(LegalBasis::LegitimateInterests),
                },
                RetentionRule {
                    id: "audit-log-retention".to_string(),
                    name: "Audit Log Retention".to_string(),
                    data_type: "audit_logs".to_string(),
                    retention_days: 2555, // 7 years
                    conditions: vec![],
                    legal_basis: Some(LegalBasis::LegalObligation),
                },
                RetentionRule {
                    id: "server-logs-retention".to_string(),
                    name: "Server Logs Retention".to_string(),
                    data_type: "server_logs".to_string(),
                    retention_days: 90,
                    conditions: vec![],
                    legal_basis: Some(LegalBasis::LegitimateInterests),
                },
            ],
            default_retention_days: 365,
            auto_deletion_enabled: true,
            notification_days_before_deletion: 30,
        };

        let mut data_retention = self.data_retention.write().await;
        *data_retention = retention_policy;
        
        info!("Initialized data retention policy");
        Ok(())
    }

    /// Create compliance audit
    pub async fn create_audit(&self, audit_type: AuditType, framework: ComplianceFramework, auditor: String) -> Result<ComplianceAudit> {
        let audit_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let audit = ComplianceAudit {
            id: audit_id.clone(),
            audit_type,
            framework,
            start_date: now,
            end_date: None,
            status: AuditStatus::Planned,
            findings: Vec::new(),
            recommendations: Vec::new(),
            auditor,
            scope: AuditScope {
                systems: vec!["guardian-platform".to_string()],
                processes: vec!["data-processing".to_string(), "access-control".to_string()],
                data_types: vec!["user-data".to_string(), "server-data".to_string()],
                time_period: TimePeriod {
                    start: now - chrono::Duration::days(90),
                    end: now,
                },
            },
        };

        let mut audits = self.audits.write().await;
        audits.push(audit.clone());
        
        info!("Created compliance audit: {}", audit_id);
        Ok(audit)
    }

    /// Add audit finding
    pub async fn add_audit_finding(&self, audit_id: &str, finding: AuditFinding) -> Result<()> {
        let mut audits = self.audits.write().await;
        if let Some(audit) = audits.iter_mut().find(|a| a.id == audit_id) {
            audit.findings.push(finding);
            info!("Added finding to audit: {}", audit_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Audit not found: {}", audit_id))
        }
    }

    /// Register data subject
    pub async fn register_data_subject(&self, identifier: String, data_types: Vec<String>) -> Result<DataSubject> {
        let subject_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let data_subject = DataSubject {
            id: subject_id.clone(),
            identifier: identifier.clone(),
            data_types: data_types.clone(),
            consent_records: Vec::new(),
            data_retention_expiry: None,
            created_at: now,
            updated_at: now,
        };

        let mut subjects = self.data_subjects.write().await;
        subjects.insert(subject_id.clone(), data_subject.clone());
        
        info!("Registered data subject: {}", identifier);
        Ok(data_subject)
    }

    /// Record consent
    pub async fn record_consent(&self, data_subject_id: String, purpose: String, data_types: Vec<String>, legal_basis: LegalBasis, evidence: String) -> Result<ConsentRecord> {
        let consent_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let consent = ConsentRecord {
            id: consent_id.clone(),
            data_subject_id: data_subject_id.clone(),
            purpose: purpose.clone(),
            data_types: data_types.clone(),
            consent_given: true,
            consent_date: now,
            withdrawal_date: None,
            legal_basis,
            evidence,
        };

        let mut consent_records = self.consent_records.write().await;
        consent_records.push(consent.clone());
        
        // Update data subject
        let mut subjects = self.data_subjects.write().await;
        if let Some(subject) = subjects.get_mut(&data_subject_id) {
            subject.consent_records.push(consent_id);
            subject.updated_at = now;
        }
        
        info!("Recorded consent for data subject: {}", data_subject_id);
        Ok(consent)
    }

    /// Withdraw consent
    pub async fn withdraw_consent(&self, consent_id: &str) -> Result<()> {
        let mut consent_records = self.consent_records.write().await;
        if let Some(consent) = consent_records.iter_mut().find(|c| c.id == consent_id) {
            consent.consent_given = false;
            consent.withdrawal_date = Some(Utc::now());
            
            info!("Withdrawn consent: {}", consent_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Consent record not found: {}", consent_id))
        }
    }

    /// Request data deletion (GDPR Article 17)
    pub async fn request_data_deletion(&self, data_subject_id: &str) -> Result<()> {
        // Check if deletion is legally required
        let consent_records = self.consent_records.read().await;
        let active_consents: Vec<_> = consent_records.iter()
            .filter(|c| c.data_subject_id == data_subject_id && c.consent_given)
            .collect();
        
        if !active_consents.is_empty() {
            return Err(anyhow::anyhow!("Cannot delete data - active consent records exist"));
        }
        
        // TODO: Implement actual data deletion
        // This would involve:
        // 1. Identifying all data related to the subject
        // 2. Anonymizing or deleting the data
        // 3. Updating audit logs
        // 4. Notifying relevant systems
        
        info!("Data deletion requested for subject: {}", data_subject_id);
        Ok(())
    }

    /// Get compliance report
    pub async fn get_compliance_report(&self, framework: ComplianceFramework) -> Result<ComplianceReport> {
        let policies = self.policies.read().await;
        let audits = self.audits.read().await;
        
        let framework_policies: Vec<_> = policies.values()
            .filter(|p| p.framework == framework)
            .cloned()
            .collect();
        
        let framework_audits: Vec<_> = audits.iter()
            .filter(|a| a.framework == framework)
            .cloned()
            .collect();
        
        let report = ComplianceReport {
            framework,
            generated_at: Utc::now(),
            policies: framework_policies,
            audits: framework_audits,
            compliance_score: self.calculate_compliance_score(&framework_policies, &framework_audits),
            recommendations: self.generate_recommendations(&framework_policies, &framework_audits),
        };
        
        Ok(report)
    }

    /// Calculate compliance score
    fn calculate_compliance_score(&self, policies: &[CompliancePolicy], audits: &[ComplianceAudit]) -> f64 {
        // Simple scoring algorithm - in production, this would be more sophisticated
        let total_rules: usize = policies.iter().map(|p| p.rules.len()).sum();
        let active_rules: usize = policies.iter()
            .flat_map(|p| &p.rules)
            .filter(|r| r.enabled)
            .count();
        
        if total_rules == 0 {
            return 0.0;
        }
        
        let rule_score = (active_rules as f64 / total_rules as f64) * 100.0;
        
        // Factor in audit findings
        let total_findings: usize = audits.iter().map(|a| a.findings.len()).sum();
        let critical_findings: usize = audits.iter()
            .flat_map(|a| &a.findings)
            .filter(|f| f.severity == RuleSeverity::Critical)
            .count();
        
        let audit_penalty = critical_findings as f64 * 10.0;
        
        (rule_score - audit_penalty).max(0.0).min(100.0)
    }

    /// Generate compliance recommendations
    fn generate_recommendations(&self, policies: &[CompliancePolicy], audits: &[ComplianceAudit]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Check for disabled rules
        for policy in policies {
            for rule in &policy.rules {
                if !rule.enabled {
                    recommendations.push(format!("Enable rule: {} in policy {}", rule.name, policy.name));
                }
            }
        }
        
        // Check for open audit findings
        for audit in audits {
            for finding in &audit.findings {
                if finding.status == FindingStatus::Open {
                    recommendations.push(format!("Address finding: {} in audit {}", finding.title, audit.id));
                }
            }
        }
        
        recommendations
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub generated_at: DateTime<Utc>,
    pub policies: Vec<CompliancePolicy>,
    pub audits: Vec<ComplianceAudit>,
    pub compliance_score: f64,
    pub recommendations: Vec<String>,
}

impl Default for DataRetentionPolicy {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_retention_days: 365,
            auto_deletion_enabled: false,
            notification_days_before_deletion: 30,
        }
    }
}
