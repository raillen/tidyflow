<script lang="ts">
  import {
    countBlueprintFilters,
    createDefaultFolderPlan,
    parseTemplateDisplayString,
    pipelineToDisplayString,
    type Blueprint,
    type BlueprintKind,
    type BlueprintOperation,
  } from "$lib/contracts/blueprint";
  import BlueprintTemplatePanel from "$lib/components/blueprints/BlueprintTemplatePanel.svelte";
  import FolderStructureEditor from "$lib/components/blueprints/FolderStructureEditor.svelte";
  import {
    EXCLUDE_PRESETS,
    type FileFilter,
    type WatchConfig as JobWatchConfig,
    type WatchDetectionMode as JobWatchDetectionMode,
    type WatchEventKind as JobWatchEventKind,
  } from "$lib/contracts/job";
  import { previewBlueprintTemplate } from "$lib/core/ipc/client";
  import { currentLocale, t } from "$lib/i18n";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowsLeftRight,
    CalendarBlank,
    CaretDoubleLeft,
    CaretDoubleRight,
    Copy,
    Eye,
    FolderOpen,
    Funnel,
    IdentificationCard,
    TextAa,
  } from "phosphor-svelte";

  type ConflictStrategy = Blueprint["conflict"];
  type NavPanel = "geral" | "busca" | "template" | "automacao";

  type Props = {
    blueprint: Blueprint;
    embedded?: boolean;
    lockedKind?: BlueprintKind;
    onsave?: () => void;
  };

  let { blueprint = $bindable(), embedded = true, lockedKind, onsave }: Props = $props();

  let activePanel = $state<NavPanel>("geral");
  let navCollapsed = $state(false);
  let templateText = $state(pipelineToDisplayString(blueprint.routing.pathTemplate));
  let renameTemplateText = $state(
    blueprint.renameTemplate ? pipelineToDisplayString(blueprint.renameTemplate) : "",
  );
  let samplePath = $state("exemplo/Docs/relatorio.PDF");
  let previewLoading = $state(false);
  let previewResult = $state<Awaited<ReturnType<typeof previewBlueprintTemplate>> | null>(null);

  const filterCount = $derived(countBlueprintFilters(blueprint.search));
  const activeLabel = $derived(
    (lockedKind ?? blueprint.kind) === "folder"
      ? t("blueprints.activeStructure")
      : t("blueprints.activeRule"),
  );

  function navItems(): { id: NavPanel; label: string; icon: typeof IdentificationCard }[] {
    return [
      { id: "geral", label: t("blueprints.nav.general"), icon: IdentificationCard },
      { id: "busca", label: t("blueprints.nav.search"), icon: Funnel },
      { id: "template", label: t("blueprints.nav.template"), icon: TextAa },
      { id: "automacao", label: t("jobs.nav.automation"), icon: Eye },
    ];
  }

  function watchEvents(): { id: JobWatchEventKind; label: string }[] {
    return [
      { id: "create", label: t("jobs.event.create") },
      { id: "modify", label: t("jobs.event.modify") },
      { id: "remove", label: t("jobs.event.remove") },
      { id: "rename", label: t("jobs.event.rename") },
    ];
  }

  function detectionOptions(): {
    value: JobWatchDetectionMode["kind"];
    label: string;
    hint: string;
  }[] {
    return [
      {
        value: "realtime",
        label: t("jobs.detection.realtime"),
        hint: t("jobs.detection.realtimeHint"),
      },
      {
        value: "polling",
        label: t("jobs.detection.polling"),
        hint: t("jobs.detection.pollingHint"),
      },
      {
        value: "hybrid",
        label: t("jobs.detection.hybrid"),
        hint: t("jobs.detection.hybridHint"),
      },
    ];
  }

  let extensionsText = $state(blueprint.search.includeExtensions.join(", "));
  let excludePatternsText = $state(blueprint.search.excludePatterns.join("\n"));

  $effect(() => {
    if (!lockedKind || blueprint.kind === lockedKind) return;
    blueprint = {
      ...blueprint,
      kind: lockedKind,
      folderPlan: lockedKind === "folder" ? blueprint.folderPlan ?? createDefaultFolderPlan() : null,
    };
  });

  $effect(() => {
    extensionsText = blueprint.search.includeExtensions.join(", ");
    excludePatternsText = blueprint.search.excludePatterns.join("\n");
    templateText = pipelineToDisplayString(blueprint.routing.pathTemplate);
    renameTemplateText = blueprint.renameTemplate
      ? pipelineToDisplayString(blueprint.renameTemplate)
      : "";
  });

  $effect(() => {
    if (activePanel !== "template") return;
    const pipeline = parseTemplateDisplayString(templateText);
    const currentSamplePath = samplePath;
    const renameTemplate = blueprint.renameTemplate;
    const timer = setTimeout(async () => {
      previewLoading = true;
      try {
        previewResult = await previewBlueprintTemplate(
          pipeline,
          currentSamplePath,
          renameTemplate,
        );
      } catch {
        previewResult = null;
      } finally {
        previewLoading = false;
      }
    }, 350);
    return () => clearTimeout(timer);
  });

  function patchSearch(partial: Partial<FileFilter>) {
    blueprint = {
      ...blueprint,
      search: { ...blueprint.search, ...partial },
      recursive: partial.recursive ?? blueprint.recursive,
    };
  }

  function patchRouting(partial: Partial<Blueprint["routing"]>) {
    blueprint = { ...blueprint, routing: { ...blueprint.routing, ...partial } };
  }

  function patchCounter(partial: Partial<Blueprint["counter"]>) {
    blueprint = { ...blueprint, counter: { ...blueprint.counter, ...partial } };
  }

  async function pickRootFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      blueprint = { ...blueprint, rootPath: selected };
    }
  }

  function setKind(kind: BlueprintKind) {
    blueprint = {
      ...blueprint,
      kind,
      folderPlan: kind === "folder" ? blueprint.folderPlan ?? createDefaultFolderPlan() : blueprint.folderPlan,
    };
  }

  function setOperation(operation: BlueprintOperation) {
    blueprint = { ...blueprint, operation };
  }

  function setConflict(conflict: ConflictStrategy) {
    blueprint = { ...blueprint, conflict };
  }

  function handleTemplateInput(value: string) {
    templateText = value;
    patchRouting({ pathTemplate: parseTemplateDisplayString(value) });
  }

  function handleRenameTemplateInput(value: string) {
    renameTemplateText = value;
    blueprint = {
      ...blueprint,
      renameTemplate: value.trim() ? parseTemplateDisplayString(value) : null,
    };
  }

  const renamePlaceholder = "{stem}_{counter}{ext}";

  function handleExtensionsInput(value: string) {
    extensionsText = value;
    patchSearch({
      includeExtensions: value
        .split(",")
        .map((ext) => ext.trim())
        .filter(Boolean),
    });
  }

  function handleExcludePatternsInput(value: string) {
    excludePatternsText = value;
    patchSearch({
      excludePatterns: value
        .split("\n")
        .map((line) => line.trim())
        .filter(Boolean),
    });
  }

  function togglePreset(id: string) {
    const ids = blueprint.search.excludePresetIds;
    patchSearch({
      excludePresetIds: ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id],
    });
  }

  function defaultWatch(): JobWatchConfig {
    return {
      enabled: true,
      settleSeconds: 2,
      detection: { kind: "realtime" },
      events: ["create"],
      autoCorrect: false,
      namingPattern: {},
    };
  }

  function patchWatch(partial: Partial<JobWatchConfig>) {
    const base = blueprint.watch ?? defaultWatch();
    blueprint = { ...blueprint, watch: { ...base, ...partial } };
  }

  function setWatchEnabled(enabled: boolean) {
    if (enabled) {
      blueprint = {
        ...blueprint,
        watch: blueprint.watch ? { ...blueprint.watch, enabled: true } : defaultWatch(),
      };
    } else if (blueprint.watch) {
      blueprint = { ...blueprint, watch: { ...blueprint.watch, enabled: false } };
    } else {
      blueprint = { ...blueprint, watch: null };
    }
  }

  function setDetectionKind(kind: JobWatchDetectionMode["kind"]) {
    switch (kind) {
      case "realtime":
        patchWatch({ detection: { kind: "realtime" } });
        break;
      case "polling":
        patchWatch({ detection: { kind: "polling", intervalSecs: 30 } });
        break;
      case "hybrid":
        patchWatch({ detection: { kind: "hybrid", pollIntervalSecs: 30 } });
        break;
    }
  }

  function toggleWatchEvent(event: JobWatchEventKind) {
    const watch = blueprint.watch ?? defaultWatch();
    const events = watch.events.includes(event)
      ? watch.events.filter((item) => item !== event)
      : [...watch.events, event];
    if (events.length === 0) return;
    patchWatch({ events });
  }
