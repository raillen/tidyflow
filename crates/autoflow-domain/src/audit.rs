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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: i64,
    pub job_id: Option<Uuid>,
    pub job_name: String,
    pub source_path: String,
    pub target_path: String,
    pub status: AuditStatus,
    pub file_size: i64,
    pub duration_ms: f64,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAuditEntry {
    pub job_id: Uuid,
    pub job_name: String,
    pub source_path: String,
    pub target_path: String,
    pub status: AuditStatus,
    pub file_size: i64,
    pub duration_ms: f64,
    pub details: Option<String>,
}
