<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { t } from "$lib/i18n";

  onMount(() => {
    const id = $page.params.id;
    void goto(id ? `/flows?edit=${id}` : "/flows", { replaceState: true });
  });
</script>

<p class="muted" aria-live="polite">{t("flows.openingEditor")}</p>

<style>
  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
