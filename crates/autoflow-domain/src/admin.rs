use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ActiveExecution, AdminAgentMode, DomainError, JobMode};

const ADMIN_TRANSPORT_SCHEMA_VERSION: &str = "admin.transport.v1";
const ADMIN_TRANSPORT_SIGNATURE_CONTEXT: &str = "autoflow-admin-transport-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminInstanceStatus {
    Online,
    Warning,
    Offline,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AdminCommandKind {
    StartJob,
    CancelExecution,
    PauseJob,
    ResumeJob,
    StopJob,
    CreateJob,
    UpdateJob,
    DeleteJob,
    ApplySettingsPolicy,
    RequestLogs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminCommandSupport {
    Available,
    Planned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandCapability {
    pub kind: AdminCommandKind,
    pub label: String,
    pub support: AdminCommandSupport,
    pub scope: String,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminHardwareProfile {
    pub host_name: String,
    pub operating_system: String,
    pub architecture: String,
    pub cpu_threads: u32,
    pub total_memory_mb: Option<u64>,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminNetworkInterface {
    pub name: String,
    pub address: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminNetworkProfile {
    pub domain: Option<String>,
    pub interfaces: Vec<AdminNetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminManagementProfile {
    pub enabled: bool,
    pub mode: AdminAgentMode,
    pub server_url: Option<String>,
    pub allow_remote_commands: bool,
    pub allow_batch_commands: bool,
    pub heartbeat_interval_secs: u32,
    pub inventory_interval_secs: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminJobRuntimeStatus {
    Running,
    Scheduled,
    Idle,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminManagedJob {
    pub id: Uuid,
    pub name: String,
    pub mode: JobMode,
    pub enabled: bool,
    pub source_path: String,
    pub target_path: String,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub status: AdminJobRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminInstanceSnapshot {
    pub instance_id: String,
    pub display_name: String,
    pub status: AdminInstanceStatus,
    pub last_seen_at: DateTime<Utc>,
    pub hardware: AdminHardwareProfile,
    pub network: AdminNetworkProfile,
    pub management: AdminManagementProfile,
    pub jobs: Vec<AdminManagedJob>,
    pub active_executions: Vec<ActiveExecution>,
    pub capabilities: Vec<AdminCommandCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdminFleetSummary {
    pub total_instances: u32,
    pub online_instances: u32,
    pub warning_instances: u32,
    pub offline_instances: u32,
    pub total_jobs: u32,
    pub running_jobs: u32,
}

impl AdminFleetSummary {
    pub fn from_instances(instances: &[AdminInstanceSnapshot]) -> Self {
        let mut summary = Self::default();
        summary.total_instances = instances.len() as u32;

        for instance in instances {
            match instance.status {
                AdminInstanceStatus::Online => summary.online_instances += 1,
                AdminInstanceStatus::Warning => summary.warning_instances += 1,
                AdminInstanceStatus::Offline => summary.offline_instances += 1,
            }
            summary.total_jobs += instance.jobs.len() as u32;
            summary.running_jobs += instance
                .jobs
                .iter()
                .filter(|job| job.status == AdminJobRuntimeStatus::Running)
                .count() as u32;
        }

        summary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminFleetSnapshot {
    pub generated_at: DateTime<Utc>,
    pub summary: AdminFleetSummary,
    pub instances: Vec<AdminInstanceSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminMachineGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instance_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminMachineGroupRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub instance_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminBatchCommandRequest {
    pub request: AdminCommandRequest,
    #[serde(default)]
    pub group_ids: Vec<Uuid>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminBatchCommandAccepted {
    pub accepted: bool,
    pub resolved_target_instance_ids: Vec<String>,
    pub command: AdminQueuedCommand,
    pub result: AdminCommandResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandPollRequest {
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandAssignment {
    pub command_id: Uuid,
    pub target_instance_id: String,
    pub request: AdminCommandRequest,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandPollResponse {
    pub assignment: Option<AdminSignedEnvelope<AdminCommandAssignment>>,
    pub pending_count: u32,
    pub polled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminCommandCompletionStatus {
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandCompletionRequest {
    pub command_id: Uuid,
    pub target_instance_id: String,
    pub status: AdminCommandCompletionStatus,
    pub message: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandCompletionAccepted {
    pub accepted: bool,
    pub command: AdminQueuedCommand,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminEnrollmentRequest {
    pub instance: AdminInstanceSnapshot,
    pub requested_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminEnrollmentTokenRequest {
    pub token: String,
    pub instance: AdminInstanceSnapshot,
    pub agent_secret: String,
    pub requested_at: DateTime<Utc>,
}

impl std::fmt::Debug for AdminEnrollmentTokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminEnrollmentTokenRequest")
            .field("token", &"<redacted>")
            .field("instance", &self.instance)
            .field("agent_secret", &"<redacted>")
            .field("requested_at", &self.requested_at)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminEnrollmentResponse {
    pub accepted: bool,
    pub instance_id: String,
    pub issued_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminHeartbeatPayload {
    pub instance: AdminInstanceSnapshot,
    pub generated_at: DateTime<Utc>,
    pub pending_command_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminHeartbeatDelivery {
    pub endpoint: String,
    pub status_code: Option<u16>,
    pub accepted: bool,
    pub message: String,
    pub sent_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAgentSecretRotationRequest {
    pub new_agent_secret: String,
    pub requested_at: DateTime<Utc>,
}

impl std::fmt::Debug for AdminAgentSecretRotationRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminAgentSecretRotationRequest")
            .field("new_agent_secret", &"<redacted>")
            .field("requested_at", &self.requested_at)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminAgentSecretRotationAccepted {
    pub accepted: bool,
    pub instance_id: String,
    pub rotated_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminHeartbeatAccepted {
    pub accepted: bool,
    pub instance_id: String,
    pub received_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AdminEnvelopeKind {
    Enrollment,
    Heartbeat,
    Command,
    SecretRotation,
}

impl AdminEnvelopeKind {
    fn as_wire_name(self) -> &'static str {
        match self {
            AdminEnvelopeKind::Enrollment => "enrollment",
            AdminEnvelopeKind::Heartbeat => "heartbeat",
            AdminEnvelopeKind::Command => "command",
            AdminEnvelopeKind::SecretRotation => "secretRotation",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminSignedEnvelope<T> {
    pub schema_version: String,
    pub kind: AdminEnvelopeKind,
    pub instance_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub nonce: Uuid,
    pub payload_hash: String,
    pub signature: String,
    pub payload: T,
}

impl<T> AdminSignedEnvelope<T>
where
    T: Serialize,
{
    pub fn sign(
        kind: AdminEnvelopeKind,
        instance_id: String,
        payload: T,
        signing_secret: &str,
        ttl_secs: u32,
    ) -> Result<Self, DomainError> {
        if signing_secret.trim().is_empty() {
            return Err(DomainError::Validation(
                "admin signing secret is required".into(),
            ));
        }
        if ttl_secs == 0 || ttl_secs > 86_400 {
            return Err(DomainError::Validation(
                "admin signed envelope ttl must be between 1 and 86400 seconds".into(),
            ));
        }

        let issued_at = Utc::now();
        let expires_at = issued_at + chrono::Duration::seconds(ttl_secs as i64);
        let nonce = Uuid::new_v4();
        let payload_hash = hash_admin_payload(&payload)?;
        let signature_input = admin_signature_input(
            kind,
            &instance_id,
            issued_at,
            expires_at,
            nonce,
            &payload_hash,
        );
        let signature = sign_admin_payload(signing_secret, &signature_input);

        Ok(Self {
            schema_version: ADMIN_TRANSPORT_SCHEMA_VERSION.into(),
            kind,
            instance_id,
            issued_at,
            expires_at,
            nonce,
            payload_hash,
            signature,
            payload,
        })
    }

    pub fn verify(&self, signing_secret: &str) -> Result<bool, DomainError> {
        if self.schema_version != ADMIN_TRANSPORT_SCHEMA_VERSION {
            return Ok(false);
        }

        let current_payload_hash = hash_admin_payload(&self.payload)?;
        if current_payload_hash != self.payload_hash {
            return Ok(false);
        }

        let signature_input = admin_signature_input(
            self.kind,
            &self.instance_id,
            self.issued_at,
            self.expires_at,
            self.nonce,
            &self.payload_hash,
        );
        Ok(sign_admin_payload(signing_secret, &signature_input) == self.signature)
    }
}

fn hash_admin_payload<T>(payload: &T) -> Result<String, DomainError>
where
    T: Serialize,
{
    let payload_bytes =
        serde_json::to_vec(payload).map_err(|error| DomainError::Validation(error.to_string()))?;
    Ok(blake3::hash(&payload_bytes).to_hex().to_string())
}

fn admin_signature_input(
    kind: AdminEnvelopeKind,
    instance_id: &str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    nonce: Uuid,
    payload_hash: &str,
) -> String {
    [
        ADMIN_TRANSPORT_SCHEMA_VERSION,
        kind.as_wire_name(),
        instance_id,
        &issued_at.to_rfc3339(),
        &expires_at.to_rfc3339(),
        &nonce.to_string(),
        payload_hash,
    ]
    .join("|")
}

fn sign_admin_payload(signing_secret: &str, signature_input: &str) -> String {
    let signing_key = blake3::derive_key(
        ADMIN_TRANSPORT_SIGNATURE_CONTEXT,
        signing_secret.trim().as_bytes(),
    );
    format!(
        "blake3:{}",
        blake3::keyed_hash(&signing_key, signature_input.as_bytes()).to_hex()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandRequest {
    pub kind: AdminCommandKind,
    #[serde(default)]
    pub target_instance_ids: Vec<String>,
    #[serde(default)]
    pub job_ids: Vec<Uuid>,
    #[serde(default)]
    pub execution_ids: Vec<Uuid>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminCommandTargetStatus {
    Accepted,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandTargetResult {
    pub target_instance_id: String,
    pub status: AdminCommandTargetStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandResult {
    pub accepted: bool,
    pub command: AdminCommandKind,
    pub results: Vec<AdminCommandTargetResult>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminQueuedCommandStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdminQueuedCommand {
    pub id: Uuid,
    pub source: String,
    pub request: AdminCommandRequest,
    pub status: AdminQueuedCommandStatus,
    pub result: Option<AdminCommandResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommandQueueSummary {
    pub pending: u32,
    pub running: u32,
    pub completed: u32,
    pub failed: u32,
    pub skipped: u32,
}

impl AdminCommandQueueSummary {
    pub fn from_commands(commands: &[AdminQueuedCommand]) -> Self {
        let mut summary = Self::default();
        for command in commands {
            match command.status {
                AdminQueuedCommandStatus::Pending => summary.pending += 1,
                AdminQueuedCommandStatus::Running => summary.running += 1,
                AdminQueuedCommandStatus::Completed => summary.completed += 1,
                AdminQueuedCommandStatus::Failed => summary.failed += 1,
                AdminQueuedCommandStatus::Skipped => summary.skipped += 1,
            }
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_instances_by_status_and_running_jobs() {
        let summary = AdminFleetSummary::from_instances(&[sample_instance()]);

        assert_eq!(summary.total_instances, 1);
        assert_eq!(summary.online_instances, 1);
        assert_eq!(summary.total_jobs, 1);
        assert_eq!(summary.running_jobs, 1);
    }

    #[test]
    fn signs_and_verifies_heartbeat_envelope() {
        let payload = AdminHeartbeatPayload {
            instance: sample_instance(),
            generated_at: Utc::now(),
            pending_command_count: 2,
        };

        let envelope = AdminSignedEnvelope::sign(
            AdminEnvelopeKind::Heartbeat,
            "local-1".into(),
            payload,
            "shared-secret",
            300,
        )
        .expect("heartbeat envelope should be signed");

        assert_eq!(envelope.kind, AdminEnvelopeKind::Heartbeat);
        assert!(envelope.verify("shared-secret").unwrap());
        assert!(!envelope.verify("wrong-secret").unwrap());

        let mut tampered = envelope.clone();
        tampered.payload.pending_command_count = 99;

        assert!(!tampered.verify("shared-secret").unwrap());
    }

    fn sample_instance() -> AdminInstanceSnapshot {
        AdminInstanceSnapshot {
            instance_id: "local-1".into(),
            display_name: "Estacao 01".into(),
            status: AdminInstanceStatus::Online,
            last_seen_at: Utc::now(),
            hardware: AdminHardwareProfile {
                host_name: "HOST".into(),
                operating_system: "windows".into(),
                architecture: "x86_64".into(),
                cpu_threads: 8,
                total_memory_mb: None,
                app_version: "0.2.0".into(),
            },
            network: AdminNetworkProfile {
                domain: None,
                interfaces: Vec::new(),
            },
            management: AdminManagementProfile {
                enabled: true,
                mode: AdminAgentMode::LocalOnly,
                server_url: None,
                allow_remote_commands: false,
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
}