</script>

{#if (lockedKind ?? blueprint.kind) === "folder"}
  <FolderStructureEditor bind:blueprint {embedded} {onsave} />
{:else}
<form
  id="blueprint-editor-form"
  class="editor"
  class:embedded
  onsubmit={(e) => {
    e.preventDefault();
    blueprint = {
      ...blueprint,
      search: { ...blueprint.search, recursive: blueprint.recursive },
    };
    onsave?.();
  }}
>
  <nav class="editor-nav" class:collapsed={navCollapsed} aria-label={t("blueprints.nav.sectionsAria")}>
    <button
      type="button"
      class="nav-toggle btn ghost"
      aria-label={navCollapsed ? t("jobs.nav.expand") : t("jobs.nav.collapse")}
      onclick={() => (navCollapsed = !navCollapsed)}
    >
      {#if navCollapsed}
        <CaretDoubleRight size={16} />
      {:else}
        <CaretDoubleLeft size={16} />
      {/if}
    </button>

    {#each navItems() as item}
      {@const Icon = item.icon}
      <button
        type="button"
        class="nav-item"
        class:active={activePanel === item.id}
        title={item.label}
        onclick={() => (activePanel = item.id)}
      >
        <Icon size={18} weight={activePanel === item.id ? "fill" : "regular"} />
        {#if !navCollapsed}
          <span class="nav-label">{item.label}</span>
          {#if item.id === "busca" && filterCount > 0}
            <span class="badge">{filterCount}</span>
          {/if}
        {:else if item.id === "busca" && filterCount > 0}
          <span class="nav-dot" aria-label={t("jobs.nav.activeFilters", { count: filterCount })}></span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="editor-content" class:template-mode={activePanel === "template"}>
    {#if activePanel === "geral"}
      <div class="panel-header">
        <h3>{t("blueprints.nav.general")}</h3>
        <p>{t("blueprints.generalDesc")}</p>
      </div>

      <details class="collapsible" open>
        <summary>{t("jobs.general.identification")}</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">{t("jobs.general.name")}</span>
            <input type="text" class="field-input" bind:value={blueprint.name} maxlength="120" required />
          </label>

          <label class="toggle">
            <input type="checkbox" bind:checked={blueprint.enabled} />
            <span>{activeLabel}</span>
          </label>

          {#if !lockedKind}
            <div class="subsection">
              <span class="field-label">{t("blueprints.typeLabel")}</span>
              <div class="option-grid">
                <button
                  type="button"
                  class="option-btn"
                  class:selected={blueprint.kind === "file"}
                  onclick={() => setKind("file")}
                >
                  {t("blueprints.files")}
                </button>
                <button
                  type="button"
                  class="option-btn"
                  class:selected={blueprint.kind === "folder"}
                  onclick={() => setKind("folder")}
                >
                  {t("blueprints.folders")}
                </button>
              </div>
            </div>
          {/if}
        </div>
      </details>

      <details class="collapsible" open>
        <summary>{t("blueprints.rootFolder")}</summary>
        <div class="collapsible-body">
          <div class="path-row">
            <label class="field">
              <span class="field-label">{t("blueprints.searchRoot")}</span>
              <input
                type="text"
                class="field-input"
                bind:value={blueprint.rootPath}
                placeholder="C:\pasta\raiz"
                required
              />
            </label>
            <button type="button" class="btn" onclick={pickRootFolder}>
              <FolderOpen size={14} />
              {t("jobs.general.choose")}
            </button>
          </div>

          <label class="toggle">
            <input
              type="checkbox"
              checked={blueprint.recursive}
              onchange={(e) => {
                const recursive = e.currentTarget.checked;
                blueprint = { ...blueprint, recursive, search: { ...blueprint.search, recursive } };
              }}
            />
            <span>{t("blueprints.recursive")}</span>
          </label>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>{t("blueprints.operation")}</summary>
        <div class="collapsible-body">
          <div class="option-grid">
            <button
              type="button"
              class="option-btn"
              class:selected={blueprint.operation === "copy"}
              onclick={() => setOperation("copy")}
            >
              <Copy size={16} />
              {t("jobs.general.copy")}
            </button>
            <button
              type="button"
              class="option-btn"
              class:selected={blueprint.operation === "move"}
              onclick={() => setOperation("move")}
            >
              <ArrowsLeftRight size={16} />
              {t("jobs.general.move")}
            </button>
          </div>

          <label class="field">
            <span class="field-label">{t("jobs.general.conflict")}</span>
            <select
              class="field-input"
              value={blueprint.conflict}
              onchange={(e) => setConflict(e.currentTarget.value as ConflictStrategy)}
            >
              <option value="skip">{t("jobs.general.conflictSkip")}</option>
              <option value="overwrite">{t("jobs.general.conflictOverwrite")}</option>
              <option value="rename">{t("jobs.general.conflictRename")}</option>
            </select>
          </label>

          <label class="field">
            <span class="field-label">{t("blueprints.renameTemplate")}</span>
            <input
              type="text"
              class="field-input mono"
              value={renameTemplateText}
              oninput={(e) => handleRenameTemplateInput(e.currentTarget.value)}
              placeholder={renamePlaceholder}
            />
            <p class="field-hint">
              {t("blueprints.renameHint")}
            </p>
          </label>
        </div>
      </details>
    {:else if activePanel === "busca"}
      <div class="panel-header">
        <h3>{t("blueprints.nav.search")}</h3>
        <p>{t("blueprints.searchDesc")}</p>
      </div>

      <details class="collapsible" open>
        <summary>{t("jobs.filters.extensions")}</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">{t("jobs.filters.includeExtensions")}</span>
            <input
              type="text"
              class="field-input"
              value={extensionsText}
              oninput={(e) => handleExtensionsInput(e.currentTarget.value)}
              placeholder=".pdf, .docx"
            />
          </label>

          <fieldset class="preset-group">
            <legend class="field-label">{t("jobs.filters.excludePresets")}</legend>
            <div class="checkbox-grid">
              {#each EXCLUDE_PRESETS as preset}
                <label class="toggle">
                  <input
                    type="checkbox"
                    checked={blueprint.search.excludePresetIds.includes(preset.id)}
                    onchange={() => togglePreset(preset.id)}
                  />
                  <span>{preset.label}</span>
                </label>
              {/each}
            </div>
          </fieldset>

          <label class="field">
            <span class="field-label">{t("jobs.filters.excludePatterns")}</span>
            <textarea
              class="field-input textarea"
              rows="4"
              value={excludePatternsText}
              oninput={(e) => handleExcludePatternsInput(e.currentTarget.value)}
            ></textarea>
          </label>
        </div>
      </details>

      <details class="collapsible">
        <summary>{t("jobs.filters.regexSize")}</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">{t("jobs.filters.nameRegex")}</span>
            <input
              type="text"
              class="field-input"
              value={blueprint.search.nameRegex ?? ""}
              oninput={(e) => patchSearch({ nameRegex: e.currentTarget.value || null })}
            />
          </label>

          <label class="field">
            <span class="field-label">{t("jobs.filters.pathRegex")}</span>
            <input
              type="text"
              class="field-input"
              value={blueprint.search.pathRegex ?? ""}
              oninput={(e) => patchSearch({ pathRegex: e.currentTarget.value || null })}
            />
          </label>

          <div class="field-row">
            <label class="field">
              <span class="field-label">{t("jobs.filters.minSize")}</span>
              <input
                class="field-input"
                type="number"
                min="0"
                value={blueprint.search.minSizeBytes ?? ""}
                oninput={(e) => {
                  const v = e.currentTarget.value;
                  patchSearch({ minSizeBytes: v === "" ? null : Number(v) });
                }}
              />
            </label>
            <label class="field">
              <span class="field-label">{t("jobs.filters.maxSize")}</span>
              <input
                class="field-input"
                type="number"
                min="0"
                value={blueprint.search.maxSizeBytes ?? ""}
                oninput={(e) => {
                  const v = e.currentTarget.value;
                  patchSearch({ maxSizeBytes: v === "" ? null : Number(v) });
                }}
              />
            </label>
          </div>
        </div>
      </details>
    {:else if activePanel === "template"}
      <div class="panel-header template-header">
        <h3>{t("blueprints.templateTitle")}</h3>
        <p>{t("blueprints.templateDesc")}</p>
      </div>

      <BlueprintTemplatePanel
        {blueprint}
        {templateText}
        {samplePath}
        {previewLoading}
        {previewResult}
        onTemplateInput={handleTemplateInput}
        onSamplePathChange={(value) => (samplePath = value)}
        onPatchRouting={patchRouting}
        onPatchCounter={patchCounter}
      />
    {:else if activePanel === "automacao"}
      <div class="panel-header">
        <h3>{t("jobs.nav.automation")}</h3>
        <p>{t("blueprints.automationDesc")}</p>
      </div>

      <details class="collapsible" open>
        <summary>{t("blueprints.watchRoot")}</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={blueprint.watch?.enabled ?? false}
              onchange={(e) => setWatchEnabled(e.currentTarget.checked)}
            />
            <span>{t("jobs.watch.enabled")}</span>
          </label>

          {#if blueprint.watch?.enabled}
            <p class="field-hint">
              {t("blueprints.watchHintPath", { path: blueprint.rootPath || "—" })}
            </p>

            <label class="field">
              <span class="field-label">{t("jobs.watch.settle")}</span>
              <input
                class="field-input"
                type="number"
                min="1"
                max="60"
                value={blueprint.watch.settleSeconds}
                oninput={(e) =>
                  patchWatch({
                    settleSeconds: Math.min(60, Math.max(1, Number(e.currentTarget.value) || 2)),
                  })}
              />
            </label>

            <label class="field">
              <span class="field-label">{t("jobs.watch.detection")}</span>
              <select
                class="field-input"
                value={blueprint.watch.detection.kind}
                onchange={(e) =>
                  setDetectionKind(e.currentTarget.value as JobWatchDetectionMode["kind"])}
              >
                {#each detectionOptions() as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </label>

            {#if blueprint.watch.detection.kind === "polling"}
              <label class="field">
                <span class="field-label">{t("blueprints.pollIntervalSecs")}</span>
                <input
                  class="field-input"
                  type="number"
                  min="5"
                  max="3600"
                  value={blueprint.watch.detection.intervalSecs}
                  oninput={(e) =>
                    patchWatch({
                      detection: {
                        kind: "polling",
                        intervalSecs: Math.max(5, Number(e.currentTarget.value) || 30),
                      },
                    })}
                />
              </label>
            {:else if blueprint.watch.detection.kind === "hybrid"}
              <label class="field">
                <span class="field-label">{t("jobs.watch.backupPoll")}</span>
                <input
                  class="field-input"
                  type="number"
                  min="5"
                  max="3600"
                  value={blueprint.watch.detection.pollIntervalSecs}
                  oninput={(e) =>
                    patchWatch({
                      detection: {
                        kind: "hybrid",
                        pollIntervalSecs: Math.max(5, Number(e.currentTarget.value) || 30),
                      },
                    })}
                />
              </label>
            {/if}

            <fieldset class="preset-group">
              <legend class="field-label">{t("blueprints.events")}</legend>
              <div class="checkbox-grid">
                {#each watchEvents() as event}
                  <label class="toggle">
                    <input
                      type="checkbox"
                      checked={blueprint.watch.events.includes(event.id)}
                      onchange={() => toggleWatchEvent(event.id)}
                    />
                    <span>{event.label}</span>
                  </label>
                {/each}
              </div>
            </fieldset>
          {/if}
        </div>
      </details>

      {#if blueprint.lastRun}
        <details class="collapsible">
          <summary>{t("blueprints.lastRun")}</summary>
          <div class="collapsible-body">
            <p class="muted">
              <CalendarBlank size={14} />
              {new Intl.DateTimeFormat(currentLocale(), { dateStyle: "short", timeStyle: "medium" }).format(
                new Date(blueprint.lastRun),
              )}
            </p>
          </div>
        </details>
      {/if}
    {/if}
  </div>
</form>
{/if}

<style>
  .editor {
    display: flex;
    min-height: 0;
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--surface-muted);
  }

  .editor.embedded {
    border: none;
    border-radius: 0;
    background: transparent;
  }

  .editor-nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    width: 13.5rem;
    padding: var(--space-3);
    border-right: 1px solid var(--border);
    background: var(--surface-muted);
    transition: width 160ms ease;
  }

  .editor-nav.collapsed {
    width: 52px;
    padding-inline: var(--space-2);
  }

  .nav-toggle {
    align-self: flex-end;
    padding: var(--space-1);
    margin-bottom: var(--space-2);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    position: relative;
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .editor-content {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    padding: var(--space-4) var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .editor-content.template-mode {
    overflow: hidden;
  }

  .template-header {
    flex-shrink: 0;
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: 650;
  }

  .panel-header p {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .collapsible {
    flex-shrink: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .collapsible > summary {
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-sm);
    font-weight: 600;
    cursor: pointer;
    list-style: none;
  }

  .collapsible-body {
    display: grid;
    gap: var(--space-4);
    padding: 0 var(--space-4) var(--space-4);
    border-top: 1px solid var(--border);
    padding-top: var(--space-4);
  }

  .field-row,
  .option-grid,
  .path-row {
    display: grid;
    gap: var(--space-3);
  }

  .option-grid,
  .path-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .path-row {
    grid-template-columns: 1fr auto;
    align-items: end;
  }

  .option-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: 0.5rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-muted);
    cursor: pointer;
  }

  .option-btn.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .preset-group {
    border: none;
    margin: 0;
    padding: 0;
  }

  .checkbox-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: var(--space-2);
  }

  .textarea {
    resize: vertical;
    min-height: 4.5rem;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .field-hint {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

</style>
