use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use autoflow_domain::{
    AdminAgentSecretRotationAccepted, AdminAgentSecretRotationRequest, AdminBatchCommandAccepted,
    AdminBatchCommandRequest, AdminCentralAuditEntry, AdminCentralAuditPage,
    AdminCentralAuditQuery, AdminCentralAuditStatus, AdminCommandAssignment,
    AdminCommandCompletionAccepted, AdminCommandCompletionRequest, AdminCommandCompletionStatus,
    AdminCommandPollRequest, AdminCommandPollResponse, AdminCommandResult,
    AdminCommandTargetResult, AdminCommandTargetStatus, AdminEnrollmentResponse,
    AdminEnrollmentTokenRequest, AdminEnvelopeKind, AdminFleetSnapshot, AdminFleetSummary,
    AdminHeartbeatAccepted, AdminHeartbeatPayload, AdminInstanceSnapshot, AdminMachineGroup,
    AdminMachineGroupRequest, AdminOperatorRole, AdminQueuedCommand, AdminQueuedCommandStatus,
    AdminSignedEnvelope,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

const ADMIN_OPERATOR_TOKEN_HEADER: &str = "x-autoflow-admin-token";

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

#[derive(Clone)]
pub struct AdminServerState {
    storage: AdminServerStorage,
    enrollment_tokens: Arc<RwLock<HashSet<String>>>,
    operator_tokens: Arc<RwLock<HashMap<String, AdminOperatorRole>>>,
}

#[derive(Default)]
struct MemoryAdminServerStore {
    instances: RwLock<HashMap<String, AdminInstanceSnapshot>>,
    agent_secrets: RwLock<HashMap<String, String>>,
    machine_groups: RwLock<HashMap<Uuid, AdminMachineGroup>>,
    command_queue: RwLock<HashMap<Uuid, AdminQueuedCommand>>,
    command_deliveries: RwLock<Vec<AdminCommandDelivery>>,
    central_audit: RwLock<Vec<AdminCentralAuditEntry>>,
}

#[derive(Debug, Clone)]
struct AdminCommandDelivery {
    command_id: Uuid,
    target_instance_id: String,
    status: AdminQueuedCommandStatus,
    assigned_at: Option<chrono::DateTime<Utc>>,
    completed_at: Option<chrono::DateTime<Utc>>,
    message: Option<String>,
}

#[derive(Debug, Clone)]
struct AdminCommandDeliveryState {
    target_instance_id: String,
    status: AdminQueuedCommandStatus,
    message: Option<String>,
}

#[derive(Debug, Clone)]
struct AdminOperatorContext {
    actor: String,
    role: Option<AdminOperatorRole>,
}

#[derive(Debug, Clone, Copy)]
enum AdminPermission {
    View,
    Operate,
    Administer,
}

#[derive(Clone)]
enum AdminServerStorage {
    Memory(Arc<MemoryAdminServerStore>),
    Sqlite(SqliteAdminServerStore),
}

#[derive(Clone)]
pub struct SqliteAdminServerStore {
    pool: SqlitePool,
}

impl Default for AdminServerState {
    fn default() -> Self {
        Self {
            storage: AdminServerStorage::Memory(Arc::new(MemoryAdminServerStore::default())),
            enrollment_tokens: Arc::new(RwLock::new(HashSet::new())),
            operator_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AdminServerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn with_sqlite_pool(pool: SqlitePool) -> Result<Self, AdminServerError> {
        let store = SqliteAdminServerStore::new(pool).await?;
        Ok(Self {
            storage: AdminServerStorage::Sqlite(store),
            enrollment_tokens: Arc::new(RwLock::new(HashSet::new())),
            operator_tokens: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn register_enrollment_token(
        &self,
        token: impl Into<String>,
    ) -> Result<(), AdminServerError> {
        let token = normalize_required_secret(token, AdminServerError::MissingEnrollmentToken)?;
        let mut tokens = self
            .enrollment_tokens
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        tokens.insert(token);
        Ok(())
    }

    pub fn register_operator_token(
        &self,
        token: impl Into<String>,
        role: AdminOperatorRole,
    ) -> Result<(), AdminServerError> {
        let token = normalize_required_secret(token, AdminServerError::MissingOperatorToken)?;
        let mut tokens = self
            .operator_tokens
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        tokens.insert(token, role);
        Ok(())
    }

    pub async fn register_agent_secret(
        &self,
        instance_id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Result<(), AdminServerError> {
        let (instance_id, secret) = normalize_agent_secret(instance_id, secret)?;

        match &self.storage {
            AdminServerStorage::Memory(store) => store.register_agent_secret(instance_id, secret),
            AdminServerStorage::Sqlite(store) => {
                store.register_agent_secret(&instance_id, &secret).await
            }
        }
    }

    pub async fn enroll_agent(
        &self,
        request: AdminEnrollmentTokenRequest,
    ) -> Result<AdminEnrollmentResponse, AdminServerError> {
        let token =
            normalize_required_secret(request.token, AdminServerError::MissingEnrollmentToken)?;
        self.ensure_enrollment_token(&token)?;

        let instance_id = normalize_instance_id(&request.instance.instance_id)?;
        let agent_secret =
            normalize_required_secret(request.agent_secret, AdminServerError::MissingAgentSecret)?;

        self.register_agent_secret(instance_id.clone(), agent_secret)
            .await?;

        let now = Utc::now();
        let mut instance = request.instance;
        instance.instance_id = instance_id.clone();
        instance.last_seen_at = now;
        self.save_instance(&instance).await?;

        Ok(AdminEnrollmentResponse {
            accepted: true,
            instance_id,
            issued_at: now,
            message: "Agent matriculado".into(),
        })
    }

    pub async fn fleet_snapshot(&self) -> Result<AdminFleetSnapshot, AdminServerError> {
        let mut instances = self.list_instances().await?;
        instances.sort_by(|left, right| left.display_name.cmp(&right.display_name));

        Ok(AdminFleetSnapshot {
            generated_at: Utc::now(),
            summary: AdminFleetSummary::from_instances(&instances),
            instances,
        })
    }

    pub async fn create_machine_group(
        &self,
        request: AdminMachineGroupRequest,
    ) -> Result<AdminMachineGroup, AdminServerError> {
        let now = Utc::now();
        let group = AdminMachineGroup {
            id: Uuid::new_v4(),
            name: normalize_group_name(request.name)?,
            description: request
                .description
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            instance_ids: normalize_instance_ids(request.instance_ids),
            created_at: now,
            updated_at: now,
        };

        self.save_machine_group(&group).await?;
        Ok(group)
    }

    pub async fn list_machine_groups(&self) -> Result<Vec<AdminMachineGroup>, AdminServerError> {
        let mut groups = match &self.storage {
            AdminServerStorage::Memory(store) => store.list_machine_groups(),
            AdminServerStorage::Sqlite(store) => store.list_machine_groups().await,
        }?;
        groups.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(groups)
    }

    pub async fn enqueue_batch_command(
        &self,
        request: AdminBatchCommandRequest,
    ) -> Result<AdminBatchCommandAccepted, AdminServerError> {
        let target_instance_ids = self.resolve_batch_targets(&request).await?;
        if target_instance_ids.is_empty() {
            return Err(AdminServerError::NoBatchTargets);
        }

        let now = Utc::now();
        let mut command_request = request.request;
        command_request.target_instance_ids = target_instance_ids.clone();
        let result = queued_batch_result(&command_request, &target_instance_ids);
        let command = AdminQueuedCommand {
            id: Uuid::new_v4(),
            source: request
                .source
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "admin-server".into()),
            request: command_request,
            status: AdminQueuedCommandStatus::Pending,
            result: None,
            created_at: now,
            updated_at: now,
        };

        self.save_batch_command(&command, &target_instance_ids)
            .await?;

        Ok(AdminBatchCommandAccepted {
            accepted: true,
            resolved_target_instance_ids: target_instance_ids,
            command,
            result,
        })
    }

    pub async fn list_queued_commands(&self) -> Result<Vec<AdminQueuedCommand>, AdminServerError> {
        let mut commands = match &self.storage {
            AdminServerStorage::Memory(store) => store.list_queued_commands(),
            AdminServerStorage::Sqlite(store) => store.list_queued_commands().await,
        }?;
        commands.sort_by_key(|command| std::cmp::Reverse(command.created_at));
        Ok(commands)
    }

    pub async fn poll_next_command(
        &self,
        path_instance_id: &str,
        envelope: AdminSignedEnvelope<AdminCommandPollRequest>,
    ) -> Result<AdminCommandPollResponse, AdminServerError> {
        let path_instance_id = path_instance_id.trim();
        let now = Utc::now();
        let secret = self
            .verify_agent_command_envelope(path_instance_id, &envelope, now)
            .await?;

        let assignment = self.assign_next_command(path_instance_id, now).await?;
        let signed_assignment = assignment
            .map(|assignment| {
                AdminSignedEnvelope::sign(
                    AdminEnvelopeKind::Command,
                    path_instance_id.to_string(),
                    assignment,
                    &secret,
                    300,
                )
            })
            .transpose()
            .map_err(AdminServerError::from)?;

        Ok(AdminCommandPollResponse {
            assignment: signed_assignment,
            pending_count: self.pending_command_count(path_instance_id).await?,
            polled_at: now,
        })
    }

    pub async fn complete_command(
        &self,
        path_instance_id: &str,
        path_command_id: Uuid,
        envelope: AdminSignedEnvelope<AdminCommandCompletionRequest>,
    ) -> Result<AdminCommandCompletionAccepted, AdminServerError> {
        let path_instance_id = path_instance_id.trim();
        let now = Utc::now();
        self.verify_agent_command_envelope(path_instance_id, &envelope, now)
            .await?;

        let completion = envelope.payload;
        if completion.command_id != path_command_id {
            return Err(AdminServerError::CommandPayloadMismatch);
        }
        if completion.target_instance_id != path_instance_id {
            return Err(AdminServerError::CommandTargetMismatch);
        }

        let command = self
            .record_command_completion(path_instance_id, path_command_id, completion, now)
            .await?;

        Ok(AdminCommandCompletionAccepted {
            accepted: true,
            command,
            recorded_at: now,
        })
    }

    pub async fn receive_heartbeat(
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

        let secret = self.agent_secret(path_instance_id).await?;
        if !envelope.verify(&secret).map_err(AdminServerError::from)? {
            return Err(AdminServerError::InvalidSignature);
        }
        if envelope.payload.instance.instance_id != path_instance_id {
            return Err(AdminServerError::PayloadInstanceMismatch);
        }

        let mut instance = envelope.payload.instance;
        instance.last_seen_at = now;

        self.save_instance(&instance).await?;

        Ok(AdminHeartbeatAccepted {
            accepted: true,
            instance_id: path_instance_id.to_string(),
            received_at: now,
            message: "Heartbeat aceito".into(),
        })
    }

    pub async fn rotate_agent_secret(
        &self,
        path_instance_id: &str,
        envelope: AdminSignedEnvelope<AdminAgentSecretRotationRequest>,
    ) -> Result<AdminAgentSecretRotationAccepted, AdminServerError> {
        let path_instance_id = path_instance_id.trim();
        if path_instance_id.is_empty() {
            return Err(AdminServerError::MissingInstanceId);
        }
        if envelope.kind != AdminEnvelopeKind::SecretRotation {
            return Err(AdminServerError::InvalidEnvelopeKind);
        }
        if envelope.instance_id != path_instance_id {
            return Err(AdminServerError::EnvelopeInstanceMismatch);
        }

        let now = Utc::now();
        if envelope.expires_at < now {
            return Err(AdminServerError::ExpiredEnvelope);
        }

        let current_secret = self.agent_secret(path_instance_id).await?;
        if !envelope
            .verify(&current_secret)
            .map_err(AdminServerError::from)?
        {
            return Err(AdminServerError::InvalidSignature);
        }

        let new_secret = normalize_required_secret(
            envelope.payload.new_agent_secret,
            AdminServerError::MissingAgentSecret,
        )?;
        self.register_agent_secret(path_instance_id.to_string(), new_secret)
            .await?;

        Ok(AdminAgentSecretRotationAccepted {
            accepted: true,
            instance_id: path_instance_id.to_string(),
            rotated_at: now,
            message: "Segredo do agent rotacionado".into(),
        })
    }

    async fn verify_agent_command_envelope<T>(
        &self,
        path_instance_id: &str,
        envelope: &AdminSignedEnvelope<T>,
        now: chrono::DateTime<Utc>,
    ) -> Result<String, AdminServerError>
    where
        T: Serialize,
    {
        if path_instance_id.is_empty() {
            return Err(AdminServerError::MissingInstanceId);
        }
        if envelope.kind != AdminEnvelopeKind::Command {
            return Err(AdminServerError::InvalidEnvelopeKind);
        }
        if envelope.instance_id != path_instance_id {
            return Err(AdminServerError::EnvelopeInstanceMismatch);
        }
        if envelope.expires_at < now {
            return Err(AdminServerError::ExpiredEnvelope);
        }

        let secret = self.agent_secret(path_instance_id).await?;
        if !envelope.verify(&secret).map_err(AdminServerError::from)? {
            return Err(AdminServerError::InvalidSignature);
        }
        Ok(secret)
    }

    fn authorize_operator(
        &self,
        headers: &HeaderMap,
        permission: AdminPermission,
    ) -> Result<AdminOperatorContext, AdminServerError> {
        let tokens = self
            .operator_tokens
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        if tokens.is_empty() {
            return Ok(AdminOperatorContext {
                actor: "dev-admin".into(),
                role: Some(AdminOperatorRole::Admin),
            });
        }

        let token = headers
            .get(ADMIN_OPERATOR_TOKEN_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(AdminServerError::OperatorAuthRequired)?;
        let role = tokens
            .get(token)
            .copied()
            .ok_or(AdminServerError::OperatorAuthRequired)?;

        if !operator_role_allows(role, permission) {
            return Err(AdminServerError::OperatorForbidden);
        }

        Ok(AdminOperatorContext {
            actor: operator_actor(token),
            role: Some(role),
        })
    }

    async fn agent_secret(&self, instance_id: &str) -> Result<String, AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.agent_secret(instance_id),
            AdminServerStorage::Sqlite(store) => store.agent_secret(instance_id).await,
        }
    }

    async fn save_instance(
        &self,
        instance: &AdminInstanceSnapshot,
    ) -> Result<(), AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.save_instance(instance),
            AdminServerStorage::Sqlite(store) => store.save_instance(instance).await,
        }
    }

    async fn list_instances(&self) -> Result<Vec<AdminInstanceSnapshot>, AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.list_instances(),
            AdminServerStorage::Sqlite(store) => store.list_instances().await,
        }
    }

    async fn save_machine_group(&self, group: &AdminMachineGroup) -> Result<(), AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.save_machine_group(group),
            AdminServerStorage::Sqlite(store) => store.save_machine_group(group).await,
        }
    }

    async fn save_batch_command(
        &self,
        command: &AdminQueuedCommand,
        target_instance_ids: &[String],
    ) -> Result<(), AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => {
                store.save_batch_command(command, target_instance_ids)
            }
            AdminServerStorage::Sqlite(store) => {
                store.save_batch_command(command, target_instance_ids).await
            }
        }
    }

    async fn assign_next_command(
        &self,
        instance_id: &str,
        assigned_at: chrono::DateTime<Utc>,
    ) -> Result<Option<AdminCommandAssignment>, AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => {
                store.assign_next_command(instance_id, assigned_at)
            }
            AdminServerStorage::Sqlite(store) => {
                store.assign_next_command(instance_id, assigned_at).await
            }
        }
    }

    async fn pending_command_count(&self, instance_id: &str) -> Result<u32, AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.pending_command_count(instance_id),
            AdminServerStorage::Sqlite(store) => store.pending_command_count(instance_id).await,
        }
    }

    async fn record_command_completion(
        &self,
        instance_id: &str,
        command_id: Uuid,
        completion: AdminCommandCompletionRequest,
        recorded_at: chrono::DateTime<Utc>,
    ) -> Result<AdminQueuedCommand, AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => {
                store.record_command_completion(instance_id, command_id, completion, recorded_at)
            }
            AdminServerStorage::Sqlite(store) => {
                store
                    .record_command_completion(instance_id, command_id, completion, recorded_at)
                    .await
            }
        }
    }

    async fn append_central_audit(
        &self,
        entry: AdminCentralAuditEntry,
    ) -> Result<(), AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.append_central_audit(entry),
            AdminServerStorage::Sqlite(store) => store.append_central_audit(entry).await,
        }
    }

    pub async fn query_central_audit(
        &self,
        query: AdminCentralAuditQuery,
    ) -> Result<AdminCentralAuditPage, AdminServerError> {
        match &self.storage {
            AdminServerStorage::Memory(store) => store.query_central_audit(query),
            AdminServerStorage::Sqlite(store) => store.query_central_audit(query).await,
        }
    }

    async fn resolve_batch_targets(
        &self,
        request: &AdminBatchCommandRequest,
    ) -> Result<Vec<String>, AdminServerError> {
        let mut target_ids = normalize_instance_ids(request.request.target_instance_ids.clone());
        let groups = self.list_machine_groups().await?;

        for group_id in &request.group_ids {
            let group = groups
                .iter()
                .find(|group| group.id == *group_id)
                .ok_or(AdminServerError::MachineGroupNotFound(*group_id))?;
            target_ids.extend(group.instance_ids.iter().cloned());
        }

        Ok(normalize_instance_ids(target_ids))
    }

    fn ensure_enrollment_token(&self, token: &str) -> Result<(), AdminServerError> {
        let tokens = self
            .enrollment_tokens
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;

        if tokens.is_empty() {
            return Err(AdminServerError::EnrollmentNotConfigured);
        }
        if !tokens.contains(token) {
            return Err(AdminServerError::InvalidEnrollmentToken);
        }

        Ok(())
    }
}

