use async_trait::async_trait;
use autoflow_domain::{DomainError, RollbackManifest};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait RollbackStore: Send + Sync {
    async fn save(&self, manifest: &RollbackManifest) -> Result<(), DomainError>;
    async fn latest_for_job(&self, job_id: Uuid) -> Result<Option<RollbackManifest>, DomainError>;
    async fn latest_for_blueprint(
        &self,
        _blueprint_id: Uuid,
    ) -> Result<Option<RollbackManifest>, DomainError> {
        Ok(None)
    }
    async fn mark_rolled_back(&self, id: Uuid, at: DateTime<Utc>) -> Result<(), DomainError>;
}
