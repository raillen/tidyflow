/*
 * Headless Tauri IPC mock for Playwright E2E.
 *
 * Injected with `page.addInitScript` before the SvelteKit bundle runs, so the
 * real `@tauri-apps/api` client talks to an in-memory backend instead of the
 * missing Rust process. Everything here is fixture data: it never touches the
 * developer's real TidyFlow database.
 */
(function installTauriMock() {
  const now = () => new Date().toISOString();
  const uuid = (n) =>
    `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`;

  const JOB_ID = uuid(1);
  const JOB_ID_DISABLED = uuid(2);
  const BLUEPRINT_FILE_ID = uuid(3);
  const BLUEPRINT_FOLDER_ID = uuid(4);
  const EXECUTION_ID = uuid(5);
  const BLUEPRINT_RUN_ID = uuid(6);

  const SETTINGS_KEY = "tidyflow-mock-settings";

  function loadPersistedSettings() {
    try {
      const raw = window.localStorage.getItem(SETTINGS_KEY);
      if (!raw) return null;
      const parsed = JSON.parse(raw);
      return parsed && typeof parsed === "object" ? parsed : null;
    } catch {
      return null;
    }
  }

  function persistSettings(next) {
    try {
      window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(next));
    } catch {
      /* ignore quota errors in tests */
    }
  }

  const emptyFilters = () => ({
    includeExtensions: [],
    excludePatterns: [],
    excludePresetIds: [],
    nameRegex: null,
    pathRegex: null,
    minSizeBytes: null,
    maxSizeBytes: null,
    maxDepth: null,
    modifiedAfter: null,
    modifiedBefore: null,
    createdAfter: null,
    createdBefore: null,
    olderThanDays: null,
    contentContains: null,
    contentMaxBytes: 5242880,
    contentExtensions: [".txt", ".md"],
    recursive: true,
    includeHidden: false,
    symlinkMode: "follow",
    skipEmptyFiles: false,
  });

  const emptyOptions = () => ({
    smartSync: false,
    strictHashSync: false,
    verifyAfterCopy: true,
    stopOnIntegrityError: false,
    encryptOutput: false,
    encryptPassword: null,
    rememberEncryptPassword: false,
    removeFilesAfterPack: false,
    packFilename: null,
  });

  const emptyScripts = () => ({ preScript: null, postScript: null, timeoutSecs: 60 });

  const emptyNotify = () => ({ enabled: false, events: ["completed", "failed"], channels: [] });

  const makeJob = (id, name, enabled, sourcePath, targetPath) => ({
    id,
    name,
    sourcePath,
    targetPath,
    mode: "copy",
    conflict: "skip",
    filters: emptyFilters(),
    options: emptyOptions(),
    schedule: null,
    watch: null,
    scripts: emptyScripts(),
    notify: emptyNotify(),
    enabled,
    lastRun: null,
    nextRun: null,
  });

  const jobs = {
    [JOB_ID]: makeJob(JOB_ID, "Backup de Documentos", true, "/home/demo/Documentos", "/home/demo/Backup"),
    [JOB_ID_DISABLED]: makeJob(JOB_ID_DISABLED, "Fluxo desativado", false, "/home/demo/tmp", "/home/demo/tmp2"),
  };

  const jobSummary = (job) => ({
    id: job.id,
    name: job.name,
    sourcePath: job.sourcePath,
    targetPath: job.targetPath,
    mode: job.mode,
    enabled: job.enabled,
    lastRun: job.lastRun,
    nextRun: job.nextRun,
    scheduleEnabled: false,
    watchEnabled: false,
  });

  const emptyTemplate = () => ({ segments: [] });

  const makeBlueprint = (id, name, kind) => ({
    id,
    name,
    kind,
    rootPath: kind === "folder" ? "/home/demo/Projetos" : "/home/demo/Entrada",
    search: emptyFilters(),
    routing: { pathTemplate: emptyTemplate(), createIntermediateDirs: true },
    operation: "move",
    recursive: true,
    conflict: "skip",
    renameTemplate: null,
    folderPlan:
      kind === "folder"
        ? {
            version: 1,
            variables: [
              { name: "projectName", label: "Nome do projeto", defaultValue: "", required: true },
            ],
            allowDeleteNonEmpty: false,
            confirmDeleteNonEmpty: false,
            nodes: [
              {
                action: "create",
                name: "{projectName}",
                sourcePath: "",
                targetPath: "",
                children: [
                  { action: "create", name: "01 Briefing", sourcePath: "", targetPath: "", children: [] },
                  { action: "create", name: "02 Entrega", sourcePath: "", targetPath: "", children: [] },
                ],
              },
            ],
          }
        : null,
    watch: null,
    counter: { scope: "global", start: 1, padding: 0 },
    enabled: true,
    lastRun: null,
  });

  const blueprints = {
    [BLUEPRINT_FILE_ID]: makeBlueprint(BLUEPRINT_FILE_ID, "Organizar notas fiscais", "file"),
    [BLUEPRINT_FOLDER_ID]: makeBlueprint(BLUEPRINT_FOLDER_ID, "Estrutura de projeto", "folder"),
  };

  const blueprintSummary = (bp) => ({
    id: bp.id,
    name: bp.name,
    kind: bp.kind,
    rootPath: bp.rootPath,
    operation: bp.operation,
    enabled: bp.enabled,
    lastRun: bp.lastRun,
    watchEnabled: Boolean(bp.watch && bp.watch.enabled),
  });

  const auditEntries = [
    {
      id: 1,
      jobId: JOB_ID,
      blueprintId: null,
      jobName: "Backup de Documentos",
      sourcePath: "/home/demo/Documentos/relatorio.pdf",
      targetPath: "/home/demo/Backup/relatorio.pdf",
      status: "COPIED",
      fileSize: 2048,
      durationMs: 12.5,
      details: null,
      createdAt: now(),
    },
    {
      id: 2,
      jobId: JOB_ID,
      blueprintId: null,
      jobName: "Backup de Documentos",
      sourcePath: "/home/demo/Documentos/bloqueado.pdf",
      targetPath: "/home/demo/Backup/bloqueado.pdf",
      status: "FAILED",
      fileSize: 0,
      durationMs: 4.1,
      details: "permissão negada",
      createdAt: now(),
    },
    {
      id: 3,
      jobId: null,
      blueprintId: BLUEPRINT_FILE_ID,
      jobName: "Organizar notas fiscais",
      sourcePath: "/home/demo/Entrada/nota.pdf",
      targetPath: "/home/demo/Entrada/2026/nota.pdf",
      status: "ORGANIZED",
      fileSize: 512,
      durationMs: 2.0,
      details: "move",
      createdAt: now(),
    },
  ];

  const auditSummary = () => ({
    total: auditEntries.length,
    copied: auditEntries.filter((e) => e.status === "COPIED").length,
    moved: auditEntries.filter((e) => e.status === "MOVED").length,
    ignored: auditEntries.filter((e) => e.status === "IGNORED").length,
    failed: auditEntries.filter((e) => e.status === "FAILED").length,
    organized: auditEntries.filter((e) => e.status === "ORGANIZED").length,
    totalBytes: auditEntries.reduce((sum, e) => sum + e.fileSize, 0),
    averageDurationMs: 6.2,
    latestAt: now(),
  });

  const defaultSettings = {
    theme: "system",
    accentColor: "#0064ff",
    language: "pt-BR",
    interfaceFont: "system",
    autostart: false,
    closeToTray: true,
    startMinimized: false,
    bandwidthLimitMbps: 0,
    maxParallelFiles: 1,
    logRetentionDays: 30,
    performance: {
      maxThreads: 2,
      memoryLimitMb: 0,
      processPriority: "normal",
      globalBandwidthLimitMbps: 0,
      queuePollIntervalMs: 500,
      pauseWhenOnBattery: false,
    },
    security: {
      pinEnabled: false,
      accessPinHash: null,
      requirePinOnStartup: false,
      lockOnMinimize: false,
      lockOnTray: false,
      encryptionEnabled: false,
      masterKeyHint: "",
      maskSensitivePaths: true,
    },
    notifications: {
      enabled: false,
      desktopEnabled: true,
      webhookEnabled: false,
      webhooks: [],
      smtp: {
        enabled: false,
        host: "",
        port: 587,
        username: "",
        password: "",
        fromAddress: "",
        toAddress: "",
        useTls: true,
        passwordConfigured: false,
      },
      adminPanelEnabled: false,
      notifyOnSuccess: false,
      notifyOnFailure: true,
    },
    maintenance: {
      logRetentionDays: 30,
      autoCompactDatabase: true,
      backupEnabled: false,
      backupDirectory: "",
      backupIntervalHours: 24,
      backupRetentionCount: 7,
      optimizeAfterCleanup: false,
    },
    support: {
      supportEmail: "suporte@tidyflow.local",
      pixKey: "",
      bankDepositInfo: "",
      buyMeCoffeeUrl: "",
      donationsEnabled: true,
    },
    about: {
      projectName: "TidyFlow",
      projectDescription: "Automação local para organizar, copiar e mover arquivos com auditoria clara.",
      creatorName: "Raillen Santos",
      creatorBio: "Criador do TidyFlow e projetos Zenith.",
      websiteUrl: "",
      githubUrl: "https://github.com/raillen",
      linkedinUrl: "",
    },
    admin: {
      enabled: true,
      mode: "localOnly",
      instanceId: "demo-instance",
      displayName: "Demo",
      serverUrl: "",
      enrollmentToken: "",
      enrollmentTokenConfigured: false,
      allowRemoteCommands: false,
      allowBatchCommands: false,
      heartbeatIntervalSecs: 30,
      inventoryIntervalSecs: 300,
      lastRegisteredAt: null,
    },
  };

  let settings = defaultSettings;

  /**
   * Overlays the persisted settings (written by a previous `settings_update`)
   * on top of the defaults. Kept lazy because Playwright runs `addInitScript`
   * hooks in registration order and a spec may seed storage after this file.
   */
  function currentSettings() {
    const persisted = loadPersistedSettings();
    if (!persisted) return settings;
    settings = {
      ...defaultSettings,
      ...persisted,
      security: { ...defaultSettings.security, ...(persisted.security ?? {}) },
      notifications: {
        ...defaultSettings.notifications,
        ...(persisted.notifications ?? {}),
      },
    };
    return settings;
  }

  const capability = (kind, support = "available") => ({
    kind,
    label: kind,
    support,
    scope: "local",
    requiresConfirmation: false,
  });

  const fleetSnapshot = () => ({
    generatedAt: now(),
    summary: {
      totalInstances: 1,
      onlineInstances: 1,
      warningInstances: 0,
      offlineInstances: 0,
      totalJobs: 2,
      runningJobs: 0,
    },
    instances: [
      {
        instanceId: "demo-instance",
        displayName: "Demo",
        status: "online",
        lastSeenAt: now(),
        hardware: {
          hostName: "demo-host",
          operatingSystem: "linux",
          architecture: "x86_64",
          cpuThreads: 8,
          totalMemoryMb: 16384,
          appVersion: "0.2.1-alpha",
        },
        network: { domain: null, interfaces: [{ name: "eth0", address: "10.0.0.2", kind: "ethernet" }] },
        management: {
          enabled: true,
          mode: "localOnly",
          serverUrl: null,
          allowRemoteCommands: false,
          allowBatchCommands: false,
          heartbeatIntervalSecs: 30,
          inventoryIntervalSecs: 300,
        },
        jobs: Object.values(jobs).map((job) => ({
          id: job.id,
          name: job.name,
          mode: job.mode,
          enabled: job.enabled,
          sourcePath: job.sourcePath,
          targetPath: job.targetPath,
          lastRun: job.lastRun,
          nextRun: job.nextRun,
          status: job.enabled ? "idle" : "disabled",
        })),
        activeExecutions: [],
        capabilities: [
          capability("startJob"),
          capability("cancelExecution"),
          capability("pauseJob"),
          capability("resumeJob"),
          capability("stopJob"),
          capability("createJob"),
          capability("updateJob"),
          capability("deleteJob"),
          capability("applySettingsPolicy", "planned"),
          capability("requestLogs", "planned"),
        ],
      },
    ],
  });

  const heartbeatPayload = () => ({
    instance: fleetSnapshot().instances[0],
    generatedAt: now(),
    pendingCommandCount: 0,
  });

  const signedEnvelope = () => ({
    schemaVersion: "admin.transport.v1",
    kind: "heartbeat",
    instanceId: "demo-instance",
    issuedAt: now(),
    expiresAt: new Date(Date.now() + 60000).toISOString(),
    nonce: uuid(77),
    payloadHash: "abc123",
    signature: "blake3:deadbeef",
    payload: heartbeatPayload(),
  });

  let uiState = {};

  const moderation = {
    activeExecutions: [],
    queuePaused: false,
    lastCompleted: null,
  };

  const listeners = new Map();
  const callbacks = new Map();
  let nextCallbackId = 1;
  let nextEventId = 1;

  function registerCallback(cb, once) {
    const id = nextCallbackId++;
    callbacks.set(id, { cb, once });
    return id;
  }

  function runCallback(id, data) {
    const entry = callbacks.get(id);
    if (!entry) return;
    if (entry.once) callbacks.delete(id);
    entry.cb(data);
  }

  function emitToFrontend(event, payload) {
    const ids = listeners.get(event) || [];
    for (const id of ids) {
      runCallback(id, { event, id: nextEventId++, payload });
    }
  }

  const handlers = {
    health: () => ({ status: "ok", version: "0.2.1-alpha", core: "autoflow-core" }),

    settings_get: () => currentSettings(),
    settings_update: ({ settings: next }) => {
      settings = next;
      persistSettings(settings);
      return settings;
    },
    security_set_pin: ({ pin }) => {
      settings = {
        ...settings,
        security: { ...settings.security, pinEnabled: true, accessPinHash: "hash-" + pin },
      };
      persistSettings(settings);
      return settings;
    },
    security_verify_pin: ({ pin }) => pin === "1234",
    security_clear_pin: () => {
      settings = { ...settings, security: { ...settings.security, pinEnabled: false, accessPinHash: null } };
      return settings;
    },

    jobs_list: () => Object.values(jobs).map(jobSummary),
    jobs_get: ({ id }) => {
      if (!jobs[id]) throw { code: "job_not_found", message: "job not found" };
      return jobs[id];
    },
    jobs_create: ({ job }) => {
      jobs[job.id] = job;
      return job;
    },
    jobs_update: ({ job }) => {
      jobs[job.id] = job;
      return job;
    },
    jobs_delete: ({ id }) => {
      delete jobs[id];
      return null;
    },
    jobs_run: () => EXECUTION_ID,
    jobs_simulate: () => ({
      filesMatched: 3,
      filesSkipped: 1,
      sample: [
        { source: "/a.txt", target: "/b/a.txt", action: "copy" },
        { source: "/c.txt", target: "/b/c.txt", action: "copy" },
      ],
      warnings: ["1 arquivo ignorado por política de conflito"],
    }),
    jobs_simulate_draft: () => ({
      filesMatched: 1,
      filesSkipped: 0,
      sample: [{ source: "/x.txt", target: "/y/x.txt", action: "copy" }],
      warnings: [],
    }),
    jobs_list_missed_schedules: () => [],
    jobs_clear_missed_schedules: () => null,

    executions_list_active: () => moderation.activeExecutions,
    executions_cancel: ({ executionId }) => {
      moderation.activeExecutions = moderation.activeExecutions.filter(
        (e) => e.executionId !== executionId,
      );
      emitToFrontend("execution_completed", {
        executionId,
        jobId: JOB_ID,
        success: false,
        processed: 0,
        failed: 0,
        errorMessage: "cancelado",
      });
      return null;
    },
    rollback_execute: () => ({ success: true, revertedCount: 2, skippedCount: 0 }),

    queue_pause: () => {
      moderation.queuePaused = true;
      return null;
    },
    queue_resume: () => {
      moderation.queuePaused = false;
      return null;
    },
    queue_is_paused: () => moderation.queuePaused,
    queue_stop_all: () => {
      moderation.activeExecutions = [];
      return null;
    },

    audit_list_recent: ({ limit = 100 }) => auditEntries.slice(0, limit),
    audit_query: ({ query = {} }) => {
      const filtered = auditEntries.filter((entry) => {
        if (query.status && entry.status !== query.status) return false;
        if (query.search) {
          const text = `${entry.jobName} ${entry.sourcePath} ${entry.targetPath}`.toLowerCase();
          if (!text.includes(String(query.search).toLowerCase())) return false;
        }
        return true;
      });
      const offset = query.offset ?? 0;
      const limit = query.limit ?? 100;
      return {
        entries: filtered.slice(offset, offset + limit),
        total: filtered.length,
        limit,
        offset,
        summary: auditSummary(),
      };
    },
    audit_export: ({ format }) => ({
      fileName: `audit.${format}`,
      mimeType: format === "csv" ? "text/csv" : "application/json",
      content: format === "csv" ? "id,status\n1,COPIED\n" : JSON.stringify(auditEntries),
    }),
    audit_clear: () => {
      auditEntries.length = 0;
      return null;
    },

    blueprints_list: () => Object.values(blueprints).map(blueprintSummary),
    blueprints_get: ({ id }) => {
      if (!blueprints[id]) throw { code: "blueprint_not_found", message: "blueprint not found" };
      return blueprints[id];
    },
    blueprints_create: ({ blueprint }) => {
      blueprints[blueprint.id] = blueprint;
      return blueprint;
    },
    blueprints_update: ({ blueprint }) => {
      blueprints[blueprint.id] = blueprint;
      return blueprint;
    },
    blueprints_delete: ({ id }) => {
      delete blueprints[id];
      return null;
    },
    blueprints_simulate: () => ({
      matched: 4,
      skipped: 0,
      plan_sample: [
        { source: "/in/a.pdf", target: "/in/2026/a.pdf", action: "move" },
        { source: "/in/b.pdf", target: "/in/2026/b.pdf", action: "move" },
      ],
      warnings: [],
      collisions: [],
    }),
    blueprints_simulate_with_variables: () => ({
      matched: 2,
      skipped: 0,
      plan_sample: [
        { source: "/in/p", target: "/in/Projeto/p", action: "create" },
        { source: "/in/p/01", target: "/in/Projeto/01 Briefing", action: "create" },
      ],
      warnings: [],
      collisions: [],
    }),
    blueprints_apply: () => ({ processed: 2, failed: 0, skipped: 0 }),
    blueprints_apply_with_variables: () => ({ processed: 3, failed: 0, skipped: 0 }),
    blueprints_rollback: () => ({ success: true, revertedCount: 2, skippedCount: 0 }),
    blueprints_preview_template: () => ({
      resultPath: "2026/relatorio.pdf",
      resultName: "relatorio.pdf",
      valid: true,
      warnings: [],
    }),
    blueprints_preview_plan: () => ({
      rootPath: "/home/demo/Projetos",
      nodes: [],
      folderCount: 2,
      valid: true,
      warnings: [],
    }),

    admin_fleet_snapshot: () => fleetSnapshot(),
    admin_heartbeat_payload: () => heartbeatPayload(),
    admin_signed_heartbeat_payload: () => signedEnvelope(),
    admin_send_signed_heartbeat_once: () => ({
      endpoint: "https://admin.local/heartbeat",
      statusCode: 200,
      accepted: true,
      message: "ok",
      sentAt: now(),
    }),
    admin_agent_secret_generate: () => ({
      secret: "s".repeat(48),
      settings,
    }),
    admin_agent_secret_set: () => settings,
    admin_agent_secret_clear: () => settings,
    admin_dispatch_command: ({ request }) => ({
      accepted: true,
      command: request.kind,
      results: [
        {
          targetInstanceId: "demo-instance",
          status: "accepted",
          message: "accepted",
        },
      ],
    }),
    admin_enqueue_command: ({ request, source }) => ({
      id: uuid(91),
      source: source ?? "local-ui",
      request,
      status: "pending",
      result: null,
      createdAt: now(),
      updatedAt: now(),
    }),
    admin_list_commands: () => [],
    admin_command_queue_summary: () => ({
      pending: 0,
      running: 0,
      completed: 0,
      failed: 0,
      skipped: 0,
    }),
    admin_process_next_command: () => null,

    ui_state_get: ({ key }) => uiState[key] ?? null,
    ui_state_save: ({ key, payload }) => {
      uiState[key] = payload;
      return null;
    },

    "plugin:dialog|open": () => "/home/demo/escolhido",
    "plugin:dialog|save": () => "/home/demo/salvar.txt",
    "plugin:dialog|message": () => null,
    "plugin:notification|is_permission_granted": () => true,
    "plugin:notification|request_permission": () => "granted",
    "plugin:notification|notify": () => null,
    "plugin:opener|open_url": () => null,
    "plugin:event|listen": ({ event, handler }) => {
      const ids = listeners.get(event) || [];
      ids.push(handler);
      listeners.set(event, ids);
      return handler;
    },
    "plugin:event|unlisten": ({ event, eventId }) => {
      const ids = listeners.get(event) || [];
      listeners.set(
        event,
        ids.filter((id) => id !== eventId),
      );
      return null;
    },
    "plugin:event|emit": ({ event, payload }) => {
      emitToFrontend(event, payload);
      return null;
    },
  };

  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args) => {
      const handler = handlers[cmd];
      if (!handler) {
        throw { code: "mock_unhandled", message: `unmocked command: ${cmd}` };
      }
      return handler(args || {});
    },
    transformCallback: (cb, once) => registerCallback(cb, once),
    unregisterCallback: (id) => callbacks.delete(id),
    runCallback: (id, data) => runCallback(id, data),
    callbacks: new Map(),
    convertFileSrc: (p) => p,
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
    plugins: {},
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: () => {},
  };

  // Test-only escape hatches so specs can simulate backend events.
  window.__TIDYFLOW_MOCK__ = {
    emit: (event, payload) => emitToFrontend(event, payload),
    pushExecution: (execution) => {
      moderation.activeExecutions = [...moderation.activeExecutions, execution];
      emitToFrontend("execution_progress", execution);
    },
    ids: { JOB_ID, JOB_ID_DISABLED, BLUEPRINT_FILE_ID, BLUEPRINT_FOLDER_ID, EXECUTION_ID },
  };
})();
