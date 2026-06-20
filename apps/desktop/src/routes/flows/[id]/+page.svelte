<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  onMount(() => {
    const id = $page.params.id;
    if (id) {
      void goto(`/flows?edit=${id}`, { replaceState: true });
    } else {
      void goto("/flows", { replaceState: true });
    }
  });
</script>

<p class="muted">Abrindo editor…</p>

<style>
  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
