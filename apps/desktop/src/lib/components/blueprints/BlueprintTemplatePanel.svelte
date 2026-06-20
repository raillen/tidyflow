<script lang="ts">
  import {
    COUNTER_SCOPE_OPTIONS,
    TEMPLATE_QUICK_PATTERNS,
    TEMPLATE_REGEX_GROUPS,
    TEMPLATE_SYNTAX_LEGEND,
    TEMPLATE_TOKEN_GROUPS,
    TEMPLATE_TRANSFORM_GROUPS,
    type Blueprint,
    type TemplatePreview,
    type TemplateToolboxItem,
  } from "$lib/contracts/blueprint";
  import { Eye, Info, Lightning, TextAa } from "phosphor-svelte";

  type ToolboxTab = "tokens" | "transforms" | "regex";

  type Props = {
    blueprint: Blueprint;
    templateText: string;
    samplePath: string;
    previewLoading: boolean;
    previewResult: TemplatePreview | null;
    onTemplateInput: (value: string) => void;
    onSamplePathChange: (value: string) => void;
    onPatchRouting: (partial: Partial<Blueprint["routing"]>) => void;
    onPatchCounter: (partial: Partial<Blueprint["counter"]>) => void;
    onAppendSegment: (suffix: string) => void;
  };

  let {
    blueprint,
    templateText,
    samplePath,
    previewLoading,
    previewResult,
    onTemplateInput,
    onSamplePathChange,
    onPatchRouting,
    onPatchCounter,
    onAppendSegment,
  }: Props = $props();

  let toolboxTab = $state<ToolboxTab>("tokens");
  let toolboxFilter = $state("");
  let templateInput: HTMLTextAreaElement | undefined = $state();

  const templatePlaceholder = "{year}/{month}/{stem}{ext}";

  const filteredTokenGroups = $derived(filterGroups(TEMPLATE_TOKEN_GROUPS, toolboxFilter));
  const filteredTransformGroups = $derived(filterGroups(TEMPLATE_TRANSFORM_GROUPS, toolboxFilter));
  const filteredRegexGroups = $derived(filterGroups(TEMPLATE_REGEX_GROUPS, toolboxFilter));

  function filterGroups(groups: typeof TEMPLATE_TOKEN_GROUPS, query: string) {
    const q = query.trim().toLowerCase();
    if (!q) return groups;
    return groups
      .map((group) => ({
        ...group,
        items: group.items.filter(
          (item) =>
            item.label.toLowerCase().includes(q) ||
            item.name.toLowerCase().includes(q) ||
            item.syntax.toLowerCase().includes(q) ||
            item.hint.toLowerCase().includes(q),
        ),
      }))
      .filter((group) => group.items.length > 0);
  }

  function handleToolboxItem(item: TemplateToolboxItem, kind: "token" | "transform" | "regexPreset") {
    if (kind === "token") {
      const cursor = templateInput?.selectionStart ?? templateText.length;
      const next =
        templateText.slice(0, cursor) + item.syntax + templateText.slice(cursor);
      onTemplateInput(next);
    } else {
      onAppendSegment(item.syntax);
    }
    templateInput?.focus();
  }
</script>

