<script lang="ts">
  import { onMount } from "svelte";
  import {
    ArrowClockwise,
    ChartBar,
    ClockCounterClockwise,
    DownloadSimple,
    Export,
    FileMagnifyingGlass,
    Funnel,
    MagnifyingGlass,
  } from "phosphor-svelte";
  import {
    AUDIT_STATUS_OPTIONS,
    auditFailureRate,
    auditStatusLabel,
    createDefaultAuditQuery,
    formatDateTime,
    formatDuration,
    formatFileSize,
    type AuditEntry,
    type AuditExportFormat,
    type AuditPage,
    type AuditQuery,
    type AuditStatus,
  } from "$lib/contracts/audit";
  import { exportAudit, queryAudit } from "$lib/core/ipc/client";

  let page = $state<AuditPage | null>(null);
  let loading = $state(true);
  let exporting = $state<AuditExportFormat | null>(null);
  let error = $state<string | null>(null);
  let selected = $state<AuditEntry | null>(null);
  let query = $state<AuditQuery>(createDefaultAuditQuery());
  let searchText = $state("");
  let statusFilter = $state<"all" | AuditStatus>("all");
  let dateFromInput = $state("");
  let dateToInput = $state("");

  const entries = $derived(page?.entries ?? []);
  const summary = $derived(page?.summary);
  const currentPage = $derived(Math.floor((query.offset ?? 0) / (query.limit ?? 100)) + 1);
  const totalPages = $derived(Math.max(1, Math.ceil((page?.total ?? 0) / (query.limit ?? 100))));

  onMount(refresh);

  function buildQuery(offset = 0): AuditQuery {
    return {
      ...query,
      search: searchText.trim() || null,
      status: statusFilter === "all" ? null : statusFilter,
      dateFrom: toIsoDate(dateFromInput),
      dateTo: toIsoDate(dateToInput),
      offset,
    };
  }

  function toIsoDate(value: string): string | null {
    if (!value) return null;
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? null : date.toISOString();
  }

  async function refresh() {
    loading = true;
    error = null;
    try {
      query = buildQuery(query.offset ?? 0);
      page = await queryAudit(query);
      if (selected && !page.entries.some((entry) => entry.id === selected?.id)) {
        selected = null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao carregar auditoria";
    } finally {
      loading = false;
    }
  }

  async function applyFilters() {
    query = buildQuery(0);
    await refresh();
  }

  async function changePage(direction: -1 | 1) {
    const limit = query.limit ?? 100;
    const nextOffset = Math.max(0, (query.offset ?? 0) + direction * limit);
    query = { ...query, offset: nextOffset };
    await refresh();
  }

  function resetFilters() {
    searchText = "";
    statusFilter = "all";
    dateFromInput = "";
    dateToInput = "";
    query = createDefaultAuditQuery();
    void refresh();
  }

  async function handleExport(format: AuditExportFormat) {
    exporting = format;
    error = null;
    try {
      const result = await exportAudit(buildQuery(0), format);
      const blob = new Blob([result.content], { type: result.mimeType });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = result.fileName;
      link.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao exportar auditoria";
    } finally {
      exporting = null;
    }
  }

  function statusClass(status: AuditStatus): string {
    switch (status) {
      case "COPIED":
      case "MOVED":
      case "ORGANIZED":
        return "success";
      case "IGNORED":
        return "muted";
      case "FAILED":
        return "danger";
    }
  }
</script>

<section class="page audit-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Auditoria e logs</h1>
      <p class="page-desc">Analytics, filtros, exportação e detalhes dos eventos processados.</p>
    </div>
    <div class="header-actions">
      <button class="btn" type="button" onclick={refresh} disabled={loading}>
        <ArrowClockwise size={14} />
        Atualizar
      </button>
      <button class="btn" type="button" onclick={() => handleExport("csv")} disabled={!!exporting}>
        <DownloadSimple size={14} />
        CSV
      </button>
      <button class="btn" type="button" onclick={() => handleExport("json")} disabled={!!exporting}>
        <Export size={14} />
        JSON
      </button>
    </div>
  </header>

  {#if summary}
    <section class="analytics-grid" aria-label="Resumo da auditoria">
      <article class="metric-card">
        <ChartBar size={18} weight="duotone" aria-hidden="true" />
        <div>
          <span>Total filtrado</span>
          <strong>{summary.total}</strong>
        </div>
      </article>
      <article class="metric-card success">
        <span>Sucesso</span>
        <strong>{summary.copied + summary.moved + summary.organized}</strong>
      </article>
      <article class="metric-card danger">
        <span>Falhas</span>
        <strong>{summary.failed}</strong>
        <small>{(auditFailureRate(summary) * 100).toFixed(1)}%</small>
      </article>
      <article class="metric-card">
        <span>Volume</span>
        <strong>{formatFileSize(summary.totalBytes)}</strong>
      </article>
      <article class="metric-card">
        <span>Duração média</span>
        <strong>{formatDuration(summary.averageDurationMs)}</strong>
      </article>
    </section>
  {/if}

  <form
    class="filters panel"
    aria-label="Filtros da auditoria"
    onsubmit={(event) => {
      event.preventDefault();
      void applyFilters();
    }}
  >
    <label class="field search-field">
      <span class="field-label">Busca</span>
      <span class="input-with-icon">
        <MagnifyingGlass size={14} aria-hidden="true" />
        <input
          class="field-input"
          placeholder="Fluxo, caminho, arquivo ou detalhe"
          bind:value={searchText}
        />
      </span>
    </label>

    <label class="field">
      <span class="field-label">Status</span>
      <select class="field-input" bind:value={statusFilter}>
        {#each AUDIT_STATUS_OPTIONS as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </label>

    <label class="field">
      <span class="field-label">De</span>
      <input class="field-input" type="datetime-local" bind:value={dateFromInput} />
    </label>

    <label class="field">
      <span class="field-label">Até</span>
      <input class="field-input" type="datetime-local" bind:value={dateToInput} />
    </label>

    <label class="field compact">
      <span class="field-label">Limite</span>
      <select
        class="field-input"
        value={query.limit}
        onchange={(event) => (query = { ...query, limit: Number(event.currentTarget.value), offset: 0 })}
      >
        <option value="50">50</option>
        <option value="100">100</option>
        <option value="250">250</option>
        <option value="500">500</option>
      </select>
    </label>

    <div class="filter-actions">
      <button class="btn primary" type="submit">
        <Funnel size={14} />
        Filtrar
      </button>
      <button class="btn ghost" type="button" onclick={resetFilters}>Limpar</button>
    </div>
  </form>

  {#if error}
    <p class="banner error" role="alert">{error}</p>
  {/if}

  <div class="audit-layout">
    <section class="panel table-panel" aria-label="Registros de auditoria">
      {#if loading}
        <p class="muted">Carregando…</p>
      {:else if entries.length === 0}
        <div class="empty">
          <ClockCounterClockwise size={28} weight="duotone" aria-hidden="true" />
          <p>Nenhum registro encontrado.</p>
          <span class="muted">Ajuste os filtros ou execute um fluxo para gerar logs.</span>
        </div>
      {:else}
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Data</th>
                <th>Fluxo</th>
                <th>Origem e destino</th>
                <th>Status</th>
                <th>Tamanho</th>
                <th>Duração</th>
                <th><span class="sr-only">Ações</span></th>
              </tr>
            </thead>
            <tbody>
              {#each entries as entry}
                <tr class:selected={selected?.id === entry.id}>
                  <td class="mono">{formatDateTime(entry.createdAt)}</td>
                  <td>{entry.jobName}</td>
                  <td class="path" title={entry.sourcePath}>
                    <span class="source">{entry.sourcePath}</span>
                    {#if entry.targetPath}
                      <span class="target">{entry.targetPath}</span>
                    {/if}
                  </td>
                  <td>
                    <span class="badge status {statusClass(entry.status)}">
                      {auditStatusLabel(entry.status)}
                    </span>
                  </td>
                  <td class="mono">{formatFileSize(entry.fileSize)}</td>
                  <td class="mono">{formatDuration(entry.durationMs)}</td>
                  <td>
                    <button class="btn ghost small" type="button" onclick={() => (selected = entry)}>
                      Detalhes
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <footer class="pagination">
          <span class="muted">Página {currentPage} de {totalPages} · {page?.total ?? 0} registro(s)</span>
          <div>
            <button class="btn" type="button" disabled={(query.offset ?? 0) === 0} onclick={() => changePage(-1)}>
              Anterior
            </button>
            <button
              class="btn"
              type="button"
              disabled={currentPage >= totalPages}
              onclick={() => changePage(1)}
            >
              Próxima
            </button>
          </div>
        </footer>
      {/if}
    </section>

    <aside class="panel detail-panel" aria-label="Detalhes do registro">
      {#if selected}
        <div class="detail-head">
          <FileMagnifyingGlass size={18} weight="duotone" aria-hidden="true" />
          <div>
            <h2>Registro #{selected.id}</h2>
            <p>{formatDateTime(selected.createdAt)}</p>
          </div>
        </div>
        <dl>
          <div>
            <dt>Status</dt>
            <dd>{auditStatusLabel(selected.status)}</dd>
          </div>
          <div>
            <dt>Fluxo</dt>
            <dd>{selected.jobName}</dd>
          </div>
          <div>
            <dt>Origem</dt>
            <dd class="mono break">{selected.sourcePath}</dd>
          </div>
          <div>
            <dt>Destino</dt>
            <dd class="mono break">{selected.targetPath || "—"}</dd>
          </div>
          <div>
            <dt>Tamanho</dt>
            <dd>{formatFileSize(selected.fileSize)}</dd>
          </div>
          <div>
            <dt>Duração</dt>
            <dd>{formatDuration(selected.durationMs)}</dd>
          </div>
          <div>
            <dt>Detalhes</dt>
            <dd>{selected.details || "Sem detalhes adicionais."}</dd>
          </div>
        </dl>
      {:else}
        <div class="empty detail-empty">
          <FileMagnifyingGlass size={30} weight="duotone" aria-hidden="true" />
          <p>Selecione um registro.</p>
          <span class="muted">Os detalhes aparecem aqui sem abrir outra tela.</span>
        </div>
      {/if}
    </aside>
  </div>
</section>

<style>
  .audit-page {
    min-width: 0;
  }

  .header-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .analytics-grid {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .metric-card {
    min-height: 5.25rem;
    padding: var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    display: grid;
    align-content: center;
    gap: var(--space-1);
  }

  .metric-card span,
  .metric-card small {
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .metric-card strong {
    font-size: var(--text-lg);
    line-height: 1.2;
  }

  .metric-card.success strong {
    color: var(--success);
  }

  .metric-card.danger strong {
    color: var(--danger);
  }

  .filters {
    display: grid;
    grid-template-columns: minmax(16rem, 1.4fr) minmax(8rem, 0.7fr) repeat(2, minmax(11rem, 0.9fr)) minmax(6rem, 0.4fr) auto;
    gap: var(--space-3);
    align-items: end;
  }

  .input-with-icon {
    position: relative;
    display: block;
  }

  .input-with-icon :global(svg) {
    position: absolute;
    left: 0.55rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
  }

  .input-with-icon input {
    width: 100%;
    padding-left: 1.8rem;
  }

  .filter-actions {
    display: flex;
    gap: var(--space-2);
  }

  .audit-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(18rem, 24rem);
    gap: var(--space-4);
    align-items: start;
  }

  .table-panel {
    padding: 0;
    overflow: hidden;
  }

  .table-wrap {
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

  tbody tr:hover,
  tbody tr.selected {
    background: var(--surface-muted);
  }

  .mono {
    font-family: var(--font-mono);
    white-space: nowrap;
  }

  .break {
    white-space: normal;
    word-break: break-all;
  }

  .path {
    max-width: 30rem;
  }

  .source,
  .target {
    display: block;
    word-break: break-all;
  }

  .target {
    color: var(--text-muted);
    margin-top: var(--space-1);
  }

  .small {
    padding: 0.25rem 0.5rem;
    font-size: var(--text-xs);
  }

  .pagination {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
  }

  .pagination div {
    display: flex;
    gap: var(--space-2);
  }

  .detail-panel {
    position: sticky;
    top: var(--space-4);
    display: grid;
    gap: var(--space-4);
  }

  .detail-head {
    display: flex;
    gap: var(--space-3);
    color: var(--accent);
  }

  .detail-head h2 {
    margin: 0;
    font-size: var(--text-base);
    color: var(--text-primary);
  }

  .detail-head p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  dl {
    display: grid;
    gap: var(--space-3);
    margin: 0;
  }

  dt {
    color: var(--text-muted);
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  dd {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
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

  .detail-empty {
    padding: var(--space-4);
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

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  @media (max-width: 1180px) {
    .analytics-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .filters,
    .audit-layout {
      grid-template-columns: 1fr;
    }

    .detail-panel {
      position: static;
    }
  }

  @media (max-width: 720px) {
    .page-header,
    .pagination {
      flex-direction: column;
      align-items: stretch;
    }

    .analytics-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