impl MemoryAdminServerStore {
    fn register_agent_secret(
        &self,
        instance_id: String,
        secret: String,
    ) -> Result<(), AdminServerError> {
        let mut secrets = self
            .agent_secrets
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        secrets.insert(instance_id, secret);
        Ok(())
    }

    fn list_instances(&self) -> Result<Vec<AdminInstanceSnapshot>, AdminServerError> {
        let instances = self
            .instances
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        Ok(instances.values().cloned().collect())
    }

    fn save_instance(&self, instance: &AdminInstanceSnapshot) -> Result<(), AdminServerError> {
        let mut instances = self
            .instances
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        instances.insert(instance.instance_id.clone(), instance.clone());
        Ok(())
    }

    fn agent_secret(&self, instance_id: &str) -> Result<String, AdminServerError> {
        let secrets = self
            .agent_secrets
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;

        secrets
            .get(instance_id)
            .cloned()
            .ok_or_else(|| AdminServerError::AgentNotRegistered(instance_id.into()))
    }

    fn save_machine_group(&self, group: &AdminMachineGroup) -> Result<(), AdminServerError> {
        let mut groups = self
            .machine_groups
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        groups.insert(group.id, group.clone());
        Ok(())
    }

    fn list_machine_groups(&self) -> Result<Vec<AdminMachineGroup>, AdminServerError> {
        let groups = self
            .machine_groups
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        Ok(groups.values().cloned().collect())
    }

