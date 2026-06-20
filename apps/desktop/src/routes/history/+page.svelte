<script lang="ts">
  import { onMount } from "svelte";
  import {
    ArrowClockwise,
    ClockCounterClockwise,
    Funnel,
  } from "phosphor-svelte";
  import {
    AUDIT_STATUS_OPTIONS,
    auditStatusLabel,
    formatDateTime,
    formatDuration,
    formatFileSize,
    type AuditEntry,
    type AuditStatus,
  } from "$lib/contracts/audit";
  import { listRecentAudit } from "$lib/core/ipc/client";

  let entries = $state<AuditEntry[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let statusFilter = $state<(typeof AUDIT_STATUS_OPTIONS)[number]["value"]>("all");

  const filtered = $derived(
    statusFilter === "all"
      ? entries
      : entries.filter((e) => e.status === statusFilter),
  );

  async function refresh() {
    loading = true;
    error = null;
    try {
      entries = await listRecentAudit(200);
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao carregar histórico";
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  function statusClass(status: AuditStatus): string {
    switch (status) {
      case "COPIED":
      case "MOVED":
        return "success";
      case "IGNORED":
        return "muted";
      case "FAILED":
        return "danger";
    }
  }
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Histórico</h1>
      <p class="page-desc">Registro de arquivos processados por execução.</p>
    </div>
    <button class="btn" type="button" onclick={refresh} disabled={loading}>
      <ArrowClockwise size={14} />
      Atualizar
    </button>
  </header>

  <div class="toolbar panel">
    <label class="filter">
      <Funnel size={14} aria-hidden="true" />
      <span class="field-label">Status</span>
      <select class="field-input" bind:value={statusFilter}>
        {#each AUDIT_STATUS_OPTIONS as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </label>
    <span class="count muted">
      {filtered.length} registro(s)
    </span>
  </div>

  {#if error}
    <p class="banner error" role="alert">{error}</p>
  {/if}

  {#if loading}
    <p class="muted">Carregando…</p>
  {:else if filtered.length === 0}
    <div class="empty panel">
      <ClockCounterClockwise size={28} weight="duotone" aria-hidden="true" />
      <p>Nenhum registro encontrado.</p>
      <span class="muted">Execute um fluxo para gerar entradas de auditoria.</span>
    </div>
  {:else}
    <div class="table-wrap panel" role="region" aria-label="Registros de auditoria">
      <table>
        <thead>
          <tr>
            <th>Data</th>
            <th>Fluxo</th>
            <th>Arquivo</th>
            <th>Status</th>
            <th>Tamanho</th>
            <th>Duração</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as entry}
            <tr>
              <td class="mono">{formatDateTime(entry.createdAt)}</td>
              <td>{entry.jobName}</td>
              <td class="path" title={entry.sourcePath}>
                <span class="source">{entry.sourcePath}</span>
                {#if entry.targetPath}
                  <span class="arrow">→</span>
                  <span class="target">{entry.targetPath}</span>
                {/if}
                {#if entry.details}
                  <span class="details" title={entry.details}>{entry.details}</span>
                {/if}
              </td>
              <td>
                <span class="badge status {statusClass(entry.status)}">
                  {auditStatusLabel(entry.status)}
                </span>
              </td>
              <td class="mono">{formatFileSize(entry.fileSize)}</td>
              <td class="mono">{formatDuration(entry.durationMs)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
  }

  .filter {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .filter select {
    min-width: 8rem;
  }

  .count {
    font-size: var(--text-xs);
  }

  .banner {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    font-size: var(--text-sm);
  }

  .banner.error {
    border-color: var(--danger);
    color: var(--danger);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6);
    text-align: center;
    color: var(--accent);
  }

  .empty p {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
  }

  .table-wrap {
    padding: 0;
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-xs);
  }

  th,
  td {
    padding: 0.55rem 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--border);
    vertical-align: top;
  }

  th {
    position: sticky;
    top: 0;
    background: var(--surface);
    color: var(--text-muted);
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.625rem;
  }

  tbody tr:hover {
    background: var(--surface-muted);
  }

  .mono {
    font-family: var(--font-mono);
    white-space: nowrap;
  }

  .path {
    max-width: 22rem;
  }

  .source,
  .target {
    display: block;
    word-break: break-all;
  }

  .arrow {
    color: var(--text-muted);
    margin: 0 var(--space-1);
  }

  .details {
    display: block;
    margin-top: var(--space-1);
    color: var(--danger);
    font-size: 0.625rem;
  }

  .badge.status.success {
    background: color-mix(in srgb, var(--success) 14%, transparent);
    color: var(--success);
  }

  .badge.status.muted {
    background: var(--surface-muted);
    color: var(--text-muted);
  }

  .badge.status.danger {
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    color: var(--danger);
  }
</style>
