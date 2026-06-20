<script lang="ts">
  import BlueprintFormModal from "$lib/components/blueprints/BlueprintFormModal.svelte";
  import BlueprintSimulationModal from "$lib/components/blueprints/BlueprintSimulationModal.svelte";
  import {
    applyBlueprint,
    deleteBlueprint,
    listBlueprints,
    simulateBlueprint,
  } from "$lib/core/ipc/client";
  import type {
    BlueprintKind,
    BlueprintSimulationReport,
    BlueprintSummary,
  } from "$lib/contracts/blueprint";
  import { blueprintKindLabel, blueprintOperationLabel } from "$lib/contracts/blueprint";
  import { onMount } from "svelte";
  import { Eye, FolderSimple, PencilSimple, Play, Plus, Trash } from "phosphor-svelte";

  let blueprints = $state<BlueprintSummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busyId = $state<string | null>(null);
  let applyResult = $state<{ processed: number; failed: number } | null>(null);
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
      error = e instanceof Error ? e.message : "Falha ao carregar blueprints";
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
    try {
      applyResult = await applyBlueprint(id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao aplicar blueprint";
    } finally {
      busyId = null;
    }
  }

  async function handleSimulate(bp: BlueprintSummary) {
    busyId = bp.id;
    error = null;
    simulationReport = null;
    try {
      simulationReport = await simulateBlueprint(bp.id);
      simulationName = bp.name;
      simulationModalOpen = true;
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha na simulação";
    } finally {
      busyId = null;
    }
  }

  async function handleDelete(id: string) {
    if (!confirm("Excluir este blueprint?")) return;
    try {
      await deleteBlueprint(id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao excluir";
    }
  }
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Blueprints</h1>
      <p class="page-desc">Organização e renomeação automática por template.</p>
    </div>
    <button type="button" class="btn primary" onclick={() => openCreateModal(kindTab === "folder" ? "folder" : "file")}>
      <Plus size={15} weight="bold" />
      Novo blueprint
    </button>
  </header>

  <div class="tabs" role="tablist" aria-label="Tipo de blueprint">
    <button
      type="button"
      class="tab"
      class:active={kindTab === "all"}
      role="tab"
      aria-selected={kindTab === "all"}
      onclick={() => (kindTab = "all")}
    >
      Todos
    </button>
    <button
      type="button"
      class="tab"
      class:active={kindTab === "file"}
      role="tab"
      aria-selected={kindTab === "file"}
      onclick={() => (kindTab = "file")}
    >
      Arquivos
    </button>
    <button
      type="button"
      class="tab"
      class:active={kindTab === "folder"}
      role="tab"
      aria-selected={kindTab === "folder"}
      onclick={() => (kindTab = "folder")}
    >
      Pastas
    </button>
  </div>

  {#if error}
    <p class="banner error" role="alert">{error}</p>
  {/if}

  {#if applyResult}
    <p class="banner success">
      Organização concluída — {applyResult.processed} ok, {applyResult.failed} falhas
    </p>
  {/if}

  {#if loading}
    <p class="muted">Carregando…</p>
  {:else if filteredBlueprints.length === 0}
    <div class="empty panel">
      <FolderSimple size={28} weight="duotone" aria-hidden="true" />
      <p>Nenhum blueprint {kindTab === "all" ? "" : kindTab === "file" ? "de arquivos" : "de pastas"}.</p>
      <button
        type="button"
        class="btn primary"
        onclick={() => openCreateModal(kindTab === "folder" ? "folder" : "file")}
      >
        <Plus size={14} weight="bold" />
        Criar blueprint
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
              <span class="badge muted">Desativado</span>
            {/if}
            {#if bp.watchEnabled}
              <span class="badge muted">Watch</span>
            {/if}
            <p class="paths muted">{bp.rootPath || "—"}</p>
          </div>
          <div class="actions">
            <button
              class="btn ghost"
              title="Editar"
              aria-label="Editar blueprint"
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
              Aplicar
            </button>
            <button class="btn ghost" disabled={busyId === bp.id} onclick={() => handleSimulate(bp)}>
              <Eye size={14} />
              Simular
            </button>
            <button class="btn ghost danger" onclick={() => handleDelete(bp.id)}>
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
