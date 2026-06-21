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
    listAdminCommands,
    processNextAdminCommand,
  } from "$lib/core/ipc/client";

  let snapshot = $state<AdminFleetSnapshot | null>(null);
  let heartbeat = $state<AdminHeartbeatPayload | null>(null);
  let queueSummary = $state<AdminCommandQueueSummary | null>(null);
  let selectedInstanceId = $state<string | null>(null);
  let selectedJobId = $state<string | null>(null);
  let queuedCommands = $state<AdminQueuedCommand[]>([]);
  let loading = $state(true);
  let message = $state<string | null>(null);

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

  async function refresh() {
    loading = true;
    message = null;
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
      message = error instanceof Error ? error.message : "Falha ao carregar painel admin";
    } finally {
      loading = false;
    }
  }

  async function startJob(instance: AdminInstanceSnapshot, job: AdminManagedJob) {
    await dispatchJobCommand(instance, job, "startJob");
  }

  async function cancelExecution(instance: AdminInstanceSnapshot, executionId: string) {
    message = null;
    const result = await dispatchAdminCommand({
      kind: "cancelExecution",
      targetInstanceIds: [instance.instanceId],
      jobIds: [],
      executionIds: [executionId],
      reason: null,
    });
    message = result.results.map((entry) => entry.message).join(" | ");
    await refresh();
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
    const result = await dispatchAdminCommand(createAdminCommandRequest(kind, instance.instanceId, job.id));
    message = result.results.map((entry) => entry.message).join(" | ");
    await refresh();
  }

  async function enqueueJobCommand(
    instance: AdminInstanceSnapshot,
    job: AdminManagedJob,
    kind: AdminCommandKind,
  ) {
    message = null;
    const queued = await enqueueAdminCommand(
      createAdminCommandRequest(kind, instance.instanceId, job.id),
      "local-ui",
    );
    message = `Comando enfileirado: ${commandKindLabel(queued.request.kind)}`;
    [queuedCommands, queueSummary] = await Promise.all([
      listAdminCommands(30),
      fetchAdminCommandQueueSummary(),
    ]);
  }

  async function processNextCommand() {
    message = null;
    const processed = await processNextAdminCommand();
    message = processed
      ? `Comando processado: ${commandKindLabel(processed.request.kind)}`
      : "Nenhum comando pendente.";
    await refresh();
  }

  async function dispatchBatchCommand(instance: AdminInstanceSnapshot, kind: AdminCommandKind) {
    if (!instance.management.allowBatchCommands) {
      message = "Ações em lote estão desativadas nesta instância.";
      return;
    }
    if ((kind === "deleteJob" || kind === "stopJob") && !confirmCommand(kind, "todos os fluxos")) {
      return;
    }

    message = null;
    const result = await dispatchAdminCommand({
      kind,
      targetInstanceIds: [instance.instanceId],
      jobIds: instance.jobs.map((job) => job.id),
      executionIds: [],
      reason: null,
    });
    message = result.results.map((entry) => entry.message).join(" | ");
    await refresh();
  }

  function confirmCommand(kind: AdminCommandKind, targetName: string): boolean {
    const label = commandKindLabel(kind).toLowerCase();
    return window.confirm(`Confirmar ${label} para ${targetName}?`);
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
    return mode === "managedAgent" ? "Agent gerenciado" : "Somente local";
  }

  async function copyHeartbeatPayload() {
    if (!heartbeat) {
      return;
    }
    try {
      await navigator.clipboard.writeText(JSON.stringify(heartbeat, null, 2));
      message = "Heartbeat copiado para a area de transferencia.";
    } catch {
      message = "Nao foi possivel copiar o heartbeat neste ambiente.";
    }
  }
</script>

