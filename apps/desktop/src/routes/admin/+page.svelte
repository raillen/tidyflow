<script lang="ts">
  import { onMount } from "svelte";
  import {
    ArrowClockwise,
    ChartBar,
    DesktopTower,
    Pause,
    Play,
    Stop,
    Trash,
    WarningCircle,
  } from "phosphor-svelte";
  import {
    commandKindLabel,
    createAdminCommandRequest,
    instanceStatusLabel,
    jobRuntimeStatusLabel,
    queuedCommandStatusLabel,
    type AdminCommandQueueSummary,
    type AdminCommandKind,
    type AdminFleetSnapshot,
    type AdminHeartbeatPayload,
    type AdminInstanceSnapshot,
    type AdminJobRuntimeStatus,
    type AdminManagedJob,
    type AdminQueuedCommand,
    type AdminQueuedCommandStatus,
  } from "$lib/contracts/admin";
  import { formatDateTime } from "$lib/contracts/audit";
  import {
    dispatchAdminCommand,
    enqueueAdminCommand,
    fetchAdminCommandQueueSummary,
    fetchAdminFleetSnapshot,
    fetchAdminHeartbeatPayload,
    fetchAdminSignedHeartbeatPayload,
    listAdminCommands,
    processNextAdminCommand,
    sendAdminSignedHeartbeatOnce,
  } from "$lib/core/ipc/client";
  import { errorMessage } from "$lib/core/errors";
  import { t } from "$lib/i18n";

  let snapshot = $state<AdminFleetSnapshot | null>(null);
  let heartbeat = $state<AdminHeartbeatPayload | null>(null);
  let queueSummary = $state<AdminCommandQueueSummary | null>(null);
  let selectedInstanceId = $state<string | null>(null);
  let selectedJobId = $state<string | null>(null);
  let queuedCommands = $state<AdminQueuedCommand[]>([]);
  let loading = $state(true);
  let message = $state<string | null>(null);
  let messageIsError = $state(false);

  const selectedInstance = $derived(
    snapshot?.instances.find((instance) => instance.instanceId === selectedInstanceId)
      ?? snapshot?.instances[0]
      ?? null,
  );
  const selectedJob = $derived(
    selectedInstance?.jobs.find((job) => job.id === selectedJobId)
      ?? selectedInstance?.jobs[0]
      ?? null,
  );
  const availableCapabilities = $derived(
    selectedInstance?.capabilities.filter((capability) => capability.support === "available") ?? [],
  );
  const plannedCapabilities = $derived(
    selectedInstance?.capabilities.filter((capability) => capability.support === "planned") ?? [],
  );

  onMount(() => {
    void refresh();
  });

  async function refresh(options: { keepMessage?: boolean } = {}) {
    loading = true;
    if (!options.keepMessage) {
      message = null;
      messageIsError = false;
    }
    try {
      const [nextSnapshot, nextHeartbeat, nextQueueSummary, nextQueuedCommands] = await Promise.all([
        fetchAdminFleetSnapshot(),
        fetchAdminHeartbeatPayload(),
        fetchAdminCommandQueueSummary(),
        listAdminCommands(30),
      ]);

      snapshot = nextSnapshot;
      heartbeat = nextHeartbeat;
      queueSummary = nextQueueSummary;
      queuedCommands = nextQueuedCommands;

      const nextSelectedInstance =
        nextSnapshot.instances.find((instance) => instance.instanceId === selectedInstanceId)
        ?? nextSnapshot.instances[0]
        ?? null;
      selectedInstanceId = nextSelectedInstance?.instanceId ?? null;
      selectedJobId =
        nextSelectedInstance?.jobs.some((job) => job.id === selectedJobId) === true
          ? selectedJobId
          : nextSelectedInstance?.jobs[0]?.id ?? null;
    } catch (error) {
      message = errorMessage(error, t("admin.loadError"));
      messageIsError = true;
    } finally {
      loading = false;
    }
  }

  /**
   * Runs a command handler, surfacing IPC failures in the shared banner instead
   * of leaving an unhandled rejection with no user feedback.
   */
  async function runCommand(action: () => Promise<void>): Promise<void> {
    try {
      await action();
    } catch (error) {
      message = errorMessage(error, t("admin.commandError"));
      messageIsError = true;
    }
  }

  async function startJob(instance: AdminInstanceSnapshot, job: AdminManagedJob) {
    await dispatchJobCommand(instance, job, "startJob");
  }

  async function cancelExecution(instance: AdminInstanceSnapshot, executionId: string) {
    message = null;
    messageIsError = false;
    const result = await dispatchAdminCommand({
      kind: "cancelExecution",
      targetInstanceIds: [instance.instanceId],
      jobIds: [],
      executionIds: [executionId],
      jobPayloads: [],
      reason: null,
    });
    message = result.results.map((entry) => entry.message).join(" | ");
    await refresh({ keepMessage: true });
  }

  async function dispatchJobCommand(
    instance: AdminInstanceSnapshot,
    job: AdminManagedJob,
    kind: AdminCommandKind,
  ) {
    if ((kind === "deleteJob" || kind === "stopJob") && !confirmCommand(kind, job.name)) {
      return;
    }

    message = null;
    messageIsError = false;
    const result = await dispatchAdminCommand(createAdminCommandRequest(kind, instance.instanceId, job.id));
    message = result.results.map((entry) => entry.message).join(" | ");
    await refresh({ keepMessage: true });
  }

  async function enqueueJobCommand(
    instance: AdminInstanceSnapshot,
    job: AdminManagedJob,
    kind: AdminCommandKind,
  ) {
    message = null;
    messageIsError = false;
    const queued = await enqueueAdminCommand(
      createAdminCommandRequest(kind, instance.instanceId, job.id),
      "local-ui",
    );
    message = t("admin.queued", { command: commandKindLabel(queued.request.kind) });
    [queuedCommands, queueSummary] = await Promise.all([
      listAdminCommands(30),
      fetchAdminCommandQueueSummary(),
    ]);
  }

  async function processNextCommand() {
    message = null;
    messageIsError = false;
    const processed = await processNextAdminCommand();
    message = processed
      ? t("admin.processed", { command: commandKindLabel(processed.request.kind) })
      : t("admin.nonePending");
    await refresh({ keepMessage: true });
  }

  async function dispatchBatchCommand(instance: AdminInstanceSnapshot, kind: AdminCommandKind) {
    if (!instance.management.allowBatchCommands) {
      message = t("admin.batchDisabled");
      messageIsError = true;
      return;
    }
    if ((kind === "deleteJob" || kind === "stopJob") && !confirmCommand(kind, t("admin.allJobs"))) {
      return;
    }

    message = null;
    messageIsError = false;
    const result = await dispatchAdminCommand({
      kind,
      targetInstanceIds: [instance.instanceId],
      jobIds: instance.jobs.map((job) => job.id),
      executionIds: [],
      jobPayloads: [],
      reason: null,
    });
    message = result.results.map((entry) => entry.message).join(" | ");
    await refresh({ keepMessage: true });
  }

  function confirmCommand(kind: AdminCommandKind, targetName: string): boolean {
    const label = commandKindLabel(kind).toLowerCase();
    return window.confirm(t("admin.confirmCommand", { command: label, target: targetName }));
  }

  function statusClass(status: AdminJobRuntimeStatus | "online" | "warning" | "offline"): string {
    switch (status) {
      case "running":
      case "online":
        return "success";
      case "scheduled":
      case "warning":
        return "warning";
      case "disabled":
      case "offline":
        return "danger";
      case "idle":
        return "muted";
    }
  }

  function queuedStatusClass(status: AdminQueuedCommandStatus): string {
    switch (status) {
      case "completed":
        return "success";
      case "running":
      case "pending":
        return "warning";
      case "failed":
        return "danger";
      case "skipped":
        return "muted";
    }
  }

  function isCommandAvailable(instance: AdminInstanceSnapshot, kind: AdminCommandKind): boolean {
    return instance.capabilities.some(
      (capability) => capability.kind === kind && capability.support === "available",
    );
  }

  function managementModeLabel(mode: "localOnly" | "managedAgent"): string {
    return mode === "managedAgent" ? t("settings.adminMode.managedAgent") : t("settings.adminMode.localOnly");
  }

  async function copyHeartbeatPayload() {
    if (!heartbeat) {
      return;
    }
    try {
      await navigator.clipboard.writeText(JSON.stringify(heartbeat, null, 2));
      message = t("admin.heartbeatCopied");
    } catch {
      message = t("admin.heartbeatCopyError");
    }
  }

  async function copySignedHeartbeatPayload() {
    message = null;
    try {
      const signedHeartbeat = await fetchAdminSignedHeartbeatPayload();
      await navigator.clipboard.writeText(JSON.stringify(signedHeartbeat, null, 2));
      message = t("admin.signedCopied");
    } catch (error) {
      message = errorMessage(error, t("admin.signedCopyError"));
      messageIsError = true;
    }
  }

  async function sendSignedHeartbeat() {
    message = null;
    try {
      const delivery = await sendAdminSignedHeartbeatOnce();
      const status = delivery.statusCode ?? t("admin.noStatus");
      message = delivery.accepted
        ? t("admin.heartbeatSent", { status })
        : t("admin.heartbeatRejected", { status, message: delivery.message });
    } catch (error) {
      message = errorMessage(error, t("admin.heartbeatSendConfigError"));
      messageIsError = true;
    }
  }
