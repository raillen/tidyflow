use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use autoflow_admin_server::{router, AdminServerError, AdminServerState};
use autoflow_domain::AdminOperatorRole;
use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions},
    ConnectOptions,
};
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

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
    if let Some(operator_token) = config.operator_token {
        state.register_operator_token(operator_token, config.operator_role)?;
    }

    if let Some(bootstrap) = config.bootstrap_agent {
        state
            .register_agent_secret(bootstrap.instance_id, bootstrap.secret)
            .await?;
    }

    let app = router(state);

    if let (Some(cert_path), Some(key_path)) = (config.tls_cert.as_ref(), config.tls_key.as_ref()) {
        let cert_pem = std::fs::read(cert_path)?;
        let key_pem = std::fs::read(key_path)?;
        let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_slice_iter(&cert_pem)
            .collect::<Result<_, _>>()
            .map_err(|error| AdminServerCliError::Tls(error.to_string()))?;
        let key = PrivateKeyDer::from_pem_slice(&key_pem)
            .map_err(|error| AdminServerCliError::Tls(error.to_string()))?;
        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|error| AdminServerCliError::Tls(error.to_string()))?;
        let acceptor = TlsAcceptor::from(Arc::new(tls_config));

        struct TlsListener {
            tcp: TcpListener,
            acceptor: TlsAcceptor,
        }

        impl axum::serve::Listener for TlsListener {
            type Io = tokio_rustls::server::TlsStream<tokio::net::TcpStream>;
            type Addr = SocketAddr;

            async fn accept(&mut self) -> (Self::Io, Self::Addr) {
                loop {
                    match self.tcp.accept().await {
                        Ok((tcp, addr)) => match self.acceptor.accept(tcp).await {
                            Ok(tls) => return (tls, addr),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    }
                }
            }

            fn local_addr(&self) -> std::io::Result<Self::Addr> {
                self.tcp.local_addr()
            }
        }

        let tcp = TcpListener::bind(config.bind_addr).await?;
        let listener = TlsListener { tcp, acceptor };

        println!(
            "AutoFlow admin server listening on https://{}",
            config.bind_addr
        );

        axum::serve(listener, app).await?;
        Ok(())
    } else {
        let listener = TcpListener::bind(config.bind_addr).await?;
        println!(
            "AutoFlow admin server listening on http://{}",
            config.bind_addr
        );

        axum::serve(listener, app).await?;
        Ok(())
    }
}

struct AdminServerConfig {
    bind_addr: SocketAddr,
    database_path: PathBuf,
    enrollment_token: Option<String>,
    operator_token: Option<String>,
    operator_role: AdminOperatorRole,
    bootstrap_agent: Option<BootstrapAgent>,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
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
        let operator_token = optional_env("AUTOFLOW_ADMIN_OPERATOR_TOKEN");
        let operator_role = optional_env("AUTOFLOW_ADMIN_OPERATOR_ROLE")
            .as_deref()
            .map(parse_operator_role)
            .transpose()?
            .unwrap_or(AdminOperatorRole::Admin);
        let bootstrap_agent = bootstrap_agent_from_env()?;
        let tls_cert = optional_env("AUTOFLOW_ADMIN_TLS_CERT").map(PathBuf::from);
        let tls_key = optional_env("AUTOFLOW_ADMIN_TLS_KEY").map(PathBuf::from);

        Ok(Self {
            bind_addr,
            database_path,
            enrollment_token,
            operator_token,
            operator_role,
            bootstrap_agent,
            tls_cert,
            tls_key,
        })
    }
}

fn parse_operator_role(raw: &str) -> Result<AdminOperatorRole, AdminServerCliError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "viewer" => Ok(AdminOperatorRole::Viewer),
        "operator" => Ok(AdminOperatorRole::Operator),
        "admin" => Ok(AdminOperatorRole::Admin),
        other => Err(AdminServerCliError::InvalidOperatorRole(other.into())),
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
    #[error("AUTOFLOW_ADMIN_OPERATOR_ROLE must be viewer, operator, or admin: {0}")]
    InvalidOperatorRole(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlite(#[from] sqlx::Error),
    #[error(transparent)]
    Server(#[from] AdminServerError),
    #[error("TLS configuration failed: {0}")]
    Tls(String),
}
