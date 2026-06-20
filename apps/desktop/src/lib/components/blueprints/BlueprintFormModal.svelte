<script lang="ts">
  import BlueprintEditor from "$lib/components/blueprints/BlueprintEditor.svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import {
    countBlueprintFilters,
    createEmptyBlueprint,
    type Blueprint,
    type BlueprintKind,
    type BlueprintSimulationReport,
  } from "$lib/contracts/blueprint";
  import {
    createBlueprint,
    getBlueprint,
    simulateBlueprint,
    updateBlueprint,
  } from "$lib/core/ipc/client";
  import { Eye, FloppyDisk } from "phosphor-svelte";

  type Props = {
    open: boolean;
    mode: "create" | "edit";
    blueprintId?: string | null;
    defaultKind?: BlueprintKind;
    onclose: () => void;
    onsaved?: () => void;
    onsimulated?: (report: BlueprintSimulationReport, blueprintName: string) => void;
  };

  let {
    open,
    mode,
    blueprintId = null,
    defaultKind = "file",
    onclose,
    onsaved,
    onsimulated,
  }: Props = $props();

  let blueprint = $state<Blueprint>(createEmptyBlueprint(defaultKind));
  let loading = $state(false);
  let saving = $state(false);
  let simulating = $state(false);
  let error = $state<string | null>(null);
  let loaded = $state(false);

  const title = $derived(
    mode === "create" ? "Novo blueprint" : blueprint.name || "Editar blueprint",
  );
  const description = $derived(
    mode === "create"
      ? "Configure busca, template de destino e automação."
      : "Ajuste regras de organização e monitoramento.",
  );

  const filterCount = $derived(countBlueprintFilters(blueprint.search));
  const filterSummary = $derived(
    filterCount === 0
      ? "Nenhum filtro ativo"
      : `${filterCount} filtro${filterCount === 1 ? "" : "s"} ativo${filterCount === 1 ? "" : "s"}`,
  );

  $effect(() => {
    if (!open) {
      loaded = false;
      return;
    }

    error = null;
    saving = false;
    loaded = false;

    if (mode === "create") {
      blueprint = createEmptyBlueprint(defaultKind);
      loading = false;
      loaded = true;
      return;
    }

    if (!blueprintId) {
      error = "ID do blueprint inválido";
      loading = false;
      return;
    }

    loading = true;
    let cancelled = false;

    void getBlueprint(blueprintId)
      .then((result) => {
        if (cancelled) return;
        blueprint = result;
        loaded = true;
      })
      .catch((e) => {
        if (cancelled) return;
        error = e instanceof Error ? e.message : "Blueprint não encontrado";
      })
      .finally(() => {
        if (!cancelled) loading = false;
      });

    return () => {
      cancelled = true;
    };
  });

  async function handleSave() {
    saving = true;
    error = null;
    try {
      if (mode === "create") {
        await createBlueprint(blueprint);
      } else {
        await updateBlueprint(blueprint);
      }
      onsaved?.();
      onclose();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao salvar blueprint";
    } finally {
      saving = false;
    }
  }

  async function handleSimulate() {
    if (mode !== "edit" || !blueprint.id) {
      error = "Salve o blueprint antes de simular.";
      return;
    }
    simulating = true;
    error = null;
    try {
      const report = await simulateBlueprint(blueprint.id);
      onsimulated?.(report, blueprint.name.trim() || "Blueprint");
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha na simulação";
    } finally {
      simulating = false;
    }
  }
</script>

<Modal {open} {title} {description} size="large" persistKey="modal.blueprintForm" {onclose}>
  {#if loading}
    <p class="state-msg muted">Carregando blueprint…</p>
  {:else if error && !loaded}
    <p class="state-msg error" role="alert">{error}</p>
  {:else}
    {#if error}
      <p class="state-msg error" role="alert">{error}</p>
    {/if}
    <BlueprintEditor bind:blueprint embedded onsave={handleSave} />
  {/if}

  {#snippet footer()}
    <span class="footer-summary muted">{filterSummary}</span>
    <div class="footer-actions">
      <button
        type="button"
        class="btn"
        disabled={simulating || !loaded || loading || mode === "create"}
        onclick={handleSimulate}
      >
        <Eye size={14} />
        {simulating ? "Simulando…" : "Simular"}
      </button>
      <button
        type="submit"
        form="blueprint-editor-form"
        class="btn primary"
        disabled={saving || !loaded || loading}
      >
        <FloppyDisk size={14} weight="bold" />
        {saving ? "Salvando…" : "Salvar"}
      </button>
    </div>
  {/snippet}
</Modal>

<style>
  .state-msg {
    margin: var(--space-4) var(--space-5);
    font-size: var(--text-sm);
  }

  .state-msg.error {
    color: var(--danger);
  }

  :global(.modal-body > form.editor) {
    flex: 1;
    min-height: 0;
  }

  :global(.modal-footer) {
    justify-content: space-between !important;
  }

  .footer-summary {
    font-size: var(--text-sm);
  }

  .footer-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
</style>
