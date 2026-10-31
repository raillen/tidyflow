use async_trait::async_trait;
use autoflow_application::ports::RollbackStore;
use autoflow_domain::{DomainError, RollbackManifest};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteRollbackStore {
    pool: SqlitePool,
}

impl SqliteRollbackStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RollbackStore for SqliteRollbackStore {
    async fn save(&self, manifest: &RollbackManifest) -> Result<(), DomainError> {
        let payload =
            serde_json::to_string(manifest).map_err(|e| DomainError::Database(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO rollback_manifests (id, job_id, execution_id, blueprint_id, payload, rolled_back_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(manifest.id.to_string())
        .bind(manifest.job_id.to_string())
        .bind(manifest.execution_id.to_string())
        .bind(manifest.blueprint_id.map(|value| value.to_string()))
        .bind(payload)
        .bind(manifest.rolled_back_at.map(|value| value.to_rfc3339()))
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    async fn latest_for_job(&self, job_id: Uuid) -> Result<Option<RollbackManifest>, DomainError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT payload FROM rollback_manifests WHERE (job_id = ? OR blueprint_id = ?) AND rolled_back_at IS NULL ORDER BY created_at DESC LIMIT 1",
        )
        .bind(job_id.to_string())
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some((payload,)) => {
                let manifest: RollbackManifest = serde_json::from_str(&payload)
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(Some(manifest))
            }
            None => Ok(None),
        }
    }

    async fn latest_for_blueprint(
        &self,
        blueprint_id: Uuid,
    ) -> Result<Option<RollbackManifest>, DomainError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT payload FROM rollback_manifests WHERE (blueprint_id = ? OR job_id = ?) AND rolled_back_at IS NULL ORDER BY created_at DESC LIMIT 1",
        )
        .bind(blueprint_id.to_string())
        .bind(blueprint_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some((payload,)) => {
                let manifest: RollbackManifest = serde_json::from_str(&payload)
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(Some(manifest))
            }
            None => Ok(None),
        }
    }

    async fn mark_rolled_back(&self, id: Uuid, at: DateTime<Utc>) -> Result<(), DomainError> {
        sqlx::query("UPDATE rollback_manifests SET rolled_back_at = ? WHERE id = ?")
            .bind(at.to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_pool;
    use autoflow_domain::{RollbackItem, RollbackManifest};
    use tempfile::TempDir;

    #[tokio::test]
    async fn saves_lists_and_marks_manifests() {
        let temp = TempDir::new().expect("tempdir");
        let pool = init_pool(temp.path()).await.expect("pool");
        let store = SqliteRollbackStore::new(pool);
        let job_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();
        let manifest = RollbackManifest::new(
            job_id,
            execution_id,
            vec![RollbackItem {
                source_path: "a.txt".into(),
                target_path: "b.txt".into(),
                mode: autoflow_domain::JobMode::Move,
                status: autoflow_domain::AuditStatus::Moved,
                target_hash: None,
                is_directory: false,
            }],
        );

        store.save(&manifest).await.expect("save");
        let latest = store.latest_for_job(job_id).await.expect("latest");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().id, manifest.id);

        store
            .mark_rolled_back(manifest.id, chrono::Utc::now())
            .await
            .expect("mark");
        let after = store.latest_for_job(job_id).await.expect("latest after");
        assert!(after.is_none());
    }
}
