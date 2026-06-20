<script lang="ts">
  import Modal from "$lib/components/ui/Modal.svelte";
  import type { SimulationReport } from "$lib/contracts/job";

  type Props = {
    open: boolean;
    report: SimulationReport | null;
    jobName?: string;
    onclose: () => void;
  };

  let { open, report, jobName = "Fluxo", onclose }: Props = $props();
</script>

<Modal
  {open}
  title="Simulação"
  description={jobName}
  size="md"
  {onclose}
>
  {#if report}
    <p class="summary">
      <strong>{report.filesMatched}</strong> arquivo(s) correspondentes,
      <strong>{report.filesSkipped}</strong> ignorados
    </p>

    {#if report.warnings.length}
      <section class="block">
        <h3>Avisos</h3>
        <ul>
          {#each report.warnings as warning}
            <li>{warning}</li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if report.sample.length}
      <section class="block">
        <h3>Amostra</h3>
        <ul class="sample">
          {#each report.sample as row}
            <li>
              <code>{row.source}</code>
              <span class="arrow">→</span>
              <code>{row.target}</code>
              <span class="action">({row.action})</span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  {:else}
    <p class="muted">Sem dados de simulação.</p>
  {/if}

  {#snippet footer()}
    <button type="button" class="btn primary" onclick={onclose}>Fechar</button>
  {/snippet}
</Modal>

<style>
  .summary {
    margin: 0 0 var(--space-4);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .block {
    margin-bottom: var(--space-4);
  }

  h3 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  ul {
    margin: 0;
    padding-left: 1rem;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .sample {
    list-style: none;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }

  .sample li {
    font-size: var(--text-xs);
    word-break: break-all;
  }

  code {
    font-family: var(--font-mono);
  }

  .arrow {
    color: var(--text-muted);
    margin: 0 var(--space-1);
  }

  .action {
    color: var(--text-muted);
  }

  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
