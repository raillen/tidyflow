use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditStatus {
    Copied,
    Moved,
    Ignored,
    Failed,
    Organized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: i64,
    pub job_id: Option<Uuid>,
    pub blueprint_id: Option<Uuid>,
    pub job_name: String,
    pub source_path: String,
    pub target_path: String,
    pub status: AuditStatus,
    pub file_size: i64,
    pub duration_ms: f64,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuditQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<AuditStatus>,
    #[serde(default)]
    pub job_id: Option<Uuid>,
    #[serde(default)]
    pub blueprint_id: Option<Uuid>,
    #[serde(default)]
    pub date_from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub date_to: Option<DateTime<Utc>>,
    #[serde(default = "default_audit_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_audit_limit() -> i64 {
    100
}

impl AuditQuery {
    pub fn normalized(&self) -> Self {
        let mut query = self.clone();
        query.search = query
            .search
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        query.limit = query.limit.clamp(1, 1_000);
        query.offset = query.offset.max(0);
        query
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditExportFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditExport {
    pub file_name: String,
    pub mime_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummary {
    pub total: i64,
    pub copied: i64,
    pub moved: i64,
    pub ignored: i64,
    pub failed: i64,
    pub organized: i64,
    pub total_bytes: i64,
    pub average_duration_ms: f64,
    pub latest_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPage {
    pub entries: Vec<AuditEntry>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub summary: AuditSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAuditEntry {
    pub job_id: Option<Uuid>,
    pub blueprint_id: Option<Uuid>,
    pub job_name: String,
    pub source_path: String,
    pub target_path: String,
    pub status: AuditStatus,
    pub file_size: i64,
    pub duration_ms: f64,
    pub details: Option<String>,
}
