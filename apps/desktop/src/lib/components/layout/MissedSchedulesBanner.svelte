<script lang="ts">
  import { clearMissedSchedules, listMissedSchedules, type MissedScheduleEntry } from "$lib/core/ipc/ui-state";
  import { Bell, X } from "phosphor-svelte";
  import { onMount } from "svelte";

  let entries = $state<MissedScheduleEntry[]>([]);
  let dismissed = $state(false);

  async function refresh() {
    try {
      entries = await listMissedSchedules();
      if (entries.length === 0) dismissed = false;
    } catch {
      entries = [];
    }
  }

  onMount(() => {
    void refresh();
  });

  async function handleDismiss() {
    try {
      await clearMissedSchedules();
    } catch {
      /* IPC indisponível fora do Tauri */
    }
    entries = [];
    dismissed = true;
  }

  const summary = $derived.by(() => {
    if (entries.length === 0) return "";
    if (entries.length === 1) {
      const entry = entries[0];
      return `O fluxo "${entry.jobName}" perdeu a execução agendada em ${formatWhen(entry.scheduledAt)}.`;
    }
    return `${entries.length} fluxos perderam execuções agendadas enquanto o app estava fechado.`;
  });

  function formatWhen(iso: string) {
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    return date.toLocaleString();
  }
</script>

{#if entries.length > 0 && !dismissed}
  <div class="missed-banner" role="status">
    <Bell size={16} weight="fill" aria-hidden="true" />
    <p>{summary}</p>
    <button type="button" class="btn ghost" onclick={handleDismiss} aria-label="Dispensar aviso">
      <X size={14} />
      Dispensar
    </button>
  </div>
{/if}

<style>
  .missed-banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
    background: color-mix(in srgb, var(--accent) 8%, var(--surface));
    color: var(--text-primary);
    font-size: var(--text-sm);
  }

  .missed-banner p {
    flex: 1;
    margin: 0;
  }
</style>