</script>

<section class="page admin-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{t("admin.pageTitle")}</h1>
      <p class="page-desc">{t("admin.subtitle")}</p>
    </div>
    <button class="btn" type="button" onclick={() => void refresh()} disabled={loading}>
      <ArrowClockwise size={14} />
      {t("admin.refresh")}
    </button>
  </header>

  {#if snapshot}
    <section class="summary-grid" aria-label={t("admin.fleetSummaryAria")}>
      <article class="metric-card">
        <DesktopTower size={18} weight="duotone" aria-hidden="true" />
        <div>
          <span>{t("admin.instances")}</span>
          <strong>{snapshot.summary.totalInstances}</strong>
        </div>
      </article>
      <article class="metric-card success">
        <span>{t("admin.online")}</span>
        <strong>{snapshot.summary.onlineInstances}</strong>
      </article>
      <article class="metric-card warning">
        <span>{t("admin.warning")}</span>
        <strong>{snapshot.summary.warningInstances}</strong>
      </article>
      <article class="metric-card">
        <ChartBar size={18} weight="duotone" aria-hidden="true" />
        <div>
          <span>{t("admin.jobs")}</span>
          <strong>{snapshot.summary.totalJobs}</strong>
        </div>
      </article>
      <article class="metric-card success">
        <span>{t("admin.running")}</span>
        <strong>{snapshot.summary.runningJobs}</strong>
      </article>
      <article class="metric-card warning">
        <span>{t("admin.queuePending")}</span>
        <strong>{queueSummary?.pending ?? 0}</strong>
      </article>
      <article class="metric-card">
        <span>{t("admin.queueDone")}</span>
        <strong>{(queueSummary?.completed ?? 0) + (queueSummary?.skipped ?? 0)}</strong>
      </article>
    </section>
  {/if}

  {#if message}
    <p class="banner" class:error={messageIsError} role="status">{message}</p>
  {/if}

  {#if loading && !snapshot}
    <p class="muted" aria-live="polite">{t("admin.loading")}</p>
  {:else if snapshot && selectedInstance}
    <div class="admin-layout">
      <section class="panel instance-list" aria-label={t("admin.managedInstancesAria")}>
        <div class="section-head">
          <h2>{t("admin.machines")}</h2>
          <span>{formatDateTime(snapshot.generatedAt)}</span>
        </div>

        <div class="machine-list">
          {#each snapshot.instances as instance}
            <button
              type="button"
              class:selected={instance.instanceId === selectedInstance.instanceId}
              onclick={() => {
                selectedInstanceId = instance.instanceId;
                selectedJobId = instance.jobs[0]?.id ?? null;
              }}
            >
              <span class="machine-name">{instance.displayName}</span>
              <span class="badge {statusClass(instance.status)}">{instanceStatusLabel(instance.status)}</span>
              <small>{instance.hardware.operatingSystem} · {instance.hardware.cpuThreads} threads</small>
            </button>
          {/each}
        </div>
      </section>

      <section class="panel instance-detail" aria-label={t("admin.instanceDetailsAria")}>
        <div class="detail-header">
          <div>
            <h2>{selectedInstance.displayName}</h2>
            <p class="muted">{selectedInstance.instanceId}</p>
          </div>
          <span class="badge {statusClass(selectedInstance.status)}">
            {instanceStatusLabel(selectedInstance.status)}
          </span>
        </div>

        <div class="profile-grid">
          <div>
            <span>{t("admin.system")}</span>
            <strong>{selectedInstance.hardware.operatingSystem}</strong>
          </div>
          <div>
            <span>{t("admin.architecture")}</span>
            <strong>{selectedInstance.hardware.architecture}</strong>
          </div>
          <div>
            <span>CPU</span>
            <strong>{selectedInstance.hardware.cpuThreads} threads</strong>
          </div>
          <div>
            <span>{t("admin.version")}</span>
            <strong>{selectedInstance.hardware.appVersion}</strong>
          </div>
        </div>

        <section class="subsection" aria-label={t("admin.management")}>
          <h3>{t("admin.management")}</h3>
          <div class="profile-grid management-grid">
            <div>
              <span>{t("admin.mode")}</span>
              <strong>{managementModeLabel(selectedInstance.management.mode)}</strong>
            </div>
            <div>
              <span>{t("admin.server")}</span>
              <strong>{selectedInstance.management.serverUrl ?? t("admin.notConfigured")}</strong>
            </div>
            <div>
              <span>{t("admin.remoteCommands")}</span>
              <strong>{selectedInstance.management.allowRemoteCommands ? t("admin.allowed") : t("admin.blocked")}</strong>
            </div>
            <div>
              <span>{t("admin.batchActions")}</span>
              <strong>{selectedInstance.management.allowBatchCommands ? t("admin.allowedFem") : t("admin.blockedFem")}</strong>
            </div>
          </div>
        </section>

        <section class="subsection" aria-label={t("admin.network")}>
          <h3>{t("admin.network")}</h3>
          <div class="network-list">
            {#each selectedInstance.network.interfaces as network}
              <span>{network.name}: {network.address ?? t("admin.noAddress")}</span>
            {/each}
          </div>
        </section>

        <section class="subsection" aria-label={t("admin.jobsAria")}>
          <div class="section-head">
            <h3>{t("admin.jobs")}</h3>
            <div class="job-actions">
              {#if selectedJob && isCommandAvailable(selectedInstance, "startJob")}
                <button class="btn primary" type="button" onclick={() => runCommand(() => startJob(selectedInstance, selectedJob))}>
                  <Play size={14} weight="fill" />
                  {t("admin.start")}
                </button>
                <button class="btn" type="button" onclick={() => runCommand(() => enqueueJobCommand(selectedInstance, selectedJob, "startJob"))}>
                  {t("admin.enqueue")}
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "pauseJob")}
                <button class="btn" type="button" onclick={() => runCommand(() => dispatchJobCommand(selectedInstance, selectedJob, "pauseJob"))}>
                  <Pause size={14} weight="fill" />
                  {t("admin.pause")}
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "resumeJob")}
                <button class="btn" type="button" onclick={() => runCommand(() => dispatchJobCommand(selectedInstance, selectedJob, "resumeJob"))}>
                  <Play size={14} />
                  {t("admin.resume")}
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "stopJob")}
                <button class="btn danger" type="button" onclick={() => runCommand(() => dispatchJobCommand(selectedInstance, selectedJob, "stopJob"))}>
                  <Stop size={14} weight="fill" />
                  {t("admin.stop")}
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "deleteJob")}
                <button class="btn danger" type="button" onclick={() => runCommand(() => dispatchJobCommand(selectedInstance, selectedJob, "deleteJob"))}>
                  <Trash size={14} />
                  {t("admin.delete")}
                </button>
              {/if}
            </div>
          </div>

          <div class="batch-actions" aria-label={t("admin.batchAria")}>
            <button class="btn" type="button" onclick={() => runCommand(() => dispatchBatchCommand(selectedInstance, "pauseJob"))}>{t("admin.pauseAll")}</button>
            <button class="btn" type="button" onclick={() => runCommand(() => dispatchBatchCommand(selectedInstance, "resumeJob"))}>{t("admin.resumeAll")}</button>
            <button class="btn danger" type="button" onclick={() => runCommand(() => dispatchBatchCommand(selectedInstance, "stopJob"))}>{t("admin.stopExecutions")}</button>
          </div>

          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>{t("history.flow")}</th>
                  <th>{t("common.status")}</th>
                  <th>{t("history.source")}</th>
                  <th>{t("history.target")}</th>
                </tr>
              </thead>
              <tbody>
                {#each selectedInstance.jobs as job}
                  <tr
                    class:selected={selectedJob?.id === job.id}
                    onclick={() => (selectedJobId = job.id)}
                  >
                    <td>{job.name}</td>
                    <td>
                      <span class="badge {statusClass(job.status)}">{jobRuntimeStatusLabel(job.status)}</span>
                    </td>
                    <td class="path">{job.sourcePath}</td>
                    <td class="path">{job.targetPath}</td>
                  </tr>
                {:else}
                  <tr>
                    <td colspan="4" class="muted">{t("admin.noJobs")}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>

        <section class="subsection" aria-label={t("admin.activeExecutionsAria")}>
          <h3>{t("admin.activeExecutionsAria")}</h3>
          <div class="execution-list">
            {#each selectedInstance.activeExecutions as execution}
              <article>
                <div>
                  <strong>{execution.jobName}</strong>
                  <span>{execution.percent.toFixed(1)}%</span>
                </div>
                <p class="muted">{execution.currentFile || t("admin.waitingFile")}</p>
                {#if isCommandAvailable(selectedInstance, "cancelExecution")}
                  <button
                    class="btn danger"
                    type="button"
                    onclick={() => runCommand(() => cancelExecution(selectedInstance, execution.executionId))}
                  >
                    <Stop size={14} weight="fill" />
                    {t("common.cancel")}
                  </button>
                {/if}
              </article>
            {:else}
              <p class="muted">{t("admin.noActive")}</p>
            {/each}
          </div>
        </section>
      </section>

      <aside class="panel command-panel" aria-label={t("admin.capabilitiesAria")}>
        <div class="section-head">
          <h2>{t("admin.commands")}</h2>
          <WarningCircle size={18} weight="duotone" aria-hidden="true" />
        </div>

        <section>
          <h3>{t("admin.availableNow")}</h3>
          <div class="capability-list">
            {#each availableCapabilities as capability}
              <span>{commandKindLabel(capability.kind)} · {capability.scope}</span>
            {:else}
              <span class="muted">{t("admin.noCommands")}</span>
            {/each}
          </div>
        </section>

        <section>
          <h3>{t("admin.planned")}</h3>
          <div class="capability-list planned">
            {#each plannedCapabilities as capability}
              <span>{commandKindLabel(capability.kind)} · {capability.scope}</span>
            {/each}
          </div>
        </section>

        <section class="heartbeat-panel" aria-label={t("admin.heartbeatAria")}>
          <div class="section-head">
            <h3>{t("admin.heartbeat")}</h3>
            <div class="heartbeat-actions">
              <button class="btn" type="button" onclick={copyHeartbeatPayload} disabled={!heartbeat}>
                {t("admin.copyJson")}
              </button>
              <button class="btn" type="button" onclick={copySignedHeartbeatPayload} disabled={!heartbeat}>
                {t("admin.copySigned")}
              </button>
              <button class="btn primary" type="button" onclick={sendSignedHeartbeat} disabled={!heartbeat}>
                {t("admin.send")}
              </button>
            </div>
          </div>
          {#if heartbeat}
            <dl>
              <div>
                <dt>{t("admin.generated")}</dt>
                <dd>{formatDateTime(heartbeat.generatedAt)}</dd>
              </div>
              <div>
                <dt>{t("admin.lastSignal")}</dt>
                <dd>{formatDateTime(heartbeat.instance.lastSeenAt)}</dd>
              </div>
              <div>
                <dt>{t("admin.pending")}</dt>
                <dd>{heartbeat.pendingCommandCount}</dd>
              </div>
              <div>
                <dt>{t("admin.mode")}</dt>
                <dd>{managementModeLabel(heartbeat.instance.management.mode)}</dd>
              </div>
            </dl>
          {:else}
            <p class="muted">{t("admin.heartbeatUnavailable")}</p>
          {/if}
        </section>

        <section>
          <div class="section-head">
            <h3>{t("admin.localQueue")}</h3>
            <button class="btn" type="button" onclick={() => runCommand(processNextCommand)}>{t("admin.processNext")}</button>
          </div>
          {#if queueSummary}
            <div class="queue-summary" aria-label={t("admin.queueSummaryAria")}>
              <span>{t("admin.pendingLabel", { count: queueSummary.pending })}</span>
              <span>{t("admin.runningLabel", { count: queueSummary.running })}</span>
              <span>{t("admin.failedLabel", { count: queueSummary.failed })}</span>
            </div>
          {/if}
          <div class="queue-list">
            {#each queuedCommands as command}
              <article>
                <strong>{commandKindLabel(command.request.kind)}</strong>
                <span class="badge {queuedStatusClass(command.status)}">
                  {queuedCommandStatusLabel(command.status)}
                </span>
                <small>{command.source} · {formatDateTime(command.createdAt)}</small>
                {#if command.result}
                  <small>{command.result.results.map((entry) => entry.message).join(" | ")}</small>
                {/if}
              </article>
            {:else}
              <span class="muted">{t("admin.noCommandsQueued")}</span>
            {/each}
          </div>
        </section>
      </aside>
    </div>
  {/if}
</section>

<style>
  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(8.5rem, 1fr));
    gap: var(--space-3);
  }

  .metric-card {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    min-height: 5rem;
    padding: var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .metric-card span,
  .profile-grid span {
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .metric-card strong,
  .profile-grid strong {
    display: block;
    font-size: var(--text-lg);
  }

  .metric-card.success strong {
    color: var(--success);
  }

  .metric-card.warning strong {
    color: #b7791f;
  }

  .admin-layout {
    display: grid;
    grid-template-columns: minmax(13rem, 16rem) minmax(0, 1fr) minmax(15rem, 19rem);
    gap: var(--space-4);
    align-items: start;
  }

  .section-head,
  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  h2,
  h3 {
    margin: 0;
    font-size: var(--text-sm);
  }

  .detail-header p {
    margin: var(--space-1) 0 0;
    word-break: break-all;
  }

  .machine-list,
  .subsection,
  .command-panel,
  .capability-list,
  .execution-list,
  .heartbeat-panel,
  .queue-list {
    display: grid;
    gap: var(--space-3);
  }

  .machine-list button {
    display: grid;
    gap: var(--space-1);
    width: 100%;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
  }

  .machine-list button.selected,
  tr.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .machine-name {
    font-weight: 650;
  }

  .profile-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-3);
    margin: var(--space-4) 0;
  }

  .management-grid {
    margin: 0;
  }

  .job-actions,
  .batch-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .batch-actions {
    justify-content: flex-start;
  }

  .profile-grid div,
  .network-list,
  .execution-list article {
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
  }

  .network-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .table-wrap {
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  th,
  td {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--border);
    text-align: left;
    vertical-align: top;
  }

  tbody tr {
    cursor: pointer;
  }

  .path {
    max-width: 14rem;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    overflow-wrap: anywhere;
  }

  .execution-list article div {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .execution-list p {
    margin: var(--space-2) 0;
  }

  .capability-list span {
    display: block;
    padding: 0.45rem 0.55rem;
    border-radius: var(--radius-sm);
    background: var(--surface-muted);
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .queue-list article {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
  }

  .heartbeat-panel dl {
    display: grid;
    gap: var(--space-2);
    margin: 0;
  }

  .heartbeat-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .heartbeat-panel dl div,
  .queue-summary {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
  }

  .heartbeat-panel dt,
  .queue-summary,
  .queue-list small {
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .heartbeat-panel dd {
    margin: 0;
    color: var(--text-primary);
    overflow-wrap: anywhere;
  }

  .queue-summary {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .capability-list.planned span {
    border: 1px dashed var(--border);
  }

  .badge.success {
    background: color-mix(in srgb, var(--success) 12%, transparent);
    color: var(--success);
  }

  .badge.warning {
    background: color-mix(in srgb, #b7791f 14%, transparent);
    color: #9a5f16;
  }

  .badge.danger {
    background: color-mix(in srgb, var(--danger) 12%, transparent);
    color: var(--danger);
  }

  .badge.muted {
    background: var(--surface-muted);
    color: var(--text-muted);
  }

  .banner {
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-secondary);
  }

  .banner.error {
    border-color: color-mix(in srgb, var(--danger) 30%, var(--border));
    color: var(--danger);
  }

  @media (max-width: 1100px) {
    .summary-grid,
    .admin-layout,
    .profile-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