    fn save_batch_command(
        &self,
        command: &AdminQueuedCommand,
        target_instance_ids: &[String],
    ) -> Result<(), AdminServerError> {
        let mut commands = self
            .command_queue
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        commands.insert(command.id, command.clone());
        let mut deliveries = self
            .command_deliveries
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        for target_instance_id in target_instance_ids {
            deliveries.push(AdminCommandDelivery {
                command_id: command.id,
                target_instance_id: target_instance_id.clone(),
                status: AdminQueuedCommandStatus::Pending,
                assigned_at: None,
                completed_at: None,
                message: None,
            });
        }
        Ok(())
    }

    fn list_queued_commands(&self) -> Result<Vec<AdminQueuedCommand>, AdminServerError> {
        let commands = self
            .command_queue
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        Ok(commands.values().cloned().collect())
    }

    fn assign_next_command(
        &self,
        instance_id: &str,
        assigned_at: chrono::DateTime<Utc>,
    ) -> Result<Option<AdminCommandAssignment>, AdminServerError> {
        let mut deliveries = self
            .command_deliveries
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        let Some(delivery) = deliveries.iter_mut().find(|delivery| {
            delivery.target_instance_id == instance_id
                && delivery.status == AdminQueuedCommandStatus::Pending
        }) else {
            return Ok(None);
        };

        delivery.status = AdminQueuedCommandStatus::Running;
        delivery.assigned_at = Some(assigned_at);

        let mut commands = self
            .command_queue
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        let command = commands
            .get_mut(&delivery.command_id)
            .ok_or(AdminServerError::CommandNotFound(delivery.command_id))?;
        command.status = AdminQueuedCommandStatus::Running;
        command.updated_at = assigned_at;

        Ok(Some(command_assignment(command, instance_id, assigned_at)))
    }

    fn pending_command_count(&self, instance_id: &str) -> Result<u32, AdminServerError> {
        let deliveries = self
            .command_deliveries
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        Ok(deliveries
            .iter()
            .filter(|delivery| {
                delivery.target_instance_id == instance_id
                    && delivery.status == AdminQueuedCommandStatus::Pending
            })
            .count() as u32)
    }

    fn record_command_completion(
        &self,
        instance_id: &str,
        command_id: Uuid,
        completion: AdminCommandCompletionRequest,
        recorded_at: chrono::DateTime<Utc>,
    ) -> Result<AdminQueuedCommand, AdminServerError> {
        let mut deliveries = self
            .command_deliveries
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        let Some(delivery) = deliveries.iter_mut().find(|delivery| {
            delivery.command_id == command_id && delivery.target_instance_id == instance_id
        }) else {
            return Err(AdminServerError::CommandTargetNotFound(
                command_id,
                instance_id.into(),
            ));
        };
        if delivery.status != AdminQueuedCommandStatus::Running {
            return Err(AdminServerError::CommandNotRunning(
                command_id,
                instance_id.into(),
            ));
        }

        delivery.status = completion_status_to_command_status(completion.status);
        delivery.completed_at = Some(completion.completed_at);
        delivery.message = Some(completion_message(completion.status, &completion.message));

        let delivery_states = command_delivery_states(&deliveries, command_id)?;

        let mut commands = self
            .command_queue
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        let command = commands
            .get_mut(&command_id)
            .ok_or(AdminServerError::CommandNotFound(command_id))?;
        command.status = aggregate_command_status(&delivery_states);
        command.result = command_result_from_deliveries(command.request.kind, &delivery_states);
        command.updated_at = recorded_at;

        Ok(command.clone())
    }

    fn append_central_audit(&self, entry: AdminCentralAuditEntry) -> Result<(), AdminServerError> {
        let mut entries = self
            .central_audit
            .write()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        entries.push(entry);
        Ok(())
    }

    fn query_central_audit(
        &self,
        query: AdminCentralAuditQuery,
    ) -> Result<AdminCentralAuditPage, AdminServerError> {
        let query = query.normalized();
        let entries = self
            .central_audit
            .read()
            .map_err(|_| AdminServerError::StateUnavailable)?;
        let mut filtered = filter_central_audit_entries(entries.iter().cloned(), &query);
        filtered.sort_by_key(|entry| std::cmp::Reverse(entry.created_at));
        Ok(page_central_audit_entries(filtered, query))
    }
}

