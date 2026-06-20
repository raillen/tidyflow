use autoflow_application::ports::AuditStore;
use autoflow_domain::{AuditEntry, AuditStatus, DomainError, NewAuditEntry};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteAuditStore {
    pool: SqlitePool,
}

impl SqliteAuditStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AuditStore for SqliteAuditStore {
    async fn append(&self, entry: NewAuditEntry) -> Result<(), DomainError> {
        let status = match entry.status {
            AuditStatus::Copied => "COPIED",
            AuditStatus::Moved => "MOVED",
            AuditStatus::Ignored => "IGNORED",
            AuditStatus::Failed => "FAILED",
            AuditStatus::Organized => "ORGANIZED",
        };

        sqlx::query(
            "INSERT INTO audit_entries (job_id, blueprint_id, job_name, source_path, target_path, status, file_size, duration_ms, details, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entry.job_id.map(|id| id.to_string()))
        .bind(entry.blueprint_id.map(|id| id.to_string()))
        .bind(entry.job_name)
        .bind(entry.source_path)
        .bind(entry.target_path)
        .bind(status)
        .bind(entry.file_size)
        .bind(entry.duration_ms)
        .bind(entry.details)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    async fn list_recent(&self, limit: i64) -> Result<Vec<AuditEntry>, DomainError> {
        let rows: Vec<(
            i64,
            Option<String>,
            Option<String>,
            String,
            String,
            String,
            String,
            i64,
            f64,
            Option<String>,
            String,
        )> = sqlx::query_as(
            "SELECT id, job_id, blueprint_id, job_name, source_path, target_path, status, file_size, duration_ms, details, created_at
             FROM audit_entries ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                Ok(AuditEntry {
                    id: row.0,
                    job_id: row.1.and_then(|id| Uuid::parse_str(&id).ok()),
                    blueprint_id: row.2.and_then(|id| Uuid::parse_str(&id).ok()),
                    job_name: row.3,
                    source_path: row.4,
                    target_path: row.5,
                    status: parse_status(&row.6)?,
                    file_size: row.7,
                    duration_ms: row.8,
                    details: row.9,
                    created_at: DateTime::parse_from_rfc3339(&row.10)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })
            .collect()
    }
}

fn parse_status(raw: &str) -> Result<AuditStatus, DomainError> {
    match raw {
        "COPIED" => Ok(AuditStatus::Copied),
        "MOVED" => Ok(AuditStatus::Moved),
        "IGNORED" => Ok(AuditStatus::Ignored),
        "FAILED" => Ok(AuditStatus::Failed),
        "ORGANIZED" => Ok(AuditStatus::Organized),
        other => Err(DomainError::Database(format!("unknown audit status: {other}"))),
    }
}
