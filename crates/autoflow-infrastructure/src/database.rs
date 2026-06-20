use std::path::{Path, PathBuf};

use autoflow_domain::DomainError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;

const MIGRATION_001: &str = include_str!("../migrations/001_init.sql");
const MIGRATION_002: &str = include_str!("../migrations/002_ui_state.sql");

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
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), DomainError> {
    for migration in [MIGRATION_001, MIGRATION_002] {
        for statement in migration.split(';').map(str::trim).filter(|s| !s.is_empty()) {
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