impl SqliteAdminServerStore {
    pub async fn new(pool: SqlitePool) -> Result<Self, AdminServerError> {
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> Result<(), AdminServerError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS admin_agents (
                instance_id TEXT PRIMARY KEY NOT NULL,
                agent_secret TEXT NOT NULL,
                snapshot_payload TEXT,
                first_seen_at TEXT NOT NULL,
                last_seen_at TEXT
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS admin_machine_groups (
                id TEXT PRIMARY KEY NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS admin_server_commands (
                id TEXT PRIMARY KEY NOT NULL,
                source TEXT NOT NULL,
                request_payload TEXT NOT NULL,
                status TEXT NOT NULL,
                result_payload TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS admin_central_audit (
                id TEXT PRIMARY KEY NOT NULL,
                actor TEXT NOT NULL,
                role TEXT,
                action TEXT NOT NULL,
                target TEXT NOT NULL,
                status TEXT NOT NULL,
                message TEXT NOT NULL,
                details TEXT,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS admin_command_deliveries (
                command_id TEXT NOT NULL,
                target_instance_id TEXT NOT NULL,
                status TEXT NOT NULL,
                assigned_at TEXT,
                completed_at TEXT,
                message TEXT,
                PRIMARY KEY (command_id, target_instance_id)
            )",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_command_delivery_message_column().await?;
        Ok(())
    }

    async fn ensure_command_delivery_message_column(&self) -> Result<(), AdminServerError> {
        let columns: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM pragma_table_info('admin_command_deliveries')")
                .fetch_all(&self.pool)
                .await?;
        let has_message = columns.iter().any(|(name,)| name == "message");
        if !has_message {
            sqlx::query("ALTER TABLE admin_command_deliveries ADD COLUMN message TEXT")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn register_agent_secret(
        &self,
        instance_id: &str,
        secret: &str,
    ) -> Result<(), AdminServerError> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO admin_agents
                (instance_id, agent_secret, snapshot_payload, first_seen_at, last_seen_at)
             VALUES (?, ?, NULL, ?, NULL)
             ON CONFLICT(instance_id) DO UPDATE SET
                agent_secret = excluded.agent_secret",
        )
        .bind(instance_id)
        .bind(secret)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn agent_secret(&self, instance_id: &str) -> Result<String, AdminServerError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT agent_secret FROM admin_agents WHERE instance_id = ?")
                .bind(instance_id)
                .fetch_optional(&self.pool)
                .await?;

        row.map(|(secret,)| secret)
            .ok_or_else(|| AdminServerError::AgentNotRegistered(instance_id.into()))
    }

    async fn save_instance(
        &self,
        instance: &AdminInstanceSnapshot,
    ) -> Result<(), AdminServerError> {
        let payload = serde_json::to_string(instance)
            .map_err(|error| AdminServerError::Database(error.to_string()))?;
        let result = sqlx::query(
            "UPDATE admin_agents
             SET snapshot_payload = ?, last_seen_at = ?
             WHERE instance_id = ?",
        )
        .bind(payload)
        .bind(instance.last_seen_at.to_rfc3339())
        .bind(&instance.instance_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AdminServerError::AgentNotRegistered(
                instance.instance_id.clone(),
            ));
        }

        Ok(())
    }

    async fn list_instances(&self) -> Result<Vec<AdminInstanceSnapshot>, AdminServerError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT snapshot_payload
             FROM admin_agents
             WHERE snapshot_payload IS NOT NULL
             ORDER BY instance_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(payload,)| {
                serde_json::from_str(&payload)
                    .map_err(|error| AdminServerError::Database(error.to_string()))
            })
            .collect()
    }

    async fn save_machine_group(&self, group: &AdminMachineGroup) -> Result<(), AdminServerError> {
        let payload = serde_json::to_string(group)
            .map_err(|error| AdminServerError::Database(error.to_string()))?;
        sqlx::query(
            "INSERT INTO admin_machine_groups
                (id, payload, created_at, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                payload = excluded.payload,
                updated_at = excluded.updated_at",
        )
        .bind(group.id.to_string())
        .bind(payload)
        .bind(group.created_at.to_rfc3339())
        .bind(group.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_machine_groups(&self) -> Result<Vec<AdminMachineGroup>, AdminServerError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT payload
             FROM admin_machine_groups
             ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(payload,)| {
                serde_json::from_str(&payload)
                    .map_err(|error| AdminServerError::Database(error.to_string()))
            })
            .collect()
    }

    async fn save_batch_command(
        &self,
        command: &AdminQueuedCommand,
        target_instance_ids: &[String],
    ) -> Result<(), AdminServerError> {
        let request_payload = serde_json::to_string(&command.request)
            .map_err(|error| AdminServerError::Database(error.to_string()))?;
        let result_payload = command
            .result
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| AdminServerError::Database(error.to_string()))?;

        sqlx::query(
            "INSERT INTO admin_server_commands
                (id, source, request_payload, status, result_payload, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(command.id.to_string())
        .bind(&command.source)
        .bind(request_payload)
        .bind(command_status_to_wire(command.status))
        .bind(result_payload)
        .bind(command.created_at.to_rfc3339())
        .bind(command.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        for target_instance_id in target_instance_ids {
            sqlx::query(
                "INSERT INTO admin_command_deliveries
                    (command_id, target_instance_id, status, assigned_at, completed_at, message)
                 VALUES (?, ?, 'pending', NULL, NULL, NULL)
                 ON CONFLICT(command_id, target_instance_id) DO NOTHING",
            )
            .bind(command.id.to_string())
            .bind(target_instance_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn list_queued_commands(&self) -> Result<Vec<AdminQueuedCommand>, AdminServerError> {
        let rows: Vec<AdminQueuedCommandRow> = sqlx::query_as(
            "SELECT id, source, request_payload, status, result_payload, created_at, updated_at
             FROM admin_server_commands
             ORDER BY created_at DESC
             LIMIT 500",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(AdminQueuedCommand::try_from).collect()
    }

    async fn assign_next_command(
        &self,
        instance_id: &str,
        assigned_at: chrono::DateTime<Utc>,
    ) -> Result<Option<AdminCommandAssignment>, AdminServerError> {
        let row: Option<AdminQueuedCommandRow> = sqlx::query_as(
            "SELECT c.id, c.source, c.request_payload, c.status, c.result_payload, c.created_at, c.updated_at
             FROM admin_server_commands c
             INNER JOIN admin_command_deliveries d ON d.command_id = c.id
             WHERE d.target_instance_id = ? AND d.status = 'pending'
             ORDER BY c.created_at ASC
             LIMIT 1",
        )
        .bind(instance_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let mut command = AdminQueuedCommand::try_from(row)?;
        command.status = AdminQueuedCommandStatus::Running;
        command.updated_at = assigned_at;

        sqlx::query(
            "UPDATE admin_command_deliveries
             SET status = 'running', assigned_at = ?
             WHERE command_id = ? AND target_instance_id = ?",
        )
        .bind(assigned_at.to_rfc3339())
        .bind(command.id.to_string())
        .bind(instance_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE admin_server_commands
             SET status = 'running', updated_at = ?
             WHERE id = ?",
        )
        .bind(assigned_at.to_rfc3339())
        .bind(command.id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(Some(command_assignment(&command, instance_id, assigned_at)))
    }

    async fn pending_command_count(&self, instance_id: &str) -> Result<u32, AdminServerError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)
             FROM admin_command_deliveries
             WHERE target_instance_id = ? AND status = 'pending'",
        )
        .bind(instance_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0.max(0) as u32)
    }

    async fn record_command_completion(
        &self,
        instance_id: &str,
        command_id: Uuid,
        completion: AdminCommandCompletionRequest,
        recorded_at: chrono::DateTime<Utc>,
    ) -> Result<AdminQueuedCommand, AdminServerError> {
        let command_id_text = command_id.to_string();
        let current_status: Option<(String,)> = sqlx::query_as(
            "SELECT status
             FROM admin_command_deliveries
             WHERE command_id = ? AND target_instance_id = ?",
        )
        .bind(&command_id_text)
        .bind(instance_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((current_status,)) = current_status else {
            self.ensure_command_exists(command_id).await?;
            return Err(AdminServerError::CommandTargetNotFound(
                command_id,
                instance_id.into(),
            ));
        };
        if command_status_from_wire(&current_status)? != AdminQueuedCommandStatus::Running {
            return Err(AdminServerError::CommandNotRunning(
                command_id,
                instance_id.into(),
            ));
        }

        let message = completion_message(completion.status, &completion.message);
        let status = completion_status_to_command_status(completion.status);
        sqlx::query(
            "UPDATE admin_command_deliveries
             SET status = ?, completed_at = ?, message = ?
             WHERE command_id = ? AND target_instance_id = ?",
        )
        .bind(command_status_to_wire(status))
        .bind(completion.completed_at.to_rfc3339())
        .bind(message)
        .bind(&command_id_text)
        .bind(instance_id)
        .execute(&self.pool)
        .await?;

        let mut command = self.fetch_command(command_id).await?;
        let delivery_states = self.fetch_command_delivery_states(command_id).await?;
        command.status = aggregate_command_status(&delivery_states);
        command.result = command_result_from_deliveries(command.request.kind, &delivery_states);
        command.updated_at = recorded_at;

        let result_payload = command
            .result
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| AdminServerError::Database(error.to_string()))?;
        sqlx::query(
            "UPDATE admin_server_commands
             SET status = ?, result_payload = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(command_status_to_wire(command.status))
        .bind(result_payload)
        .bind(command.updated_at.to_rfc3339())
        .bind(&command_id_text)
        .execute(&self.pool)
        .await?;

        Ok(command)
    }

    async fn ensure_command_exists(&self, command_id: Uuid) -> Result<(), AdminServerError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT id
             FROM admin_server_commands
             WHERE id = ?",
        )
        .bind(command_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|_| ())
            .ok_or(AdminServerError::CommandNotFound(command_id))
    }

    async fn fetch_command(
        &self,
        command_id: Uuid,
    ) -> Result<AdminQueuedCommand, AdminServerError> {
        let row: Option<AdminQueuedCommandRow> = sqlx::query_as(
            "SELECT id, source, request_payload, status, result_payload, created_at, updated_at
             FROM admin_server_commands
             WHERE id = ?",
        )
        .bind(command_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or(AdminServerError::CommandNotFound(command_id))?
            .try_into()
    }

    async fn fetch_command_delivery_states(
        &self,
        command_id: Uuid,
    ) -> Result<Vec<AdminCommandDeliveryState>, AdminServerError> {
        let rows: Vec<AdminCommandDeliveryRow> = sqlx::query_as(
            "SELECT target_instance_id, status, message
             FROM admin_command_deliveries
             WHERE command_id = ?
             ORDER BY target_instance_id",
        )
        .bind(command_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        if rows.is_empty() {
            return Err(AdminServerError::CommandNotFound(command_id));
        }

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn append_central_audit(
        &self,
        entry: AdminCentralAuditEntry,
    ) -> Result<(), AdminServerError> {
        sqlx::query(
            "INSERT INTO admin_central_audit
                (id, actor, role, action, target, status, message, details, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entry.id.to_string())
        .bind(entry.actor)
        .bind(entry.role.map(operator_role_to_wire))
        .bind(entry.action)
        .bind(entry.target)
        .bind(central_audit_status_to_wire(entry.status))
        .bind(entry.message)
        .bind(entry.details)
        .bind(entry.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn query_central_audit(
        &self,
        query: AdminCentralAuditQuery,
    ) -> Result<AdminCentralAuditPage, AdminServerError> {
        let query = query.normalized();
        let rows: Vec<AdminCentralAuditRow> = sqlx::query_as(
            "SELECT id, actor, role, action, target, status, message, details, created_at
             FROM admin_central_audit
             ORDER BY created_at DESC
             LIMIT 5000",
        )
        .fetch_all(&self.pool)
        .await?;
        let entries: Result<Vec<AdminCentralAuditEntry>, AdminServerError> =
            rows.into_iter().map(TryInto::try_into).collect();
        let filtered = filter_central_audit_entries(entries?, &query);
        Ok(page_central_audit_entries(filtered, query))
    }
}

#[derive(sqlx::FromRow)]
struct AdminQueuedCommandRow {
    id: String,
    source: String,
    request_payload: String,
    status: String,
    result_payload: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct AdminCommandDeliveryRow {
    target_instance_id: String,
    status: String,
    message: Option<String>,
}

#[derive(sqlx::FromRow)]
struct AdminCentralAuditRow {
    id: String,
    actor: String,
    role: Option<String>,
    action: String,
    target: String,
    status: String,
    message: String,
    details: Option<String>,
    created_at: String,
}

impl TryFrom<AdminQueuedCommandRow> for AdminQueuedCommand {
    type Error = AdminServerError;

    fn try_from(row: AdminQueuedCommandRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&row.id)
                .map_err(|error| AdminServerError::Database(error.to_string()))?,
            source: row.source,
            request: serde_json::from_str(&row.request_payload)
                .map_err(|error| AdminServerError::Database(error.to_string()))?,
            status: command_status_from_wire(&row.status)?,
            result: row
                .result_payload
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|error| AdminServerError::Database(error.to_string()))?,
            created_at: parse_datetime(&row.created_at)?,
            updated_at: parse_datetime(&row.updated_at)?,
        })
    }
}

impl TryFrom<AdminCommandDeliveryRow> for AdminCommandDeliveryState {
    type Error = AdminServerError;

    fn try_from(row: AdminCommandDeliveryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            target_instance_id: row.target_instance_id,
            status: command_status_from_wire(&row.status)?,
            message: row.message,
        })
    }
}

impl TryFrom<AdminCentralAuditRow> for AdminCentralAuditEntry {
    type Error = AdminServerError;

    fn try_from(row: AdminCentralAuditRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&row.id)
                .map_err(|error| AdminServerError::Database(error.to_string()))?,
            actor: row.actor,
            role: row
                .role
                .as_deref()
                .map(operator_role_from_wire)
                .transpose()?,
            action: row.action,
            target: row.target,
            status: central_audit_status_from_wire(&row.status)?,
            message: row.message,
            details: row.details,
            created_at: parse_datetime(&row.created_at)?,
        })
    }
}

fn normalize_agent_secret(
    instance_id: impl Into<String>,
    secret: impl Into<String>,
) -> Result<(String, String), AdminServerError> {
    let instance_id = normalize_instance_id(&instance_id.into())?;
    let secret = normalize_required_secret(secret, AdminServerError::MissingAgentSecret)?;

    Ok((instance_id, secret))
}

fn normalize_group_name(name: String) -> Result<String, AdminServerError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AdminServerError::MissingMachineGroupName);
    }
    Ok(name)
}

fn normalize_instance_id(instance_id: &str) -> Result<String, AdminServerError> {
    let instance_id = instance_id.trim().to_string();
    if instance_id.is_empty() {
        return Err(AdminServerError::MissingInstanceId);
    }
    Ok(instance_id)
}

fn normalize_required_secret(
    secret: impl Into<String>,
    missing_error: AdminServerError,
) -> Result<String, AdminServerError> {
    let secret = secret.into().trim().to_string();
    if secret.is_empty() {
        return Err(missing_error);
    }
    Ok(secret)
}

fn normalize_instance_ids(instance_ids: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for instance_id in instance_ids {
        let instance_id = instance_id.trim().to_string();
        if instance_id.is_empty() || normalized.contains(&instance_id) {
            continue;
        }
        normalized.push(instance_id);
    }
    normalized
}

fn operator_role_allows(role: AdminOperatorRole, permission: AdminPermission) -> bool {
    matches!(
        (role, permission),
        (AdminOperatorRole::Admin, _)
            | (
                AdminOperatorRole::Operator,
                AdminPermission::View | AdminPermission::Operate
            )
            | (AdminOperatorRole::Viewer, AdminPermission::View)
    )
}

fn operator_actor(token: &str) -> String {
    let visible_suffix: String = token
        .chars()
        .rev()
        .take(6)
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();
    format!("operator:{}", visible_suffix)
}

fn operator_role_to_wire(role: AdminOperatorRole) -> &'static str {
    match role {
        AdminOperatorRole::Viewer => "viewer",
        AdminOperatorRole::Operator => "operator",
        AdminOperatorRole::Admin => "admin",
    }
}

