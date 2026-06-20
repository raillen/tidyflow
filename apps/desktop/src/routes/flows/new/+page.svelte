<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    void goto("/flows?new=1", { replaceState: true });
  });
</script>

<p class="muted">Abrindo editor…</p>

<style>
  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
