<script lang="ts">
  import "../app.css";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import {
    executionCompletedSchema,
    executionProgressSchema,
    fetchSettings,
    listActiveExecutions,
  } from "$lib/core/ipc/client";
  import {
    handleCompleted,
    setActiveExecutions,
    upsertExecution,
  } from "$lib/core/stores/executions";
  import { applyAppearance } from "$lib/core/stores/theme";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(() => {
    void (async () => {
      try {
        const settings = await fetchSettings();
        applyAppearance(settings);
      } catch {
        applyAppearance({ theme: "system", accentColor: "#0064ff" });
      }

      try {
        const active = await listActiveExecutions();
        setActiveExecutions(active);
      } catch {
        /* IPC indisponível fora do Tauri */
      }
    })();

    let unlistenProgress = () => {};
    let unlistenCompleted = () => {};

    void listen("execution_progress", (event) => {
      const progress = executionProgressSchema.parse(event.payload);
      upsertExecution(progress);
    }).then((unlisten) => {
      unlistenProgress = unlisten;
    });

    void listen("execution_completed", (event) => {
      const completed = executionCompletedSchema.parse(event.payload);
      handleCompleted(completed);
    }).then((unlisten) => {
      unlistenCompleted = unlisten;
    });

    return () => {
      unlistenProgress();
      unlistenCompleted();
    };
  });
</script>

<AppShell>
  {@render children()}
</AppShell>