fn operator_role_from_wire(role: &str) -> Result<AdminOperatorRole, AdminServerError> {
    match role {
        "viewer" => Ok(AdminOperatorRole::Viewer),
        "operator" => Ok(AdminOperatorRole::Operator),
        "admin" => Ok(AdminOperatorRole::Admin),
        other => Err(AdminServerError::Database(format!(
            "unknown admin operator role: {other}"
        ))),
    }
}

fn central_audit_status_to_wire(status: AdminCentralAuditStatus) -> &'static str {
    match status {
        AdminCentralAuditStatus::Accepted => "accepted",
        AdminCentralAuditStatus::Rejected => "rejected",
        AdminCentralAuditStatus::Failed => "failed",
    }
}

fn central_audit_status_from_wire(
    status: &str,
) -> Result<AdminCentralAuditStatus, AdminServerError> {
    match status {
        "accepted" => Ok(AdminCentralAuditStatus::Accepted),
        "rejected" => Ok(AdminCentralAuditStatus::Rejected),
        "failed" => Ok(AdminCentralAuditStatus::Failed),
        other => Err(AdminServerError::Database(format!(
            "unknown admin central audit status: {other}"
        ))),
    }
}

fn central_audit_entry(
    operator: &AdminOperatorContext,
    action: &str,
    target: &str,
    status: AdminCentralAuditStatus,
    message: &str,
    details: Option<serde_json::Value>,
) -> AdminCentralAuditEntry {
    AdminCentralAuditEntry {
        id: Uuid::new_v4(),
        actor: operator.actor.clone(),
        role: operator.role,
        action: action.into(),
        target: target.into(),
        status,
        message: message.into(),
        details: details.map(|value| value.to_string()),
        created_at: Utc::now(),
    }
}

fn agent_audit_context(instance_id: &str) -> AdminOperatorContext {
    AdminOperatorContext {
        actor: format!("agent:{instance_id}"),
        role: None,
    }
}

fn filter_central_audit_entries(
    entries: impl IntoIterator<Item = AdminCentralAuditEntry>,
    query: &AdminCentralAuditQuery,
) -> Vec<AdminCentralAuditEntry> {
    entries
        .into_iter()
        .filter(|entry| central_audit_entry_matches(entry, query))
        .collect()
}

fn central_audit_entry_matches(
    entry: &AdminCentralAuditEntry,
    query: &AdminCentralAuditQuery,
) -> bool {
    if let Some(actor) = &query.actor {
        if !entry.actor.eq_ignore_ascii_case(actor) {
            return false;
        }
    }
    if let Some(action) = &query.action {
        if !entry.action.eq_ignore_ascii_case(action) {
            return false;
        }
    }
    if let Some(status) = query.status {
        if entry.status != status {
            return false;
        }
    }
    if let Some(search) = &query.search {
        let search = search.to_ascii_lowercase();
        let haystack = [
            entry.actor.as_str(),
            entry.action.as_str(),
            entry.target.as_str(),
            entry.message.as_str(),
            entry.details.as_deref().unwrap_or_default(),
        ]
        .join(" ")
        .to_ascii_lowercase();
        if !haystack.contains(&search) {
            return false;
        }
    }
    true
}

fn page_central_audit_entries(
    entries: Vec<AdminCentralAuditEntry>,
    query: AdminCentralAuditQuery,
) -> AdminCentralAuditPage {
    let total = entries.len() as i64;
    let offset = query.offset as usize;
    let limit = query.limit as usize;
    let entries = entries.into_iter().skip(offset).take(limit).collect();
    AdminCentralAuditPage {
        entries,
        total,
        limit: query.limit,
        offset: query.offset,
    }
}

fn command_delivery_states(
    deliveries: &[AdminCommandDelivery],
    command_id: Uuid,
) -> Result<Vec<AdminCommandDeliveryState>, AdminServerError> {
    let states: Vec<AdminCommandDeliveryState> = deliveries
        .iter()
        .filter(|delivery| delivery.command_id == command_id)
        .map(|delivery| AdminCommandDeliveryState {
            target_instance_id: delivery.target_instance_id.clone(),
            status: delivery.status,
            message: delivery.message.clone(),
        })
        .collect();
    if states.is_empty() {
        return Err(AdminServerError::CommandNotFound(command_id));
    }
    Ok(states)
}

fn completion_status_to_command_status(
    status: AdminCommandCompletionStatus,
) -> AdminQueuedCommandStatus {
    match status {
        AdminCommandCompletionStatus::Completed => AdminQueuedCommandStatus::Completed,
        AdminCommandCompletionStatus::Failed => AdminQueuedCommandStatus::Failed,
        AdminCommandCompletionStatus::Skipped => AdminQueuedCommandStatus::Skipped,
    }
}

fn completion_message(status: AdminCommandCompletionStatus, message: &str) -> String {
    let message = message.trim();
    if !message.is_empty() {
        return message.to_string();
    }

    match status {
        AdminCommandCompletionStatus::Completed => "Comando concluido".into(),
        AdminCommandCompletionStatus::Failed => "Comando falhou".into(),
        AdminCommandCompletionStatus::Skipped => "Comando ignorado".into(),
    }
}

fn aggregate_command_status(states: &[AdminCommandDeliveryState]) -> AdminQueuedCommandStatus {
    if states.iter().any(|state| {
        matches!(
            state.status,
            AdminQueuedCommandStatus::Pending | AdminQueuedCommandStatus::Running
        )
    }) {
        return AdminQueuedCommandStatus::Running;
    }
    if states
        .iter()
        .any(|state| state.status == AdminQueuedCommandStatus::Failed)
    {
        return AdminQueuedCommandStatus::Failed;
    }
    if states
        .iter()
        .all(|state| state.status == AdminQueuedCommandStatus::Skipped)
    {
        return AdminQueuedCommandStatus::Skipped;
    }
    AdminQueuedCommandStatus::Completed
}

fn command_result_from_deliveries(
    command: autoflow_domain::AdminCommandKind,
    states: &[AdminCommandDeliveryState],
) -> Option<AdminCommandResult> {
    let results: Vec<AdminCommandTargetResult> = states
        .iter()
        .filter_map(command_target_result_from_delivery)
        .collect();
    if results.is_empty() {
        return None;
    }

    Some(AdminCommandResult {
        accepted: !results
            .iter()
            .any(|result| result.status == AdminCommandTargetStatus::Error),
        command,
        results,
    })
}

fn command_target_result_from_delivery(
    delivery: &AdminCommandDeliveryState,
) -> Option<AdminCommandTargetResult> {
    let status = match delivery.status {
        AdminQueuedCommandStatus::Completed => AdminCommandTargetStatus::Accepted,
        AdminQueuedCommandStatus::Failed => AdminCommandTargetStatus::Error,
        AdminQueuedCommandStatus::Skipped => AdminCommandTargetStatus::Skipped,
        AdminQueuedCommandStatus::Pending | AdminQueuedCommandStatus::Running => return None,
    };

    Some(AdminCommandTargetResult {
        target_instance_id: delivery.target_instance_id.clone(),
        status,
        message: delivery
            .message
            .clone()
            .unwrap_or_else(|| "Comando finalizado".into()),
    })
}

fn queued_batch_result(
    request: &autoflow_domain::AdminCommandRequest,
    target_instance_ids: &[String],
) -> AdminCommandResult {
    AdminCommandResult {
        accepted: true,
        command: request.kind,
        results: target_instance_ids
            .iter()
            .map(|target_instance_id| AdminCommandTargetResult {
                target_instance_id: target_instance_id.clone(),
                status: AdminCommandTargetStatus::Accepted,
                message: "Comando enfileirado no servidor admin".into(),
            })
            .collect(),
    }
}

fn command_assignment(
    command: &AdminQueuedCommand,
    target_instance_id: &str,
    assigned_at: chrono::DateTime<Utc>,
) -> AdminCommandAssignment {
    let mut request = command.request.clone();
    request.target_instance_ids = vec![target_instance_id.to_string()];

    AdminCommandAssignment {
        command_id: command.id,
        target_instance_id: target_instance_id.to_string(),
        request,
        assigned_at,
    }
}

fn command_status_to_wire(status: AdminQueuedCommandStatus) -> &'static str {
    match status {
        AdminQueuedCommandStatus::Pending => "pending",
        AdminQueuedCommandStatus::Running => "running",
        AdminQueuedCommandStatus::Completed => "completed",
        AdminQueuedCommandStatus::Failed => "failed",
        AdminQueuedCommandStatus::Skipped => "skipped",
    }
}

fn command_status_from_wire(status: &str) -> Result<AdminQueuedCommandStatus, AdminServerError> {
    match status {
        "pending" => Ok(AdminQueuedCommandStatus::Pending),
        "running" => Ok(AdminQueuedCommandStatus::Running),
        "completed" => Ok(AdminQueuedCommandStatus::Completed),
        "failed" => Ok(AdminQueuedCommandStatus::Failed),
        "skipped" => Ok(AdminQueuedCommandStatus::Skipped),
        other => Err(AdminServerError::Database(format!(
            "unknown admin command status: {other}"
        ))),
    }
}

