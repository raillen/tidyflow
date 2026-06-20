<script lang="ts">
  import { onMount } from "svelte";
  import {
    ArrowRight,
    BookOpen,
    ClockCounterClockwise,
    ListChecks,
    Pulse,
  } from "phosphor-svelte";
  import {
    auditStatusLabel,
    formatDateTime,
    type AuditEntry,
  } from "$lib/contracts/audit";
  import { fetchHealth, listRecentAudit } from "$lib/core/ipc/client";
  import type { HealthStatus } from "$lib/contracts/settings";

  let health = $state<HealthStatus | null>(null);
  let recentAudit = $state<AuditEntry[]>([]);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      health = await fetchHealth();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao contactar o core Rust";
    }

    try {
      recentAudit = await listRecentAudit(5);
    } catch {
      /* fora do Tauri */
    }
  });
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Dashboard</h1>
      <p class="page-desc">Visão geral do AutoFlow v2.</p>
    </div>
  </header>

  <div class="grid">
    <article class="card">
      <div class="card-head">
        <Pulse size={15} weight="duotone" aria-hidden="true" />
        <h2>Core Rust</h2>
      </div>
      {#if health}
        <p class="stat ok">{health.status.toUpperCase()}</p>
        <dl>
          <div><dt>Versão</dt><dd>{health.version}</dd></div>
          <div><dt>Módulo</dt><dd>{health.core}</dd></div>
        </dl>
      {:else if error}
        <p class="stat error" role="alert">{error}</p>
        <p class="muted">Execute via <code>pnpm dev</code> na raiz do monorepo.</p>
      {:else}
        <p class="muted" aria-live="polite">A contactar o agente…</p>
      {/if}
    </article>

    <article class="card">
      <div class="card-head">
        <ClockCounterClockwise size={15} weight="duotone" aria-hidden="true" />
        <h2>Atividade recente</h2>
      </div>
      {#if recentAudit.length === 0}
        <p class="muted">Nenhuma execução registrada ainda.</p>
      {:else}
        <ul class="activity">
          {#each recentAudit as entry}
            <li>
              <span class="badge">{auditStatusLabel(entry.status)}</span>
              <span class="name">{entry.jobName}</span>
              <span class="time">{formatDateTime(entry.createdAt)}</span>
            </li>
          {/each}
        </ul>
        <a class="link" href="/history">
          Ver histórico completo
          <ArrowRight size={12} />
        </a>
      {/if}
    </article>

    <article class="card">
      <div class="card-head">
        <ListChecks size={15} weight="duotone" aria-hidden="true" />
        <h2>Próximas entregas</h2>
      </div>
      <ul class="roadmap">
        <li>Watch folder + agendamento</li>
        <li>Blueprints e templates</li>
        <li>Rollback de operações</li>
      </ul>
    </article>

    <article class="card">
      <div class="card-head">
        <BookOpen size={15} weight="duotone" aria-hidden="true" />
        <h2>Documentação</h2>
      </div>
      <p class="muted">
        Spec em <code>docs/v2/</code> — arquitetura, IPC e roadmap.
      </p>
    </article>
  </div>
</section>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--accent);
    margin-bottom: var(--space-3);
  }

  .card h2 {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 650;
  }

  .stat {
    font-size: var(--text-lg);
    font-weight: 700;
    margin: 0 0 var(--space-3);
    letter-spacing: -0.01em;
  }

  .stat.ok {
    color: var(--success);
  }

  .stat.error {
    color: var(--danger);
  }

  dl {
    margin: 0;
    display: grid;
    gap: var(--space-2);
  }

  dl div {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  dt {
    color: var(--text-muted);
  }

  .roadmap {
    margin: 0;
    padding-left: 1rem;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .activity {
    list-style: none;
    margin: 0 0 var(--space-3);
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }

  .activity li {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-1) var(--space-2);
    align-items: center;
    font-size: var(--text-xs);
  }

  .activity .name {
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .activity .time {
    grid-column: 2;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 0.625rem;
  }

  .link {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--accent);
  }

  code {
    font-family: var(--font-mono);
    font-size: 0.92em;
    background: var(--surface-muted);
    padding: 0.08rem 0.3rem;
    border-radius: 4px;
  }

  @media (max-width: 900px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
