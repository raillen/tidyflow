<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { t } from "$lib/i18n";

  onMount(() => {
    void goto("/flows?new=1", { replaceState: true });
  });
</script>

<p class="muted" aria-live="polite">{t("flows.openingEditor")}</p>

<style>
  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
