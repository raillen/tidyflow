<script lang="ts">
  import BlueprintFormModal from "$lib/components/blueprints/BlueprintFormModal.svelte";
  import { errorMessage } from "$lib/core/errors";
  import BlueprintRunVariablesModal from "$lib/components/blueprints/BlueprintRunVariablesModal.svelte";
  import BlueprintSimulationModal from "$lib/components/blueprints/BlueprintSimulationModal.svelte";
  import {
    applyBlueprint,
    applyBlueprintWithVariables,
    deleteBlueprint,
    getBlueprint,
    listBlueprints,
    simulateBlueprint,
    simulateBlueprintWithVariables,
  } from "$lib/core/ipc/client";
  import type {
    Blueprint,
    BlueprintApplyResult,
    BlueprintKind,
    BlueprintSimulationReport,
    BlueprintSummary,
  } from "$lib/contracts/blueprint";
  import { blueprintKindLabel, blueprintOperationLabel } from "$lib/contracts/blueprint";
  import { onMount } from "svelte";
  import { Eye, FolderSimple, PencilSimple, Play, Plus, Trash } from "phosphor-svelte";
  import { t } from "$lib/i18n";

  let blueprints = $state<BlueprintSummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busyId = $state<string | null>(null);
  let applyResult = $state<BlueprintApplyResult | null>(null);
  let runModalOpen = $state(false);
  let runAction = $state<"apply" | "simulate">("apply");
  let runBlueprint = $state<Blueprint | null>(null);
  let runLoading = $state(false);
  let watchActiveMessage = $state<string | null>(null);
  let kindTab = $state<BlueprintKind | "all">("all");

  let modalOpen = $state(false);
  let modalMode = $state<"create" | "edit">("create");
  let modalId = $state<string | null>(null);
  let createKind = $state<BlueprintKind>("file");

  let simulationModalOpen = $state(false);
  let simulationReport = $state<BlueprintSimulationReport | null>(null);
  let simulationName = $state("");

  const filteredBlueprints = $derived(
    kindTab === "all" ? blueprints : blueprints.filter((bp) => bp.kind === kindTab),
  );

  async function refresh() {
    loading = true;
    error = null;
    try {
      blueprints = await listBlueprints();
    } catch (e) {
      error = errorMessage(e, t("blueprints.loadError"));
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  function openCreateModal(kind: BlueprintKind = "file") {
    modalMode = "create";
    modalId = null;
    createKind = kind;
    modalOpen = true;
  }

  function openEditModal(id: string) {
    modalMode = "edit";
    modalId = id;
    modalOpen = true;
  }

  function closeModal() {
    modalOpen = false;
  }

  function handleSaved() {
    void refresh();
  }

  async function handleApply(id: string) {
    busyId = id;
    error = null;
    applyResult = null;
    watchActiveMessage = null;
    try {
      const blueprint = await getBlueprint(id);
      if (blueprint.kind === "folder" && blueprint.watch?.enabled) {
        watchActiveMessage = t("blueprints.watchActive");
        return;
      }
      if (blueprint.kind === "folder" && !blueprint.folderPlan?.variables.length) {
        await executeFolderRun(blueprint, "apply", {});
        return;
      }
      if (blueprint.kind === "folder") {
        runBlueprint = blueprint;
        runAction = "apply";
        runModalOpen = true;
        return;
      }
      applyResult = await applyBlueprint(id);
      await refresh();
    } catch (e) {
      error = errorMessage(e, t("blueprints.applyError"));
    } finally {
      busyId = null;
    }
  }

  async function handleSimulate(bp: BlueprintSummary) {
    busyId = bp.id;
    error = null;
    simulationReport = null;
    watchActiveMessage = null;
    try {
      const blueprint = await getBlueprint(bp.id);
      if (blueprint.kind === "folder" && !blueprint.folderPlan?.variables.length) {
        await executeFolderRun(blueprint, "simulate", {});
        return;
      }
      if (blueprint.kind === "folder") {
        runBlueprint = blueprint;
        runAction = "simulate";
        runModalOpen = true;
        return;
      }
      simulationReport = await simulateBlueprint(bp.id);
      simulationName = bp.name;
      simulationModalOpen = true;
    } catch (e) {
      error = errorMessage(e, t("blueprints.simulateError"));
    } finally {
      busyId = null;
    }
  }

  async function executeFolderRun(
    blueprint: Blueprint,
    action: "apply" | "simulate",
    variables: Record<string, string>,
  ) {
    runLoading = true;
    busyId = blueprint.id;
    error = null;
    try {
      if (action === "apply") {
        applyResult = await applyBlueprintWithVariables(blueprint.id, variables);
        runModalOpen = false;
        runBlueprint = null;
        await refresh();
      } else {
        simulationReport = await simulateBlueprintWithVariables(blueprint.id, variables);
        simulationName = blueprint.name;
        simulationModalOpen = true;
        runModalOpen = false;
        runBlueprint = null;
      }
    } catch (e) {
      error = errorMessage(e, t("blueprints.executeError"));
    } finally {
      runLoading = false;
      busyId = null;
    }
  }

  function closeRunModal() {
    if (runLoading) return;
    runModalOpen = false;
    runBlueprint = null;
  }

  async function handleDelete(id: string) {
    if (!confirm(t("blueprints.deleteConfirm"))) return;
    try {
      await deleteBlueprint(id);
      await refresh();
    } catch (e) {
      error = errorMessage(e, t("blueprints.deleteError"));
    }
  }
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{t("blueprints.title")}</h1>
      <p class="page-desc">{t("blueprints.subtitle")}</p>
    </div>
    <button type="button" class="btn primary" onclick={() => openCreateModal(kindTab === "folder" ? "folder" : "file")}>
      <Plus size={15} weight="bold" />
      {t("blueprints.newBlueprint")}
    </button>
  </header>

  <div class="tabs" role="tablist" aria-label={t("blueprints.typeAria")}>
    <button
      type="button"
      class="tab"
      class:active={kindTab === "all"}
      role="tab"
      aria-selected={kindTab === "all"}
      onclick={() => (kindTab = "all")}
    >
      {t("blueprints.all")}
    </button>
    <button
      type="button"
      class="tab"
      class:active={kindTab === "file"}
      role="tab"
      aria-selected={kindTab === "file"}
      onclick={() => (kindTab = "file")}
    >
      {t("blueprints.files")}
    </button>
    <button
      type="button"
      class="tab"
      class:active={kindTab === "folder"}
      role="tab"
      aria-selected={kindTab === "folder"}
      onclick={() => (kindTab = "folder")}
    >
      {t("blueprints.folders")}
    </button>
  </div>

  {#if error}
    <p class="banner error" role="alert">{error}</p>
  {/if}

  {#if watchActiveMessage}
    <p class="banner info">{watchActiveMessage}</p>
  {/if}

  {#if applyResult}
    <p class="banner success">
      {t("blueprints.resultSummary", { processed: applyResult.processed, skipped: applyResult.skipped, failed: applyResult.failed })}
    </p>
  {/if}

  {#if loading}
    <p class="muted">{t("common.loading")}</p>
  {:else if filteredBlueprints.length === 0}
    <div class="empty panel">
      <FolderSimple size={28} weight="duotone" aria-hidden="true" />
      <p>{kindTab === "all" ? t("blueprints.noBlueprints") : kindTab === "file" ? t("blueprints.noFiles") : t("blueprints.noFolders")}</p>
      <button
        type="button"
        class="btn primary"
        onclick={() => openCreateModal(kindTab === "folder" ? "folder" : "file")}
      >
        <Plus size={14} weight="bold" />
        {t("blueprints.createFirst")}
      </button>
    </div>
  {:else}
    <ul class="bp-list">
      {#each filteredBlueprints as bp}
        <li class="bp-card">
          <div class="bp-info">
            <button type="button" class="bp-name" onclick={() => openEditModal(bp.id)}>
              {bp.name}
            </button>
            <span class="badge">{blueprintKindLabel(bp.kind)}</span>
            <span class="badge">{blueprintOperationLabel(bp.operation)}</span>
            {#if !bp.enabled}
              <span class="badge muted">{t("common.disabled")}</span>
            {/if}
            {#if bp.watchEnabled}
              <span class="badge muted">{t("blueprints.watch")}</span>
            {/if}
            <p class="paths muted">{bp.rootPath || "—"}</p>
          </div>
          <div class="actions">
            <button
              class="btn ghost"
              title={t("common.edit")}
              aria-label={t("blueprints.edit")}
              onclick={() => openEditModal(bp.id)}
            >
              <PencilSimple size={14} />
            </button>
            <button
              class="btn primary"
              disabled={busyId === bp.id || !bp.enabled}
              onclick={() => handleApply(bp.id)}
            >
              <Play size={14} weight="fill" />
              {t("blueprints.apply")}
            </button>
            <button class="btn ghost" disabled={busyId === bp.id} onclick={() => handleSimulate(bp)}>
              <Eye size={14} />
              {t("blueprints.simulate")}
            </button>
            <button class="btn ghost danger" onclick={() => handleDelete(bp.id)}>
              <Trash size={14} />
              {t("common.delete")}
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<BlueprintFormModal
  open={modalOpen}
  mode={modalMode}
  blueprintId={modalId}
  defaultKind={createKind}
  onclose={closeModal}
  onsaved={handleSaved}
  onsimulated={(report, name) => {
    simulationReport = report;
    simulationName = name;
    simulationModalOpen = true;
  }}
/>

<BlueprintSimulationModal
  open={simulationModalOpen}
  report={simulationReport}
  blueprintName={simulationName}
  onclose={() => {
    simulationModalOpen = false;
  }}
/>

<BlueprintRunVariablesModal
  open={runModalOpen}
  blueprint={runBlueprint}
  action={runAction}
  loading={runLoading}
  onclose={closeRunModal}
  onrun={(variables) => {
    if (!runBlueprint) return;
    void executeFolderRun(runBlueprint, runAction, variables);
  }}
/>

<style>
  .tabs {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .tab {
    padding: 0.35rem 0.75rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .tab.active {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .banner {
    margin: 0 0 var(--space-4);
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

  .banner.info {
    border-color: var(--accent);
    color: var(--accent);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-5);
    color: var(--accent);
  }

  .empty p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .bp-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .bp-card {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: center;
    padding: var(--space-4);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
  }

  .bp-name {
    padding: 0;
    border: none;
    background: none;
    font-weight: 600;
    font-size: var(--text-sm);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }

  .bp-name:hover {
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

  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
