use std::{env, net::SocketAddr, path::PathBuf};

use autoflow_admin_server::{router, AdminServerError, AdminServerState};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions},
    ConnectOptions,
};
use thiserror::Error;
use tokio::net::TcpListener;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:7840";
const DEFAULT_DATABASE_PATH: &str = "autoflow-admin.sqlite";

#[tokio::main]
async fn main() -> Result<(), AdminServerCliError> {
    let config = AdminServerConfig::from_env()?;
    let pool = open_pool(&config.database_path).await?;
    let state = AdminServerState::with_sqlite_pool(pool).await?;

    if let Some(enrollment_token) = config.enrollment_token {
        state.register_enrollment_token(enrollment_token)?;
    }

    if let Some(bootstrap) = config.bootstrap_agent {
        state
            .register_agent_secret(bootstrap.instance_id, bootstrap.secret)
            .await?;
    }

    let listener = TcpListener::bind(config.bind_addr).await?;
    println!(
        "AutoFlow admin server listening on http://{}",
        config.bind_addr
    );

    axum::serve(listener, router(state)).await?;
    Ok(())
}

struct AdminServerConfig {
    bind_addr: SocketAddr,
    database_path: PathBuf,
    enrollment_token: Option<String>,
    bootstrap_agent: Option<BootstrapAgent>,
}

struct BootstrapAgent {
    instance_id: String,
    secret: String,
}

impl AdminServerConfig {
    fn from_env() -> Result<Self, AdminServerCliError> {
        let bind_addr = env::var("AUTOFLOW_ADMIN_BIND")
            .unwrap_or_else(|_| DEFAULT_BIND_ADDR.into())
            .parse::<SocketAddr>()
            .map_err(|error| AdminServerCliError::InvalidBind(error.to_string()))?;
        let database_path = env::var("AUTOFLOW_ADMIN_DB")
            .unwrap_or_else(|_| DEFAULT_DATABASE_PATH.into())
            .into();
        let enrollment_token = optional_env("AUTOFLOW_ADMIN_ENROLLMENT_TOKEN");
        let bootstrap_agent = bootstrap_agent_from_env()?;

        Ok(Self {
            bind_addr,
            database_path,
            enrollment_token,
            bootstrap_agent,
        })
    }
}

fn bootstrap_agent_from_env() -> Result<Option<BootstrapAgent>, AdminServerCliError> {
    let instance_id = optional_env("AUTOFLOW_ADMIN_BOOTSTRAP_INSTANCE_ID");
    let secret = optional_env("AUTOFLOW_ADMIN_BOOTSTRAP_SECRET");

    match (instance_id, secret) {
        (Some(instance_id), Some(secret)) => Ok(Some(BootstrapAgent {
            instance_id,
            secret,
        })),
        (None, None) => Ok(None),
        _ => Err(AdminServerCliError::IncompleteBootstrap),
    }
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn open_pool(database_path: &PathBuf) -> Result<SqlitePool, AdminServerCliError> {
    if let Some(parent) = database_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .disable_statement_logging();

    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?)
}

#[derive(Debug, Error)]
enum AdminServerCliError {
    #[error("AUTOFLOW_ADMIN_BIND must be host:port, for example 127.0.0.1:7840: {0}")]
    InvalidBind(String),
    #[error("set both AUTOFLOW_ADMIN_BOOTSTRAP_INSTANCE_ID and AUTOFLOW_ADMIN_BOOTSTRAP_SECRET")]
    IncompleteBootstrap,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlite(#[from] sqlx::Error),
    #[error(transparent)]
    Server(#[from] AdminServerError),
}
