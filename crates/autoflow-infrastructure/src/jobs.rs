use autoflow_application::ports::JobStore;
use autoflow_domain::{DomainError, Job, JobSummary};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteJobStore {
    pool: SqlitePool,
}

impl SqliteJobStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn deserialize(payload: &str) -> Result<Job, DomainError> {
        let mut job: Job = serde_json::from_str(payload)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        job.normalize();
        Ok(job)
    }
}

#[async_trait::async_trait]
impl JobStore for SqliteJobStore {
    async fn list(&self) -> Result<Vec<JobSummary>, DomainError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT payload FROM jobs ORDER BY updated_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|(payload,)| Self::deserialize(&payload).ok())
            .map(|job| job.summary())
            .collect())
    }

    async fn get(&self, id: Uuid) -> Result<Job, DomainError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT payload FROM jobs WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some((payload,)) => Self::deserialize(&payload),
            None => Err(DomainError::JobNotFound),
        }
    }

    async fn save(&self, job: &Job) -> Result<(), DomainError> {
        job.validate()?;
        let payload = serde_json::to_string(job)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO jobs (id, payload, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at",
        )
        .bind(job.id.to_string())
        .bind(payload)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::JobNotFound);
        }
        Ok(())
    }

    async fn authorized_roots(&self) -> Result<Vec<std::path::PathBuf>, DomainError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT payload FROM jobs")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut roots = Vec::new();
        for (payload,) in rows {
            if let Ok(job) = Self::deserialize(&payload) {
                roots.push(std::path::PathBuf::from(&job.source_path));
                roots.push(std::path::PathBuf::from(&job.target_path));
            }
        }
        roots.sort();
        roots.dedup_by(|a, b| a == b);
        Ok(roots)
    }
}
