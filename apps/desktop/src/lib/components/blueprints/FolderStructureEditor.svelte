<script lang="ts">
  import {
    createDefaultFolderPlan,
    createFolderActionNode,
    createFolderVariableValues,
    folderActionOptions,
    folderActionLabel,
    parseTemplateDisplayString,
    pipelineToDisplayString,
    type Blueprint,
    type FolderAction,
    type FolderActionNode,
    type FolderPlanPreviewNode,
    type FolderPlanVariable,
  } from "$lib/contracts/blueprint";
  import {
    type WatchConfig as JobWatchConfig,
    type WatchDetectionMode as JobWatchDetectionMode,
    type WatchEventKind as JobWatchEventKind,
  } from "$lib/contracts/job";
  import { defaultWatch as buildDefaultWatch, detectionOptions, watchEvents } from "$lib/contracts/job-options";
  import { previewBlueprintPlan } from "$lib/core/ipc/client";
  import { currentLocale, t } from "$lib/i18n";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    CalendarBlank,
    CaretDoubleLeft,
    CaretDoubleRight,
    Eye,
    FolderOpen,
    FolderSimple,
    IdentificationCard,
    Plus,
    ShieldWarning,
    Sliders,
    Trash,
    TreeStructure,
  } from "phosphor-svelte";

  type NavPanel = "geral" | "variaveis" | "arvore" | "seguranca" | "automacao";

  type Props = {
    blueprint: Blueprint;
    embedded?: boolean;
    onsave?: () => void;
  };

  let { blueprint = $bindable(), embedded = true, onsave }: Props = $props();

  let activePanel = $state<NavPanel>("arvore");
  let navCollapsed = $state(false);
  let planPreviewLoading = $state(false);
  let renameTemplateText = $state("");
  let renameTemplateFocused = $state(false);

  $effect(() => {
    const pipeline = blueprint.watch?.namingPattern?.templateOverride ?? null;
    if (!renameTemplateFocused) {
      renameTemplateText = pipeline ? pipelineToDisplayString(pipeline) : "";
    }
  });
  let planPreviewResult = $state<Awaited<ReturnType<typeof previewBlueprintPlan>> | null>(null);

  function navItems(): { id: NavPanel; label: string; icon: typeof IdentificationCard }[] {
    return [
      { id: "geral", label: t("folders.nav.general"), icon: IdentificationCard },
      { id: "variaveis", label: t("folders.nav.variables"), icon: Sliders },
      { id: "arvore", label: t("folders.nav.tree"), icon: TreeStructure },
      { id: "seguranca", label: t("folders.nav.security"), icon: ShieldWarning },
      { id: "automacao", label: t("jobs.nav.automation"), icon: Eye },
    ];
  }

  $effect(() => {
    ensureFolderPlan();
  });

  const previewVariables = $derived(createFolderVariableValues(blueprint.folderPlan));

  $effect(() => {
    if (activePanel !== "arvore" && activePanel !== "seguranca" && activePanel !== "variaveis") return;
    const rootPath = blueprint.rootPath;
    const folderPlan = blueprint.folderPlan ?? createDefaultFolderPlan();
    const variables = previewVariables;
    const timer = setTimeout(async () => {
      planPreviewLoading = true;
      try {
        planPreviewResult = await previewBlueprintPlan(rootPath, folderPlan, variables);
      } catch {
        planPreviewResult = null;
      } finally {
        planPreviewLoading = false;
      }
    }, 250);
    return () => clearTimeout(timer);
  });

  async function pickRootFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      blueprint = { ...blueprint, rootPath: selected };
    }
  }

  function ensureFolderPlan() {
    if (!blueprint.folderPlan) {
      blueprint = { ...blueprint, kind: "folder", folderPlan: createDefaultFolderPlan() };
    }
  }

  function patchFolderPlan(partial: Partial<NonNullable<Blueprint["folderPlan"]>>) {
    const folderPlan = blueprint.folderPlan ?? createDefaultFolderPlan();
    blueprint = { ...blueprint, folderPlan: { ...folderPlan, ...partial } };
  }

  function patchWatch(partial: Partial<JobWatchConfig>) {
    const base = blueprint.watch ?? defaultWatch();
    blueprint = { ...blueprint, watch: { ...base, ...partial } };
  }

  /** Folder watches default to auto-correct, unlike file jobs. */
  function defaultWatch(): JobWatchConfig {
    return { ...buildDefaultWatch(), autoCorrect: true };
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
    if (kind === "realtime") patchWatch({ detection: { kind: "realtime" } });
    if (kind === "polling") patchWatch({ detection: { kind: "polling", intervalSecs: 30 } });
    if (kind === "hybrid") patchWatch({ detection: { kind: "hybrid", pollIntervalSecs: 30 } });
  }

  function toggleWatchEvent(event: JobWatchEventKind) {
    const watch = blueprint.watch ?? defaultWatch();
    const events = watch.events.includes(event)
      ? watch.events.filter((item) => item !== event)
      : [...watch.events, event];
    if (events.length === 0) return;
    patchWatch({ events });
  }

  function patchNamingPattern(partial: Partial<NonNullable<JobWatchConfig["namingPattern"]>>) {
    const watch = blueprint.watch ?? defaultWatch();
    blueprint = {
      ...blueprint,
      watch: { ...watch, namingPattern: { ...watch.namingPattern, ...partial } },
    };
  }

  function syncRenameTemplateText() {
    const pipeline = blueprint.watch?.namingPattern?.templateOverride ?? null;
    renameTemplateText = pipeline ? pipelineToDisplayString(pipeline) : "";
  }

  function handleRenameTemplateInput(value: string) {
    renameTemplateText = value;
    patchNamingPattern({
      templateOverride: value.trim() ? parseTemplateDisplayString(value) : null,
    });
  }

  function nodesSnapshot() {
    ensureFolderPlan();
    return $state.snapshot(blueprint.folderPlan!.nodes);
  }

  function variablesSnapshot() {
    ensureFolderPlan();
    return $state.snapshot(blueprint.folderPlan!.variables);
  }

  function findNode(nodes: FolderActionNode[], path: number[]) {
    let current = nodes[path[0]!]!;
    for (let i = 1; i < path.length; i++) {
      current = current.children[path[i]!]!;
    }
    return current;
  }

  function patchNode(path: number[], partial: Partial<FolderActionNode>) {
    const nodes = nodesSnapshot();
    const node = findNode(nodes, path);
    Object.assign(node, partial);
    if (partial.action === "delete") {
      node.children = [];
    }
    patchFolderPlan({ nodes });
  }

  function addRootAction(action: FolderAction = "create") {
    const nodes = nodesSnapshot();
    nodes.push(createFolderActionNode(action, action === "create" ? t("folders.defaultNewFolder") : ""));
    patchFolderPlan({ nodes });
  }

  function addChildAction(path: number[]) {
    const nodes = nodesSnapshot();
    const node = findNode(nodes, path);
    node.children.push(createFolderActionNode("create", t("folders.defaultSubfolder")));
    patchFolderPlan({ nodes });
  }

  function removeNode(path: number[]) {
    const nodes = nodesSnapshot();
    if (path.length === 1) {
      nodes.splice(path[0]!, 1);
    } else {
      const parent = findNode(nodes, path.slice(0, -1));
      parent.children.splice(path[path.length - 1]!, 1);
    }
    patchFolderPlan({ nodes });
  }

  function addVariable() {
    const variables = variablesSnapshot();
    variables.push({
      name: t("folders.defaultVariableName"),
      label: t("folders.defaultVariableLabel"),
      defaultValue: "",
      required: true,
    });
    patchFolderPlan({ variables });
  }

  function patchVariable(index: number, partial: Partial<FolderPlanVariable>) {
    const variables = variablesSnapshot();
    variables[index] = { ...variables[index]!, ...partial };
    patchFolderPlan({ variables });
  }

  function removeVariable(index: number) {
    const variables = variablesSnapshot();
    variables.splice(index, 1);
    patchFolderPlan({ variables });
  }