<section class="page admin-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Painel administrativo</h1>
      <p class="page-desc">Instâncias, hardware, rede, fluxos, execuções e comandos de operação.</p>
    </div>
    <button class="btn" type="button" onclick={refresh} disabled={loading}>
      <ArrowClockwise size={14} />
      Atualizar
    </button>
  </header>

  {#if snapshot}
    <section class="summary-grid" aria-label="Resumo da frota">
      <article class="metric-card">
        <DesktopTower size={18} weight="duotone" aria-hidden="true" />
        <div>
          <span>Instâncias</span>
          <strong>{snapshot.summary.totalInstances}</strong>
        </div>
      </article>
      <article class="metric-card success">
        <span>Online</span>
        <strong>{snapshot.summary.onlineInstances}</strong>
      </article>
      <article class="metric-card warning">
        <span>Atenção</span>
        <strong>{snapshot.summary.warningInstances}</strong>
      </article>
      <article class="metric-card">
        <ChartBar size={18} weight="duotone" aria-hidden="true" />
        <div>
          <span>Fluxos</span>
          <strong>{snapshot.summary.totalJobs}</strong>
        </div>
      </article>
      <article class="metric-card success">
        <span>Rodando</span>
        <strong>{snapshot.summary.runningJobs}</strong>
      </article>
      <article class="metric-card warning">
        <span>Fila pendente</span>
        <strong>{queueSummary?.pending ?? 0}</strong>
      </article>
      <article class="metric-card">
        <span>Fila finalizada</span>
        <strong>{(queueSummary?.completed ?? 0) + (queueSummary?.skipped ?? 0)}</strong>
      </article>
    </section>
  {/if}

  {#if message}
    <p class="banner" class:error={!snapshot} role="status">{message}</p>
  {/if}

  {#if loading && !snapshot}
    <p class="muted" aria-live="polite">Carregando painel administrativo...</p>
  {:else if snapshot && selectedInstance}
    <div class="admin-layout">
      <section class="panel instance-list" aria-label="Instâncias gerenciadas">
        <div class="section-head">
          <h2>Máquinas</h2>
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

      <section class="panel instance-detail" aria-label="Detalhes da instância">
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
            <span>Sistema</span>
            <strong>{selectedInstance.hardware.operatingSystem}</strong>
          </div>
          <div>
            <span>Arquitetura</span>
            <strong>{selectedInstance.hardware.architecture}</strong>
          </div>
          <div>
            <span>CPU</span>
            <strong>{selectedInstance.hardware.cpuThreads} threads</strong>
          </div>
          <div>
            <span>Versão</span>
            <strong>{selectedInstance.hardware.appVersion}</strong>
          </div>
        </div>

        <section class="subsection" aria-label="Gerenciamento">
          <h3>Gerenciamento</h3>
          <div class="profile-grid management-grid">
            <div>
              <span>Modo</span>
              <strong>{managementModeLabel(selectedInstance.management.mode)}</strong>
            </div>
            <div>
              <span>Servidor</span>
              <strong>{selectedInstance.management.serverUrl ?? "não configurado"}</strong>
            </div>
            <div>
              <span>Comandos remotos</span>
              <strong>{selectedInstance.management.allowRemoteCommands ? "permitidos" : "bloqueados"}</strong>
            </div>
            <div>
              <span>Ações em lote</span>
              <strong>{selectedInstance.management.allowBatchCommands ? "permitidas" : "bloqueadas"}</strong>
            </div>
          </div>
        </section>

        <section class="subsection" aria-label="Rede">
          <h3>Rede</h3>
          <div class="network-list">
            {#each selectedInstance.network.interfaces as network}
              <span>{network.name}: {network.address ?? "sem endereço"}</span>
            {/each}
          </div>
        </section>

        <section class="subsection" aria-label="Fluxos da instância">
          <div class="section-head">
            <h3>Fluxos</h3>
            <div class="job-actions">
              {#if selectedJob && isCommandAvailable(selectedInstance, "startJob")}
                <button class="btn primary" type="button" onclick={() => startJob(selectedInstance, selectedJob)}>
                  <Play size={14} weight="fill" />
                  Iniciar
                </button>
                <button class="btn" type="button" onclick={() => enqueueJobCommand(selectedInstance, selectedJob, "startJob")}>
                  Enfileirar
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "pauseJob")}
                <button class="btn" type="button" onclick={() => dispatchJobCommand(selectedInstance, selectedJob, "pauseJob")}>
                  <Pause size={14} weight="fill" />
                  Pausar
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "resumeJob")}
                <button class="btn" type="button" onclick={() => dispatchJobCommand(selectedInstance, selectedJob, "resumeJob")}>
                  <Play size={14} />
                  Continuar
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "stopJob")}
                <button class="btn danger" type="button" onclick={() => dispatchJobCommand(selectedInstance, selectedJob, "stopJob")}>
                  <Stop size={14} weight="fill" />
                  Parar
                </button>
              {/if}
              {#if selectedJob && isCommandAvailable(selectedInstance, "deleteJob")}
                <button class="btn danger" type="button" onclick={() => dispatchJobCommand(selectedInstance, selectedJob, "deleteJob")}>
                  <Trash size={14} />
                  Deletar
                </button>
              {/if}
            </div>
          </div>

          <div class="batch-actions" aria-label="Ações em lote locais">
            <button class="btn" type="button" onclick={() => dispatchBatchCommand(selectedInstance, "pauseJob")}>Pausar todos</button>
            <button class="btn" type="button" onclick={() => dispatchBatchCommand(selectedInstance, "resumeJob")}>Continuar todos</button>
            <button class="btn danger" type="button" onclick={() => dispatchBatchCommand(selectedInstance, "stopJob")}>Parar execuções</button>
          </div>

          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Fluxo</th>
                  <th>Status</th>
                  <th>Origem</th>
                  <th>Destino</th>
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
                    <td colspan="4" class="muted">Nenhum fluxo cadastrado nesta instância.</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>

        <section class="subsection" aria-label="Execuções ativas">
          <h3>Execuções ativas</h3>
          <div class="execution-list">
            {#each selectedInstance.activeExecutions as execution}
              <article>
                <div>
                  <strong>{execution.jobName}</strong>
                  <span>{execution.percent.toFixed(1)}%</span>
                </div>
                <p class="muted">{execution.currentFile || "Aguardando arquivo"}</p>
                {#if isCommandAvailable(selectedInstance, "cancelExecution")}
                  <button
                    class="btn danger"
                    type="button"
                    onclick={() => cancelExecution(selectedInstance, execution.executionId)}
                  >
                    <Stop size={14} weight="fill" />
                    Cancelar
                  </button>
                {/if}
              </article>
            {:else}
              <p class="muted">Nenhuma execução ativa.</p>
            {/each}
          </div>
        </section>
      </section>

      <aside class="panel command-panel" aria-label="Capacidades administrativas">
        <div class="section-head">
          <h2>Comandos</h2>
          <WarningCircle size={18} weight="duotone" aria-hidden="true" />
        </div>

        <section>
          <h3>Disponíveis agora</h3>
          <div class="capability-list">
            {#each availableCapabilities as capability}
              <span>{commandKindLabel(capability.kind)} · {capability.scope}</span>
            {:else}
              <span class="muted">Nenhum comando local disponível.</span>
            {/each}
          </div>
        </section>

        <section>
          <h3>Planejados para web/agente</h3>
          <div class="capability-list planned">
            {#each plannedCapabilities as capability}
              <span>{commandKindLabel(capability.kind)} · {capability.scope}</span>
            {/each}
          </div>
        </section>

        <section class="heartbeat-panel" aria-label="Heartbeat local">
          <div class="section-head">
            <h3>Heartbeat</h3>
            <button class="btn" type="button" onclick={copyHeartbeatPayload} disabled={!heartbeat}>
              Copiar JSON
            </button>
          </div>
          {#if heartbeat}
            <dl>
              <div>
                <dt>Gerado</dt>
                <dd>{formatDateTime(heartbeat.generatedAt)}</dd>
              </div>
              <div>
                <dt>Ultimo sinal</dt>
                <dd>{formatDateTime(heartbeat.instance.lastSeenAt)}</dd>
              </div>
              <div>
                <dt>Pendentes</dt>
                <dd>{heartbeat.pendingCommandCount}</dd>
              </div>
              <div>
                <dt>Modo</dt>
                <dd>{managementModeLabel(heartbeat.instance.management.mode)}</dd>
              </div>
            </dl>
          {:else}
            <p class="muted">Heartbeat indisponivel.</p>
          {/if}
        </section>

        <section>
          <div class="section-head">
            <h3>Fila local</h3>
            <button class="btn" type="button" onclick={processNextCommand}>Processar próximo</button>
          </div>
          {#if queueSummary}
            <div class="queue-summary" aria-label="Resumo da fila local">
              <span>Pendentes: {queueSummary.pending}</span>
              <span>Executando: {queueSummary.running}</span>
              <span>Falhas: {queueSummary.failed}</span>
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
              <span class="muted">Nenhum comando na fila.</span>
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
