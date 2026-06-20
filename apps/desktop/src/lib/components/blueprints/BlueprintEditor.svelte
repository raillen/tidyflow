<script lang="ts">
  import {
    countBlueprintFilters,
    parseTemplateDisplayString,
    pipelineToDisplayString,
    TEMPLATE_REGEX_PRESETS,
    TEMPLATE_TOKENS,
    TEMPLATE_TRANSFORMS,
    type Blueprint,
    type BlueprintKind,
    type BlueprintOperation,
    type FolderNode,
    type FolderPlanPreviewNode,
  } from "$lib/contracts/blueprint";
  import {
    EXCLUDE_PRESETS,
    type FileFilter,
    type WatchConfig as JobWatchConfig,
    type WatchDetectionMode as JobWatchDetectionMode,
    type WatchEventKind as JobWatchEventKind,
  } from "$lib/contracts/job";
  import { previewBlueprintPlan, previewBlueprintTemplate } from "$lib/core/ipc/client";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowsLeftRight,
    CalendarBlank,
    CaretDoubleLeft,
    CaretDoubleRight,
    Copy,
    Eye,
    FolderOpen,
    FolderSimple,
    Funnel,
    IdentificationCard,
    Plus,
    TextAa,
    Trash,
  } from "phosphor-svelte";

  type ConflictStrategy = Blueprint["conflict"];
  type NavPanel = "geral" | "busca" | "template" | "pastas" | "automacao";

  type Props = {
    blueprint: Blueprint;
    embedded?: boolean;
    onsave?: () => void;
  };

  let { blueprint = $bindable(), embedded = true, onsave }: Props = $props();

  let activePanel = $state<NavPanel>("geral");
  let navCollapsed = $state(false);
  let templateText = $state(pipelineToDisplayString(blueprint.routing.pathTemplate));
  let renameTemplateText = $state(
    blueprint.renameTemplate ? pipelineToDisplayString(blueprint.renameTemplate) : "",
  );
  let samplePath = $state("C:\\exemplo\\Docs\\relatorio.PDF");
  let previewLoading = $state(false);
  let previewResult = $state<Awaited<ReturnType<typeof previewBlueprintTemplate>> | null>(null);
  let planPreviewLoading = $state(false);
  let planPreviewResult = $state<Awaited<ReturnType<typeof previewBlueprintPlan>> | null>(null);
  let templateInput: HTMLTextAreaElement | undefined = $state();

  const renamePlaceholder = "{stem}_{counter}{ext}";
  const templatePlaceholder = "{year}/{month}/{stem}{ext}";

  const filterCount = $derived(countBlueprintFilters(blueprint.search));

  const NAV_ITEMS: { id: NavPanel; label: string; icon: typeof IdentificationCard; folderOnly?: boolean }[] = [
    { id: "geral", label: "Geral", icon: IdentificationCard },
    { id: "busca", label: "Busca", icon: Funnel },
    { id: "template", label: "Template", icon: TextAa },
    { id: "pastas", label: "Pastas", icon: FolderSimple, folderOnly: true },
    { id: "automacao", label: "Automação", icon: Eye },
  ];

  const visibleNavItems = $derived(
    NAV_ITEMS.filter((item) => !item.folderOnly || blueprint.kind === "folder"),
  );

  const WATCH_EVENTS: { id: JobWatchEventKind; label: string }[] = [
    { id: "create", label: "Criação" },
    { id: "modify", label: "Modificação" },
    { id: "remove", label: "Remoção" },
    { id: "rename", label: "Renomeação" },
  ];

  const DETECTION_OPTIONS: { value: JobWatchDetectionMode["kind"]; label: string; hint: string }[] = [
    { value: "realtime", label: "Tempo real", hint: "Eventos nativos do SO — ideal para disco local." },
    { value: "polling", label: "Polling", hint: "Varredura periódica — ideal para rede/USB." },
    { value: "hybrid", label: "Híbrido", hint: "Tempo real + polling de backup — máxima confiabilidade." },
  ];

  let extensionsText = $state(blueprint.search.includeExtensions.join(", "));
  let excludePatternsText = $state(blueprint.search.excludePatterns.join("\n"));

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
    const timer = setTimeout(async () => {
      previewLoading = true;
      try {
        previewResult = await previewBlueprintTemplate(pipeline, samplePath);
      } catch {
        previewResult = null;
      } finally {
        previewLoading = false;
      }
    }, 350);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (activePanel !== "pastas" || blueprint.kind !== "folder") return;
    const rootPath = blueprint.rootPath;
    const folderPlan = blueprint.folderPlan ?? { nodes: [] };
    const timer = setTimeout(async () => {
      planPreviewLoading = true;
      try {
        planPreviewResult = await previewBlueprintPlan(rootPath, folderPlan);
      } catch {
        planPreviewResult = null;
      } finally {
        planPreviewLoading = false;
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
      folderPlan: kind === "folder" ? blueprint.folderPlan ?? { nodes: [] } : blueprint.folderPlan,
    };
    if (kind === "file" && activePanel === "pastas") {
      activePanel = "geral";
    }
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

  function insertToken(token: string) {
    const cursor = templateInput?.selectionStart ?? templateText.length;
    const next = templateText.slice(0, cursor) + `{${token}}` + templateText.slice(cursor);
    handleTemplateInput(next);
  }

  function appendTransform(name: string, kind: "transform" | "regexPreset") {
    const suffix = kind === "transform" ? `[${name}]` : `/${name}/`;
    handleTemplateInput(templateText + suffix);
  }

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

  function ensureFolderPlan() {
    if (!blueprint.folderPlan) {
      blueprint = { ...blueprint, folderPlan: { nodes: [] } };
    }
  }

  function addRootFolderNode() {
    ensureFolderPlan();
    blueprint = {
      ...blueprint,
      folderPlan: {
        nodes: [...(blueprint.folderPlan?.nodes ?? []), { name: "Nova pasta", children: [] }],
      },
    };
  }

  function updateFolderNode(path: number[], name: string) {
    ensureFolderPlan();
    const nodes = structuredClone(blueprint.folderPlan!.nodes);
    let current = nodes;
    for (let i = 0; i < path.length - 1; i++) {
      current = current[path[i]!]!.children;
    }
    current[path[path.length - 1]!]!.name = name;
    blueprint = { ...blueprint, folderPlan: { nodes } };
  }

  function addChildFolderNode(path: number[]) {
    ensureFolderPlan();
    const nodes = structuredClone(blueprint.folderPlan!.nodes);
    let current = nodes;
    for (const index of path) {
      current = current[index]!.children;
    }
    current.push({ name: "Subpasta", children: [] });
    blueprint = { ...blueprint, folderPlan: { nodes } };
  }

  function removeFolderNode(path: number[]) {
    ensureFolderPlan();
    const nodes = structuredClone(blueprint.folderPlan!.nodes);
    if (path.length === 1) {
      nodes.splice(path[0]!, 1);
    } else {
      let current = nodes;
      for (let i = 0; i < path.length - 1; i++) {
        current = current[path[i]!]!.children;
      }
      current.splice(path[path.length - 1]!, 1);
    }
    blueprint = { ...blueprint, folderPlan: { nodes } };
  }

  function renderFolderNodes(nodes: FolderNode[], path: number[] = []) {
    return nodes.map((node, index) => ({ node, path: [...path, index] }));
  }
</script>

{#snippet planPreviewNode(node: FolderPlanPreviewNode, depth: number)}
  <li class="plan-preview-item" style:padding-left={`${depth * 1.25}rem`}>
    <code>{node.relativePath}</code>
    {#if node.children.length}
      <ul>
        {#each node.children as child}
          {@render planPreviewNode(child, depth + 1)}
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
    blueprint = {
      ...blueprint,
      search: { ...blueprint.search, recursive: blueprint.recursive },
    };
    onsave?.();
  }}
>
  <nav class="editor-nav" class:collapsed={navCollapsed} aria-label="Seções do blueprint">
    <button
      type="button"
      class="nav-toggle btn ghost"
      aria-label={navCollapsed ? "Expandir menu" : "Recolher menu"}
      onclick={() => (navCollapsed = !navCollapsed)}
    >
      {#if navCollapsed}
        <CaretDoubleRight size={16} />
      {:else}
        <CaretDoubleLeft size={16} />
      {/if}
    </button>

    {#each visibleNavItems as item}
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
          <span class="nav-dot" aria-label="{filterCount} filtros ativos"></span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="editor-content">
    {#if activePanel === "geral"}
      <div class="panel-header">
        <h3>Geral</h3>
        <p>Nome, pasta raiz e operação.</p>
      </div>

      <details class="collapsible" open>
        <summary>Identificação</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Nome</span>
            <input class="field-input" bind:value={blueprint.name} maxlength="120" required />
          </label>

          <label class="toggle">
            <input type="checkbox" bind:checked={blueprint.enabled} />
            <span>Blueprint ativo</span>
          </label>

          <div class="subsection">
            <span class="field-label">Tipo</span>
            <div class="option-grid">
              <button
                type="button"
                class="option-btn"
                class:selected={blueprint.kind === "file"}
                onclick={() => setKind("file")}
              >
                Arquivos
              </button>
              <button
                type="button"
                class="option-btn"
                class:selected={blueprint.kind === "folder"}
                onclick={() => setKind("folder")}
              >
                Pastas
              </button>
            </div>
          </div>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Pasta raiz</summary>
        <div class="collapsible-body">
          <div class="path-row">
            <label class="field">
              <span class="field-label">Raiz de busca</span>
              <input
                class="field-input"
                bind:value={blueprint.rootPath}
                placeholder="C:\pasta\raiz"
                required
              />
            </label>
            <button type="button" class="btn" onclick={pickRootFolder}>
              <FolderOpen size={14} />
              Escolher
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
            <span>Busca recursiva em subpastas</span>
          </label>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Operação</summary>
        <div class="collapsible-body">
          <div class="option-grid">
            <button
              type="button"
              class="option-btn"
              class:selected={blueprint.operation === "copy"}
              onclick={() => setOperation("copy")}
            >
              <Copy size={16} />
              Cópia
            </button>
            <button
              type="button"
              class="option-btn"
              class:selected={blueprint.operation === "move"}
              onclick={() => setOperation("move")}
            >
              <ArrowsLeftRight size={16} />
              Mover
            </button>
          </div>

          <label class="field">
            <span class="field-label">Conflito</span>
            <select
              class="field-input"
              value={blueprint.conflict}
              onchange={(e) => setConflict(e.currentTarget.value as ConflictStrategy)}
            >
              <option value="skip">Ignorar existentes</option>
              <option value="overwrite">Sobrescrever</option>
              <option value="rename">Renomear</option>
            </select>
          </label>

          {#if blueprint.conflict === "rename"}
            <label class="field">
              <span class="field-label">Template de renomeação (opcional)</span>
              <input
                class="field-input mono"
                value={renameTemplateText}
                oninput={(e) => handleRenameTemplateInput(e.currentTarget.value)}
                placeholder={renamePlaceholder}
              />
            </label>
          {/if}
        </div>
      </details>
    {:else if activePanel === "busca"}
      <div class="panel-header">
        <h3>Busca</h3>
        <p>Filtros aplicados na varredura recursiva.</p>
      </div>

      <details class="collapsible" open>
        <summary>Extensões e exclusões</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Extensões incluídas (vazio = todas)</span>
            <input
              class="field-input"
              value={extensionsText}
              oninput={(e) => handleExtensionsInput(e.currentTarget.value)}
              placeholder=".pdf, .docx"
            />
          </label>

          <fieldset class="preset-group">
            <legend class="field-label">Presets de exclusão</legend>
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
            <span class="field-label">Padrões de exclusão (um por linha)</span>
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
        <summary>Regex e tamanho</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Regex do nome</span>
            <input
              class="field-input"
              value={blueprint.search.nameRegex ?? ""}
              oninput={(e) => patchSearch({ nameRegex: e.currentTarget.value || null })}
            />
          </label>

          <label class="field">
            <span class="field-label">Regex do caminho</span>
            <input
              class="field-input"
              value={blueprint.search.pathRegex ?? ""}
              oninput={(e) => patchSearch({ pathRegex: e.currentTarget.value || null })}
            />
          </label>

          <div class="field-row">
            <label class="field">
              <span class="field-label">Tamanho mínimo (bytes)</span>
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
              <span class="field-label">Tamanho máximo (bytes)</span>
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
      <div class="panel-header">
        <h3>Template de destino</h3>
        <p>Define subpastas e nome final com tokens.</p>
      </div>

      <details class="collapsible" open>
        <summary>Caminho de destino</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Template</span>
            <textarea
              bind:this={templateInput}
              class="field-input textarea mono"
              rows="3"
              value={templateText}
              oninput={(e) => handleTemplateInput(e.currentTarget.value)}
              placeholder={templatePlaceholder}
            ></textarea>
          </label>

          <label class="toggle">
            <input
              type="checkbox"
              checked={blueprint.routing.createIntermediateDirs}
              onchange={(e) =>
                patchRouting({ createIntermediateDirs: e.currentTarget.checked })}
            />
            <span>Criar pastas intermediárias</span>
          </label>

          <div class="toolbox">
            <span class="field-label">Tokens</span>
            <div class="chip-row">
              {#each TEMPLATE_TOKENS as token}
                <button type="button" class="chip-btn" onclick={() => insertToken(token.name)}>
                  {token.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="toolbox">
            <span class="field-label">Transformações</span>
            <div class="chip-row">
              {#each TEMPLATE_TRANSFORMS as transform}
                <button
                  type="button"
                  class="chip-btn"
                  onclick={() => appendTransform(transform.name, "transform")}
                >
                  {transform.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="toolbox">
            <span class="field-label">Regex presets</span>
            <div class="chip-row">
              {#each TEMPLATE_REGEX_PRESETS as preset}
                <button
                  type="button"
                  class="chip-btn"
                  onclick={() => appendTransform(preset.name, "regexPreset")}
                >
                  {preset.label}
                </button>
              {/each}
            </div>
          </div>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Contador</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Escopo</span>
            <select
              class="field-input"
              value={blueprint.counter.scope}
              onchange={(e) =>
                patchCounter({
                  scope: e.currentTarget.value as Blueprint["counter"]["scope"],
                })}
            >
              <option value="global">Global</option>
              <option value="perDay">Por dia</option>
              <option value="perFolder">Por pasta destino</option>
              <option value="perParent">Por pasta pai</option>
            </select>
          </label>

          <div class="field-row">
            <label class="field">
              <span class="field-label">Início</span>
              <input
                class="field-input"
                type="number"
                min="0"
                value={blueprint.counter.start}
                oninput={(e) => patchCounter({ start: Number(e.currentTarget.value) || 1 })}
              />
            </label>
            <label class="field">
              <span class="field-label">Padding (zeros)</span>
              <input
                class="field-input"
                type="number"
                min="0"
                max="12"
                value={blueprint.counter.padding}
                oninput={(e) => patchCounter({ padding: Number(e.currentTarget.value) || 0 })}
              />
            </label>
          </div>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Preview</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Arquivo de exemplo</span>
            <input class="field-input mono" bind:value={samplePath} />
          </label>

          {#if previewLoading}
            <p class="muted">Calculando preview…</p>
          {:else if previewResult}
            <div class="preview-box" class:invalid={!previewResult.valid}>
              <p><strong>Destino:</strong> <code>{previewResult.resultPath}</code></p>
              <p><strong>Nome:</strong> <code>{previewResult.resultName}</code></p>
              {#if previewResult.warnings.length}
                <ul>
                  {#each previewResult.warnings as warning}
                    <li>{warning}</li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        </div>
      </details>
    {:else if activePanel === "pastas"}
      <div class="panel-header">
        <h3>Plano de pastas</h3>
        <p>Scaffolding opcional para blueprints de pastas.</p>
      </div>

      <details class="collapsible" open>
        <summary>Árvore</summary>
        <div class="collapsible-body">
          <button type="button" class="btn" onclick={addRootFolderNode}>
            <Plus size={14} />
            Pasta raiz
          </button>

          {#if !blueprint.folderPlan?.nodes.length}
            <p class="muted">Nenhuma pasta definida. O template de destino ainda funciona sozinho.</p>
          {:else}
            <ul class="folder-tree">
              {#each renderFolderNodes(blueprint.folderPlan.nodes) as { node, path }}
                <li class="folder-row">
                  <input
                    class="field-input"
                    value={node.name}
                    oninput={(e) => updateFolderNode(path, e.currentTarget.value)}
                  />
                  <button type="button" class="btn ghost" onclick={() => addChildFolderNode(path)}>
                    <Plus size={12} />
                  </button>
                  <button type="button" class="btn ghost danger" onclick={() => removeFolderNode(path)}>
                    <Trash size={12} />
                  </button>
                </li>
                {#each renderFolderNodes(node.children, path) as child}
                  <li class="folder-row nested">
                    <input
                      class="field-input"
                      value={child.node.name}
                      oninput={(e) => updateFolderNode(child.path, e.currentTarget.value)}
                    />
                    <button type="button" class="btn ghost" onclick={() => addChildFolderNode(child.path)}>
                      <Plus size={12} />
                    </button>
                    <button type="button" class="btn ghost danger" onclick={() => removeFolderNode(child.path)}>
                      <Trash size={12} />
                    </button>
                  </li>
                {/each}
              {/each}
            </ul>
          {/if}
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Preview do scaffolding</summary>
        <div class="collapsible-body">
          {#if planPreviewLoading}
            <p class="muted">Calculando árvore…</p>
          {:else if planPreviewResult}
            <div class="preview-box" class:invalid={!planPreviewResult.valid}>
              <p>
                <strong>{planPreviewResult.folderCount}</strong>
                pasta(s) sob
                <code>{planPreviewResult.rootPath || blueprint.rootPath || "—"}</code>
              </p>
              {#if planPreviewResult.warnings.length}
                <ul>
                  {#each planPreviewResult.warnings as warning}
                    <li>{warning}</li>
                  {/each}
                </ul>
              {/if}
              {#if planPreviewResult.nodes.length}
                <ul class="plan-preview-tree">
                  {#each planPreviewResult.nodes as node}
                    {@render planPreviewNode(node, 0)}
                  {/each}
                </ul>
              {:else}
                <p class="muted">Nenhuma pasta no plano.</p>
              {/if}
            </div>
          {/if}
        </div>
      </details>
    {:else if activePanel === "automacao"}
      <div class="panel-header">
        <h3>Automação</h3>
        <p>Monitoramento da pasta raiz.</p>
      </div>

      <details class="collapsible" open>
        <summary>Watch na raiz</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={blueprint.watch?.enabled ?? false}
              onchange={(e) => setWatchEnabled(e.currentTarget.checked)}
            />
            <span>Monitoramento ativo</span>
          </label>

          {#if blueprint.watch?.enabled}
            <p class="field-hint">
              Monitora <code>{blueprint.rootPath || "—"}</code>.
            </p>

            <label class="field">
              <span class="field-label">Estabilização (segundos)</span>
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
              <span class="field-label">Método de detecção</span>
              <select
                class="field-input"
                value={blueprint.watch.detection.kind}
                onchange={(e) =>
                  setDetectionKind(e.currentTarget.value as JobWatchDetectionMode["kind"])}
              >
                {#each DETECTION_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </label>

            {#if blueprint.watch.detection.kind === "polling"}
              <label class="field">
                <span class="field-label">Intervalo (segundos)</span>
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
                <span class="field-label">Polling de backup (segundos)</span>
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
              <legend class="field-label">Eventos</legend>
              <div class="checkbox-grid">
                {#each WATCH_EVENTS as event}
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
          <summary>Última execução</summary>
          <div class="collapsible-body">
            <p class="muted">
              <CalendarBlank size={14} />
              {new Intl.DateTimeFormat("pt-BR", { dateStyle: "short", timeStyle: "medium" }).format(
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

  .toolbox {
    display: grid;
    gap: var(--space-2);
  }

  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .chip-btn {
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-muted);
    font-size: var(--text-xs);
    cursor: pointer;
  }

  .preview-box {
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--success);
    background: color-mix(in srgb, var(--success) 8%, transparent);
    font-size: var(--text-sm);
  }

  .preview-box.invalid {
    border-color: var(--danger);
    background: color-mix(in srgb, var(--danger) 8%, transparent);
  }

  .preview-box p {
    margin: 0 0 var(--space-2);
  }

  .preview-box ul {
    margin: 0;
    padding-left: 1rem;
  }

  .folder-tree {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }

  .folder-row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: var(--space-2);
    align-items: center;
  }

  .folder-row.nested {
    margin-left: var(--space-4);
  }

  .plan-preview-tree {
    list-style: none;
    margin: var(--space-2) 0 0;
    padding: 0;
    display: grid;
    gap: var(--space-1);
  }

  .plan-preview-item {
    font-size: var(--text-xs);
  }

  .plan-preview-item ul {
    list-style: none;
    margin: var(--space-1) 0 0;
    padding: 0;
    display: grid;
    gap: var(--space-1);
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

  code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
</style>
