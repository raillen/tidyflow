<script lang="ts">
  import { onMount } from "svelte";
  import {
    CheckCircle,
    Desktop,
    Moon,
    Palette,
    Power,
    SlidersHorizontal,
    Sun,
    Translate,
  } from "phosphor-svelte";
  import {
    ACCENT_PRESETS,
    THEME_OPTIONS,
    type AppSettings,
    type ThemeMode,
  } from "$lib/contracts/settings";
  import { fetchSettings, saveSettings } from "$lib/core/ipc/client";
  import { applyAppearance } from "$lib/core/stores/theme";

  let settings = $state<AppSettings | null>(null);
  let message = $state<string | null>(null);
  let saving = $state(false);

  onMount(async () => {
    settings = await fetchSettings();
    applyAppearance(settings);
  });

  async function handleSave() {
    if (!settings) return;
    saving = true;
    message = null;
    try {
      settings = await saveSettings(settings);
      applyAppearance(settings);
      message = "Configurações salvas.";
    } catch (e) {
      message = e instanceof Error ? e.message : "Erro ao salvar";
    } finally {
      saving = false;
    }
  }

  function setTheme(theme: ThemeMode) {
    if (!settings) return;
    settings = { ...settings, theme };
    applyAppearance(settings);
  }

  function setAccent(accentColor: string) {
    if (!settings) return;
    settings = { ...settings, accentColor };
    applyAppearance(settings);
  }

  function themeIcon(theme: ThemeMode) {
    if (theme === "light") return Sun;
    if (theme === "dark") return Moon;
    return Desktop;
  }
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Configurações</h1>
      <p class="page-desc">Aparência, idioma e preferências globais do app.</p>
    </div>
  </header>

  {#if settings}
    <form
      class="settings-grid"
      onsubmit={(event) => {
        event.preventDefault();
        void handleSave();
      }}
    >
      <section class="panel section">
        <div class="section-head">
          <Palette size={16} weight="duotone" aria-hidden="true" />
          <div>
            <h2>Aparência</h2>
            <p>Cor de destaque e tema da interface.</p>
          </div>
        </div>

        <div class="subsection">
          <span class="field-label">Cor de destaque</span>
          <div class="accent-grid" role="radiogroup" aria-label="Cor de destaque">
            {#each ACCENT_PRESETS as preset}
              <button
                type="button"
                class="accent-swatch"
                class:selected={settings.accentColor.toLowerCase() === preset.value.toLowerCase()}
                style="--swatch: {preset.value}"
                aria-label={preset.label}
                aria-pressed={settings.accentColor.toLowerCase() === preset.value.toLowerCase()}
                onclick={() => setAccent(preset.value)}
              >
                <span class="dot"></span>
                <span class="label">{preset.label}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="subsection">
          <span class="field-label">Tema</span>
          <div class="theme-grid">
            {#each THEME_OPTIONS as option}
              {@const Icon = themeIcon(option.id)}
              <button
                type="button"
                class="theme-option"
                class:selected={settings.theme === option.id}
                aria-pressed={settings.theme === option.id}
                onclick={() => setTheme(option.id)}
              >
                <Icon size={18} weight={settings.theme === option.id ? "fill" : "regular"} />
                {option.label}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <section class="panel section">
        <div class="section-head">
          <Translate size={16} weight="duotone" aria-hidden="true" />
          <div>
            <h2>Idioma</h2>
            <p>Idioma da interface (Paraglide em fases futuras).</p>
          </div>
        </div>

        <label class="field">
          <span class="field-label">Idioma</span>
          <select class="field-input" bind:value={settings.language}>
            <option value="pt-BR">Português (BR)</option>
            <option value="en-US">English (US)</option>
          </select>
        </label>
      </section>

      <section class="panel section">
        <div class="section-head">
          <SlidersHorizontal size={16} weight="duotone" aria-hidden="true" />
          <div>
            <h2>Execução</h2>
            <p>Limites e retenção de dados.</p>
          </div>
        </div>

        <div class="field-row">
          <label class="field">
            <span class="field-label">Arquivos paralelos por job</span>
            <input
              class="field-input"
              type="number"
              min="1"
              max="16"
              bind:value={settings.maxParallelFiles}
            />
          </label>

          <label class="field">
            <span class="field-label">Retenção de logs (dias)</span>
            <input
              class="field-input"
              type="number"
              min="0"
              bind:value={settings.logRetentionDays}
            />
          </label>
        </div>
      </section>

      <section class="panel section">
        <div class="section-head">
          <Power size={16} weight="duotone" aria-hidden="true" />
          <div>
            <h2>Sistema</h2>
            <p>Comportamento ao iniciar o Windows.</p>
          </div>
        </div>

        <label class="toggle">
          <input type="checkbox" bind:checked={settings.autostart} />
          <span>Iniciar com o sistema</span>
        </label>
      </section>

      <footer class="form-footer">
        <button type="submit" class="btn primary" disabled={saving}>
          <CheckCircle size={16} weight="bold" />
          {saving ? "Salvando…" : "Salvar alterações"}
        </button>

        {#if message}
          <p class="feedback" aria-live="polite">{message}</p>
        {/if}
      </footer>
    </form>
  {:else}
    <p class="muted" aria-live="polite">Carregando configurações…</p>
  {/if}
</section>

<style>
  .settings-grid {
    display: grid;
    gap: var(--space-4);
    max-width: 42rem;
  }

  .section {
    display: grid;
    gap: var(--space-4);
  }

  .section-head {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    color: var(--accent);
  }

  .section-head h2 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-primary);
  }

  .section-head p {
    margin: 2px 0 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .subsection {
    display: grid;
    gap: var(--space-3);
  }

  .accent-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
  }

  .accent-swatch {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.45rem 0.55rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-muted);
    cursor: pointer;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    transition: border-color 120ms ease, background 120ms ease;
  }

  .accent-swatch:hover {
    border-color: color-mix(in srgb, var(--swatch) 40%, var(--border));
  }

  .accent-swatch.selected {
    border-color: var(--swatch);
    background: color-mix(in srgb, var(--swatch) 10%, var(--surface-muted));
    color: var(--text-primary);
    font-weight: 600;
  }

  .dot {
    width: 0.85rem;
    height: 0.85rem;
    border-radius: 999px;
    background: var(--swatch);
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
    flex-shrink: 0;
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
  }

  .theme-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: 0.55rem 0.4rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-muted);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease, color 120ms ease;
  }

  .theme-option.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .field-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .form-footer {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex-wrap: wrap;
  }

  .feedback {
    margin: 0;
    color: var(--success);
    font-size: var(--text-sm);
  }

  @media (max-width: 640px) {
    .accent-grid,
    .theme-grid,
    .field-row {
      grid-template-columns: 1fr;
    }
  }
</style>
