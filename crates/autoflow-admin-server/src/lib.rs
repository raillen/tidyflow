use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use autoflow_domain::{
    AdminEnvelopeKind, AdminFleetSnapshot, AdminFleetSummary, AdminHeartbeatAccepted,
    AdminHeartbeatPayload, AdminInstanceSnapshot, AdminSignedEnvelope,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdminServerHealth {
    pub status: String,
    pub version: String,
    pub service: String,
}

impl AdminServerHealth {
    pub fn ok(version: &str) -> Self {
        Self {
            status: "ok".into(),
            version: version.into(),
            service: "autoflow-admin-server".into(),
        }
    }
}

#[derive(Clone, Default)]
pub struct AdminServerState {
    inner: Arc<AdminServerStateInner>,
}

#[derive(Default)]
struct AdminServerStateInner {
    instances: RwLock<HashMap<String, AdminInstanceSnapshot>>,
    agent_secrets: RwLock<HashMap<String, String>>,
}

impl AdminServerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_agent_secret(
        &self,
        instance_id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Result<(), AdminServerError> {
        let instance_id = instance_id.into().trim().to_string();
        let secret = secret.into().trim().to_string();

        if instance_id.is_empty() {
            return Err(AdminServerError::MissingInstanceId);
        }
        if secret.is_empty() {
            return Err(AdminServerError::MissingAgentSecret);
        }

        let mut secrets = self
            .inner
            .agent_secrets
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        secrets.insert(instance_id, secret);
        Ok(())
    }

    pub fn fleet_snapshot(&self) -> Result<AdminFleetSnapshot, AdminServerError> {
        let instances = self
            .inner
            .instances
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        let mut instances = instances.values().cloned().collect::<Vec<_>>();
        instances.sort_by(|left, right| left.display_name.cmp(&right.display_name));

        Ok(AdminFleetSnapshot {
            generated_at: Utc::now(),
            summary: AdminFleetSummary::from_instances(&instances),
            instances,
        })
    }

    pub fn receive_heartbeat(
        &self,
        path_instance_id: &str,
        envelope: AdminSignedEnvelope<AdminHeartbeatPayload>,
    ) -> Result<AdminHeartbeatAccepted, AdminServerError> {
        let path_instance_id = path_instance_id.trim();
        if path_instance_id.is_empty() {
            return Err(AdminServerError::MissingInstanceId);
        }
        if envelope.kind != AdminEnvelopeKind::Heartbeat {
            return Err(AdminServerError::InvalidEnvelopeKind);
        }
        if envelope.instance_id != path_instance_id {
            return Err(AdminServerError::EnvelopeInstanceMismatch);
        }

        let now = Utc::now();
        if envelope.expires_at < now {
            return Err(AdminServerError::ExpiredEnvelope);
        }

        let secret = self.agent_secret(path_instance_id)?;
        if !envelope.verify(&secret).map_err(AdminServerError::from)? {
            return Err(AdminServerError::InvalidSignature);
        }
        if envelope.payload.instance.instance_id != path_instance_id {
            return Err(AdminServerError::PayloadInstanceMismatch);
        }

        let mut instance = envelope.payload.instance;
        instance.last_seen_at = now;

        let mut instances = self
            .inner
            .instances
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        instances.insert(path_instance_id.to_string(), instance);

        Ok(AdminHeartbeatAccepted {
            accepted: true,
            instance_id: path_instance_id.to_string(),
            received_at: now,
            message: "Heartbeat aceito".into(),
        })
    }

    fn agent_secret(&self, instance_id: &str) -> Result<String, AdminServerError> {
        let secrets = self
            .inner
            .agent_secrets
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;

        secrets
            .get(instance_id)
            .cloned()
            .ok_or_else(|| AdminServerError::AgentNotRegistered(instance_id.into()))
    }
}

pub fn router(state: AdminServerState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/fleet", get(fleet_snapshot))
        .route(
            "/api/agents/{instance_id}/heartbeat",
            post(receive_heartbeat),
        )
        .with_state(state)
}

async fn health() -> Json<AdminServerHealth> {
    Json(AdminServerHealth::ok(env!("CARGO_PKG_VERSION")))
}

async fn fleet_snapshot(
    State(state): State<AdminServerState>,
) -> Result<Json<AdminFleetSnapshot>, AdminServerError> {
    Ok(Json(state.fleet_snapshot()?))
}

async fn receive_heartbeat(
    State(state): State<AdminServerState>,
    Path(instance_id): Path<String>,
    Json(envelope): Json<AdminSignedEnvelope<AdminHeartbeatPayload>>,
) -> Result<(StatusCode, Json<AdminHeartbeatAccepted>), AdminServerError> {
    let accepted = state.receive_heartbeat(&instance_id, envelope)?;
    Ok((StatusCode::ACCEPTED, Json(accepted)))
}

#[derive(Debug, Error)]
pub enum AdminServerError {
    #[error("instance id is required")]
    MissingInstanceId,
    #[error("agent secret is required")]
    MissingAgentSecret,
    #[error("agent is not registered: {0}")]
    AgentNotRegistered(String),
    #[error("envelope kind must be heartbeat")]
    InvalidEnvelopeKind,
    #[error("path instance id does not match envelope instance id")]
    EnvelopeInstanceMismatch,
    #[error("payload instance id does not match path instance id")]
    PayloadInstanceMismatch,
    #[error("admin signed envelope is expired")]
    ExpiredEnvelope,
    #[error("admin signed envelope signature is invalid")]
    InvalidSignature,
    #[error("admin signed envelope validation failed: {0}")]
    EnvelopeValidation(String),
    #[error("admin server state is unavailable")]
    StateUnavailable,
}

impl From<autoflow_domain::DomainError> for AdminServerError {
    fn from(error: autoflow_domain::DomainError) -> Self {
        Self::EnvelopeValidation(error.to_string())
    }
}

impl AdminServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingInstanceId
            | Self::MissingAgentSecret
            | Self::InvalidEnvelopeKind
            | Self::EnvelopeInstanceMismatch
            | Self::PayloadInstanceMismatch
            | Self::ExpiredEnvelope
            | Self::EnvelopeValidation(_) => StatusCode::BAD_REQUEST,
            Self::AgentNotRegistered(_) => StatusCode::NOT_FOUND,
            Self::InvalidSignature => StatusCode::UNAUTHORIZED,
            Self::StateUnavailable => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::MissingInstanceId => "missingInstanceId",
            Self::MissingAgentSecret => "missingAgentSecret",
            Self::AgentNotRegistered(_) => "agentNotRegistered",
            Self::InvalidEnvelopeKind => "invalidEnvelopeKind",
            Self::EnvelopeInstanceMismatch => "envelopeInstanceMismatch",
            Self::PayloadInstanceMismatch => "payloadInstanceMismatch",
            Self::ExpiredEnvelope => "expiredEnvelope",
            Self::InvalidSignature => "invalidSignature",
            Self::EnvelopeValidation(_) => "envelopeValidation",
            Self::StateUnavailable => "stateUnavailable",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminServerErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for AdminServerError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = AdminServerErrorBody {
            code: self.code(),
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autoflow_domain::{
        AdminAgentMode, AdminHardwareProfile, AdminInstanceStatus, AdminJobRuntimeStatus,
        AdminManagedJob, AdminManagementProfile, AdminNetworkProfile, JobMode,
    };
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn accepts_signed_heartbeat_and_updates_fleet_snapshot() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "shared-secret")
            .unwrap();
        let app = router(state.clone());
        let envelope = signed_heartbeat("local-1", "shared-secret", 300);

        let response = app
            .oneshot(json_post(
                "/api/agents/local-1/heartbeat",
                serde_json::to_vec(&envelope).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let accepted = parse_body::<AdminHeartbeatAccepted>(response).await;
        assert!(accepted.accepted);
        assert_eq!(accepted.instance_id, "local-1");

        let snapshot = state.fleet_snapshot().unwrap();
        assert_eq!(snapshot.summary.total_instances, 1);
        assert_eq!(snapshot.summary.online_instances, 1);
        assert_eq!(snapshot.summary.total_jobs, 1);
        assert_eq!(snapshot.summary.running_jobs, 1);
        assert_eq!(snapshot.instances[0].instance_id, "local-1");
    }

    #[tokio::test]
    async fn rejects_heartbeat_for_unknown_agent() {
        let state = AdminServerState::new();
        let app = router(state.clone());
        let envelope = signed_heartbeat("local-1", "shared-secret", 300);

        let response = app
            .oneshot(json_post(
                "/api/agents/local-1/heartbeat",
                serde_json::to_vec(&envelope).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(state.fleet_snapshot().unwrap().summary.total_instances, 0);
    }

    #[tokio::test]
    async fn rejects_heartbeat_with_wrong_signature() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "shared-secret")
            .unwrap();
        let app = router(state.clone());
        let envelope = signed_heartbeat("local-1", "wrong-secret", 300);

        let response = app
            .oneshot(json_post(
                "/api/agents/local-1/heartbeat",
                serde_json::to_vec(&envelope).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(state.fleet_snapshot().unwrap().summary.total_instances, 0);
    }

    #[tokio::test]
    async fn rejects_heartbeat_when_path_and_envelope_do_not_match() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "shared-secret")
            .unwrap();
        let app = router(state.clone());
        let envelope = signed_heartbeat("local-1", "shared-secret", 300);

        let response = app
            .oneshot(json_post(
                "/api/agents/local-2/heartbeat",
                serde_json::to_vec(&envelope).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(state.fleet_snapshot().unwrap().summary.total_instances, 0);
    }

    #[tokio::test]
    async fn exposes_health_and_empty_fleet() {
        let state = AdminServerState::new();
        let app = router(state);

        let health = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::OK);

        let fleet = app
            .oneshot(
                Request::builder()
                    .uri("/api/fleet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(fleet.status(), StatusCode::OK);
        let fleet = parse_body::<AdminFleetSnapshot>(fleet).await;
        assert_eq!(fleet.summary.total_instances, 0);
    }

    fn signed_heartbeat(
        instance_id: &str,
        secret: &str,
        ttl_secs: u32,
    ) -> AdminSignedEnvelope<AdminHeartbeatPayload> {
        let payload = AdminHeartbeatPayload {
            instance: sample_instance(instance_id),
            generated_at: Utc::now(),
            pending_command_count: 2,
        };

        AdminSignedEnvelope::sign(
            AdminEnvelopeKind::Heartbeat,
            instance_id.into(),
            payload,
            secret,
            ttl_secs,
        )
        .unwrap()
    }

    fn sample_instance(instance_id: &str) -> AdminInstanceSnapshot {
        AdminInstanceSnapshot {
            instance_id: instance_id.into(),
            display_name: "Estacao 01".into(),
            status: AdminInstanceStatus::Online,
            last_seen_at: Utc::now(),
            hardware: AdminHardwareProfile {
                host_name: "HOST".into(),
                operating_system: "windows".into(),
                architecture: "x86_64".into(),
                cpu_threads: 8,
                total_memory_mb: Some(16_384),
                app_version: "0.2.0".into(),
            },
            network: AdminNetworkProfile {
                domain: Some("AUTO".into()),
                interfaces: Vec::new(),
            },
            management: AdminManagementProfile {
                enabled: true,
                mode: AdminAgentMode::ManagedAgent,
                server_url: Some("https://admin.autoflow.local".into()),
                allow_remote_commands: true,
                allow_batch_commands: false,
                heartbeat_interval_secs: 30,
                inventory_interval_secs: 300,
            },
            jobs: vec![AdminManagedJob {
                id: Uuid::new_v4(),
                name: "Backup".into(),
                mode: JobMode::Copy,
                enabled: true,
                source_path: "C:/in".into(),
                target_path: "D:/out".into(),
                last_run: None,
                next_run: None,
                status: AdminJobRuntimeStatus::Running,
            }],
            active_executions: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    fn json_post(uri: &str, body: Vec<u8>) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    async fn parse_body<T>(response: Response) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }
}
