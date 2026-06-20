use autoflow_domain::{AppSettings, DomainError, Job, JobSummary};
use uuid::Uuid;

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
    async fn list_recent(&self, limit: i64) -> Result<Vec<autoflow_domain::AuditEntry>, DomainError>;
}
