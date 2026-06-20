<script lang="ts">
  import JobEditor from "$lib/components/jobs/JobEditor.svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import { countActiveFilters, createEmptyJob, type Job, type SimulationReport } from "$lib/contracts/job";
  import { createJob, getJob, simulateJob, simulateJobDraft, updateJob } from "$lib/core/ipc/client";
  import { Eye, FloppyDisk } from "phosphor-svelte";

  type Props = {
    open: boolean;
    mode: "create" | "edit";
    jobId?: string | null;
    onclose: () => void;
    onsaved?: () => void;
    onsimulated?: (report: SimulationReport, jobName: string) => void;
  };

  let { open, mode, jobId = null, onclose, onsaved, onsimulated }: Props = $props();

  let job = $state<Job>(createEmptyJob());
  let loading = $state(false);
  let saving = $state(false);
  let simulating = $state(false);
  let error = $state<string | null>(null);
  let loaded = $state(false);

  const title = $derived(mode === "create" ? "Novo fluxo" : job.name || "Editar fluxo");
  const description = $derived(
    mode === "create"
      ? "Configure origem, destino e regras de processamento."
      : "Ajuste caminhos, modo e filtros do job.",
  );

  const filterCount = $derived(countActiveFilters(job.filters));
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
      job = createEmptyJob();
      loading = false;
      loaded = true;
      return;
    }

    if (!jobId) {
      error = "ID do fluxo inválido";
      loading = false;
      return;
    }

    loading = true;
    let cancelled = false;

    void getJob(jobId)
      .then((result) => {
        if (cancelled) return;
        job = result;
        loaded = true;
      })
      .catch((e) => {
        if (cancelled) return;
        error = e instanceof Error ? e.message : "Fluxo não encontrado";
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
        await createJob(job);
      } else {
        await updateJob(job);
      }
      onsaved?.();
      onclose();
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha ao salvar fluxo";
    } finally {
      saving = false;
    }
  }

  async function handleSimulate() {
    simulating = true;
    error = null;
    try {
      const report =
        mode === "edit" && job.id
          ? await simulateJob(job.id)
          : await simulateJobDraft(job);
      onsimulated?.(report, job.name.trim() || "Fluxo");
    } catch (e) {
      error = e instanceof Error ? e.message : "Falha na simulação";
    } finally {
      simulating = false;
    }
  }
</script>

<Modal {open} {title} {description} size="large" persistKey="modal.jobForm" {onclose}>
  {#if loading}
    <p class="state-msg muted">Carregando fluxo…</p>
  {:else if error && !loaded}
    <p class="state-msg error" role="alert">{error}</p>
  {:else}
    {#if error}
      <p class="state-msg error" role="alert">{error}</p>
    {/if}
    <JobEditor bind:job embedded onsave={handleSave} />
  {/if}

  {#snippet footer()}
    <span class="footer-summary muted">{filterSummary}</span>
    <div class="footer-actions">
      <button
        type="button"
        class="btn"
        disabled={simulating || !loaded || loading}
        onclick={handleSimulate}
      >
        <Eye size={14} />
        {simulating ? "Simulando…" : "Simular"}
      </button>
      <button
        type="submit"
        form="job-editor-form"
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
