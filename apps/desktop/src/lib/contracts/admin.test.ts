import { describe, expect, it } from "vitest";
import {
  adminCommandRequestSchema,
  adminCommandResultSchema,
  adminAgentSecretRotationAcceptedSchema,
  adminBatchCommandAcceptedSchema,
  adminBatchCommandRequestSchema,
  adminCentralAuditPageSchema,
  adminCentralAuditQuerySchema,
  adminCommandCompletionAcceptedSchema,
  adminCommandPollResponseSchema,
  adminEnrollmentTokenRequestSchema,
  adminHeartbeatDeliverySchema,
  adminHeartbeatPayloadSchema,
  adminMachineGroupRequestSchema,
  adminMachineGroupSchema,
  adminQueuedCommandSchema,
  adminSignedCommandCompletionEnvelopeSchema,
  adminSignedCommandPollEnvelopeSchema,
  adminSignedHeartbeatEnvelopeSchema,
  adminSignedSecretRotationEnvelopeSchema,
  adminFleetSnapshotSchema,
  commandKindLabel,
  createAdminCommandRequest,
  instanceStatusLabel,
} from "./admin";

describe("admin contracts", () => {
  it("accepts a local fleet snapshot", () => {
    const parsed = adminFleetSnapshotSchema.parse({
      generatedAt: "2026-06-21T10:00:00.000Z",
      summary: {
        totalInstances: 1,
        onlineInstances: 1,
        warningInstances: 0,
        offlineInstances: 0,
        totalJobs: 1,
        runningJobs: 0,
      },
      instances: [
        {
          instanceId: "local-abc",
          displayName: "WORKSTATION-01",
          status: "online",
          lastSeenAt: "2026-06-21T10:00:00.000Z",
          hardware: {
            hostName: "WORKSTATION-01",
            operatingSystem: "windows",
            architecture: "x86_64",
            cpuThreads: 8,
            totalMemoryMb: null,
            appVersion: "0.2.1-alpha",
          },
          network: {
            domain: "REDE",
            interfaces: [{ name: "hostname", address: "WORKSTATION-01", kind: "host" }],
          },
          management: {
            enabled: true,
            mode: "localOnly",
            serverUrl: null,
            allowRemoteCommands: false,
            allowBatchCommands: false,
            heartbeatIntervalSecs: 30,
            inventoryIntervalSecs: 300,
          },
          jobs: [
            {
              id: "550e8400-e29b-41d4-a716-446655440000",
              name: "Backup",
              mode: "copy",
              enabled: true,
              sourcePath: "C:/in",
              targetPath: "D:/out",
              lastRun: null,
              nextRun: null,
              status: "idle",
            },
          ],
          activeExecutions: [],
          capabilities: [
            {
              kind: "startJob",
              label: "Iniciar fluxo",
              support: "available",
              scope: "maquina/fluxo",
              requiresConfirmation: false,
            },
          ],
        },
      ],
    });

    expect(parsed.summary.onlineInstances).toBe(1);
    expect(instanceStatusLabel(parsed.instances[0].status)).toBe("Online");
  });

  it("builds a start command request", () => {
    const request = createAdminCommandRequest(
      "startJob",
      "local-abc",
      "550e8400-e29b-41d4-a716-446655440000",
    );

    expect(adminCommandRequestSchema.parse(request).jobIds).toHaveLength(1);
    expect(commandKindLabel(request.kind)).toBe("Iniciar");
  });

  it("accepts command results", () => {
    const parsed = adminCommandResultSchema.parse({
      accepted: true,
      command: "startJob",
      results: [{ targetInstanceId: "local-abc", status: "accepted", message: "ok" }],
    });

    expect(parsed.results[0].status).toBe("accepted");
  });

  it("accepts heartbeat and queued command payloads", () => {
    const snapshot = adminFleetSnapshotSchema.parse({
      generatedAt: "2026-06-21T10:00:00.000Z",
      summary: {
        totalInstances: 0,
        onlineInstances: 0,
        warningInstances: 0,
        offlineInstances: 0,
        totalJobs: 0,
        runningJobs: 0,
      },
      instances: [],
    });

    expect(snapshot.instances).toHaveLength(0);

    const command = adminQueuedCommandSchema.parse({
      id: "550e8400-e29b-41d4-a716-446655440000",
      source: "server",
      request: { kind: "requestLogs", targetInstanceIds: [], jobIds: [], executionIds: [], reason: null },
      status: "pending",
      result: null,
      createdAt: "2026-06-21T10:00:00.000Z",
      updatedAt: "2026-06-21T10:00:00.000Z",
    });

    expect(command.status).toBe("pending");
    const heartbeatInstance = {
      instanceId: "local-abc",
      displayName: "WORKSTATION-01",
      status: "online",
      lastSeenAt: "2026-06-21T10:00:00.000Z",
      hardware: {
        hostName: "WORKSTATION-01",
        operatingSystem: "windows",
        architecture: "x86_64",
        cpuThreads: 8,
        totalMemoryMb: null,
        appVersion: "0.2.1-alpha",
      },
      network: {
        domain: null,
        interfaces: [{ name: "hostname", address: "WORKSTATION-01", kind: "host" }],
      },
      management: {
        enabled: true,
        mode: "localOnly",
        serverUrl: null,
        allowRemoteCommands: false,
        allowBatchCommands: false,
        heartbeatIntervalSecs: 30,
        inventoryIntervalSecs: 300,
      },
      jobs: [],
      activeExecutions: [],
      capabilities: [],
    };
    const heartbeat = adminHeartbeatPayloadSchema.parse({
      instance: heartbeatInstance,
      generatedAt: "2026-06-21T10:00:00.000Z",
      pendingCommandCount: 0,
    });
    const envelope = adminSignedHeartbeatEnvelopeSchema.parse({
      schemaVersion: "admin.transport.v1",
      kind: "heartbeat",
      instanceId: "local-abc",
      issuedAt: "2026-06-21T10:00:00.000Z",
      expiresAt: "2026-06-21T10:05:00.000Z",
      nonce: "550e8400-e29b-41d4-a716-446655440000",
      payloadHash: "abc123",
      signature: "blake3:abc123",
      payload: heartbeat,
    });

    expect(envelope.kind).toBe("heartbeat");
    const delivery = adminHeartbeatDeliverySchema.parse({
      endpoint: "https://admin.tidyflow.local/api/agents/local-abc/heartbeat",
      statusCode: 202,
      accepted: true,
      message: "accepted",
      sentAt: "2026-06-21T10:00:00.000Z",
    });

    expect(delivery.accepted).toBe(true);
    expect(() =>
      adminHeartbeatPayloadSchema.parse({
        instance: null,
        generatedAt: "2026-06-21T10:00:00.000Z",
        pendingCommandCount: 0,
      }),
    ).toThrow();
  });

  it("accepts enrollment and secret rotation payloads", () => {
    const instance = {
      instanceId: "local-abc",
      displayName: "WORKSTATION-01",
      status: "online",
      lastSeenAt: "2026-06-21T10:00:00.000Z",
      hardware: {
        hostName: "WORKSTATION-01",
        operatingSystem: "windows",
        architecture: "x86_64",
        cpuThreads: 8,
        totalMemoryMb: null,
        appVersion: "0.2.1-alpha",
      },
      network: {
        domain: null,
        interfaces: [{ name: "hostname", address: "WORKSTATION-01", kind: "host" }],
      },
      management: {
        enabled: true,
        mode: "managedAgent",
        serverUrl: "https://admin.tidyflow.local",
        allowRemoteCommands: true,
        allowBatchCommands: false,
        heartbeatIntervalSecs: 30,
        inventoryIntervalSecs: 300,
      },
      jobs: [],
      activeExecutions: [],
      capabilities: [],
    };

    const enrollment = adminEnrollmentTokenRequestSchema.parse({
      token: "invite-token",
      instance,
      agentSecret: "af_secret",
      requestedAt: "2026-06-21T10:00:00.000Z",
    });

    expect(enrollment.instance.instanceId).toBe("local-abc");

    const rotationEnvelope = adminSignedSecretRotationEnvelopeSchema.parse({
      schemaVersion: "admin.transport.v1",
      kind: "secretRotation",
      instanceId: "local-abc",
      issuedAt: "2026-06-21T10:00:00.000Z",
      expiresAt: "2026-06-21T10:05:00.000Z",
      nonce: "550e8400-e29b-41d4-a716-446655440000",
      payloadHash: "abc123",
      signature: "blake3:abc123",
      payload: {
        newAgentSecret: "af_new_secret",
        requestedAt: "2026-06-21T10:00:00.000Z",
      },
    });

    expect(rotationEnvelope.kind).toBe("secretRotation");
    expect(
      adminAgentSecretRotationAcceptedSchema.parse({
        accepted: true,
        instanceId: "local-abc",
        rotatedAt: "2026-06-21T10:00:00.000Z",
        message: "ok",
      }).accepted,
    ).toBe(true);
  });

  it("accepts machine groups and batch command payloads", () => {
    const group = adminMachineGroupSchema.parse({
      id: "550e8400-e29b-41d4-a716-446655440000",
      name: "Financeiro",
      description: null,
      instanceIds: ["local-abc", "local-def"],
      createdAt: "2026-06-21T10:00:00.000Z",
      updatedAt: "2026-06-21T10:00:00.000Z",
    });

    expect(group.instanceIds).toHaveLength(2);
    expect(
      adminMachineGroupRequestSchema.parse({
        name: "Operacao",
        instanceIds: ["local-abc"],
      }).name,
    ).toBe("Operacao");

    const batchRequest = adminBatchCommandRequestSchema.parse({
      request: {
        kind: "requestLogs",
        targetInstanceIds: ["local-direct"],
        jobIds: [],
        executionIds: [],
        reason: "auditoria",
      },
      groupIds: [group.id],
      source: "admin-server",
    });

    expect(batchRequest.groupIds).toEqual([group.id]);

    const accepted = adminBatchCommandAcceptedSchema.parse({
      accepted: true,
      resolvedTargetInstanceIds: ["local-direct", "local-abc", "local-def"],
      command: {
        id: "650e8400-e29b-41d4-a716-446655440000",
        source: "admin-server",
        request: batchRequest.request,
        status: "pending",
        result: null,
        createdAt: "2026-06-21T10:00:00.000Z",
        updatedAt: "2026-06-21T10:00:00.000Z",
      },
      result: {
        accepted: true,
        command: "requestLogs",
        results: [
          {
            targetInstanceId: "local-direct",
            status: "accepted",
            message: "queued",
          },
        ],
      },
    });

    expect(accepted.command.status).toBe("pending");

    const pollRequest = adminSignedCommandPollEnvelopeSchema.parse({
      schemaVersion: "admin.transport.v1",
      kind: "command",
      instanceId: "local-direct",
      issuedAt: "2026-06-21T10:00:00.000Z",
      expiresAt: "2026-06-21T10:05:00.000Z",
      nonce: "740e8400-e29b-41d4-a716-446655440000",
      payloadHash: "abc123",
      signature: "blake3:abc123",
      payload: {
        requestedAt: "2026-06-21T10:00:00.000Z",
      },
    });

    expect(pollRequest.payload.requestedAt).toBe("2026-06-21T10:00:00.000Z");

    const poll = adminCommandPollResponseSchema.parse({
      assignment: {
        schemaVersion: "admin.transport.v1",
        kind: "command",
        instanceId: "local-direct",
        issuedAt: "2026-06-21T10:00:00.000Z",
        expiresAt: "2026-06-21T10:05:00.000Z",
        nonce: "750e8400-e29b-41d4-a716-446655440000",
        payloadHash: "abc123",
        signature: "blake3:abc123",
        payload: {
          commandId: accepted.command.id,
          targetInstanceId: "local-direct",
          request: {
            ...batchRequest.request,
            targetInstanceIds: ["local-direct"],
          },
          assignedAt: "2026-06-21T10:00:00.000Z",
        },
      },
      pendingCount: 0,
      polledAt: "2026-06-21T10:00:00.000Z",
    });

    expect(poll.assignment?.payload.targetInstanceId).toBe("local-direct");

    const remoteJobPreview = adminCommandRequestSchema.parse({
      kind: "createJob",
      targetInstanceIds: ["local-direct"],
      jobPayloads: [
        {
          previewOnly: true,
          job: {
            id: "860e8400-e29b-41d4-a716-446655440000",
            name: "Coleta remota",
            sourcePath: "C:/Entrada",
            targetPath: "D:/Saida",
            mode: "copy",
            conflict: "skip",
            enabled: true,
            lastRun: null,
            nextRun: null,
          },
        },
      ],
    });

    expect(remoteJobPreview.jobPayloads[0]?.previewOnly).toBe(true);

    const completionEnvelope = adminSignedCommandCompletionEnvelopeSchema.parse({
      schemaVersion: "admin.transport.v1",
      kind: "command",
      instanceId: "local-direct",
      issuedAt: "2026-06-21T10:01:00.000Z",
      expiresAt: "2026-06-21T10:06:00.000Z",
      nonce: "760e8400-e29b-41d4-a716-446655440000",
      payloadHash: "abc123",
      signature: "blake3:abc123",
      payload: {
        commandId: accepted.command.id,
        targetInstanceId: "local-direct",
        status: "completed",
        message: "logs uploaded",
        completedAt: "2026-06-21T10:01:00.000Z",
      },
    });

    expect(completionEnvelope.payload.status).toBe("completed");

    const completion = adminCommandCompletionAcceptedSchema.parse({
      accepted: true,
      command: {
        ...accepted.command,
        status: "completed",
        result: {
          accepted: true,
          command: "requestLogs",
          results: [
            {
              targetInstanceId: "local-direct",
              status: "accepted",
              message: "logs uploaded",
            },
          ],
        },
        updatedAt: "2026-06-21T10:01:00.000Z",
      },
      recordedAt: "2026-06-21T10:01:00.000Z",
    });

    expect(completion.command.result?.results[0]?.message).toBe("logs uploaded");
  });

  it("accepts central admin audit pages", () => {
    const query = adminCentralAuditQuerySchema.parse({
      search: "logs",
      status: "accepted",
      limit: 50,
      offset: 0,
    });

    expect(query.status).toBe("accepted");

    const page = adminCentralAuditPageSchema.parse({
      entries: [
        {
          id: "850e8400-e29b-41d4-a716-446655440000",
          actor: "operator:abc123",
          role: "admin",
          action: "command.batch.enqueue",
          target: "650e8400-e29b-41d4-a716-446655440000",
          status: "accepted",
          message: "Comando em lote enfileirado",
          details: "{\"targets\":[\"local-1\"]}",
          createdAt: "2026-06-21T10:00:00.000Z",
        },
      ],
      total: 1,
      limit: 50,
      offset: 0,
    });

    expect(page.entries[0]?.action).toBe("command.batch.enqueue");
  });
});
