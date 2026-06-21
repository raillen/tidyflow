use autoflow_domain::{
    AdminCommandQueueSummary, AdminCommandRequest, AdminCommandResult, AdminQueuedCommand,
    AdminQueuedCommandStatus, DomainError,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteAdminCommandStore {
    pool: SqlitePool,
}

impl SqliteAdminCommandStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(
        &self,
        source: &str,
        request: AdminCommandRequest,
    ) -> Result<AdminQueuedCommand, DomainError> {
        let command = AdminQueuedCommand {
            id: Uuid::new_v4(),
            source: source.into(),
            request,
            status: AdminQueuedCommandStatus::Pending,
            result: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.save(&command).await?;
        Ok(command)
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<AdminQueuedCommand>, DomainError> {
        let rows: Vec<AdminCommandRow> = sqlx::query_as(
            "SELECT id, source, request_payload, status, result_payload, created_at, updated_at
             FROM admin_commands
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(|error| DomainError::Database(error.to_string()))?;

        rows.into_iter().map(AdminQueuedCommand::try_from).collect()
    }

    pub async fn summary(&self) -> Result<AdminCommandQueueSummary, DomainError> {
        let commands = self.list_recent(500).await?;
        Ok(AdminCommandQueueSummary::from_commands(&commands))
    }

    pub async fn next_pending(&self) -> Result<Option<AdminQueuedCommand>, DomainError> {
        let row: Option<AdminCommandRow> = sqlx::query_as(
            "SELECT id, source, request_payload, status, result_payload, created_at, updated_at
             FROM admin_commands
             WHERE status = 'pending'
             ORDER BY created_at ASC
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| DomainError::Database(error.to_string()))?;

        row.map(AdminQueuedCommand::try_from).transpose()
    }

    pub async fn mark_running(&self, id: Uuid) -> Result<(), DomainError> {
        self.update_status(id, AdminQueuedCommandStatus::Running, None)
            .await
    }

    pub async fn finish(
        &self,
        id: Uuid,
        result: AdminCommandResult,
    ) -> Result<AdminQueuedCommand, DomainError> {
        let status = if result.accepted {
            AdminQueuedCommandStatus::Completed
        } else if result
            .results
            .iter()
            .any(|entry| entry.status == autoflow_domain::AdminCommandTargetStatus::Error)
        {
            AdminQueuedCommandStatus::Failed
        } else {
            AdminQueuedCommandStatus::Skipped
        };
        self.update_status(id, status, Some(result)).await?;
        self.get(id).await
    }

    async fn get(&self, id: Uuid) -> Result<AdminQueuedCommand, DomainError> {
        let row: AdminCommandRow = sqlx::query_as(
            "SELECT id, source, request_payload, status, result_payload, created_at, updated_at
             FROM admin_commands
             WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|error| DomainError::Database(error.to_string()))?;

        AdminQueuedCommand::try_from(row)
    }

    async fn save(&self, command: &AdminQueuedCommand) -> Result<(), DomainError> {
        let request_payload = serde_json::to_string(&command.request)
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let result_payload = command
            .result
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| DomainError::Database(error.to_string()))?;

        sqlx::query(
            "INSERT INTO admin_commands
                (id, source, request_payload, status, result_payload, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(command.id.to_string())
        .bind(&command.source)
        .bind(request_payload)
        .bind(status_to_db(command.status))
        .bind(result_payload)
        .bind(command.created_at.to_rfc3339())
        .bind(command.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|error| DomainError::Database(error.to_string()))?;

        Ok(())
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: AdminQueuedCommandStatus,
        result: Option<AdminCommandResult>,
    ) -> Result<(), DomainError> {
        let result_payload = result
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| DomainError::Database(error.to_string()))?;

        sqlx::query(
            "UPDATE admin_commands
             SET status = ?, result_payload = COALESCE(?, result_payload), updated_at = ?
             WHERE id = ?",
        )
        .bind(status_to_db(status))
        .bind(result_payload)
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|error| DomainError::Database(error.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AdminCommandRow {
    id: String,
    source: String,
    request_payload: String,
    status: String,
    result_payload: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<AdminCommandRow> for AdminQueuedCommand {
    type Error = DomainError;

    fn try_from(row: AdminCommandRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&row.id)
                .map_err(|error| DomainError::Database(error.to_string()))?,
            source: row.source,
            request: serde_json::from_str(&row.request_payload)
                .map_err(|error| DomainError::Database(error.to_string()))?,
            status: parse_status(&row.status)?,
            result: row
                .result_payload
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|error| DomainError::Database(error.to_string()))?,
            created_at: parse_datetime(&row.created_at)?,
            updated_at: parse_datetime(&row.updated_at)?,
        })
    }
}

fn status_to_db(status: AdminQueuedCommandStatus) -> &'static str {
    match status {
        AdminQueuedCommandStatus::Pending => "pending",
        AdminQueuedCommandStatus::Running => "running",
        AdminQueuedCommandStatus::Completed => "completed",
        AdminQueuedCommandStatus::Failed => "failed",
        AdminQueuedCommandStatus::Skipped => "skipped",
    }
}

fn parse_status(raw: &str) -> Result<AdminQueuedCommandStatus, DomainError> {
    match raw {
        "pending" => Ok(AdminQueuedCommandStatus::Pending),
        "running" => Ok(AdminQueuedCommandStatus::Running),
        "completed" => Ok(AdminQueuedCommandStatus::Completed),
        "failed" => Ok(AdminQueuedCommandStatus::Failed),
        "skipped" => Ok(AdminQueuedCommandStatus::Skipped),
        other => Err(DomainError::Database(format!(
            "unknown admin command status: {other}"
        ))),
    }
}

fn parse_datetime(raw: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| DomainError::Database(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_pool;
    use autoflow_domain::AdminCommandKind;

    #[tokio::test]
    async fn enqueues_and_finishes_admin_command() {
        let temp = tempfile::tempdir().unwrap();
        let pool = init_pool(temp.path()).await.unwrap();
        let store = SqliteAdminCommandStore::new(pool);

        let command = store
            .enqueue(
                "test",
                AdminCommandRequest {
                    kind: AdminCommandKind::RequestLogs,
                    target_instance_ids: vec!["local".into()],
                    job_ids: Vec::new(),
                    execution_ids: Vec::new(),
                    reason: None,
                    job_payloads: Vec::new(),
                },
            )
            .await
            .unwrap();

        assert_eq!(store.summary().await.unwrap().pending, 1);
        store.mark_running(command.id).await.unwrap();
        let finished = store
            .finish(
                command.id,
                AdminCommandResult {
                    accepted: false,
                    command: AdminCommandKind::RequestLogs,
                    results: Vec::new(),
                },
            )
            .await
            .unwrap();

        assert_eq!(finished.status, AdminQueuedCommandStatus::Skipped);
    }
}
