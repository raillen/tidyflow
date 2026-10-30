use autoflow_domain::{AppSettings, AuditPage, AuditQuery, DomainError, Job, JobSummary};
use uuid::Uuid;

pub mod blueprint_store;
pub mod rollback_store;

pub use blueprint_store::BlueprintStore;
pub use rollback_store::RollbackStore;

pub trait SettingsStore: Send + Sync {
    fn get(&self) -> AppSettings;
    fn update(&self, settings: AppSettings) -> Result<(), DomainError>;
}

#[async_trait::async_trait]
pub trait JobStore: Send + Sync {
    async fn list(&self) -> Result<Vec<JobSummary>, DomainError>;
    async fn get(&self, id: Uuid) -> Result<Job, DomainError>;
    async fn save(&self, job: &Job) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn authorized_roots(&self) -> Result<Vec<std::path::PathBuf>, DomainError>;
}

#[async_trait::async_trait]
pub trait AuditStore: Send + Sync {
    async fn append(&self, entry: autoflow_domain::NewAuditEntry) -> Result<(), DomainError>;
    async fn list_recent(
        &self,
        limit: i64,
    ) -> Result<Vec<autoflow_domain::AuditEntry>, DomainError>;
    async fn query(&self, query: AuditQuery) -> Result<AuditPage, DomainError>;
    async fn clear(&self) -> Result<(), DomainError>;
    async fn purge_older_than(&self, days: u32) -> Result<u64, DomainError>;
}
