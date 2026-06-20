<script lang="ts">
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import MissedSchedulesBanner from "$lib/components/layout/MissedSchedulesBanner.svelte";

  let { children } = $props();
</script>

<div class="shell">
  <Sidebar />
  <main class="content">
    <MissedSchedulesBanner />
    {@render children()}
  </main>
</div>

<style>
  .shell {
    display: flex;
    min-height: 100vh;
  }

  .content {
    flex: 1;
    padding: var(--space-5) var(--space-6);
    min-width: 0;
  }
</style>
