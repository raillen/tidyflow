use async_trait::async_trait;
use autoflow_domain::{Blueprint, BlueprintSummary, DomainError};
use uuid::Uuid;

#[async_trait]
pub trait BlueprintStore: Send + Sync {
    async fn list(&self) -> Result<Vec<BlueprintSummary>, DomainError>;
    async fn get(&self, id: Uuid) -> Result<Blueprint, DomainError>;
    async fn save(&self, blueprint: &Blueprint) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn authorized_roots(&self) -> Result<Vec<std::path::PathBuf>, DomainError>;
    async fn get_counter(&self, blueprint_id: Uuid, scope_key: &str) -> Result<u64, DomainError>;
    async fn set_counter(
        &self,
        blueprint_id: Uuid,
        scope_key: &str,
        value: u64,
    ) -> Result<(), DomainError>;
    async fn update_last_run(&self, id: Uuid) -> Result<(), DomainError>;
}
