<script lang="ts">
  import {
    countActiveFilters,
    EXCLUDE_PRESETS,
    type FileFilter,
    type Job,
    type NotifyChannel,
    type NotifyConfig,
    type ScheduleConfig,
    type ScheduleRule,
    type ScriptsConfig,
    type TransferOptions,
    type WatchConfig,
    type WatchDetectionMode,
    type WatchEventKind,
  } from "$lib/contracts/job";

  type JobMode = Job["mode"];
  type ConflictStrategy = Job["conflict"];
  type SymlinkMode = FileFilter["symlinkMode"];
  type NotifyEvent = NotifyConfig["events"][number];
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowsLeftRight,
    Bell,
    Calendar,
    CaretDoubleLeft,
    CaretDoubleRight,
    Code,
    Copy,
    Eye,
    FolderOpen,
    Funnel,
    IdentificationCard,
    Plus,
    ShieldCheck,
    Trash,
  } from "phosphor-svelte";

  type NavPanel = "geral" | "filtros" | "automacao" | "monitoramento" | "seguranca" | "avancado";

  type Props = {
    job: Job;
    embedded?: boolean;
    onsave?: () => void;
  };

  let { job = $bindable(), embedded = true, onsave }: Props = $props();

  let activePanel = $state<NavPanel>("geral");
  let navCollapsed = $state(false);

  const filterCount = $derived(countActiveFilters(job.filters));

  const NAV_ITEMS: { id: NavPanel; label: string; icon: typeof IdentificationCard }[] = [
    { id: "geral", label: "Geral", icon: IdentificationCard },
    { id: "filtros", label: "Filtros", icon: Funnel },
    { id: "automacao", label: "Automação", icon: Calendar },
    { id: "monitoramento", label: "Monitoramento", icon: Eye },
    { id: "seguranca", label: "Segurança", icon: ShieldCheck },
    { id: "avancado", label: "Avançado", icon: Code },
  ];

  const WEEKDAYS = [
    { value: 0, label: "Dom", fullLabel: "Domingo" },
    { value: 1, label: "Seg", fullLabel: "Segunda-feira" },
    { value: 2, label: "Ter", fullLabel: "Terça-feira" },
    { value: 3, label: "Qua", fullLabel: "Quarta-feira" },
    { value: 4, label: "Qui", fullLabel: "Quinta-feira" },
    { value: 5, label: "Sex", fullLabel: "Sexta-feira" },
    { value: 6, label: "Sáb", fullLabel: "Sábado" },
  ] as const;

  const WATCH_EVENTS: { id: WatchEventKind; label: string }[] = [
    { id: "create", label: "Criação" },
    { id: "modify", label: "Modificação" },
    { id: "remove", label: "Remoção" },
    { id: "rename", label: "Renomeação" },
  ];

  const DETECTION_OPTIONS: { value: WatchDetectionMode["kind"]; label: string; hint: string }[] = [
    { value: "realtime", label: "Tempo real", hint: "Eventos nativos do SO — ideal para disco local." },
    { value: "polling", label: "Polling", hint: "Varredura periódica — ideal para rede/USB." },
    { value: "hybrid", label: "Híbrido", hint: "Tempo real + polling de backup — máxima confiabilidade." },
  ];

  const RULE_KIND_OPTIONS: { value: ScheduleRule["kind"]; label: string }[] = [
    { value: "interval", label: "Intervalo" },
    { value: "daily", label: "Diário" },
    { value: "weekly", label: "Dias da Semana" },
  ];

  const NOTIFY_EVENTS: { id: NotifyEvent; label: string }[] = [
    { id: "started", label: "Início" },
    { id: "completed", label: "Concluído" },
    { id: "failed", label: "Falha" },
  ];

  let extensionsText = $state(job.filters.includeExtensions.join(", "));
  let contentExtensionsText = $state(job.filters.contentExtensions.join(", "));
  let excludePatternsText = $state(job.filters.excludePatterns.join("\n"));

  $effect(() => {
    extensionsText = job.filters.includeExtensions.join(", ");
    contentExtensionsText = job.filters.contentExtensions.join(", ");
    excludePatternsText = job.filters.excludePatterns.join("\n");
  });

  function patchFilters(partial: Partial<FileFilter>) {
    job = { ...job, filters: { ...job.filters, ...partial } };
  }

  function patchOptions(partial: Partial<TransferOptions>) {
    job = { ...job, options: { ...job.options, ...partial } };
  }

  function patchScripts(partial: Partial<ScriptsConfig>) {
    job = { ...job, scripts: { ...job.scripts, ...partial } };
  }

  function patchNotify(partial: Partial<typeof job.notify>) {
    job = { ...job, notify: { ...job.notify, ...partial } };
  }

  function patchSchedule(partial: Partial<ScheduleConfig>) {
    const base = job.schedule ?? defaultSchedule();
    job = { ...job, schedule: { ...base, ...partial } };
  }

  function patchRule(rule: ScheduleRule) {
    patchSchedule({ rule });
  }

  function defaultSchedule(): ScheduleConfig {
    return {
      enabled: true,
      timezone: "local",
      rule: { kind: "interval", minutes: 60 },
    };
  }

  async function pickFolder(field: "sourcePath" | "targetPath") {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      job = { ...job, [field]: selected };
    }
  }

  function setMode(mode: JobMode) {
    job = { ...job, mode };
  }

  function setConflict(conflict: ConflictStrategy) {
    job = { ...job, conflict };
  }

  function handleExtensionsInput(value: string) {
    extensionsText = value;
    patchFilters({
      includeExtensions: value
        .split(",")
        .map((ext) => ext.trim())
        .filter(Boolean),
    });
  }

  function handleContentExtensionsInput(value: string) {
    contentExtensionsText = value;
    patchFilters({
      contentExtensions: value
        .split(",")
        .map((ext) => ext.trim())
        .filter(Boolean),
    });
  }

  function handleExcludePatternsInput(value: string) {
    excludePatternsText = value;
    patchFilters({
      excludePatterns: value
        .split("\n")
        .map((line) => line.trim())
        .filter(Boolean),
    });
  }

  function togglePreset(id: string) {
    const ids = job.filters.excludePresetIds;
    patchFilters({
      excludePresetIds: ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id],
    });
  }

  function isoToLocal(iso: string | null | undefined): string {
    if (!iso) return "";
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "";
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  function localToIso(local: string): string | null {
    if (!local) return null;
    const d = new Date(local);
    if (Number.isNaN(d.getTime())) return null;
    return d.toISOString();
  }

  function defaultWatch(): WatchConfig {
    return {
      enabled: true,
      settleSeconds: 2,
      detection: { kind: "realtime" },
      events: ["create"],
    };
  }

  function patchWatch(partial: Partial<WatchConfig>) {
    const base = job.watch ?? defaultWatch();
    job = { ...job, watch: { ...base, ...partial } };
  }

  function setWatchEnabled(enabled: boolean) {
    if (enabled) {
      job = {
        ...job,
        watch: job.watch ? { ...job.watch, enabled: true } : defaultWatch(),
        schedule: job.schedule ? { ...job.schedule, enabled: false } : job.schedule,
      };
    } else if (job.watch) {
      job = { ...job, watch: { ...job.watch, enabled: false } };
    } else {
      job = { ...job, watch: null };
    }
  }

  function setDetectionKind(kind: WatchDetectionMode["kind"]) {
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

  function toggleWatchEvent(event: WatchEventKind) {
    const watch = job.watch ?? defaultWatch();
    const events = watch.events.includes(event)
      ? watch.events.filter((item) => item !== event)
      : [...watch.events, event];
    if (events.length === 0) return;
    patchWatch({ events });
  }

  function setScheduleEnabled(enabled: boolean) {
    if (enabled) {
      job = {
        ...job,
        watch: job.watch ? { ...job.watch, enabled: false } : job.watch,
        schedule: job.schedule
          ? { ...job.schedule, enabled: true }
          : defaultSchedule(),
      };
    } else if (job.schedule) {
      job = { ...job, schedule: { ...job.schedule, enabled: false } };
    } else {
      job = { ...job, schedule: null };
    }
  }

  function setRuleKind(kind: ScheduleRule["kind"]) {
    switch (kind) {
      case "interval":
        patchRule({ kind: "interval", minutes: 60 });
        break;
      case "daily":
        patchRule({ kind: "daily", hour: 9, minute: 0 });
        break;
      case "weekly":
        patchRule({ kind: "weekly", days: [1, 2, 3, 4, 5], hour: 9, minute: 0 });
        break;
    }
  }

  function toggleWeekday(day: number) {
    const rule = job.schedule?.rule;
    if (!rule || rule.kind !== "weekly") return;
    const selected = rule.days.includes(day);
    const days = selected
      ? rule.days.filter((d) => d !== day)
      : [...rule.days, day].sort((a, b) => a - b);
    if (days.length === 0) return;
    patchRule({ ...rule, days });
  }

  function selectedWeekdayLabels(days: number[]): string {
    return WEEKDAYS.filter((d) => days.includes(d.value))
      .map((d) => d.fullLabel)
      .join(", ");
  }

  function toggleNotifyEvent(event: NotifyEvent) {
    const events = job.notify.events;
    patchNotify({
      events: events.includes(event)
        ? events.filter((e) => e !== event)
        : [...events, event],
    });
  }

  function addChannel(type: NotifyChannel["type"]) {
    let channel: NotifyChannel;
    switch (type) {
      case "discord":
        channel = { type: "discord", webhookUrl: "https://discord.com/api/webhooks/" };
        break;
      case "telegram":
        channel = { type: "telegram", botToken: "", chatId: "", rememberToken: false };
        break;
      case "generic":
        channel = { type: "generic", url: "https://", headers: {} };
        break;
    }
    patchNotify({ channels: [...job.notify.channels, channel] });
  }

  function removeChannel(index: number) {
    patchNotify({ channels: job.notify.channels.filter((_, i) => i !== index) });
  }

  function updateChannel(index: number, channel: NotifyChannel) {
    const channels = [...job.notify.channels];
    channels[index] = channel;
    patchNotify({ channels });
  }

  function headersToText(headers: Record<string, string>): string {
    return Object.entries(headers)
      .map(([k, v]) => `${k}: ${v}`)
      .join("\n");
  }

  function textToHeaders(text: string): Record<string, string> {
    const headers: Record<string, string> = {};
    for (const line of text.split("\n")) {
      const idx = line.indexOf(":");
      if (idx <= 0) continue;
      const key = line.slice(0, idx).trim();
      const value = line.slice(idx + 1).trim();
      if (key) headers[key] = value;
    }
    return headers;
  }
</script>

<form
  id="job-editor-form"
  class="editor"
  class:embedded
  onsubmit={(e) => {
    e.preventDefault();
    onsave?.();
  }}
>
  <nav class="editor-nav" class:collapsed={navCollapsed} aria-label="Seções do fluxo">
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

    {#each NAV_ITEMS as item}
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
          {#if item.id === "filtros" && filterCount > 0}
            <span class="badge">{filterCount}</span>
          {/if}
        {:else if item.id === "filtros" && filterCount > 0}
          <span class="nav-dot" aria-label="{filterCount} filtros ativos"></span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="editor-content">
    {#if activePanel === "geral"}
      <div class="panel-header">
        <h3>Geral</h3>
        <p>Nome, pastas e operação básica.</p>
      </div>

      <details class="collapsible" open>
        <summary>Identificação</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Nome</span>
            <input class="field-input" bind:value={job.name} maxlength="120" required />
          </label>

          <label class="toggle">
            <input type="checkbox" bind:checked={job.enabled} />
            <span>Fluxo ativo</span>
          </label>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Pastas</summary>
        <div class="collapsible-body">
          <div class="path-row">
            <label class="field">
              <span class="field-label">Origem</span>
              <input
                class="field-input"
                bind:value={job.sourcePath}
                placeholder="C:\pasta\origem"
                required
              />
            </label>
            <button type="button" class="btn" onclick={() => pickFolder("sourcePath")}>
              <FolderOpen size={14} />
              Escolher
            </button>
          </div>

          <div class="path-row">
            <label class="field">
              <span class="field-label">Destino</span>
              <input
                class="field-input"
                bind:value={job.targetPath}
                placeholder="D:\pasta\destino"
                required
              />
            </label>
            <button type="button" class="btn" onclick={() => pickFolder("targetPath")}>
              <FolderOpen size={14} />
              Escolher
            </button>
          </div>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Operação</summary>
        <div class="collapsible-body">
          <div class="subsection">
            <span class="field-label">Modo</span>
            <div class="option-grid">
              <button
                type="button"
                class="option-btn"
                class:selected={job.mode === "copy"}
                aria-pressed={job.mode === "copy"}
                onclick={() => setMode("copy")}
              >
                <Copy size={16} weight={job.mode === "copy" ? "fill" : "regular"} />
                Cópia
              </button>
              <button
                type="button"
                class="option-btn"
                class:selected={job.mode === "move"}
                aria-pressed={job.mode === "move"}
                onclick={() => setMode("move")}
              >
                <ArrowsLeftRight
                  size={16}
                  weight={job.mode === "move" ? "fill" : "regular"}
                />
                Mover
              </button>
            </div>
          </div>

          <label class="field">
            <span class="field-label">Conflito</span>
            <select
              class="field-input"
              value={job.conflict}
              onchange={(e) => setConflict(e.currentTarget.value as ConflictStrategy)}
            >
              <option value="skip">Ignorar existentes</option>
              <option value="overwrite">Sobrescrever</option>
              <option value="rename">Renomear</option>
            </select>
          </label>
        </div>
      </details>
    {:else if activePanel === "filtros"}
      <div class="panel-header">
        <h3>Filtros</h3>
        <p>
          Escopo de arquivos processados.
          {#if filterCount > 0}
            <span class="badge">{filterCount} ativo{filterCount === 1 ? "" : "s"}</span>
          {/if}
        </p>
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
                    checked={job.filters.excludePresetIds.includes(preset.id)}
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
              placeholder="*.tmp&#10;Thumbs.db"
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
              value={job.filters.nameRegex ?? ""}
              oninput={(e) =>
                patchFilters({ nameRegex: e.currentTarget.value || null })}
              placeholder="^backup_.*"
            />
          </label>

          <label class="field">
            <span class="field-label">Regex do caminho</span>
            <input
              class="field-input"
              value={job.filters.pathRegex ?? ""}
              oninput={(e) =>
                patchFilters({ pathRegex: e.currentTarget.value || null })}
              placeholder="/projetos/.*/src/"
            />
          </label>

          <div class="field-row">
            <label class="field">
              <span class="field-label">Tamanho mínimo (bytes)</span>
              <input
                class="field-input"
                type="number"
                min="0"
                value={job.filters.minSizeBytes ?? ""}
                oninput={(e) => {
                  const v = e.currentTarget.value;
                  patchFilters({ minSizeBytes: v === "" ? null : Number(v) });
                }}
              />
            </label>
            <label class="field">
              <span class="field-label">Tamanho máximo (bytes)</span>
              <input
                class="field-input"
                type="number"
                min="0"
                value={job.filters.maxSizeBytes ?? ""}
                oninput={(e) => {
                  const v = e.currentTarget.value;
                  patchFilters({ maxSizeBytes: v === "" ? null : Number(v) });
                }}
              />
            </label>
          </div>

          <label class="field">
            <span class="field-label">Profundidade máxima</span>
            <input
              class="field-input"
              type="number"
              min="0"
              value={job.filters.maxDepth ?? ""}
              oninput={(e) => {
                const v = e.currentTarget.value;
                patchFilters({ maxDepth: v === "" ? null : Number(v) });
              }}
            />
          </label>
        </div>
      </details>

      <details class="collapsible">
        <summary>Datas</summary>
        <div class="collapsible-body">
          <div class="field-row">
            <label class="field">
              <span class="field-label">Modificado após</span>
              <input
                class="field-input"
                type="datetime-local"
                value={isoToLocal(job.filters.modifiedAfter)}
                oninput={(e) =>
                  patchFilters({ modifiedAfter: localToIso(e.currentTarget.value) })}
              />
            </label>
            <label class="field">
              <span class="field-label">Modificado antes</span>
              <input
                class="field-input"
                type="datetime-local"
                value={isoToLocal(job.filters.modifiedBefore)}
                oninput={(e) =>
                  patchFilters({ modifiedBefore: localToIso(e.currentTarget.value) })}
              />
            </label>
          </div>

          <div class="field-row">
            <label class="field">
              <span class="field-label">Criado após</span>
              <input
                class="field-input"
                type="datetime-local"
                value={isoToLocal(job.filters.createdAfter)}
                oninput={(e) =>
                  patchFilters({ createdAfter: localToIso(e.currentTarget.value) })}
              />
            </label>
            <label class="field">
              <span class="field-label">Criado antes</span>
              <input
                class="field-input"
                type="datetime-local"
                value={isoToLocal(job.filters.createdBefore)}
                oninput={(e) =>
                  patchFilters({ createdBefore: localToIso(e.currentTarget.value) })}
              />
            </label>
          </div>

          <label class="field">
            <span class="field-label">Mais antigo que (dias)</span>
            <input
              class="field-input"
              type="number"
              min="0"
              value={job.filters.olderThanDays ?? ""}
              oninput={(e) => {
                const v = e.currentTarget.value;
                patchFilters({ olderThanDays: v === "" ? null : Number(v) });
              }}
            />
          </label>
        </div>
      </details>

      <details class="collapsible">
        <summary>Conteúdo</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Contém texto</span>
            <input
              class="field-input"
              value={job.filters.contentContains ?? ""}
              oninput={(e) =>
                patchFilters({ contentContains: e.currentTarget.value || null })}
              placeholder="ERROR"
            />
          </label>

          <label class="field">
            <span class="field-label">Extensões para busca de conteúdo</span>
            <input
              class="field-input"
              value={contentExtensionsText}
              oninput={(e) => handleContentExtensionsInput(e.currentTarget.value)}
              placeholder=".txt, .log, .json"
            />
          </label>
        </div>
      </details>

      <details class="collapsible">
        <summary>Comportamento</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.filters.recursive}
              onchange={(e) => patchFilters({ recursive: e.currentTarget.checked })}
            />
            <span>Incluir subpastas</span>
          </label>

          <label class="toggle">
            <input
              type="checkbox"
              checked={job.filters.includeHidden}
              onchange={(e) => patchFilters({ includeHidden: e.currentTarget.checked })}
            />
            <span>Incluir ocultos</span>
          </label>

          <label class="toggle">
            <input
              type="checkbox"
              checked={job.filters.skipEmptyFiles}
              onchange={(e) => patchFilters({ skipEmptyFiles: e.currentTarget.checked })}
            />
            <span>Ignorar arquivos vazios</span>
          </label>

          <label class="field">
            <span class="field-label">Links simbólicos</span>
            <select
              class="field-input"
              value={job.filters.symlinkMode}
              onchange={(e) =>
                patchFilters({ symlinkMode: e.currentTarget.value as SymlinkMode })}
            >
              <option value="follow">Seguir destino</option>
              <option value="copyLink">Copiar link</option>
              <option value="skip">Ignorar</option>
            </select>
          </label>
        </div>
      </details>
    {:else if activePanel === "automacao"}
      <div class="panel-header">
        <h3>Automação</h3>
        <p>Agendamento periódico. Mutuamente exclusivo com monitoramento.</p>
      </div>

      <details class="collapsible" open>
        <summary>Agendamento</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.schedule?.enabled ?? false}
              onchange={(e) => setScheduleEnabled(e.currentTarget.checked)}
            />
            <span>Agendamento ativo</span>
          </label>

          {#if job.schedule?.enabled}
            <p class="field-hint">Incompatível com monitoramento ativo — um desativa o outro ao salvar.</p>
            <label class="field">
              <span class="field-label">Fuso horário</span>
              <select
                class="field-input"
                value={job.schedule.timezone}
                onchange={(e) => patchSchedule({ timezone: e.currentTarget.value })}
              >
                <option value="local">Local do sistema</option>
                <option value="UTC">UTC</option>
                <option value="America/Sao_Paulo">America/Sao_Paulo</option>
                <option value="Europe/Lisbon">Europe/Lisbon</option>
              </select>
            </label>

            <label class="field">
              <span class="field-label">Tipo de regra</span>
              <select
                class="field-input"
                value={job.schedule.rule.kind}
                onchange={(e) =>
                  setRuleKind(e.currentTarget.value as ScheduleRule["kind"])}
              >
                {#each RULE_KIND_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </label>

            {#if job.schedule.rule.kind === "weekly"}
              <p class="field-hint">
                Escolha em quais dias o fluxo deve rodar automaticamente, no horário definido abaixo.
              </p>
            {/if}

            {#if job.schedule.rule.kind === "interval"}
              <label class="field">
                <span class="field-label">Intervalo (minutos)</span>
                <input
                  class="field-input"
                  type="number"
                  min="1"
                  value={job.schedule.rule.minutes}
                  oninput={(e) =>
                    patchRule({
                      kind: "interval",
                      minutes: Math.max(1, Number(e.currentTarget.value) || 1),
                    })}
                />
              </label>
            {:else if job.schedule.rule.kind === "daily"}
              <div class="field-row">
                <label class="field">
                  <span class="field-label">Hora</span>
                  <input
                    class="field-input"
                    type="number"
                    min="0"
                    max="23"
                    value={job.schedule.rule.hour}
                    oninput={(e) =>
                      patchRule({
                        ...job.schedule!.rule,
                        kind: "daily",
                        hour: Number(e.currentTarget.value),
                      } as ScheduleRule)}
                  />
                </label>
                <label class="field">
                  <span class="field-label">Minuto</span>
                  <input
                    class="field-input"
                    type="number"
                    min="0"
                    max="59"
                    value={job.schedule.rule.minute}
                    oninput={(e) =>
                      patchRule({
                        ...job.schedule!.rule,
                        kind: "daily",
                        minute: Number(e.currentTarget.value),
                      } as ScheduleRule)}
                  />
                </label>
              </div>
            {:else if job.schedule.rule.kind === "weekly"}
              <fieldset class="preset-group">
                <legend class="field-label">Dias selecionados</legend>
                <div class="weekday-grid" role="group" aria-label="Dias da semana">
                  {#each WEEKDAYS as day}
                    <button
                      type="button"
                      class="weekday-btn"
                      class:selected={job.schedule.rule.days.includes(day.value)}
                      title={day.fullLabel}
                      aria-pressed={job.schedule.rule.days.includes(day.value)}
                      onclick={() => toggleWeekday(day.value)}
                    >
                      {day.label}
                    </button>
                  {/each}
                </div>
                <p class="weekday-summary muted">
                  {selectedWeekdayLabels(job.schedule.rule.days)}
                </p>
              </fieldset>
              <div class="field-row">
                <label class="field">
                  <span class="field-label">Hora</span>
                  <input
                    class="field-input"
                    type="number"
                    min="0"
                    max="23"
                    value={job.schedule.rule.hour}
                    oninput={(e) =>
                      patchRule({
                        ...job.schedule!.rule,
                        kind: "weekly",
                        hour: Number(e.currentTarget.value),
                      } as ScheduleRule)}
                  />
                </label>
                <label class="field">
                  <span class="field-label">Minuto</span>
                  <input
                    class="field-input"
                    type="number"
                    min="0"
                    max="59"
                    value={job.schedule.rule.minute}
                    oninput={(e) =>
                      patchRule({
                        ...job.schedule!.rule,
                        kind: "weekly",
                        minute: Number(e.currentTarget.value),
                      } as ScheduleRule)}
                  />
                </label>
              </div>
            {/if}
          {/if}
        </div>
      </details>
    {:else if activePanel === "monitoramento"}
      <div class="panel-header">
        <h3>Monitoramento</h3>
        <p>Dispara o fluxo quando a pasta de origem muda. Exclusivo com agendamento.</p>
      </div>

      <details class="collapsible" open>
        <summary>Watch na origem</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.watch?.enabled ?? false}
              onchange={(e) => setWatchEnabled(e.currentTarget.checked)}
            />
            <span>Monitoramento ativo</span>
          </label>

          {#if job.watch?.enabled}
            <p class="field-hint">
              Monitora <code>{job.sourcePath || "—"}</code>. Profundidade segue o filtro
              &quot;Incluir subpastas&quot;.
            </p>

            <label class="field">
              <span class="field-label">Estabilização (segundos)</span>
              <input
                class="field-input"
                type="number"
                min="1"
                max="60"
                value={job.watch.settleSeconds}
                oninput={(e) =>
                  patchWatch({
                    settleSeconds: Math.min(
                      60,
                      Math.max(1, Number(e.currentTarget.value) || 2),
                    ),
                  })}
              />
            </label>

            <label class="field">
              <span class="field-label">Método de detecção</span>
              <select
                class="field-input"
                value={job.watch.detection.kind}
                onchange={(e) =>
                  setDetectionKind(e.currentTarget.value as WatchDetectionMode["kind"])}
              >
                {#each DETECTION_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </label>
            <p class="field-hint">
              {DETECTION_OPTIONS.find((o) => o.value === job.watch?.detection.kind)?.hint}
            </p>

            {#if job.watch.detection.kind === "polling"}
              <label class="field">
                <span class="field-label">Intervalo de polling (segundos)</span>
                <input
                  class="field-input"
                  type="number"
                  min="5"
                  max="3600"
                  value={job.watch.detection.intervalSecs}
                  oninput={(e) =>
                    patchWatch({
                      detection: {
                        kind: "polling",
                        intervalSecs: Math.max(5, Number(e.currentTarget.value) || 30),
                      },
                    })}
                />
              </label>
            {:else if job.watch.detection.kind === "hybrid"}
              <label class="field">
                <span class="field-label">Polling de backup (segundos)</span>
                <input
                  class="field-input"
                  type="number"
                  min="5"
                  max="3600"
                  value={job.watch.detection.pollIntervalSecs}
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
              <legend class="field-label">Eventos que disparam o fluxo</legend>
              <div class="checkbox-grid">
                {#each WATCH_EVENTS as event}
                  <label class="toggle">
                    <input
                      type="checkbox"
                      checked={job.watch.events.includes(event.id)}
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
    {:else if activePanel === "seguranca"}
      <div class="panel-header">
        <h3>Segurança</h3>
        <p>Sincronização, verificação e criptografia.</p>
      </div>

      <details class="collapsible" open>
        <summary>Sincronização</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.options.smartSync}
              onchange={(e) => patchOptions({ smartSync: e.currentTarget.checked })}
            />
            <span>Smart sync (copiar apenas alterados)</span>
          </label>

          <label class="toggle">
            <input
              type="checkbox"
              checked={job.options.strictHashSync}
              onchange={(e) => patchOptions({ strictHashSync: e.currentTarget.checked })}
            />
            <span>Hash estrito na comparação</span>
          </label>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>Integridade</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.options.verifyAfterCopy}
              onchange={(e) => patchOptions({ verifyAfterCopy: e.currentTarget.checked })}
            />
            <span>Verificar após cópia</span>
          </label>

          <label class="toggle">
            <input
              type="checkbox"
              checked={job.options.stopOnIntegrityError}
              onchange={(e) =>
                patchOptions({ stopOnIntegrityError: e.currentTarget.checked })}
            />
            <span>Parar em erro de integridade</span>
          </label>
        </div>
      </details>

      <details class="collapsible">
        <summary>Criptografia do pacote</summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.options.encryptOutput}
              onchange={(e) => patchOptions({ encryptOutput: e.currentTarget.checked })}
            />
            <span>Criptografar saída</span>
          </label>

          {#if job.options.encryptOutput}
            <label class="field">
              <span class="field-label">Senha</span>
              <input
                class="field-input"
                type="password"
                value={job.options.encryptPassword ?? ""}
                oninput={(e) =>
                  patchOptions({ encryptPassword: e.currentTarget.value || null })}
                autocomplete="new-password"
              />
            </label>

            <label class="toggle">
              <input
                type="checkbox"
                checked={job.options.rememberEncryptPassword}
                onchange={(e) =>
                  patchOptions({ rememberEncryptPassword: e.currentTarget.checked })}
              />
              <span>Lembrar senha</span>
            </label>

            <label class="field">
              <span class="field-label">Nome do pacote</span>
              <input
                class="field-input"
                value={job.options.packFilename ?? ""}
                oninput={(e) =>
                  patchOptions({ packFilename: e.currentTarget.value || null })}
                placeholder="backup.zip.enc"
              />
            </label>

            <label class="toggle">
              <input
                type="checkbox"
                checked={job.options.removeFilesAfterPack}
                onchange={(e) =>
                  patchOptions({ removeFilesAfterPack: e.currentTarget.checked })}
              />
              <span>Remover arquivos após empacotar</span>
            </label>
          {/if}
        </div>
      </details>
    {:else if activePanel === "avancado"}
      <div class="panel-header">
        <h3>Avançado</h3>
        <p>Scripts e notificações.</p>
      </div>

      <details class="collapsible" open>
        <summary>Scripts</summary>
        <div class="collapsible-body">
          <label class="field">
            <span class="field-label">Script pré-execução</span>
            <input
              class="field-input"
              value={job.scripts.preScript ?? ""}
              oninput={(e) =>
                patchScripts({ preScript: e.currentTarget.value || null })}
              placeholder="pre_backup.ps1"
            />
          </label>

          <label class="field">
            <span class="field-label">Script pós-execução</span>
            <input
              class="field-input"
              value={job.scripts.postScript ?? ""}
              oninput={(e) =>
                patchScripts({ postScript: e.currentTarget.value || null })}
              placeholder="post_backup.ps1"
            />
          </label>

          <label class="field">
            <span class="field-label">Timeout (segundos)</span>
            <input
              class="field-input"
              type="number"
              min="5"
              max="600"
              value={job.scripts.timeoutSecs}
              oninput={(e) =>
                patchScripts({
                  timeoutSecs: Math.min(600, Math.max(5, Number(e.currentTarget.value) || 60)),
                })}
            />
          </label>
        </div>
      </details>

      <details class="collapsible" open>
        <summary>
          <Bell size={14} weight="duotone" />
          Notificações
        </summary>
        <div class="collapsible-body">
          <label class="toggle">
            <input
              type="checkbox"
              checked={job.notify.enabled}
              onchange={(e) => patchNotify({ enabled: e.currentTarget.checked })}
            />
            <span>Notificações ativas</span>
          </label>

          {#if job.notify.enabled}
            <fieldset class="preset-group">
              <legend class="field-label">Eventos</legend>
              <div class="checkbox-grid">
                {#each NOTIFY_EVENTS as ev}
                  <label class="toggle">
                    <input
                      type="checkbox"
                      checked={job.notify.events.includes(ev.id)}
                      onchange={() => toggleNotifyEvent(ev.id)}
                    />
                    <span>{ev.label}</span>
                  </label>
                {/each}
              </div>
            </fieldset>

            <div class="channel-list">
              {#each job.notify.channels as channel, i}
                <div class="channel-card panel">
                  <div class="channel-head">
                    <span class="badge">{channel.type}</span>
                    <button
                      type="button"
                      class="btn ghost danger"
                      aria-label="Remover canal"
                      onclick={() => removeChannel(i)}
                    >
                      <Trash size={14} />
                    </button>
                  </div>

                  {#if channel.type === "discord"}
                    <label class="field">
                      <span class="field-label">Webhook URL</span>
                      <input
                        class="field-input"
                        value={channel.webhookUrl}
                        oninput={(e) =>
                          updateChannel(i, {
                            type: "discord",
                            webhookUrl: e.currentTarget.value,
                          })}
                      />
                    </label>
                  {:else if channel.type === "telegram"}
                    <label class="field">
                      <span class="field-label">Bot token</span>
                      <input
                        class="field-input"
                        value={channel.botToken}
                        oninput={(e) =>
                          updateChannel(i, {
                            ...channel,
                            botToken: e.currentTarget.value,
                          })}
                      />
                    </label>
                    <label class="field">
                      <span class="field-label">Chat ID</span>
                      <input
                        class="field-input"
                        value={channel.chatId}
                        oninput={(e) =>
                          updateChannel(i, {
                            ...channel,
                            chatId: e.currentTarget.value,
                          })}
                      />
                    </label>
                    <label class="toggle">
                      <input
                        type="checkbox"
                        checked={channel.rememberToken}
                        onchange={(e) =>
                          updateChannel(i, {
                            ...channel,
                            rememberToken: e.currentTarget.checked,
                          })}
                      />
                      <span>Lembrar token</span>
                    </label>
                  {:else if channel.type === "generic"}
                    <label class="field">
                      <span class="field-label">URL</span>
                      <input
                        class="field-input"
                        value={channel.url}
                        oninput={(e) =>
                          updateChannel(i, {
                            type: "generic",
                            url: e.currentTarget.value,
                            headers: channel.headers,
                          })}
                      />
                    </label>
                    <label class="field">
                      <span class="field-label">Headers (um por linha: Nome: valor)</span>
                      <textarea
                        class="field-input textarea"
                        rows="3"
                        value={headersToText(channel.headers)}
                        oninput={(e) =>
                          updateChannel(i, {
                            type: "generic",
                            url: channel.url,
                            headers: textToHeaders(e.currentTarget.value),
                          })}
                      ></textarea>
                    </label>
                  {/if}
                </div>
              {/each}
            </div>

            <div class="channel-actions">
              <button type="button" class="btn" onclick={() => addChannel("discord")}>
                <Plus size={14} />
                Discord
              </button>
              <button type="button" class="btn" onclick={() => addChannel("telegram")}>
                <Plus size={14} />
                Telegram
              </button>
              <button type="button" class="btn" onclick={() => addChannel("generic")}>
                <Plus size={14} />
                Genérico
              </button>
            </div>
          {/if}
        </div>
      </details>
    {/if}
  </div>
</form>

<style>
  .editor {
    display: flex;
    flex-direction: row;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .editor.embedded {
    flex: 1;
    min-height: 0;
  }

  .editor-nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    width: 168px;
    flex-shrink: 0;
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

  .editor-nav.collapsed .nav-toggle {
    align-self: center;
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
    transition: background 120ms ease, color 120ms ease;
  }

  .nav-item:hover {
    background: var(--surface);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .editor-nav.collapsed .nav-item {
    justify-content: center;
    padding-inline: var(--space-2);
  }

  .nav-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-dot {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
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
    align-items: stretch;
  }

  .panel-header {
    flex-shrink: 0;
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: 650;
    color: var(--text-primary);
  }

  .panel-header p {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .collapsible {
    flex-shrink: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .collapsible > summary {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    list-style: none;
    user-select: none;
  }

  .collapsible > summary::-webkit-details-marker {
    display: none;
  }

  .collapsible > summary::before {
    content: "▸";
    color: var(--text-muted);
    transition: transform 120ms ease;
  }

  .collapsible[open] > summary::before {
    transform: rotate(90deg);
  }

  .collapsible-body {
    display: grid;
    gap: var(--space-4);
    padding: 0 var(--space-4) var(--space-4);
    border-top: 1px solid var(--border);
    padding-top: var(--space-4);
  }

  .subsection {
    display: grid;
    gap: var(--space-2);
  }

  .path-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-2);
    align-items: end;
  }

  .field-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .option-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-2);
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
    color: var(--text-secondary);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease, color 120ms ease;
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
    color: var(--text-secondary);
    cursor: pointer;
  }

  .preset-group {
    border: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--space-2);
  }

  .checkbox-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: var(--space-2);
  }

  .textarea {
    resize: vertical;
    min-height: 4.5rem;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .weekday-grid {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .weekday-btn {
    min-width: 2.5rem;
    padding: 0.35rem 0.5rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-muted);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: 600;
    cursor: pointer;
  }

  .weekday-btn.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
  }

  .field-hint {
    margin: calc(-1 * var(--space-2)) 0 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .weekday-summary {
    margin: var(--space-2) 0 0;
    font-size: var(--text-xs);
  }

  .channel-list {
    display: grid;
    gap: var(--space-3);
  }

  .channel-card {
    padding: var(--space-3);
    display: grid;
    gap: var(--space-3);
  }

  .channel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .channel-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  @media (max-width: 640px) {
    .editor {
      flex-direction: column;
    }

    .editor-nav {
      flex-direction: row;
      width: 100%;
      overflow-x: auto;
      border-right: none;
      border-bottom: 1px solid var(--border);
    }

    .editor-nav.collapsed {
      width: 100%;
    }

    .nav-toggle {
      display: none;
    }

    .field-row {
      grid-template-columns: 1fr;
    }
  }
</style>
