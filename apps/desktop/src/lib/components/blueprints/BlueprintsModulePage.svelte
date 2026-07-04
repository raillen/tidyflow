<script lang="ts">
  import BlueprintFormModal from "$lib/components/blueprints/BlueprintFormModal.svelte";
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
  import { blueprintOperationLabel } from "$lib/contracts/blueprint";
  import type {
    Blueprint,
    BlueprintKind,
    BlueprintSimulationReport,
    BlueprintSummary,
  } from "$lib/contracts/blueprint";
  import { onMount } from "svelte";
  import { Eye, FolderSimple, PencilSimple, Play, Plus, Trash } from "phosphor-svelte";

  type Props = {
    kind: BlueprintKind;
    title: string;
    description: string;
    createLabel: string;
    emptyMessage: string;
    emptyActionLabel: string;
  };

  let {
    kind,
    title,
    description,
    createLabel,
    emptyMessage,
    emptyActionLabel,
  }: Props = $props();

  let blueprints = $state<BlueprintSummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busyId = $state<string | null>(null);
  let applyResult = $state<{ processed: number; failed: number; skipped: number } | null>(null);

  let modalOpen = $state(false);
  let modalMode = $state<"create" | "edit">("create");
  let modalId = $state<string | null>(null);

  let simulationModalOpen = $state(false);
  let simulationReport = $state<BlueprintSimulationReport | null>(null);
  let simulationName = $state("");
  let runModalOpen = $state(false);
  let runAction = $state<"apply" | "simulate">("apply");
  let runBlueprint = $state<Blueprint | null>(null);
  let runLoading = $state(false);
  let watchActiveMessage = $state<string | null>(null);

  const moduleItemName = $derived(kind === "file" ? "regra de arquivos" : "estrutura de pastas");
  const filteredBlueprints = $derived(blueprints.filter((blueprint) => blueprint.kind === kind));

  async function refresh() {
    loading = true;
    error = null;
    try {
      blueprints = await listBlueprints();
    } catch (e) {
      error = e instanceof Error ? e.message : `Falha ao carregar ${moduleItemName}`;
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  function openCreateModal() {
    modalMode = "create";
    modalId = null;
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
    if (kind === "folder") {
      const blueprint = await getBlueprint(id);
      if (blueprint.watch?.enabled) {
        watchActiveMessage = "Monitoramento ativo — o watch já está em execução.";
        applyResult = null;
        error = null;
        return;
      }
      if (!blueprint.folderPlan?.variables.length) {
        await executeFolderRun(blueprint, "apply", {});
        return;
      }
      runBlueprint = blueprint;
      runAction = "apply";
      runModalOpen = true;
      return;
    }
    busyId = id;
    error = null;
    applyResult = null;
    try {
      applyResult = await applyBlueprint(id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : `Falha ao aplicar ${moduleItemName}`;
    } finally {
      busyId = null;
    }
  }

  async function handleSimulate(blueprint: BlueprintSummary) {
    if (kind === "folder") {
      await prepareFolderRun(blueprint.id, "simulate");
      return;
    }
    busyId = blueprint.id;
    error = null;
    simulationReport = null;
    try {
      simulationReport = await simulateBlueprint(blueprint.id);
      simulationName = blueprint.name;
      simulationModalOpen = true;
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha na simulação";
    } finally {
      busyId = null;
    }
  }

  async function handleDelete(id: string) {
    if (!confirm(`Excluir esta ${moduleItemName}?`)) return;
    try {
      await deleteBlueprint(id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao excluir";
    }
  }

  async function prepareFolderRun(id: string, action: "apply" | "simulate") {
    busyId = id;
    error = null;
    applyResult = null;
    try {
      const blueprint = await getBlueprint(id);
      if (!blueprint.folderPlan?.variables.length) {
        await executeFolderRun(blueprint, action, {});
        return;
      }
      runBlueprint = blueprint;
      runAction = action;
      runModalOpen = true;
    } catch (e) {
      error = e instanceof Error ? e.message : `Falha ao preparar ${moduleItemName}`;
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
      error = e instanceof Error ? e.message : `Falha ao executar ${moduleItemName}`;
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
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">{title}</h1>
      <p class="page-desc">{description}</p>
    </div>
    <button type="button" class="btn primary" onclick={openCreateModal}>
      <Plus size={15} weight="bold" />
      {createLabel}
    </button>
  </header>

  {#if error}
    <p class="banner error" role="alert">{error}</p>
  {/if}

  {#if watchActiveMessage}
    <p class="banner info">{watchActiveMessage}</p>
  {/if}

  {#if applyResult}
    <p class="banner success">
      Aplicação concluída: {applyResult.processed} ok{#if applyResult.skipped}, {applyResult.skipped} ignorado{applyResult.skipped === 1 ? "" : "s"}{/if}{#if applyResult.failed}, {applyResult.failed} falha{applyResult.failed === 1 ? "" : "s"}{/if}
    </p>
  {/if}

  {#if loading}
    <p class="muted">Carregando...</p>
  {:else if filteredBlueprints.length === 0}
    <div class="empty panel">
      <FolderSimple size={28} weight="duotone" aria-hidden="true" />
      <p>{emptyMessage}</p>
      <button type="button" class="btn primary" onclick={openCreateModal}>
        <Plus size={14} weight="bold" />
        {emptyActionLabel}
      </button>
    </div>
  {:else}
    <ul class="bp-list">
      {#each filteredBlueprints as blueprint}
        <li class="bp-card">
          <div class="bp-info">
            <button type="button" class="bp-name" onclick={() => openEditModal(blueprint.id)}>
              {blueprint.name}
            </button>
            <span class="badge">{kind === "folder" ? "Receita" : blueprintOperationLabel(blueprint.operation)}</span>
            {#if !blueprint.enabled}
              <span class="badge muted">Desativado</span>
            {/if}
            {#if blueprint.watchEnabled}
              <span class="badge muted">Monitorado</span>
            {/if}
            <p class="paths muted">{blueprint.rootPath || "-"}</p>
          </div>
          <div class="actions">
            <button
              class="btn ghost"
              title="Editar"
              aria-label={`Editar ${moduleItemName}`}
              onclick={() => openEditModal(blueprint.id)}
            >
              <PencilSimple size={14} />
            </button>
            <button
              class="btn primary"
              disabled={busyId === blueprint.id || !blueprint.enabled}
              onclick={() => handleApply(blueprint.id)}
            >
              <Play size={14} weight="fill" />
              Aplicar
            </button>
            <button class="btn ghost" disabled={busyId === blueprint.id} onclick={() => handleSimulate(blueprint)}>
              <Eye size={14} />
              Simular
            </button>
            <button class="btn ghost danger" onclick={() => handleDelete(blueprint.id)}>
              <Trash size={14} />
              Excluir
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
  defaultKind={kind}
  fixedKind={kind}
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

  @media (max-width: 760px) {
    .bp-card {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
