<script lang="ts">
  import { t } from "$lib/i18n";
  type Props = {
    error?: string | null;
    loading?: boolean;
    onsubmit: (pin: string) => void;
  };

  let { error = null, loading = false, onsubmit }: Props = $props();
  let pin = $state("");

  function submit() {
    if (loading || !pin.trim()) return;
    onsubmit(pin.trim());
    pin = "";
  }
</script>

<div class="lock-screen" role="dialog" aria-modal="true" aria-labelledby="pin-title">
  <div class="lock-card">
    <h1 id="pin-title">{t("lock.title")}</h1>
    <p class="muted">{t("lock.prompt")}</p>
    <label class="field">
      <span class="field-label">{t("lock.pin")}</span>
      <input
        class="field-input"
        type="password"
        inputmode="numeric"
        autocomplete="current-password"
        bind:value={pin}
        onkeydown={(event) => {
          if (event.key === "Enter") submit();
        }}
      />
    </label>
    {#if error}
      <p class="banner error" role="alert">{error}</p>
    {/if}
    <button type="button" class="btn primary" onclick={submit} disabled={loading || !pin.trim()}>
      {loading ? t("lock.checking") : t("lock.unlock")}
    </button>
  </div>
</div>

<style>
  .lock-screen {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: var(--surface);
    z-index: 1000;
  }

  .lock-card {
    width: min(24rem, 90vw);
    padding: var(--space-5);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-muted);
    display: grid;
    gap: var(--space-3);
  }

  .lock-card h1 {
    margin: 0;
    font-size: var(--text-lg);
  }

  .field {
    display: grid;
    gap: 0.35rem;
  }

  .field-label {
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .banner.error {
    border-color: var(--danger);
    color: var(--danger);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--danger);
    font-size: var(--text-sm);
  }
</style>