</script>

{#snippet planPreviewNode(node: FolderPlanPreviewNode, depth: number)}
  <li class="preview-item" style:padding-left={`${depth * 1.1}rem`}>
    <span class:danger={node.action === "delete"}>{folderActionLabel(node.action)}</span>
    <code>{node.relativePath}</code>
    {#if node.sourcePath}
      <small>{t("folders.previewSource")}: {node.sourcePath}</small>
    {/if}
    {#if node.children.length}
      <ul>
        {#each node.children as child}
          {@render planPreviewNode(child, depth + 1)}
        {/each}
      </ul>
    {/if}
  </li>
{/snippet}

{#snippet folderNodeEditor(node: FolderActionNode, path: number[], depth: number)}
  <li class="folder-node" style:margin-left={`${depth * 1.25}rem`}>
    <div class="node-grid">
      <label class="field compact">
        <span class="field-label">{t("folders.action")}</span>
        <select
          class="field-input"
          value={node.action}
          onchange={(e) => patchNode(path, { action: e.currentTarget.value as FolderAction })}
        >
          {#each folderActionOptions() as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </label>

      {#if node.action === "create"}
        <label class="field">
          <span class="field-label">{t("folders.nameTemplate")}</span>
          <input
            type="text"
            class="field-input mono"
            value={node.name}
            placeholder={"{projectName}"}
            oninput={(e) => patchNode(path, { name: e.currentTarget.value })}
          />
        </label>
      {:else if node.action === "rename"}
        <label class="field">
          <span class="field-label">{t("folders.sourceFolder")}</span>
          <input
            type="text"
            class="field-input mono"
            value={node.sourcePath}
            placeholder={t("folders.placeholderOldProject")}
            oninput={(e) => patchNode(path, { sourcePath: e.currentTarget.value })}
          />
        </label>
        <label class="field">
          <span class="field-label">{t("folders.newNameTemplate")}</span>
          <input
            type="text"
            class="field-input mono"
            value={node.name}
            placeholder={"{projectName}"}
            oninput={(e) => patchNode(path, { name: e.currentTarget.value })}
          />
        </label>
      {:else if node.action === "move" || node.action === "copy"}
        <label class="field">
          <span class="field-label">{t("folders.sourceFolder")}</span>
          <input
            type="text"
            class="field-input mono"
            value={node.sourcePath}
            placeholder={t("folders.placeholderSourceProject")}
            oninput={(e) => patchNode(path, { sourcePath: e.currentTarget.value })}
          />
        </label>
        <label class="field">
          <span class="field-label">{t("folders.targetTemplate")}</span>
          <input
            type="text"
            class="field-input mono"
            value={node.targetPath}
            placeholder={"{projectName}"}
            oninput={(e) => patchNode(path, { targetPath: e.currentTarget.value })}
          />
        </label>
      {:else}
        <label class="field">
          <span class="field-label">{t("folders.folderToDelete")}</span>
          <input
            type="text"
            class="field-input mono"
            value={node.sourcePath}
            placeholder={t("folders.placeholderOldProject")}
            oninput={(e) => patchNode(path, { sourcePath: e.currentTarget.value })}
          />
        </label>
      {/if}

      <div class="node-actions">
        {#if node.action !== "delete"}
          <button type="button" class="btn ghost" onclick={() => addChildAction(path)}>
            <Plus size={12} />
            {t("folders.childAction")}
          </button>
        {/if}
        <button
          type="button"
          class="btn ghost danger"
          aria-label={t("folders.removeNode")}
          onclick={() => removeNode(path)}
        >
          <Trash size={12} />
        </button>
      </div>
    </div>

    {#if node.children.length}
      <ul class="folder-tree">
        {#each node.children as child, index}
          {@render folderNodeEditor(child, [...path, index], depth + 1)}
        {/each}
      </ul>
    {/if}
  </li>
{/snippet}

<form
  id="blueprint-editor-form"
  class="editor"
  class:embedded
  onsubmit={(e) => {
    e.preventDefault();
    blueprint = { ...blueprint, kind: "folder", folderPlan: blueprint.folderPlan ?? createDefaultFolderPlan() };
    onsave?.();
  }}
>
  <nav class="editor-nav" class:collapsed={navCollapsed} aria-label={t("folders.nav.sectionsAria")}>
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
        {/if}
      </button>
    {/each}
  </nav>

  <div class="editor-content">
    {#if activePanel === "geral"}
      <div class="panel-header">
        <h3>{t("folders.nav.general")}</h3>
        <p>{t("folders.generalDesc")}</p>
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
            <span>{t("blueprints.activeStructure")}</span>
          </label>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>{t("folders.rootFolder")}</summary>
        <div class="collapsible-body">
          <div class="path-row">
            <label class="field">
              <span class="field-label">{t("folders.recipeRoot")}</span>
              <input type="text" class="field-input" bind:value={blueprint.rootPath} placeholder="C:\pasta\raiz" required />
            </label>
            <button type="button" class="btn" onclick={pickRootFolder}>
              <FolderOpen size={14} />
              {t("jobs.general.choose")}
            </button>
          </div>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>{t("folders.conflicts")}</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">{t("folders.whenTargetExists")}</span>
            <select
              class="field-input"
              value={blueprint.conflict}
              onchange={(e) =>
                (blueprint = {
                  ...blueprint,
                  conflict: e.currentTarget.value as Blueprint["conflict"],
                })}
            >
              <option value="skip">{t("folders.conflictIgnore")}</option>
              <option value="overwrite">{t("jobs.general.conflictOverwrite")}</option>
              <option value="rename">{t("folders.conflictAutoRename")}</option>
            </select>
          </label>
          <p class="muted">
            {t("folders.conflictHint")}
          </p>
        </div>
      </details>
    {:else if activePanel === "variaveis"}
      <div class="panel-header">
        <h3>{t("folders.nav.variables")}</h3>
        <p>{t("folders.variablesDesc")}</p>
      </div>

      <details class="collapsible" open>
        <summary>{t("folders.recipeFields")}</summary>
        <div class="collapsible-body">
          <button type="button" class="btn" onclick={addVariable}>
            <Plus size={14} />
            {t("folders.variable")}
          </button>

          {#if !blueprint.folderPlan?.variables.length}
            <p class="muted">{t("folders.noVariables")}</p>
          {:else}
            <ul class="variable-list">
              {#each blueprint.folderPlan.variables as variable, index}
                <li class="variable-row">
                  <label class="field">
                    <span class="field-label">{t("folders.token")}</span>
                    <input
                      type="text"
                      class="field-input mono"
                      value={variable.name}
                      placeholder="projectName"
                      pattern="[A-Za-z_][A-Za-z0-9_]*"
                      title={t("folders.tokenTitle")}
                      oninput={(e) => patchVariable(index, { name: e.currentTarget.value })}
                    />
                  </label>
                  <label class="field">
                    <span class="field-label">{t("folders.label")}</span>
                    <input
                      type="text"
                      class="field-input"
                      value={variable.label}
                      placeholder={t("folders.labelPlaceholder")}
                      oninput={(e) => patchVariable(index, { label: e.currentTarget.value })}
                    />
                  </label>
                  <label class="field">
                    <span class="field-label">{t("folders.defaultValue")}</span>
                    <input
                      type="text"
                      class="field-input"
                      value={variable.defaultValue}
                      placeholder={t("folders.defaultValuePlaceholder")}
                      oninput={(e) => patchVariable(index, { defaultValue: e.currentTarget.value })}
                    />
                  </label>
                  <label class="toggle variable-required">
                    <input
                      type="checkbox"
                      checked={variable.required}
                      onchange={(e) => patchVariable(index, { required: e.currentTarget.checked })}
                    />
                    <span>{t("folders.required")}</span>
                  </label>
                  <button
                    type="button"
                    class="btn ghost danger"
                    aria-label={t("folders.removeNode")}
                    onclick={() => removeVariable(index)}
                  >
                    <Trash size={12} />
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      </details>
    {:else if activePanel === "arvore"}
      <div class="panel-header">
        <h3>{t("folders.nav.tree")}</h3>
        <p>{t("folders.treeDesc")}</p>
      </div>

      <div class="tree-layout">
        <details class="collapsible" open>
          <summary>{t("folders.actions")}</summary>
          <div class="collapsible-body">
            <div class="toolbar">
              <button type="button" class="btn" onclick={() => addRootAction("create")}>
                <Plus size={14} />
                {t("folders.createFolder")}
              </button>
              <button type="button" class="btn ghost" onclick={() => addRootAction("rename")}>{t("folders.action.rename")}</button>
              <button type="button" class="btn ghost" onclick={() => addRootAction("move")}>{t("folders.action.move")}</button>
              <button type="button" class="btn ghost" onclick={() => addRootAction("copy")}>{t("folders.action.copy")}</button>
              <button type="button" class="btn ghost danger" onclick={() => addRootAction("delete")}>{t("folders.action.delete")}</button>
            </div>

            {#if !blueprint.folderPlan?.nodes.length}
              <p class="muted">{t("folders.noActions")}</p>
            {:else}
              <ul class="folder-tree">
                {#each blueprint.folderPlan.nodes as node, index}
                  {@render folderNodeEditor(node, [index], 0)}
                {/each}
              </ul>
            {/if}
          </div>
        </details>

        <aside class="preview-panel">
          <h4>{t("folders.preview")}</h4>
          {#if planPreviewLoading}
            <p class="muted">{t("folders.calculating")}</p>
          {:else if planPreviewResult}
            <p class="preview-summary">
              {t("folders.previewSummary", {
                count: planPreviewResult.folderCount,
                path: planPreviewResult.rootPath || blueprint.rootPath || "-",
              })}
            </p>
            {#if planPreviewResult.warnings.length}
              <ul class="warning-list">
                {#each planPreviewResult.warnings as warning}
                  <li>{warning}</li>
                {/each}
              </ul>
            {/if}
            {#if planPreviewResult.nodes.length}
              <ul class="preview-tree">
                {#each planPreviewResult.nodes as node}
                  {@render planPreviewNode(node, 0)}
                {/each}
              </ul>
            {/if}
          {:else}
            <p class="muted">{t("folders.noPreview")}</p>
          {/if}
        </aside>
      </div>
    {:else if activePanel === "seguranca"}
      <div class="panel-header">
        <h3>{t("folders.nav.security")}</h3>
        <p>{t("folders.securityDesc")}</p>
      </div>

      <details class="collapsible" open>
        <summary>{t("folders.deleteWithContent")}</summary>
        <div class="collapsible-body">
          <label class="toggle danger-text">
            <input
              type="checkbox"
              checked={blueprint.folderPlan?.allowDeleteNonEmpty ?? false}
              onchange={(e) => patchFolderPlan({ allowDeleteNonEmpty: e.currentTarget.checked })}
            />
            <span>{t("folders.allowDeleteNonEmpty")}</span>
          </label>

          <label class="toggle danger-text">
            <input
              type="checkbox"
              checked={blueprint.folderPlan?.confirmDeleteNonEmpty ?? false}
              onchange={(e) => patchFolderPlan({ confirmDeleteNonEmpty: e.currentTarget.checked })}
            />
            <span>{t("folders.confirmDeleteNonEmpty")}</span>
          </label>

          <p class="muted">
            {t("folders.deleteHint")}
          </p>
        </div>
      </details>
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
              {t("blueprints.watchHintPath", { path: blueprint.rootPath || "-" })}
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
                onchange={(e) => setDetectionKind(e.currentTarget.value as JobWatchDetectionMode["kind"])}
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

            <div class="auto-correct-section">
              <label class="toggle">
                <input
                  type="checkbox"
                  checked={blueprint.watch?.autoCorrect ?? false}
                  onchange={(e) => patchWatch({ autoCorrect: e.currentTarget.checked })}
                />
                <span>{t("folders.autoCorrect")}</span>
              </label>
              <p class="field-hint">
                {t("folders.autoCorrectHint")}
              </p>

              {#if blueprint.watch?.autoCorrect}
                <label class="field">
                  <span class="field-label">{t("folders.validationRegex")}</span>
                  <input
                    class="field-input"
                    type="text"
                    placeholder="^\\d{4}\\s+.*"
                    value={blueprint.watch?.namingPattern?.regex ?? ""}
                    oninput={(e) =>
                      patchNamingPattern({ regex: e.currentTarget.value || undefined })}
                  />
                </label>
                <p class="field-hint">
                  {t("folders.validationRegexHint")}
                </p>

                <label class="field">
                  <span class="field-label">{t("folders.renameTemplate")}</span>
                  <input
                    class="field-input mono"
                    type="text"
                    placeholder={"{year}-{month} {stem}"}
                    value={renameTemplateText}
                    onfocus={() => (renameTemplateFocused = true)}
                    onblur={() => {
                      renameTemplateFocused = false;
                      syncRenameTemplateText();
                    }}
                    oninput={(e) => handleRenameTemplateInput(e.currentTarget.value)}
                  />
                </label>
                <p class="field-hint">
                  {t("folders.renameTemplateHint")}
                </p>
              {/if}
            </div>
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

  .collapsible,
  .preview-panel {
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
    padding: var(--space-4);
    border-top: 1px solid var(--border);
  }

  .path-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-3);
    align-items: end;
  }

  .tree-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.25fr) minmax(18rem, 0.75fr);
    gap: var(--space-4);
    align-items: start;
  }

  .toolbar,
  .node-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .folder-tree,
  .variable-list,
  .preview-tree,
  .preview-item ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }

  .folder-node {
    display: grid;
    gap: var(--space-2);
  }

  .node-grid,
  .variable-row {
    display: grid;
    grid-template-columns: minmax(7rem, 0.45fr) minmax(0, 1fr) minmax(0, 1fr) auto;
    gap: var(--space-2);
    align-items: end;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
  }

  .variable-row {
    grid-template-columns: minmax(7rem, 0.7fr) minmax(8rem, 1fr) minmax(8rem, 1fr) auto auto;
  }

  .field.compact {
    min-width: 7rem;
  }

  .variable-required {
    align-self: center;
  }

  .preview-panel {
    padding: var(--space-4);
    position: sticky;
    top: 0;
  }

  .preview-panel h4 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
  }

  .preview-summary {
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
  }

  .preview-item {
    display: grid;
    gap: var(--space-1);
    font-size: var(--text-xs);
  }

  .preview-item span {
    font-weight: 600;
    color: var(--accent);
  }

  .preview-item span.danger,
  .danger-text {
    color: var(--danger);
  }

  .preview-item small {
    color: var(--text-muted);
  }

  .warning-list {
    margin: 0 0 var(--space-3);
    padding-left: 1rem;
    color: var(--danger);
    font-size: var(--text-xs);
  }

  .checkbox-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: var(--space-2);
  }

  .preset-group {
    border: none;
    margin: 0;
    padding: 0;
  }

  .auto-correct-section {
    margin-top: var(--space-3);
    padding-top: var(--space-3);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-hint,
  .muted {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .mono,
  code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  @media (max-width: 980px) {
    .tree-layout,
    .node-grid,
    .variable-row {
      grid-template-columns: 1fr;
    }

    .preview-panel {
      position: static;
    }
  }
</style>
