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
  import { loadLocale, setLocale, t } from "$lib/i18n";
  import PinLockScreen from "$lib/components/security/PinLockScreen.svelte";
  import { verifySecurityPin } from "$lib/core/ipc/client";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import type { AppSettings } from "$lib/contracts/settings";

  let { children } = $props();
  let settings = $state<AppSettings | null>(null);
  let locked = $state(false);
  let pinError = $state<string | null>(null);
  let pinLoading = $state(false);

  onMount(() => {
    loadLocale();
    void (async () => {
      try {
        const loaded = await fetchSettings();
        settings = loaded;
        applyAppearance(loaded);
        if (loaded.language === "pt-BR" || loaded.language === "en-US") {
          setLocale(loaded.language as "pt-BR" | "en-US");
        }
        if (
          loaded.security.pinEnabled &&
          loaded.security.requirePinOnStartup &&
          loaded.security.accessPinHash
        ) {
          locked = true;
        }
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
    let unlistenSecurityLock = () => {};

    void listen("execution_progress", (event) => {
      const progress = executionProgressSchema.parse(event.payload);
      upsertExecution(progress);
    }).then((unlisten) => {
      unlistenProgress = unlisten;
    });

    void listen("execution_completed", async (event) => {
      const completed = executionCompletedSchema.parse(event.payload);
      handleCompleted(completed);
      await notifyDesktop(completed);
    }).then((unlisten) => {
      unlistenCompleted = unlisten;
    });

    void listen("security_lock", () => {
      locked = true;
      pinError = null;
    }).then((unlisten) => {
      unlistenSecurityLock = unlisten;
    });

    return () => {
      unlistenProgress();
      unlistenCompleted();
      unlistenSecurityLock();
    };
  });

  async function unlock(pin: string): Promise<void> {
    pinLoading = true;
    pinError = null;
    try {
      const ok = await verifySecurityPin(pin);
      if (ok) {
        locked = false;
      } else {
        pinError = "PIN incorreto.";
      }
    } catch (e) {
      pinError = e instanceof Error ? e.message : t("pin.verifyFailed");
    } finally {
      pinLoading = false;
    }
  }

  async function notifyDesktop(completed: {
    success: boolean;
    processed: number;
    failed: number;
    errorMessage: string | null;
  }): Promise<void> {
    const config = settings?.notifications;
    if (!config?.enabled || !config.desktopEnabled) return;
    if (completed.success && !config.notifyOnSuccess) return;
    if (!completed.success && !config.notifyOnFailure) return;

    try {
      const granted = await invoke("plugin:notification|is_permission_granted");
      if (!granted) {
        const permission = await invoke("plugin:notification|request_permission");
        if (permission !== "granted") return;
      }
      await invoke("plugin:notification|notify", {
        options: {
          title: "TidyFlow",
          body: t("notifications.jobBody", {
            status: completed.success
              ? t("notifications.completed")
              : t("notifications.failed"),
            processed: completed.processed,
            failed: completed.failed,
            error: completed.errorMessage ? ` — ${completed.errorMessage}` : "",
          }),
        },
      });
    } catch {
      // Notificações são best-effort; não devem quebrar a UI.
    }
  }
</script>

{#if locked}
  <PinLockScreen error={pinError} loading={pinLoading} onsubmit={unlock} />
{:else}
  <AppShell>
    {@render children()}
  </AppShell>
{/if}
