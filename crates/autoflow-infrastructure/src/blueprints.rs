use autoflow_application::ports::BlueprintStore;
use autoflow_domain::{Blueprint, BlueprintSummary, DomainError};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteBlueprintStore {
    pool: SqlitePool,
}

impl SqliteBlueprintStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn deserialize(payload: &str) -> Result<Blueprint, DomainError> {
        let mut blueprint: Blueprint = serde_json::from_str(payload)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        blueprint.normalize();
        Ok(blueprint)
    }

    fn kind_str(kind: autoflow_domain::BlueprintKind) -> &'static str {
        match kind {
            autoflow_domain::BlueprintKind::File => "file",
            autoflow_domain::BlueprintKind::Folder => "folder",
        }
    }
}

#[async_trait::async_trait]
impl BlueprintStore for SqliteBlueprintStore {
    async fn list(&self) -> Result<Vec<BlueprintSummary>, DomainError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT payload FROM blueprints ORDER BY updated_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(payload,)| Self::deserialize(&payload).ok())
            .map(|bp| bp.summary())
            .collect())
    }

    async fn get(&self, id: Uuid) -> Result<Blueprint, DomainError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT payload FROM blueprints WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some((payload,)) => Self::deserialize(&payload),
            None => Err(DomainError::BlueprintNotFound),
        }
    }

    async fn save(&self, blueprint: &Blueprint) -> Result<(), DomainError> {
        blueprint.validate()?;
        let payload = serde_json::to_string(blueprint)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO blueprints (id, kind, payload, enabled, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               kind = excluded.kind,
               payload = excluded.payload,
               enabled = excluded.enabled,
               updated_at = excluded.updated_at",
        )
        .bind(blueprint.id.to_string())
        .bind(Self::kind_str(blueprint.kind))
        .bind(payload)
        .bind(if blueprint.enabled { 1 } else { 0 })
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM blueprints WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::BlueprintNotFound);
        }

        sqlx::query("DELETE FROM blueprint_counters WHERE blueprint_id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn authorized_roots(&self) -> Result<Vec<std::path::PathBuf>, DomainError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT payload FROM blueprints")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut roots = Vec::new();
        for (payload,) in rows {
            if let Ok(bp) = Self::deserialize(&payload) {
                roots.push(std::path::PathBuf::from(&bp.root_path));
            }
        }
        roots.sort();
        roots.dedup_by(|a, b| a == b);
        Ok(roots)
    }

    async fn get_counter(&self, blueprint_id: Uuid, scope_key: &str) -> Result<u64, DomainError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT value FROM blueprint_counters WHERE blueprint_id = ? AND scope_key = ?",
        )
        .bind(blueprint_id.to_string())
        .bind(scope_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|(v,)| v as u64).unwrap_or(0))
    }

    async fn set_counter(
        &self,
        blueprint_id: Uuid,
        scope_key: &str,
        value: u64,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO blueprint_counters (blueprint_id, scope_key, value)
             VALUES (?, ?, ?)
             ON CONFLICT(blueprint_id, scope_key) DO UPDATE SET value = excluded.value",
        )
        .bind(blueprint_id.to_string())
        .bind(scope_key)
        .bind(value as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    async fn update_last_run(&self, id: Uuid) -> Result<(), DomainError> {
        let mut blueprint = self.get(id).await?;
        blueprint.last_run = Some(Utc::now());
        self.save(&blueprint).await
    }
}