fn parse_datetime(raw: &str) -> Result<chrono::DateTime<Utc>, AdminServerError> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| AdminServerError::Database(error.to_string()))
}

pub fn router(state: AdminServerState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/fleet", get(fleet_snapshot))
        .route("/api/admin-audit", get(query_central_audit))
        .route(
            "/api/machine-groups",
            get(list_machine_groups).post(create_machine_group),
        )
        .route("/api/admin-commands", get(list_queued_commands))
        .route("/api/admin-commands/batch", post(enqueue_batch_command))
        .route("/api/enrollments", post(enroll_agent))
        .route(
            "/api/agents/{instance_id}/commands/next",
            post(poll_next_command),
        )
        .route(
            "/api/agents/{instance_id}/commands/{command_id}/completion",
            post(complete_command),
        )
        .route(
            "/api/agents/{instance_id}/heartbeat",
            post(receive_heartbeat),
        )
        .route(
            "/api/agents/{instance_id}/secret-rotation",
            post(rotate_agent_secret),
        )
        .with_state(state)
}

async fn health() -> Json<AdminServerHealth> {
    Json(AdminServerHealth::ok(env!("CARGO_PKG_VERSION")))
}

async fn fleet_snapshot(
    State(state): State<AdminServerState>,
    headers: HeaderMap,
) -> Result<Json<AdminFleetSnapshot>, AdminServerError> {
    state.authorize_operator(&headers, AdminPermission::View)?;
    Ok(Json(state.fleet_snapshot().await?))
}

async fn list_machine_groups(
    State(state): State<AdminServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AdminMachineGroup>>, AdminServerError> {
    state.authorize_operator(&headers, AdminPermission::View)?;
    Ok(Json(state.list_machine_groups().await?))
}

async fn create_machine_group(
    State(state): State<AdminServerState>,
    headers: HeaderMap,
    Json(request): Json<AdminMachineGroupRequest>,
) -> Result<(StatusCode, Json<AdminMachineGroup>), AdminServerError> {
    let operator = state.authorize_operator(&headers, AdminPermission::Operate)?;
    let group = state.create_machine_group(request).await?;
    state
        .append_central_audit(central_audit_entry(
            &operator,
            "machineGroup.create",
            &group.id.to_string(),
            AdminCentralAuditStatus::Accepted,
            "Grupo de maquinas criado",
            Some(serde_json::json!({
                "name": &group.name,
                "instanceIds": &group.instance_ids,
            })),
        ))
        .await?;
    Ok((StatusCode::CREATED, Json(group)))
}

async fn enqueue_batch_command(
    State(state): State<AdminServerState>,
    headers: HeaderMap,
    Json(request): Json<AdminBatchCommandRequest>,
) -> Result<(StatusCode, Json<AdminBatchCommandAccepted>), AdminServerError> {
    let operator = state.authorize_operator(&headers, AdminPermission::Operate)?;
    let accepted = state.enqueue_batch_command(request).await?;
    state
        .append_central_audit(central_audit_entry(
            &operator,
            "command.batch.enqueue",
            &accepted.command.id.to_string(),
            AdminCentralAuditStatus::Accepted,
            "Comando em lote enfileirado",
            Some(serde_json::json!({
                "source": &accepted.command.source,
                "kind": accepted.command.request.kind,
                "targets": &accepted.resolved_target_instance_ids,
            })),
        ))
        .await?;
    Ok((StatusCode::ACCEPTED, Json(accepted)))
}

async fn list_queued_commands(
    State(state): State<AdminServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AdminQueuedCommand>>, AdminServerError> {
    state.authorize_operator(&headers, AdminPermission::View)?;
    Ok(Json(state.list_queued_commands().await?))
}

async fn query_central_audit(
    State(state): State<AdminServerState>,
    headers: HeaderMap,
    Query(query): Query<AdminCentralAuditQuery>,
) -> Result<Json<AdminCentralAuditPage>, AdminServerError> {
    state.authorize_operator(&headers, AdminPermission::Administer)?;
    Ok(Json(state.query_central_audit(query).await?))
}

async fn poll_next_command(
    State(state): State<AdminServerState>,
    Path(instance_id): Path<String>,
    Json(envelope): Json<AdminSignedEnvelope<AdminCommandPollRequest>>,
) -> Result<Json<AdminCommandPollResponse>, AdminServerError> {
    Ok(Json(state.poll_next_command(&instance_id, envelope).await?))
}

async fn complete_command(
    State(state): State<AdminServerState>,
    Path((instance_id, command_id)): Path<(String, Uuid)>,
    Json(envelope): Json<AdminSignedEnvelope<AdminCommandCompletionRequest>>,
) -> Result<(StatusCode, Json<AdminCommandCompletionAccepted>), AdminServerError> {
    let accepted = state
        .complete_command(&instance_id, command_id, envelope)
        .await?;
    state
        .append_central_audit(central_audit_entry(
            &agent_audit_context(&instance_id),
            "command.complete",
            &command_id.to_string(),
            AdminCentralAuditStatus::Accepted,
            "Agent registrou resultado de comando",
            Some(serde_json::json!({
                "instanceId": instance_id,
                "status": accepted.command.status,
                "result": &accepted.command.result,
            })),
        ))
        .await?;
    Ok((StatusCode::ACCEPTED, Json(accepted)))
}

async fn enroll_agent(
    State(state): State<AdminServerState>,
    Json(request): Json<AdminEnrollmentTokenRequest>,
) -> Result<(StatusCode, Json<AdminEnrollmentResponse>), AdminServerError> {
    let response = state.enroll_agent(request).await?;
    state
        .append_central_audit(central_audit_entry(
            &agent_audit_context(&response.instance_id),
            "agent.enroll",
            &response.instance_id,
            AdminCentralAuditStatus::Accepted,
            "Agent matriculado por token",
            None,
        ))
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn receive_heartbeat(
    State(state): State<AdminServerState>,
    Path(instance_id): Path<String>,
    Json(envelope): Json<AdminSignedEnvelope<AdminHeartbeatPayload>>,
) -> Result<(StatusCode, Json<AdminHeartbeatAccepted>), AdminServerError> {
    let accepted = state.receive_heartbeat(&instance_id, envelope).await?;
    Ok((StatusCode::ACCEPTED, Json(accepted)))
}

async fn rotate_agent_secret(
    State(state): State<AdminServerState>,
    Path(instance_id): Path<String>,
    Json(envelope): Json<AdminSignedEnvelope<AdminAgentSecretRotationRequest>>,
) -> Result<(StatusCode, Json<AdminAgentSecretRotationAccepted>), AdminServerError> {
    let accepted = state.rotate_agent_secret(&instance_id, envelope).await?;
    state
        .append_central_audit(central_audit_entry(
            &agent_audit_context(&instance_id),
            "agent.secret.rotate",
            &instance_id,
            AdminCentralAuditStatus::Accepted,
            "Segredo do agent rotacionado",
            None,
        ))
        .await?;
    Ok((StatusCode::OK, Json(accepted)))
}

#[derive(Debug, Error)]
pub enum AdminServerError {
    #[error("instance id is required")]
    MissingInstanceId,
    #[error("agent secret is required")]
    MissingAgentSecret,
    #[error("enrollment token is required")]
    MissingEnrollmentToken,
    #[error("operator token is required")]
    MissingOperatorToken,
    #[error("enrollment token is not configured")]
    EnrollmentNotConfigured,
    #[error("enrollment token is invalid")]
    InvalidEnrollmentToken,
    #[error("operator authentication is required")]
    OperatorAuthRequired,
    #[error("operator role is not allowed for this action")]
    OperatorForbidden,
    #[error("agent is not registered: {0}")]
    AgentNotRegistered(String),
    #[error("machine group name is required")]
    MissingMachineGroupName,
    #[error("machine group not found: {0}")]
    MachineGroupNotFound(Uuid),
    #[error("batch command needs at least one target instance")]
    NoBatchTargets,
    #[error("admin command not found: {0}")]
    CommandNotFound(Uuid),
    #[error("admin command completion payload does not match the route")]
    CommandPayloadMismatch,
    #[error("admin command completion target does not match the route")]
    CommandTargetMismatch,
    #[error("admin command target not found: {0} / {1}")]
    CommandTargetNotFound(Uuid, String),
    #[error("admin command target is not running: {0} / {1}")]
    CommandNotRunning(Uuid, String),
    #[error("envelope kind is not valid for this endpoint")]
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
    #[error("admin server database error: {0}")]
    Database(String),
    #[error("admin server state is unavailable")]
    StateUnavailable,
}

impl From<autoflow_domain::DomainError> for AdminServerError {
    fn from(error: autoflow_domain::DomainError) -> Self {
        Self::EnvelopeValidation(error.to_string())
    }
}

impl From<sqlx::Error> for AdminServerError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error.to_string())
    }
}

