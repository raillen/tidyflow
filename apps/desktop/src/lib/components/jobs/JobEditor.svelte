<script lang="ts">
  /**
   * Job editor shell: the section navigation plus the active panel.
   *
   * The panels live in `./panels/*`; this file owns only navigation state and
   * the shared filter counter so each section stays independently testable.
   */
  import {
    CaretDoubleLeft,
    CaretDoubleRight,
    Code,
    Calendar,
    Eye,
    Funnel,
    IdentificationCard,
    ShieldCheck,
  } from "phosphor-svelte";
  import { countActiveFilters, type Job } from "$lib/contracts/job";
  import { t } from "$lib/i18n";
  import JobGeneralPanel from "./panels/JobGeneralPanel.svelte";
  import JobFiltersPanel from "./panels/JobFiltersPanel.svelte";
  import JobAutomationPanel from "./panels/JobAutomationPanel.svelte";
  import JobMonitoringPanel from "./panels/JobMonitoringPanel.svelte";
  import JobSecurityPanel from "./panels/JobSecurityPanel.svelte";
  import JobAdvancedPanel from "./panels/JobAdvancedPanel.svelte";

  type NavPanel = "geral" | "filtros" | "automacao" | "monitoramento" | "seguranca" | "avancado";

  type Props = {
    job: Job;
    embedded?: boolean;
    onsave?: () => void;
  };

  let { job = $bindable(), embedded = true, onsave }: Props = $props();

  let activePanel = $state<NavPanel>("geral");
  let navCollapsed = $state(false);

  const filterCount = $derived(countActiveFilters(job.filters));

  const navItems = $derived<{ id: NavPanel; label: string; icon: typeof IdentificationCard }[]>([
    { id: "geral", label: t("jobs.nav.general"), icon: IdentificationCard },
    { id: "filtros", label: t("jobs.nav.filters"), icon: Funnel },
    { id: "automacao", label: t("jobs.nav.automation"), icon: Calendar },
    { id: "monitoramento", label: t("jobs.nav.monitoring"), icon: Eye },
    { id: "seguranca", label: t("jobs.nav.security"), icon: ShieldCheck },
    { id: "avancado", label: t("jobs.nav.advanced"), icon: Code },
  ]);
</script>

<form
  id="job-editor-form"
  class="editor"
  class:embedded
  onsubmit={(event) => {
    event.preventDefault();
    onsave?.();
  }}
>
  <nav class="editor-nav" class:collapsed={navCollapsed} aria-label={t("jobs.nav.sectionsAria")}>
    <button
      type="button"
      class="nav-toggle btn ghost"
      aria-label={navCollapsed ? t("jobs.nav.expand") : t("jobs.nav.collapse")}
      onclick={() => (navCollapsed = !navCollapsed)}
    >
      {#if navCollapsed}
        <CaretDoubleRight size={16} />
      {:else}
        <CaretDoubleLeft size={16} />
      {/if}
    </button>

    {#each navItems as item}
      {@const Icon = item.icon}
      <button
        type="button"
        class="nav-item"
        class:active={activePanel === item.id}
        title={item.label}
        onclick={() => (activePanel = item.id)}
      >
        <Icon size={18} weight={activePanel === item.id ? "fill" : "regular"} />
        {#if !navCollapsed}
          <span class="nav-label">{item.label}</span>
          {#if item.id === "filtros" && filterCount > 0}
            <span class="badge">{filterCount}</span>
          {/if}
        {:else if item.id === "filtros" && filterCount > 0}
          <span class="nav-dot" aria-label={t("jobs.nav.activeFilters", { count: filterCount })}></span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="editor-content">
    {#if activePanel === "geral"}
      <JobGeneralPanel bind:job />
    {:else if activePanel === "filtros"}
      <JobFiltersPanel bind:job {filterCount} />
    {:else if activePanel === "automacao"}
      <JobAutomationPanel bind:job />
    {:else if activePanel === "monitoramento"}
      <JobMonitoringPanel bind:job />
    {:else if activePanel === "seguranca"}
      <JobSecurityPanel bind:job />
    {:else if activePanel === "avancado"}
      <JobAdvancedPanel bind:job />
    {/if}
  </div>
</form>

<style>
  .editor-nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    width: 168px;
    flex-shrink: 0;
    padding: var(--space-3);
    border-right: 1px solid var(--border);
    background: var(--surface-muted);
    transition: width 160ms ease;
  }

  .editor-nav.collapsed {
    width: 52px;
    padding-inline: var(--space-2);
  }

  :global(.editor-nav .nav-toggle) {
    align-self: flex-end;
    padding: var(--space-1);
    margin-bottom: var(--space-2);
  }

  .editor-nav.collapsed :global(.nav-toggle) {
    align-self: center;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    position: relative;
    transition: background 120ms ease, color 120ms ease;
  }

  .nav-item:hover {
    background: var(--surface);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .editor-nav.collapsed .nav-item {
    justify-content: center;
    padding-inline: var(--space-2);
  }

  .nav-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-dot {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  @media (max-width: 640px) {
    .editor-nav {
      flex-direction: row;
      width: 100%;
      overflow-x: auto;
      border-right: none;
      border-bottom: 1px solid var(--border);
    }

    .editor-nav.collapsed {
      width: 100%;
    }

    :global(.nav-toggle) {
      display: none;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .editor-nav,
    .nav-item {
      transition: none;
    }
  }
</style>
