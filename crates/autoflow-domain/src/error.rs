use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("settings store error: {0}")]
    Settings(String),
    #[error("job not found")]
    JobNotFound,
    #[error("job is disabled")]
    JobDisabled,
    #[error("source not found: {0}")]
    SourceNotFound(String),
    #[error("job already running")]
    AlreadyRunning,
    #[error("database error: {0}")]
    Database(String),
    #[error("execution not found")]
    ExecutionNotFound,
}
