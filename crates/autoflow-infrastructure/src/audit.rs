use autoflow_application::ports::AuditStore;
use autoflow_domain::{
    AuditEntry, AuditPage, AuditQuery, AuditStatus, AuditSummary, DomainError, NewAuditEntry,
};
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
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
        sqlx::query(
            "INSERT INTO audit_entries (job_id, blueprint_id, job_name, source_path, target_path, status, file_size, duration_ms, details, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entry.job_id.map(|id| id.to_string()))
        .bind(entry.blueprint_id.map(|id| id.to_string()))
        .bind(entry.job_name)
        .bind(entry.source_path)
        .bind(entry.target_path)
        .bind(status_to_db(entry.status))
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
        Ok(self
            .query(AuditQuery {
                limit,
                ..AuditQuery::default()
            })
            .await?
            .entries)
    }

    async fn query(&self, query: AuditQuery) -> Result<AuditPage, DomainError> {
        let query = query.normalized();
        let summary = self.query_summary(&query).await?;
        let entries = self.query_entries(&query).await?;

        Ok(AuditPage {
            entries,
            total: summary.total,
            limit: query.limit,
            offset: query.offset,
            summary,
        })
    }
}

impl SqliteAuditStore {
    async fn query_entries(&self, query: &AuditQuery) -> Result<Vec<AuditEntry>, DomainError> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, job_id, blueprint_id, job_name, source_path, target_path, status, file_size, duration_ms, details, created_at FROM audit_entries",
        );
        push_filters(&mut builder, query);
        builder.push(" ORDER BY created_at DESC, id DESC LIMIT ");
        builder.push_bind(query.limit);
        builder.push(" OFFSET ");
        builder.push_bind(query.offset);

        let rows: Vec<AuditRow> = builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter().map(AuditEntry::try_from).collect()
    }

    async fn query_summary(&self, query: &AuditQuery) -> Result<AuditSummary, DomainError> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT
                COUNT(*) AS total,
                COALESCE(SUM(CASE WHEN status = 'COPIED' THEN 1 ELSE 0 END), 0) AS copied,
                COALESCE(SUM(CASE WHEN status = 'MOVED' THEN 1 ELSE 0 END), 0) AS moved,
                COALESCE(SUM(CASE WHEN status = 'IGNORED' THEN 1 ELSE 0 END), 0) AS ignored,
                COALESCE(SUM(CASE WHEN status = 'FAILED' THEN 1 ELSE 0 END), 0) AS failed,
                COALESCE(SUM(CASE WHEN status = 'ORGANIZED' THEN 1 ELSE 0 END), 0) AS organized,
                COALESCE(SUM(file_size), 0) AS total_bytes,
                COALESCE(AVG(duration_ms), 0) AS average_duration_ms,
                MAX(created_at) AS latest_at
             FROM audit_entries",
        );
        push_filters(&mut builder, query);

        let row: SummaryRow = builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(AuditSummary {
            total: row.total,
            copied: row.copied,
            moved: row.moved,
            ignored: row.ignored,
            failed: row.failed,
            organized: row.organized,
            total_bytes: row.total_bytes,
            average_duration_ms: row.average_duration_ms,
            latest_at: row.latest_at.as_deref().and_then(parse_datetime_or_none),
        })
    }
}

fn push_filters(builder: &mut QueryBuilder<Sqlite>, query: &AuditQuery) {
    let mut has_filter = false;

    if let Some(search) = &query.search {
        push_filter_prefix(builder, &mut has_filter);
        let pattern = format!("%{}%", search.to_ascii_lowercase());
        builder
            .push("(LOWER(job_name) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR LOWER(source_path) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR LOWER(target_path) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR LOWER(COALESCE(details, '')) LIKE ")
            .push_bind(pattern)
            .push(")");
    }

    if let Some(status) = query.status {
        push_filter_prefix(builder, &mut has_filter);
        builder.push("status = ").push_bind(status_to_db(status));
    }

    if let Some(job_id) = query.job_id {
        push_filter_prefix(builder, &mut has_filter);
        builder.push("job_id = ").push_bind(job_id.to_string());
    }

    if let Some(blueprint_id) = query.blueprint_id {
        push_filter_prefix(builder, &mut has_filter);
        builder
            .push("blueprint_id = ")
            .push_bind(blueprint_id.to_string());
    }

    if let Some(date_from) = query.date_from {
        push_filter_prefix(builder, &mut has_filter);
        builder
            .push("created_at >= ")
            .push_bind(date_from.to_rfc3339());
    }

    if let Some(date_to) = query.date_to {
        push_filter_prefix(builder, &mut has_filter);
        builder
            .push("created_at <= ")
            .push_bind(date_to.to_rfc3339());
    }
}

