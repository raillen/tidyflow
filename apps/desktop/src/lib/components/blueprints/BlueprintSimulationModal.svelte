<script lang="ts">
  import Modal from "$lib/components/ui/Modal.svelte";
  import type { BlueprintSimulationReport } from "$lib/contracts/blueprint";
  import { t } from "$lib/i18n";

  type Props = {
    open: boolean;
    report: BlueprintSimulationReport | null;
    blueprintName?: string;
    onclose: () => void;
  };

  let { open, report, blueprintName = "Blueprint", onclose }: Props = $props();
</script>

<Modal
  {open}
  title={t("blueprintSim.title")}
  description={blueprintName}
  size="md"
  {onclose}
>
  {#if report}
    <p class="summary">
      {t("blueprintSim.summary", { matched: report.matched, skipped: report.skipped })}
    </p>

    {#if report.warnings.length}
      <section class="block">
        <h3>{t("simulation.warnings")}</h3>
        <ul>
          {#each report.warnings as warning}
            <li>{warning}</li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if report.collisions.length}
      <section class="block">
        <h3>{t("blueprintSim.collisions")}</h3>
        <ul class="sample">
          {#each report.collisions as row}
            <li>
              <code>{row.source}</code>
              <span class="arrow">→</span>
              <code>{row.target}</code>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if report.planSample.length}
      <section class="block">
        <h3>{t("blueprintSim.planSample")}</h3>
        <ul class="sample">
          {#each report.planSample as row}
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
    <p class="muted">{t("simulation.empty")}</p>
  {/if}

  {#snippet footer()}
    <button type="button" class="btn primary" onclick={onclose}>{t("common.close")}</button>
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
