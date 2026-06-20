use autoflow_domain::DomainError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

pub struct SqliteUiStateStore {
    pool: SqlitePool,
}

impl SqliteUiStateStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Value>, DomainError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT payload FROM ui_state WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some((payload,)) => serde_json::from_str(&payload)
                .map(Some)
                .map_err(|e| DomainError::Database(e.to_string())),
            None => Ok(None),
        }
    }

    pub async fn save(&self, key: &str, payload: Value) -> Result<Value, DomainError> {
        let serialized = serde_json::to_string(&payload)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO ui_state (key, payload, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET payload = excluded.payload, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(serialized)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(payload)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissedScheduleEntry {
    pub id: i64,
    pub job_id: String,
    pub job_name: String,
    pub scheduled_at: String,
    pub detected_at: String,
}

pub struct SqliteMissedScheduleStore {
    pool: SqlitePool,
}

impl SqliteMissedScheduleStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        job_id: uuid::Uuid,
        job_name: &str,
        scheduled_at: chrono::DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO missed_schedules (job_id, job_name, scheduled_at, detected_at, acknowledged)
             SELECT ?, ?, ?, ?, 0
             WHERE NOT EXISTS (
                SELECT 1 FROM missed_schedules
                WHERE job_id = ? AND scheduled_at = ? AND acknowledged = 0
             )",
        )
        .bind(job_id.to_string())
        .bind(job_name)
        .bind(scheduled_at.to_rfc3339())
        .bind(&now)
        .bind(job_id.to_string())
        .bind(scheduled_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_unacknowledged(&self) -> Result<Vec<MissedScheduleEntry>, DomainError> {
        let rows: Vec<(i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, job_id, job_name, scheduled_at, detected_at
             FROM missed_schedules
             WHERE acknowledged = 0
             ORDER BY scheduled_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(id, job_id, job_name, scheduled_at, detected_at)| MissedScheduleEntry {
                id,
                job_id,
                job_name,
                scheduled_at,
                detected_at,
            })
            .collect())
    }

    pub async fn clear(&self) -> Result<(), DomainError> {
        sqlx::query("UPDATE missed_schedules SET acknowledged = 1 WHERE acknowledged = 0")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