fn push_filter_prefix(builder: &mut QueryBuilder<Sqlite>, has_filter: &mut bool) {
    if *has_filter {
        builder.push(" AND ");
    } else {
        builder.push(" WHERE ");
        *has_filter = true;
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: i64,
    job_id: Option<String>,
    blueprint_id: Option<String>,
    job_name: String,
    source_path: String,
    target_path: String,
    status: String,
    file_size: i64,
    duration_ms: f64,
    details: Option<String>,
    created_at: String,
}

impl TryFrom<AuditRow> for AuditEntry {
    type Error = DomainError;

    fn try_from(row: AuditRow) -> Result<Self, Self::Error> {
        Ok(AuditEntry {
            id: row.id,
            job_id: row.job_id.and_then(|id| Uuid::parse_str(&id).ok()),
            blueprint_id: row.blueprint_id.and_then(|id| Uuid::parse_str(&id).ok()),
            job_name: row.job_name,
            source_path: row.source_path,
            target_path: row.target_path,
            status: parse_status(&row.status)?,
            file_size: row.file_size,
            duration_ms: row.duration_ms,
            details: row.details,
            created_at: parse_datetime_or_none(&row.created_at).unwrap_or_else(Utc::now),
        })
    }
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    total: i64,
    copied: i64,
    moved: i64,
    ignored: i64,
    failed: i64,
    organized: i64,
    total_bytes: i64,
    average_duration_ms: f64,
    latest_at: Option<String>,
}

fn status_to_db(status: AuditStatus) -> &'static str {
    match status {
        AuditStatus::Copied => "COPIED",
        AuditStatus::Moved => "MOVED",
        AuditStatus::Ignored => "IGNORED",
        AuditStatus::Failed => "FAILED",
        AuditStatus::Organized => "ORGANIZED",
    }
}

fn parse_status(raw: &str) -> Result<AuditStatus, DomainError> {
    match raw {
        "COPIED" => Ok(AuditStatus::Copied),
        "MOVED" => Ok(AuditStatus::Moved),
        "IGNORED" => Ok(AuditStatus::Ignored),
        "FAILED" => Ok(AuditStatus::Failed),
        "ORGANIZED" => Ok(AuditStatus::Organized),
        other => Err(DomainError::Database(format!(
            "unknown audit status: {other}"
        ))),
    }
}

fn parse_datetime_or_none(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_pool;
    use autoflow_domain::NewAuditEntry;
    use uuid::Uuid;

    #[tokio::test]
    async fn query_filters_and_summarizes_audit_entries() {
        let temp = tempfile::tempdir().unwrap();
        let pool = init_pool(temp.path()).await.unwrap();
        let store = SqliteAuditStore::new(pool);
        let job_id = Uuid::new_v4();

        store
            .append(NewAuditEntry {
                job_id: Some(job_id),
                blueprint_id: None,
                job_name: "Fotos".into(),
                source_path: "C:/Entrada/foto.jpg".into(),
                target_path: "D:/Saida/foto.jpg".into(),
                status: AuditStatus::Copied,
                file_size: 2048,
                duration_ms: 10.0,
                details: None,
            })
            .await
            .unwrap();
        store
            .append(NewAuditEntry {
                job_id: Some(job_id),
                blueprint_id: None,
                job_name: "Fotos".into(),
                source_path: "C:/Entrada/quebrado.jpg".into(),
                target_path: "D:/Saida/quebrado.jpg".into(),
                status: AuditStatus::Failed,
                file_size: 1024,
                duration_ms: 30.0,
                details: Some("Erro de permissao".into()),
            })
            .await
            .unwrap();

        let page = store
            .query(AuditQuery {
                search: Some("permissao".into()),
                status: Some(AuditStatus::Failed),
                job_id: Some(job_id),
                limit: 10,
                ..AuditQuery::default()
            })
            .await
            .unwrap();

        assert_eq!(page.total, 1);
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.summary.failed, 1);
        assert_eq!(page.summary.total_bytes, 1024);
        assert_eq!(page.entries[0].status, AuditStatus::Failed);
    }
}