impl AdminServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingInstanceId
            | Self::MissingAgentSecret
            | Self::MissingEnrollmentToken
            | Self::MissingOperatorToken
            | Self::MissingMachineGroupName
            | Self::NoBatchTargets
            | Self::CommandPayloadMismatch
            | Self::CommandTargetMismatch
            | Self::InvalidEnvelopeKind
            | Self::EnvelopeInstanceMismatch
            | Self::PayloadInstanceMismatch
            | Self::ExpiredEnvelope
            | Self::EnvelopeValidation(_) => StatusCode::BAD_REQUEST,
            Self::AgentNotRegistered(_)
            | Self::CommandNotFound(_)
            | Self::CommandTargetNotFound(_, _)
            | Self::MachineGroupNotFound(_) => StatusCode::NOT_FOUND,
            Self::CommandNotRunning(_, _) => StatusCode::CONFLICT,
            Self::EnrollmentNotConfigured
            | Self::InvalidEnrollmentToken
            | Self::OperatorAuthRequired
            | Self::InvalidSignature => StatusCode::UNAUTHORIZED,
            Self::OperatorForbidden => StatusCode::FORBIDDEN,
            Self::Database(_) | Self::StateUnavailable => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::MissingInstanceId => "missingInstanceId",
            Self::MissingAgentSecret => "missingAgentSecret",
            Self::MissingEnrollmentToken => "missingEnrollmentToken",
            Self::MissingOperatorToken => "missingOperatorToken",
            Self::EnrollmentNotConfigured => "enrollmentNotConfigured",
            Self::InvalidEnrollmentToken => "invalidEnrollmentToken",
            Self::OperatorAuthRequired => "operatorAuthRequired",
            Self::OperatorForbidden => "operatorForbidden",
            Self::AgentNotRegistered(_) => "agentNotRegistered",
            Self::MissingMachineGroupName => "missingMachineGroupName",
            Self::MachineGroupNotFound(_) => "machineGroupNotFound",
            Self::NoBatchTargets => "noBatchTargets",
            Self::CommandNotFound(_) => "commandNotFound",
            Self::CommandPayloadMismatch => "commandPayloadMismatch",
            Self::CommandTargetMismatch => "commandTargetMismatch",
            Self::CommandTargetNotFound(_, _) => "commandTargetNotFound",
            Self::CommandNotRunning(_, _) => "commandNotRunning",
            Self::InvalidEnvelopeKind => "invalidEnvelopeKind",
            Self::EnvelopeInstanceMismatch => "envelopeInstanceMismatch",
            Self::PayloadInstanceMismatch => "payloadInstanceMismatch",
            Self::ExpiredEnvelope => "expiredEnvelope",
            Self::InvalidSignature => "invalidSignature",
            Self::EnvelopeValidation(_) => "envelopeValidation",
            Self::Database(_) => "database",
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
        AdminAgentMode, AdminAgentSecretRotationRequest, AdminBatchCommandRequest,
        AdminCommandKind, AdminCommandRequest, AdminEnrollmentTokenRequest, AdminHardwareProfile,
        AdminInstanceStatus, AdminJobRuntimeStatus, AdminMachineGroupRequest, AdminManagedJob,
        AdminManagementProfile, AdminNetworkProfile, JobMode,
    };
    use axum::{body::Body, http::Request};
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn accepts_signed_heartbeat_and_updates_fleet_snapshot() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "shared-secret")
            .await
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

        let snapshot = state.fleet_snapshot().await.unwrap();
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
        assert_eq!(
            state
                .fleet_snapshot()
                .await
                .unwrap()
                .summary
                .total_instances,
            0
        );
    }

    #[tokio::test]
    async fn rejects_heartbeat_with_wrong_signature() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "shared-secret")
            .await
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
        assert_eq!(
            state
                .fleet_snapshot()
                .await
                .unwrap()
                .summary
                .total_instances,
            0
        );
    }

    #[tokio::test]
    async fn rejects_heartbeat_when_path_and_envelope_do_not_match() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "shared-secret")
            .await
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
        assert_eq!(
            state
                .fleet_snapshot()
                .await
                .unwrap()
                .summary
                .total_instances,
            0
        );
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

    #[tokio::test]
    async fn enrolls_agent_with_valid_token() {
        let state = AdminServerState::new();
        state.register_enrollment_token("invite-token").unwrap();
        let app = router(state.clone());
        let request = enrollment_request("invite-token", "local-1", "shared-secret");

        let response = app
            .oneshot(json_post(
                "/api/enrollments",
                serde_json::to_vec(&request).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let accepted = parse_body::<AdminEnrollmentResponse>(response).await;
        assert!(accepted.accepted);
        assert_eq!(accepted.instance_id, "local-1");

        let snapshot = state.fleet_snapshot().await.unwrap();
        assert_eq!(snapshot.summary.total_instances, 1);

        state
            .receive_heartbeat("local-1", signed_heartbeat("local-1", "shared-secret", 300))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn creates_group_and_enqueues_batch_command() {
        let state = AdminServerState::new();
        let app = router(state.clone());
        let group_request =
            machine_group_request("Financeiro", vec!["local-1", "local-2", "local-1"]);

        let group_response = app
            .clone()
            .oneshot(json_post(
                "/api/machine-groups",
                serde_json::to_vec(&group_request).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(group_response.status(), StatusCode::CREATED);
        let group = parse_body::<AdminMachineGroup>(group_response).await;
        assert_eq!(group.instance_ids, vec!["local-1", "local-2"]);

        let batch_response = app
            .clone()
            .oneshot(json_post(
                "/api/admin-commands/batch",
                serde_json::to_vec(&batch_request(group.id, vec!["local-3"])).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(batch_response.status(), StatusCode::ACCEPTED);
        let accepted = parse_body::<AdminBatchCommandAccepted>(batch_response).await;
        assert_eq!(
            accepted.resolved_target_instance_ids,
            vec!["local-3", "local-1", "local-2"]
        );
        assert_eq!(accepted.command.status, AdminQueuedCommandStatus::Pending);
        assert!(accepted.result.accepted);

        let commands_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/admin-commands")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let commands = parse_body::<Vec<AdminQueuedCommand>>(commands_response).await;

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].request.target_instance_ids.len(), 3);
    }

    #[tokio::test]
    async fn enforces_operator_rbac_and_records_central_audit() {
        let state = AdminServerState::new();
        state
            .register_operator_token("viewer-token", AdminOperatorRole::Viewer)
            .unwrap();
        state
            .register_operator_token("operator-token", AdminOperatorRole::Operator)
            .unwrap();
        state
            .register_operator_token("admin-token", AdminOperatorRole::Admin)
            .unwrap();
        let app = router(state.clone());
        let group_request = machine_group_request("Financeiro", vec!["local-1"]);

        let unauthorized = app
            .clone()
            .oneshot(json_post(
                "/api/machine-groups",
                serde_json::to_vec(&group_request).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let forbidden = app
            .clone()
            .oneshot(json_post_with_operator(
                "/api/admin-commands/batch",
                serde_json::to_vec(&batch_request(Uuid::new_v4(), vec!["local-1"])).unwrap(),
                "viewer-token",
            ))
            .await
            .unwrap();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

        let group_response = app
            .clone()
            .oneshot(json_post_with_operator(
                "/api/machine-groups",
                serde_json::to_vec(&group_request).unwrap(),
                "operator-token",
            ))
            .await
            .unwrap();
        assert_eq!(group_response.status(), StatusCode::CREATED);
        let group = parse_body::<AdminMachineGroup>(group_response).await;

        let batch_response = app
            .clone()
            .oneshot(json_post_with_operator(
                "/api/admin-commands/batch",
                serde_json::to_vec(&batch_request(group.id, Vec::new())).unwrap(),
                "operator-token",
            ))
            .await
            .unwrap();
        assert_eq!(batch_response.status(), StatusCode::ACCEPTED);

        let operator_audit = app
            .clone()
            .oneshot(get_with_operator("/api/admin-audit", "operator-token"))
            .await
            .unwrap();
        assert_eq!(operator_audit.status(), StatusCode::FORBIDDEN);

        let audit = app
            .oneshot(get_with_operator("/api/admin-audit", "admin-token"))
            .await
            .unwrap();
        assert_eq!(audit.status(), StatusCode::OK);
        let audit = parse_body::<AdminCentralAuditPage>(audit).await;
        assert_eq!(audit.total, 2);
        assert!(audit
            .entries
            .iter()
            .any(|entry| entry.action == "command.batch.enqueue"));
    }

    #[tokio::test]
    async fn agent_polls_signed_pending_command_assignment() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "agent-secret-1")
            .await
            .unwrap();
        state
            .register_agent_secret("local-2", "agent-secret-2")
            .await
            .unwrap();
        let app = router(state.clone());
        let group = state
            .create_machine_group(machine_group_request(
                "Operacao",
                vec!["local-1", "local-2"],
            ))
            .await
            .unwrap();
        state
            .enqueue_batch_command(batch_request(group.id, Vec::new()))
            .await
            .unwrap();

        let response = app
            .oneshot(json_post(
                "/api/agents/local-1/commands/next",
                serde_json::to_vec(&signed_command_poll("local-1", "agent-secret-1")).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let poll = parse_body::<AdminCommandPollResponse>(response).await;
        let assignment = poll.assignment.expect("agent should receive one command");
        assert!(assignment.verify("agent-secret-1").unwrap());
        assert_eq!(assignment.kind, AdminEnvelopeKind::Command);
        assert_eq!(assignment.payload.target_instance_id, "local-1");
        assert_eq!(
            assignment.payload.request.target_instance_ids,
            vec!["local-1"]
        );
        assert_eq!(poll.pending_count, 0);
    }

    #[tokio::test]
    async fn agent_reports_signed_command_completion() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "agent-secret-1")
            .await
            .unwrap();
        state
            .register_agent_secret("local-2", "agent-secret-2")
            .await
            .unwrap();
        let app = router(state.clone());
        let group = state
            .create_machine_group(machine_group_request(
                "Operacao",
                vec!["local-1", "local-2"],
            ))
            .await
            .unwrap();
        state
            .enqueue_batch_command(batch_request(group.id, Vec::new()))
            .await
            .unwrap();

        let first_poll_response = app
            .clone()
            .oneshot(json_post(
                "/api/agents/local-1/commands/next",
                serde_json::to_vec(&signed_command_poll("local-1", "agent-secret-1")).unwrap(),
            ))
            .await
            .unwrap();
        let first_poll = parse_body::<AdminCommandPollResponse>(first_poll_response).await;
        let first_assignment = first_poll.assignment.unwrap().payload;
        let command_id = first_assignment.command_id;

        let first_completion_uri = format!("/api/agents/local-1/commands/{command_id}/completion");
        let first_completion_response = app
            .clone()
            .oneshot(json_post(
                &first_completion_uri,
                serde_json::to_vec(&signed_command_completion(
                    "local-1",
                    "agent-secret-1",
                    command_id,
                    AdminCommandCompletionStatus::Completed,
                    "logs enviados",
                ))
                .unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(first_completion_response.status(), StatusCode::ACCEPTED);
        let first_completion =
            parse_body::<AdminCommandCompletionAccepted>(first_completion_response).await;
        assert_eq!(
            first_completion.command.status,
            AdminQueuedCommandStatus::Running
        );
        assert_eq!(
            first_completion.command.result.unwrap().results[0].target_instance_id,
            "local-1"
        );

        let second_poll_response = app
            .clone()
            .oneshot(json_post(
                "/api/agents/local-2/commands/next",
                serde_json::to_vec(&signed_command_poll("local-2", "agent-secret-2")).unwrap(),
            ))
            .await
            .unwrap();
        let second_poll = parse_body::<AdminCommandPollResponse>(second_poll_response).await;
        assert_eq!(
            second_poll.assignment.unwrap().payload.command_id,
            command_id
        );

        let second_completion_uri = format!("/api/agents/local-2/commands/{command_id}/completion");
        let second_completion_response = app
            .oneshot(json_post(
                &second_completion_uri,
                serde_json::to_vec(&signed_command_completion(
                    "local-2",
                    "agent-secret-2",
                    command_id,
                    AdminCommandCompletionStatus::Failed,
                    "falha ao coletar logs",
                ))
                .unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(second_completion_response.status(), StatusCode::ACCEPTED);
        let second_completion =
            parse_body::<AdminCommandCompletionAccepted>(second_completion_response).await;
        assert_eq!(
            second_completion.command.status,
            AdminQueuedCommandStatus::Failed
        );
        let result = second_completion.command.result.unwrap();
        assert!(!result.accepted);
        assert_eq!(result.results.len(), 2);
        assert!(result
            .results
            .iter()
            .any(|target| target.status == AdminCommandTargetStatus::Error));
    }

    #[tokio::test]
    async fn rejects_command_poll_with_wrong_signature() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "agent-secret")
            .await
            .unwrap();
        let app = router(state);

        let response = app
            .oneshot(json_post(
                "/api/agents/local-1/commands/next",
                serde_json::to_vec(&signed_command_poll("local-1", "wrong-secret")).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_batch_command_without_targets() {
        let state = AdminServerState::new();
        let app = router(state);
        let request = AdminBatchCommandRequest {
            request: AdminCommandRequest {
                kind: AdminCommandKind::RequestLogs,
                target_instance_ids: Vec::new(),
                job_ids: Vec::new(),
                execution_ids: Vec::new(),
                reason: None,
            },
            group_ids: Vec::new(),
            source: Some("test".into()),
        };

        let response = app
            .oneshot(json_post(
                "/api/admin-commands/batch",
                serde_json::to_vec(&request).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_enrollment_with_invalid_token() {
        let state = AdminServerState::new();
        state.register_enrollment_token("invite-token").unwrap();
        let app = router(state.clone());
        let request = enrollment_request("wrong-token", "local-1", "shared-secret");

        let response = app
            .oneshot(json_post(
                "/api/enrollments",
                serde_json::to_vec(&request).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            state
                .fleet_snapshot()
                .await
                .unwrap()
                .summary
                .total_instances,
            0
        );
    }

    #[tokio::test]
    async fn sqlite_state_persists_agent_snapshot() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let state = AdminServerState::with_sqlite_pool(pool.clone())
            .await
            .unwrap();
        state
            .register_agent_secret("local-1", "shared-secret")
            .await
            .unwrap();

        state
            .receive_heartbeat("local-1", signed_heartbeat("local-1", "shared-secret", 300))
            .await
            .unwrap();

        let restored = AdminServerState::with_sqlite_pool(pool).await.unwrap();
        let snapshot = restored.fleet_snapshot().await.unwrap();

        assert_eq!(snapshot.summary.total_instances, 1);
        assert_eq!(snapshot.instances[0].instance_id, "local-1");
    }

    #[tokio::test]
    async fn sqlite_state_persists_groups_and_batch_commands() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let state = AdminServerState::with_sqlite_pool(pool.clone())
            .await
            .unwrap();
        let group = state
            .create_machine_group(machine_group_request("Operacao", vec!["local-1"]))
            .await
            .unwrap();
        state
            .enqueue_batch_command(batch_request(group.id, vec!["local-2"]))
            .await
            .unwrap();

        let restored = AdminServerState::with_sqlite_pool(pool).await.unwrap();
        let groups = restored.list_machine_groups().await.unwrap();
        let commands = restored.list_queued_commands().await.unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0].request.target_instance_ids,
            vec!["local-2", "local-1"]
        );
    }

    #[tokio::test]
    async fn sqlite_state_persists_command_completion() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let state = AdminServerState::with_sqlite_pool(pool.clone())
            .await
            .unwrap();
        state
            .register_agent_secret("local-1", "agent-secret")
            .await
            .unwrap();
        let group = state
            .create_machine_group(machine_group_request("Operacao", vec!["local-1"]))
            .await
            .unwrap();
        state
            .enqueue_batch_command(batch_request(group.id, Vec::new()))
            .await
            .unwrap();

        let poll = state
            .poll_next_command("local-1", signed_command_poll("local-1", "agent-secret"))
            .await
            .unwrap();
        let command_id = poll.assignment.unwrap().payload.command_id;
        state
            .complete_command(
                "local-1",
                command_id,
                signed_command_completion(
                    "local-1",
                    "agent-secret",
                    command_id,
                    AdminCommandCompletionStatus::Completed,
                    "logs enviados",
                ),
            )
            .await
            .unwrap();

        let restored = AdminServerState::with_sqlite_pool(pool).await.unwrap();
        let commands = restored.list_queued_commands().await.unwrap();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].status, AdminQueuedCommandStatus::Completed);
        let result = commands[0].result.as_ref().unwrap();
        assert!(result.accepted);
        assert_eq!(result.results[0].message, "logs enviados");
    }

    #[tokio::test]
    async fn rotates_agent_secret_with_signed_request() {
        let state = AdminServerState::new();
        state
            .register_agent_secret("local-1", "old-secret")
            .await
            .unwrap();
        let app = router(state.clone());
        let envelope = signed_secret_rotation("local-1", "old-secret", "new-secret");

        let response = app
            .oneshot(json_post(
                "/api/agents/local-1/secret-rotation",
                serde_json::to_vec(&envelope).unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let accepted = parse_body::<AdminAgentSecretRotationAccepted>(response).await;
        assert!(accepted.accepted);
        assert_eq!(accepted.instance_id, "local-1");

        assert!(matches!(
            state
                .receive_heartbeat("local-1", signed_heartbeat("local-1", "old-secret", 300))
                .await,
            Err(AdminServerError::InvalidSignature)
        ));
        state
            .receive_heartbeat("local-1", signed_heartbeat("local-1", "new-secret", 300))
            .await
            .unwrap();
    }

    fn enrollment_request(
        token: &str,
        instance_id: &str,
        agent_secret: &str,
    ) -> AdminEnrollmentTokenRequest {
        AdminEnrollmentTokenRequest {
            token: token.into(),
            instance: sample_instance(instance_id),
            agent_secret: agent_secret.into(),
            requested_at: Utc::now(),
        }
    }

    fn machine_group_request(name: &str, instance_ids: Vec<&str>) -> AdminMachineGroupRequest {
        AdminMachineGroupRequest {
            name: name.into(),
            description: None,
            instance_ids: instance_ids.into_iter().map(str::to_string).collect(),
        }
    }

    fn batch_request(group_id: Uuid, target_instance_ids: Vec<&str>) -> AdminBatchCommandRequest {
        AdminBatchCommandRequest {
            request: AdminCommandRequest {
                kind: AdminCommandKind::RequestLogs,
                target_instance_ids: target_instance_ids
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                job_ids: Vec::new(),
                execution_ids: Vec::new(),
                reason: Some("auditoria".into()),
            },
            group_ids: vec![group_id],
            source: Some("test".into()),
        }
    }

    fn signed_command_poll(
        instance_id: &str,
        secret: &str,
    ) -> AdminSignedEnvelope<AdminCommandPollRequest> {
        AdminSignedEnvelope::sign(
            AdminEnvelopeKind::Command,
            instance_id.into(),
            AdminCommandPollRequest {
                requested_at: Utc::now(),
            },
            secret,
            300,
        )
        .unwrap()
    }

    fn signed_command_completion(
        instance_id: &str,
        secret: &str,
        command_id: Uuid,
        status: AdminCommandCompletionStatus,
        message: &str,
    ) -> AdminSignedEnvelope<AdminCommandCompletionRequest> {
        AdminSignedEnvelope::sign(
            AdminEnvelopeKind::Command,
            instance_id.into(),
            AdminCommandCompletionRequest {
                command_id,
                target_instance_id: instance_id.into(),
                status,
                message: message.into(),
                completed_at: Utc::now(),
            },
            secret,
            300,
        )
        .unwrap()
    }

    fn signed_secret_rotation(
        instance_id: &str,
        current_secret: &str,
        new_secret: &str,
    ) -> AdminSignedEnvelope<AdminAgentSecretRotationRequest> {
        let payload = AdminAgentSecretRotationRequest {
            new_agent_secret: new_secret.into(),
            requested_at: Utc::now(),
        };

        AdminSignedEnvelope::sign(
            AdminEnvelopeKind::SecretRotation,
            instance_id.into(),
            payload,
            current_secret,
            300,
        )
        .unwrap()
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

    fn json_post_with_operator(uri: &str, body: Vec<u8>, token: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .header(ADMIN_OPERATOR_TOKEN_HEADER, token)
            .body(Body::from(body))
            .unwrap()
    }

    fn get_with_operator(uri: &str, token: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .header(ADMIN_OPERATOR_TOKEN_HEADER, token)
            .body(Body::empty())
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
