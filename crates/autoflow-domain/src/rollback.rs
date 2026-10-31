use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuditStatus, JobMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RollbackItem {
    pub source_path: String,
    pub target_path: String,
    pub mode: JobMode,
    pub status: AuditStatus,
    #[serde(default)]
    pub target_hash: Option<String>,
    #[serde(default)]
    pub is_directory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RollbackManifest {
    pub id: Uuid,
    pub job_id: Uuid,
    #[serde(default)]
    pub blueprint_id: Option<Uuid>,
    pub execution_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub rolled_back_at: Option<DateTime<Utc>>,
    pub items: Vec<RollbackItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackResult {
    pub success: bool,
    pub reverted_count: u32,
    pub skipped_count: u32,
}

impl RollbackManifest {
    pub fn new(job_id: Uuid, execution_id: Uuid, items: Vec<RollbackItem>) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id,
            blueprint_id: None,
            execution_id,
            created_at: Utc::now(),
            rolled_back_at: None,
            items,
        }
    }

    pub fn new_for_blueprint(
        blueprint_id: Uuid,
        execution_id: Uuid,
        items: Vec<RollbackItem>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id: blueprint_id,
            blueprint_id: Some(blueprint_id),
            execution_id,
            created_at: Utc::now(),
            rolled_back_at: None,
            items,
        }
    }

    pub fn new_with_targets(
        job_id: Option<Uuid>,
        blueprint_id: Option<Uuid>,
        execution_id: Uuid,
        items: Vec<RollbackItem>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id: job_id.or(blueprint_id).unwrap_or_else(Uuid::nil),
            blueprint_id,
            execution_id,
            created_at: Utc::now(),
            rolled_back_at: None,
            items,
        }
    }
}

pub fn rollback_available(manifest: &RollbackManifest) -> bool {
    manifest.rolled_back_at.is_none() && !manifest.items.is_empty()
}