<div class="template-workspace">
  <section class="template-col template-col-editor" aria-label="Editor de template">
    <div class="panel-card">
      <div class="card-head">
        <TextAa size={16} weight="duotone" aria-hidden="true" />
        <div>
          <h4>Caminho de destino</h4>
          <p>Monte o caminho relativo dentro da pasta raiz.</p>
        </div>
      </div>

      <label class="field">
        <span class="field-label">Template</span>
        <textarea
          bind:this={templateInput}
          class="field-input textarea mono template-input"
          rows="4"
          value={templateText}
          oninput={(e) => onTemplateInput(e.currentTarget.value)}
          placeholder={templatePlaceholder}
          spellcheck="false"
        ></textarea>
      </label>

      <div class="syntax-legend">
        {#each TEMPLATE_SYNTAX_LEGEND as row}
          <div class="syntax-row" title={row.hint}>
            <code>{row.syntax}</code>
            <span>{row.label}</span>
          </div>
        {/each}
      </div>

      <label class="toggle">
        <input
          type="checkbox"
          checked={blueprint.routing.createIntermediateDirs}
          onchange={(e) => onPatchRouting({ createIntermediateDirs: e.currentTarget.checked })}
        />
        <span>Criar pastas intermediárias automaticamente</span>
      </label>
    </div>

    <div class="panel-card">
      <div class="card-head compact">
        <Lightning size={15} aria-hidden="true" />
        <h4>Padrões rápidos</h4>
      </div>
      <div class="quick-patterns">
        {#each TEMPLATE_QUICK_PATTERNS as pattern}
          <button
            type="button"
            class="quick-btn"
            title={`Aplicar: ${pattern.value}`}
            onclick={() => onTemplateInput(pattern.value)}
          >
            {pattern.label}
          </button>
        {/each}
      </div>
    </div>

    <details class="panel-card counter-card">
      <summary>
        <span>Contador</span>
        <span class="muted summary-hint">Escopo e formatação de {"{counter}"}</span>
      </summary>
      <div class="counter-body">
        <label class="field">
          <span class="field-label">Escopo</span>
          <select
            class="field-input"
            value={blueprint.counter.scope}
            onchange={(e) =>
              onPatchCounter({
                scope: e.currentTarget.value as Blueprint["counter"]["scope"],
              })}
          >
            {#each COUNTER_SCOPE_OPTIONS as option}
              <option value={option.value} title={option.hint}>{option.label}</option>
            {/each}
          </select>
          <p class="field-hint">
            {COUNTER_SCOPE_OPTIONS.find((o) => o.value === blueprint.counter.scope)?.hint}
          </p>
        </label>

        <div class="field-row">
          <label class="field">
            <span class="field-label">Início</span>
            <input
              class="field-input"
              type="number"
              min="0"
              value={blueprint.counter.start}
              oninput={(e) => onPatchCounter({ start: Number(e.currentTarget.value) || 1 })}
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
              oninput={(e) => onPatchCounter({ padding: Number(e.currentTarget.value) || 0 })}
            />
          </label>
        </div>
      </div>
    </details>
  </section>

  <section class="template-col template-col-toolbox" aria-label="Toolbox de template">
    <div class="panel-card toolbox-card">
      <div class="card-head">
        <Info size={16} aria-hidden="true" />
        <div>
          <h4>Toolbox</h4>
          <p>Clique para inserir no cursor. Passe o mouse para ver detalhes.</p>
        </div>
      </div>

      <div class="toolbox-tabs" role="tablist" aria-label="Categorias da toolbox">
        <button
          type="button"
          class="toolbox-tab"
          class:active={toolboxTab === "tokens"}
          role="tab"
          aria-selected={toolboxTab === "tokens"}
          onclick={() => (toolboxTab = "tokens")}
        >
          Tokens
        </button>
        <button
          type="button"
          class="toolbox-tab"
          class:active={toolboxTab === "transforms"}
          role="tab"
          aria-selected={toolboxTab === "transforms"}
          onclick={() => (toolboxTab = "transforms")}
        >
          Estilo
        </button>
        <button
          type="button"
          class="toolbox-tab"
          class:active={toolboxTab === "regex"}
          role="tab"
          aria-selected={toolboxTab === "regex"}
          onclick={() => (toolboxTab = "regex")}
        >
          Regex
        </button>
      </div>

      <label class="field toolbox-search">
        <span class="sr-only">Filtrar toolbox</span>
        <input
          class="field-input"
          type="search"
          placeholder="Filtrar…"
          bind:value={toolboxFilter}
        />
      </label>

      <div class="toolbox-scroll">
        {#if toolboxTab === "tokens"}
          {#each filteredTokenGroups as group}
            <div class="toolbox-group">
              <h5>{group.label}</h5>
              <div class="toolbox-grid">
                {#each group.items as item}
                  <button
                    type="button"
                    class="toolbox-chip"
                    title={`${item.hint}${item.example ? ` Ex.: ${item.example}` : ""}`}
                    aria-label={`${item.label}: ${item.hint}`}
                    onclick={() => handleToolboxItem(item, "token")}
                  >
                    <code>{item.syntax}</code>
                    <span>{item.label}</span>
                  </button>
                {/each}
              </div>
            </div>
          {:else}
            <p class="muted empty-toolbox">Nenhum token corresponde ao filtro.</p>
          {/each}
        {:else if toolboxTab === "transforms"}
          {#each filteredTransformGroups as group}
            <div class="toolbox-group">
              <h5>{group.label}</h5>
              <div class="toolbox-grid">
                {#each group.items as item}
                  <button
                    type="button"
                    class="toolbox-chip"
                    title={`${item.hint}${item.example ? ` Ex.: ${item.example}` : ""}`}
                    aria-label={`${item.label}: ${item.hint}`}
                    onclick={() => handleToolboxItem(item, "transform")}
                  >
                    <code>{item.syntax}</code>
                    <span>{item.label}</span>
                  </button>
                {/each}
              </div>
            </div>
          {:else}
            <p class="muted empty-toolbox">Nenhuma transformação corresponde ao filtro.</p>
          {/each}
        {:else}
          {#each filteredRegexGroups as group}
            <div class="toolbox-group">
              <h5>{group.label}</h5>
              <div class="toolbox-grid">
                {#each group.items as item}
                  <button
                    type="button"
                    class="toolbox-chip"
                    title={`${item.hint}${item.example ? ` Ex.: ${item.example}` : ""}`}
                    aria-label={`${item.label}: ${item.hint}`}
                    onclick={() => handleToolboxItem(item, "regexPreset")}
                  >
                    <code>{item.syntax}</code>
                    <span>{item.label}</span>
                  </button>
                {/each}
              </div>
            </div>
          {:else}
            <p class="muted empty-toolbox">Nenhum preset corresponde ao filtro.</p>
          {/each}
        {/if}
      </div>
    </div>
  </section>

  <aside class="template-col template-col-preview" aria-label="Preview ao vivo">
    <div class="panel-card preview-card">
      <div class="card-head">
        <Eye size={16} weight="duotone" aria-hidden="true" />
        <div>
          <h4>Preview ao vivo</h4>
          <p>Atualiza enquanto você edita o template.</p>
        </div>
      </div>

      <label class="field">
        <span class="field-label">Arquivo de exemplo</span>
        <input
          class="field-input mono"
          value={samplePath}
          oninput={(e) => onSamplePathChange(e.currentTarget.value)}
          placeholder="C:\exemplo\Docs\relatorio.PDF"
        />
      </label>

      <div class="preview-live" aria-live="polite">
        {#if previewLoading}
          <p class="preview-state muted">Calculando…</p>
        {:else if previewResult}
          <div class="preview-result" class:invalid={!previewResult.valid}>
            <div class="preview-row">
              <span class="preview-label">Destino</span>
              <code class="preview-value">{previewResult.resultPath || "—"}</code>
            </div>
            <div class="preview-row">
              <span class="preview-label">Nome</span>
              <code class="preview-value">{previewResult.resultName || "—"}</code>
            </div>
            {#if !previewResult.valid}
              <p class="preview-warning">Caminho inválido — revise tokens ou caracteres.</p>
            {/if}
            {#if previewResult.warnings.length}
              <ul class="preview-warnings">
                {#each previewResult.warnings as warning}
                  <li>{warning}</li>
                {/each}
              </ul>
            {/if}
          </div>
        {:else}
          <p class="preview-state muted">Informe um template e um arquivo de exemplo.</p>
        {/if}
      </div>

      <p class="preview-footnote muted">
        Transformações e regex presets aplicam-se ao trecho acumulado à esquerda no pipeline.
      </p>
    </div>
  </aside>
</div>

<style>
  .template-workspace {
    display: grid;
    grid-template-columns: minmax(0, 1.1fr) minmax(260px, 320px) minmax(240px, 280px);
    gap: var(--space-3);
    flex: 1;
    min-height: 0;
    align-items: stretch;
  }

  .template-col {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-height: 0;
    min-width: 0;
  }

  .template-col-preview {
    position: sticky;
    top: 0;
    align-self: start;
    max-height: calc(100vh - 12rem);
  }

  .panel-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    padding: var(--space-3) var(--space-4);
    display: grid;
    gap: var(--space-3);
  }

  .card-head {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    color: var(--accent);
  }

  .card-head.compact {
    align-items: center;
  }

  .card-head h4 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-primary);
  }

  .card-head p {
    margin: 0.15rem 0 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .template-input {
    min-height: 6.5rem;
    resize: vertical;
  }

  .syntax-legend {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-2);
  }

  .syntax-row {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--surface-muted);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    cursor: help;
  }

  .syntax-row code {
    font-family: var(--font-mono);
    color: var(--accent);
    font-size: var(--text-xs);
  }

  .quick-patterns {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .quick-btn {
    padding: 0.3rem 0.6rem;
    border-radius: 999px;
    border: 1px dashed var(--border);
    background: var(--surface-muted);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .quick-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .counter-card > summary {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: 600;
    list-style: none;
  }

  .counter-card > summary::-webkit-details-marker {
    display: none;
  }

  .summary-hint {
    font-size: var(--text-xs);
    font-weight: 400;
  }

  .counter-body {
    display: grid;
    gap: var(--space-3);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border);
  }

  .toolbox-card {
    flex: 1;
    min-height: 0;
    grid-template-rows: auto auto auto 1fr;
  }

  .toolbox-tabs {
    display: flex;
    gap: var(--space-1);
    padding: 0.15rem;
    border-radius: var(--radius-md);
    background: var(--surface-muted);
  }

  .toolbox-tab {
    flex: 1;
    padding: 0.35rem 0.5rem;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
  }

  .toolbox-tab.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: 0 1px 2px rgb(0 0 0 / 6%);
  }

  .toolbox-search {
    margin: 0;
  }

  .toolbox-scroll {
    overflow-y: auto;
    min-height: 12rem;
    max-height: 100%;
    display: grid;
    gap: var(--space-3);
    padding-right: 0.15rem;
  }

  .toolbox-group h5 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .toolbox-grid {
    display: grid;
    gap: var(--space-2);
  }

  .toolbox-chip {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    text-align: left;
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease;
  }

  .toolbox-chip:hover,
  .toolbox-chip:focus-visible {
    border-color: var(--accent);
    background: var(--accent-soft);
    outline: none;
  }

  .toolbox-chip code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--accent);
    white-space: nowrap;
  }

  .toolbox-chip span {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .preview-card {
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .preview-live {
    min-height: 8rem;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    border: 1px solid var(--border);
  }

  .preview-result {
    display: grid;
    gap: var(--space-3);
  }

  .preview-result.invalid {
    border-color: var(--danger);
  }

  .preview-row {
    display: grid;
    gap: var(--space-1);
  }

  .preview-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .preview-value {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    word-break: break-all;
    color: var(--text-primary);
  }

  .preview-warning {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--danger);
  }

  .preview-warnings {
    margin: 0;
    padding-left: 1rem;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .preview-state,
  .preview-footnote {
    margin: 0;
    font-size: var(--text-xs);
  }

  .preview-footnote {
    line-height: 1.45;
  }

  .empty-toolbox {
    margin: 0;
    font-size: var(--text-sm);
  }

  .field-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .field-hint {
    margin: var(--space-1) 0 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .toggle {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .muted {
    color: var(--text-muted);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  @media (max-width: 1100px) {
    .template-workspace {
      grid-template-columns: 1fr 1fr;
    }

    .template-col-preview {
      grid-column: 1 / -1;
      position: static;
      max-height: none;
    }
  }

  @media (max-width: 720px) {
    .template-workspace {
      grid-template-columns: 1fr;
    }

    .syntax-legend {
      grid-template-columns: 1fr;
    }
  }
</style>
