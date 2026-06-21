use std::path::{Path, PathBuf};

use autoflow_domain::DomainError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;

const MIGRATION_001: &str = include_str!("../migrations/001_init.sql");
const MIGRATION_002: &str = include_str!("../migrations/002_ui_state.sql");
const MIGRATION_003: &str = include_str!("../migrations/003_blueprints.sql");
const MIGRATION_004: &str = include_str!("../migrations/004_admin_commands.sql");

pub async fn init_pool(data_dir: &Path) -> Result<SqlitePool, DomainError> {
    std::fs::create_dir_all(data_dir).map_err(|e| DomainError::Database(e.to_string()))?;

    let db_path = data_dir.join("autoflow.db");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true)
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    run_migrations(&pool).await?;
    ensure_audit_blueprint_column(&pool).await?;
    Ok(pool)
}

async fn ensure_audit_blueprint_column(pool: &SqlitePool) -> Result<(), DomainError> {
    let columns: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(audit_entries)")
            .fetch_all(pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

    if columns.iter().any(|col| col.1 == "blueprint_id") {
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_blueprint_id ON audit_entries(blueprint_id)",
        )
        .execute(pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
        return Ok(());
    }

    sqlx::query("ALTER TABLE audit_entries ADD COLUMN blueprint_id TEXT")
        .execute(pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_blueprint_id ON audit_entries(blueprint_id)")
        .execute(pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;
    Ok(())
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), DomainError> {
    for migration in [MIGRATION_001, MIGRATION_002, MIGRATION_003, MIGRATION_004] {
        for statement in migration
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            sqlx::query(statement)
                .execute(pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;
        }
    }
    Ok(())
}

pub fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("autoflow.db")
}
