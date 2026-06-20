<script lang="ts">
  import JobFormModal from "$lib/components/jobs/JobFormModal.svelte";
  import SimulationModal from "$lib/components/jobs/SimulationModal.svelte";
  import {
    cancelExecution,
    deleteJob,
    listJobs,
    runJob,
    simulateJob,
  } from "$lib/core/ipc/client";
  import type { JobSummary, SimulationReport } from "$lib/contracts/job";
  import {
    activeExecutions,
    lastCompleted,
  } from "$lib/core/stores/executions";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { Eye, PencilSimple, Play, Plus, Trash } from "phosphor-svelte";

  let jobs = $state<JobSummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busyId = $state<string | null>(null);

  let jobModalOpen = $state(false);
  let jobModalMode = $state<"create" | "edit">("create");
  let jobModalId = $state<string | null>(null);

  let simulationModalOpen = $state(false);
  let simulationReport = $state<SimulationReport | null>(null);
  let simulationJobName = $state("");

  async function refresh() {
    loading = true;
    error = null;
    try {
      jobs = await listJobs();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao carregar fluxos";
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  function openCreateModal() {
    jobModalMode = "create";
    jobModalId = null;
    jobModalOpen = true;
  }

  function openEditModal(id: string) {
    jobModalMode = "edit";
    jobModalId = id;
    jobModalOpen = true;
  }

  function closeJobModal() {
    jobModalOpen = false;
    const params = $page.url.searchParams;
    if (params.has("new") || params.has("edit")) {
      void goto("/flows", { replaceState: true, keepFocus: true, noScroll: true });
    }
  }

  function handleJobSaved() {
    void refresh();
  }

  $effect(() => {
    const params = $page.url.searchParams;
    const newParam = params.get("new");
    const editParam = params.get("edit");

    if (newParam === "1") {
      jobModalMode = "create";
      jobModalId = null;
      jobModalOpen = true;
    } else if (editParam) {
      jobModalMode = "edit";
      jobModalId = editParam;
      jobModalOpen = true;
    }
  });

  async function handleRun(id: string) {
    busyId = id;
    error = null;
    try {
      await runJob(id);
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao executar";
    } finally {
      busyId = null;
    }
  }

  async function handleSimulate(job: JobSummary) {
    busyId = job.id;
    error = null;
    simulationReport = null;
    try {
      simulationReport = await simulateJob(job.id);
      simulationJobName = job.name;
      simulationModalOpen = true;
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha na simulação";
    } finally {
      busyId = null;
    }
  }

  async function handleDelete(id: string) {
    if (!confirm("Excluir este fluxo?")) return;
    try {
      await deleteJob(id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao excluir";
    }
  }

  async function handleCancel(executionId: string) {
    try {
      await cancelExecution(executionId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao cancelar";
    }
  }

  function modeLabel(mode: JobSummary["mode"]) {
    return mode === "copy" ? "Cópia" : "Mover";
  }
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Fluxos</h1>
      <p class="page-desc">Jobs de cópia e movimentação de arquivos.</p>
    </div>
    <button type="button" class="btn primary" onclick={openCreateModal}>
      <Plus size={15} weight="bold" />
      Novo fluxo
    </button>
  </header>

  {#if error}
    <p class="banner error" role="alert">{error}</p>
  {/if}

  {#if $lastCompleted}
    <p class="banner" class:success={$lastCompleted.success} class:fail={!$lastCompleted.success}>
      Execução concluída — {$lastCompleted.processed} ok, {$lastCompleted.failed} falhas
      {#if $lastCompleted.errorMessage}
        ({$lastCompleted.errorMessage})
      {/if}
    </p>
  {/if}

  {#if Object.keys($activeExecutions).length > 0}
    <section class="panel" aria-label="Execuções ativas">
      <h2>Em execução</h2>
      <ul class="exec-list">
        {#each Object.values($activeExecutions) as exec}
          <li>
            <div class="exec-main">
              <strong>{exec.jobName}</strong>
              <span class="muted">{exec.currentFile}</span>
              <progress max="100" value={exec.percent}></progress>
              <span class="percent">{exec.percent.toFixed(0)}%</span>
            </div>
            <button class="btn ghost" onclick={() => handleCancel(exec.executionId)}>
              Cancelar
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if loading}
    <p class="muted">Carregando…</p>
  {:else if jobs.length === 0}
    <div class="empty panel">
      <p>Nenhum fluxo cadastrado.</p>
      <button type="button" class="btn primary" onclick={openCreateModal}>
        <Plus size={14} weight="bold" />
        Criar primeiro fluxo
      </button>
    </div>
  {:else}
    <ul class="job-list">
      {#each jobs as job}
        <li class="job-card">
          <div class="job-info">
            <button type="button" class="job-name" onclick={() => openEditModal(job.id)}>
              {job.name}
            </button>
            <span class="badge">{modeLabel(job.mode)}</span>
            {#if !job.enabled}
              <span class="badge muted">Desativado</span>
            {/if}
            <p class="paths muted">
              {job.sourcePath || "—"} → {job.targetPath || "—"}
            </p>
          </div>
          <div class="actions">
            <button
              class="btn ghost"
              title="Editar"
              aria-label="Editar fluxo"
              onclick={() => openEditModal(job.id)}
            >
              <PencilSimple size={14} />
            </button>
            <button
              class="btn primary"
              disabled={busyId === job.id || !job.enabled}
              onclick={() => handleRun(job.id)}
            >
              <Play size={14} weight="fill" />
              Executar
            </button>
            <button
              class="btn ghost"
              disabled={busyId === job.id}
              onclick={() => handleSimulate(job)}
            >
              <Eye size={14} />
              Simular
            </button>
            <button class="btn ghost danger" onclick={() => handleDelete(job.id)}>
              <Trash size={14} />
              Excluir
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<JobFormModal
  open={jobModalOpen}
  mode={jobModalMode}
  jobId={jobModalId}
  onclose={closeJobModal}
  onsaved={handleJobSaved}
  onsimulated={(report, jobName) => {
    simulationReport = report;
    simulationJobName = jobName;
    simulationModalOpen = true;
  }}
/>

<SimulationModal
  open={simulationModalOpen}
  report={simulationReport}
  jobName={simulationJobName}
  onclose={() => {
    simulationModalOpen = false;
  }}
/>

<style>
  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
    font-weight: 650;
  }

  .banner {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    border: 1px solid var(--border);
    font-size: var(--text-sm);
  }

  .banner.error {
    border-color: var(--danger);
    color: var(--danger);
  }

  .banner.success {
    border-color: var(--success);
  }

  .banner.fail {
    border-color: var(--danger);
  }

  .panel {
    padding: var(--space-4);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-5);
  }

  .empty p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .job-list,
  .exec-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .job-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .job-card {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: center;
    padding: var(--space-4);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
  }

  .job-name {
    padding: 0;
    border: none;
    background: none;
    font-weight: 600;
    font-size: var(--text-sm);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }

  .job-name:hover {
    color: var(--accent);
  }

  .paths {
    font-size: var(--text-xs);
    margin-top: var(--space-1);
    word-break: break-all;
  }

  .badge {
    margin-left: var(--space-2);
  }

  .badge.muted {
    background: var(--surface-muted);
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .exec-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .exec-list li {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: center;
  }

  .exec-main {
    flex: 1;
    display: grid;
    gap: var(--space-1);
    font-size: var(--text-sm);
  }

  progress {
    width: 100%;
    height: 5px;
    accent-color: var(--accent);
  }

  .percent {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
