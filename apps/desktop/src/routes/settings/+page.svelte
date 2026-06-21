<script lang="ts">
  import { onMount } from "svelte";
  import {
    Bell,
    CheckCircle,
    ClockCounterClockwise,
    Coffee,
    Desktop,
    DesktopTower,
    FloppyDisk,
    GearSix,
    Info,
    LockKey,
    Moon,
    Palette,
    Plus,
    ShieldCheck,
    SlidersHorizontal,
    Sun,
    Trash,
  } from "phosphor-svelte";
  import {
    ACCENT_PRESETS,
    ADMIN_AGENT_MODE_OPTIONS,
    CHANGELOG_ENTRIES,
    INTERFACE_FONT_OPTIONS,
    LANGUAGE_OPTIONS,
    PROCESS_PRIORITY_OPTIONS,
    THEME_OPTIONS,
    hashPin,
    type AppSettings,
    type ChangelogFilter,
    type ThemeMode,
  } from "$lib/contracts/settings";
  import NumberField from "$lib/components/settings/NumberField.svelte";
  import SectionTitle from "$lib/components/settings/SectionTitle.svelte";
  import ToggleField from "$lib/components/settings/ToggleField.svelte";
  import {
    clearAdminAgentSecret,
    fetchHealth,
    fetchSettings,
    generateAdminAgentSecret,
    saveSettings,
    setAdminAgentSecret,
  } from "$lib/core/ipc/client";
  import { applyAppearance } from "$lib/core/stores/theme";
  import type { HealthStatus } from "$lib/contracts/settings";
  import type { Component } from "svelte";

  type SettingsTab =
    | "general"
    | "performance"
    | "security"
    | "notifications"
    | "admin"
    | "maintenance"
    | "support"
    | "about"
    | "changelog";

  const tabs: { id: SettingsTab; label: string; icon: Component }[] = [
    { id: "general", label: "Geral", icon: GearSix },
    { id: "performance", label: "Performance", icon: SlidersHorizontal },
    { id: "security", label: "Segurança", icon: LockKey },
    { id: "notifications", label: "Notificações", icon: Bell },
    { id: "admin", label: "Admin", icon: DesktopTower },
    { id: "maintenance", label: "Manutenção", icon: ClockCounterClockwise },
    { id: "support", label: "Suporte", icon: Coffee },
    { id: "about", label: "Sobre", icon: Info },
    { id: "changelog", label: "Changelog", icon: FloppyDisk },
  ];

  let settings = $state<AppSettings | null>(null);
  let health = $state<HealthStatus | null>(null);
  let activeTab = $state<SettingsTab>("general");
  let message = $state<string | null>(null);
  let saving = $state(false);
  let newPin = $state("");
  let confirmPin = $state("");
  let adminSecretInput = $state("");
  let generatedAdminSecret = $state<string | null>(null);
  let changelogSearch = $state("");
  let changelogFilter = $state<ChangelogFilter>("all");
  let expandedVersions = $state<string[]>([CHANGELOG_ENTRIES[0]?.version ?? ""]);

  const filteredChangelog = $derived(
    CHANGELOG_ENTRIES.map((entry) => ({
      ...entry,
      groups: entry.groups.filter((group) => {
        const matchesType = changelogFilter === "all" || group.type === changelogFilter;
        const text = `${entry.version} ${entry.date} ${group.label} ${group.items.join(" ")}`.toLowerCase();
        const matchesSearch = !changelogSearch.trim() || text.includes(changelogSearch.trim().toLowerCase());
        return matchesType && matchesSearch;
      }),
    })).filter((entry) => entry.groups.length > 0),
  );

  onMount(async () => {
    try {
      settings = await fetchSettings();
      health = await fetchHealth();
      applyAppearance(settings);
    } catch (e) {
      message = e instanceof Error ? e.message : "Erro ao carregar configuracoes";
    }
  });

  async function handleSave() {
    if (!settings) return;
    saving = true;
    message = null;
    try {
      let next: AppSettings = {
        ...settings,
        bandwidthLimitMbps: settings.performance.globalBandwidthLimitMbps,
        logRetentionDays: settings.maintenance.logRetentionDays,
      };

      if (!next.security.pinEnabled) {
        next.security = { ...next.security, accessPinHash: null };
      } else if (newPin || confirmPin) {
        if (newPin.length < 4) throw new Error("O PIN precisa ter pelo menos 4 caracteres.");
        if (newPin !== confirmPin) throw new Error("A confirmação do PIN não confere.");
        next.security = { ...next.security, accessPinHash: await hashPin(newPin) };
      }

      settings = await saveSettings(next);
      newPin = "";
      confirmPin = "";
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

  function setInterfaceFont(interfaceFont: string) {
    if (!settings) return;
    settings = { ...settings, interfaceFont };
    applyAppearance(settings);
  }

  function themeIcon(theme: ThemeMode) {
    if (theme === "light") return Sun;
    if (theme === "dark") return Moon;
    return Desktop;
  }

  function addWebhook() {
    if (!settings) return;
    settings.notifications.webhooks = [
      ...settings.notifications.webhooks,
      { name: "Webhook", url: "https://example.com/webhook", enabled: true, secretConfigured: false },
    ];
  }

  function removeWebhook(index: number) {
    if (!settings) return;
    settings.notifications.webhooks = settings.notifications.webhooks.filter((_, i) => i !== index);
  }

  async function saveAdminSecret() {
    if (!settings) return;
    if (adminSecretInput.trim().length < 32) {
      message = "O segredo do agent precisa ter pelo menos 32 caracteres.";
      return;
    }

    saving = true;
    message = null;
    try {
      settings = await setAdminAgentSecret(adminSecretInput);
      adminSecretInput = "";
      generatedAdminSecret = null;
      message = "Segredo do agent salvo no cofre do sistema.";
    } catch (e) {
      message = e instanceof Error ? e.message : "Erro ao salvar segredo do agent";
    } finally {
      saving = false;
    }
  }

  async function generateAdminSecret() {
    saving = true;
    message = null;
    try {
      const generated = await generateAdminAgentSecret();
      settings = generated.settings;
      generatedAdminSecret = generated.secret;
      adminSecretInput = "";
      message = "Segredo gerado e salvo. Copie agora se precisar cadastrar no servidor.";
    } catch (e) {
      message = e instanceof Error ? e.message : "Erro ao gerar segredo do agent";
    } finally {
      saving = false;
    }
  }

  async function clearAdminSecret() {
    if (!window.confirm("Remover o segredo do agent salvo neste computador?")) {
      return;
    }

    saving = true;
    message = null;
    try {
      settings = await clearAdminAgentSecret();
      adminSecretInput = "";
      generatedAdminSecret = null;
      message = "Segredo do agent removido do cofre.";
    } catch (e) {
      message = e instanceof Error ? e.message : "Erro ao remover segredo do agent";
    } finally {
      saving = false;
    }
  }

  async function copyGeneratedAdminSecret() {
    if (!generatedAdminSecret) return;
    try {
      await navigator.clipboard.writeText(generatedAdminSecret);
      message = "Segredo copiado para a area de transferencia.";
    } catch {
      message = "Nao foi possivel copiar o segredo neste ambiente.";
    }
  }

  function toggleVersion(version: string) {
    expandedVersions = expandedVersions.includes(version)
      ? expandedVersions.filter((item) => item !== version)
      : [...expandedVersions, version];
  }
</script>

<section class="page settings-page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Configurações</h1>
      <p class="page-desc">Controle geral, segurança, logs, suporte e informações do projeto.</p>
    </div>
    <button class="btn primary" type="button" onclick={handleSave} disabled={!settings || saving}>
      <CheckCircle size={16} weight="bold" />
      {saving ? "Salvando…" : "Salvar"}
    </button>
  </header>

  {#if settings}
    <div class="settings-layout">
      <nav class="settings-nav panel" aria-label="Seções de configuração">
        {#each tabs as tab}
          {@const Icon = tab.icon}
          <button
            type="button"
            class:active={activeTab === tab.id}
            aria-current={activeTab === tab.id ? "page" : undefined}
            onclick={() => (activeTab = tab.id)}
          >
            <Icon size={16} weight={activeTab === tab.id ? "fill" : "regular"} />
            {tab.label}
          </button>
        {/each}
      </nav>

      <form
        class="settings-content"
        onsubmit={(event) => {
          event.preventDefault();
          void handleSave();
        }}
      >
        {#if activeTab === "general"}
          <section class="panel section">
            <SectionTitle icon={Palette} title="Geral" text="Tema, idioma, fonte e comportamento da janela." />

            <div class="subsection">
              <span class="field-label">Tema</span>
              <div class="choice-grid three">
                {#each THEME_OPTIONS as option}
                  {@const Icon = themeIcon(option.id)}
                  <button
                    type="button"
                    class="choice"
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
                    <span>{preset.label}</span>
                  </button>
                {/each}
              </div>
            </div>

            <div class="field-row">
              <label class="field">
                <span class="field-label">Idioma</span>
                <select class="field-input" bind:value={settings.language}>
                  {#each LANGUAGE_OPTIONS as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>

              <label class="field">
                <span class="field-label">Fonte da interface</span>
                <select
                  class="field-input"
                  value={settings.interfaceFont}
                  onchange={(event) => setInterfaceFont(event.currentTarget.value)}
                >
                  {#each INTERFACE_FONT_OPTIONS as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>
            </div>

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.autostart} label="Iniciar com o sistema" />
              <ToggleField bind:checked={settings.closeToTray} label="Fechar para a bandeja" />
              <ToggleField bind:checked={settings.startMinimized} label="Iniciar minimizado" />
            </div>
          </section>
        {:else if activeTab === "performance"}
          <section class="panel section">
            <SectionTitle icon={SlidersHorizontal} title="Performance" text="Limites para CPU, memória, fila e banda." />

            <div class="field-row three">
              <NumberField bind:value={settings.maxParallelFiles} label="Arquivos paralelos por job" min={1} max={64} />
              <NumberField bind:value={settings.performance.maxThreads} label="Threads máximas" min={1} max={128} />
              <NumberField bind:value={settings.performance.memoryLimitMb} label="Memória máxima (MB, 0 = livre)" min={0} />
            </div>

            <div class="field-row three">
              <label class="field">
                <span class="field-label">Prioridade do processo</span>
                <select class="field-input" bind:value={settings.performance.processPriority}>
                  {#each PROCESS_PRIORITY_OPTIONS as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>
              <NumberField bind:value={settings.performance.globalBandwidthLimitMbps} label="Banda global (Mbps, 0 = livre)" min={0} />
              <NumberField bind:value={settings.performance.queuePollIntervalMs} label="Intervalo da fila (ms)" min={100} />
            </div>

            <ToggleField bind:checked={settings.performance.pauseWhenOnBattery} label="Pausar tarefas pesadas na bateria" />
          </section>
        {:else if activeTab === "security"}
          <section class="panel section">
            <SectionTitle icon={ShieldCheck} title="Segurança" text="PIN, bloqueio automático e preparação para criptografia." />

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.security.pinEnabled} label="Exigir PIN de acesso" />
              <ToggleField bind:checked={settings.security.requirePinOnStartup} label="Pedir PIN ao iniciar" />
              <ToggleField bind:checked={settings.security.lockOnMinimize} label="Bloquear ao minimizar" />
              <ToggleField bind:checked={settings.security.lockOnTray} label="Bloquear ao ir para a bandeja" />
              <ToggleField bind:checked={settings.security.encryptionEnabled} label="Usar chave mestre de criptografia" />
              <ToggleField bind:checked={settings.security.maskSensitivePaths} label="Mascarar caminhos sensíveis nos logs" />
            </div>

            {#if settings.security.pinEnabled}
              <div class="field-row">
                <label class="field">
                  <span class="field-label">Novo PIN</span>
                  <input class="field-input" type="password" bind:value={newPin} autocomplete="new-password" />
                </label>
                <label class="field">
                  <span class="field-label">Confirmar PIN</span>
                  <input class="field-input" type="password" bind:value={confirmPin} autocomplete="new-password" />
                </label>
              </div>
              <p class="hint">
                O PIN é salvo como hash SHA-256. Para produção, a próxima camada deve trocar isso por KDF com salt.
              </p>
            {/if}

            <label class="field">
              <span class="field-label">Dica da chave mestre</span>
              <input class="field-input" bind:value={settings.security.masterKeyHint} placeholder="Ex.: chave do cofre local" />
            </label>
          </section>
        {:else if activeTab === "notifications"}
          <section class="panel section">
            <SectionTitle icon={Bell} title="Notificações" text="Desktop, webhooks, SMTP/e-mail e futuro painel admin." />

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.notifications.enabled} label="Ativar notificações" />
              <ToggleField bind:checked={settings.notifications.desktopEnabled} label="Notificações desktop" />
              <ToggleField bind:checked={settings.notifications.notifyOnSuccess} label="Avisar sucesso" />
              <ToggleField bind:checked={settings.notifications.notifyOnFailure} label="Avisar falhas" />
              <ToggleField bind:checked={settings.notifications.webhookEnabled} label="Enviar para webhooks" />
              <ToggleField bind:checked={settings.notifications.adminPanelEnabled} label="Preparar painel admin futuro" />
            </div>

            <div class="subsection">
              <div class="section-line">
                <h3>Webhooks</h3>
                <button class="btn" type="button" onclick={addWebhook}>
                  <Plus size={14} />
                  Adicionar
                </button>
              </div>
              {#each settings.notifications.webhooks as webhook, index}
                <div class="nested-card">
                  <div class="field-row">
                    <label class="field">
                      <span class="field-label">Nome</span>
                      <input class="field-input" bind:value={webhook.name} />
                    </label>
                    <label class="field">
                      <span class="field-label">URL</span>
                      <input class="field-input" bind:value={webhook.url} placeholder="https://..." />
                    </label>
                  </div>
                  <div class="nested-actions">
                    <ToggleField bind:checked={webhook.enabled} label="Ativo" />
                    <button class="btn ghost danger" type="button" onclick={() => removeWebhook(index)}>
                      <Trash size={14} />
                      Remover
                    </button>
                  </div>
                </div>
              {:else}
                <p class="muted">Nenhum webhook configurado.</p>
              {/each}
            </div>

            <div class="subsection">
              <h3>SMTP / e-mail</h3>
              <ToggleField bind:checked={settings.notifications.smtp.enabled} label="Ativar SMTP" />
              <div class="field-row three">
                <label class="field">
                  <span class="field-label">Host</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.host} placeholder="smtp.exemplo.com" />
                </label>
                <NumberField bind:value={settings.notifications.smtp.port} label="Porta" min={1} max={65535} />
                <label class="field">
                  <span class="field-label">Remetente</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.fromAddress} placeholder="autoflow@exemplo.com" />
                </label>
              </div>
              <div class="field-row">
                <label class="field">
                  <span class="field-label">Usuário</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.username} />
                </label>
                <ToggleField bind:checked={settings.notifications.smtp.useTls} label="Usar TLS" />
              </div>
            </div>
          </section>
        {:else if activeTab === "admin"}
          <section class="panel section">
            <SectionTitle icon={DesktopTower} title="Admin" text="Identidade da instância, servidor web e permissões do agent." />

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.admin.enabled} label="Ativar monitoramento admin" />
              <ToggleField bind:checked={settings.admin.allowRemoteCommands} label="Permitir comandos remotos" />
              <ToggleField bind:checked={settings.admin.allowBatchCommands} label="Permitir ações em lote" />
            </div>

            <div class="field-row">
              <label class="field">
                <span class="field-label">Modo</span>
                <select class="field-input" bind:value={settings.admin.mode}>
                  {#each ADMIN_AGENT_MODE_OPTIONS as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>
              <label class="field">
                <span class="field-label">Nome exibido no painel</span>
                <input class="field-input" bind:value={settings.admin.displayName} placeholder="Ex.: Financeiro 01" />
              </label>
            </div>

            <div class="field-row">
              <label class="field">
                <span class="field-label">ID da instância</span>
                <input class="field-input" value={settings.admin.instanceId ?? "gerado no próximo início"} readonly />
              </label>
              <label class="field">
                <span class="field-label">Servidor Admin Web</span>
                <input class="field-input" bind:value={settings.admin.serverUrl} placeholder="https://admin.autoflow.local" />
              </label>
            </div>

            <div class="field-row three">
              <NumberField bind:value={settings.admin.heartbeatIntervalSecs} label="Heartbeat (s)" min={10} max={3600} />
              <NumberField bind:value={settings.admin.inventoryIntervalSecs} label="Inventário (s)" min={60} max={86400} />
              <label class="field">
                <span class="field-label">Token de matrícula</span>
                <input
                  class="field-input"
                  value={settings.admin.enrollmentTokenConfigured ? "Configurado" : "Não configurado"}
                  readonly
                />
              </label>
            </div>

            <section class="subsection nested-card" aria-label="Segredo do agent admin">
              <div class="section-line">
                <h3>Segredo do agent</h3>
                <span class="status-pill" class:enabled={settings.admin.enrollmentTokenConfigured}>
                  {settings.admin.enrollmentTokenConfigured ? "No cofre" : "Nao configurado"}
                </span>
              </div>

              <label class="field">
                <span class="field-label">Inserir segredo recebido do servidor</span>
                <input
                  class="field-input"
                  type="password"
                  bind:value={adminSecretInput}
                  autocomplete="new-password"
                  placeholder="Cole aqui o segredo de matricula ou assinatura"
                />
              </label>

              <div class="secret-actions">
                <button class="btn primary" type="button" onclick={saveAdminSecret} disabled={saving || adminSecretInput.trim().length < 32}>
                  Salvar no cofre
                </button>
                <button class="btn" type="button" onclick={generateAdminSecret} disabled={saving}>
                  Gerar local
                </button>
                <button class="btn danger" type="button" onclick={clearAdminSecret} disabled={saving || !settings.admin.enrollmentTokenConfigured}>
                  Limpar
                </button>
              </div>

              {#if generatedAdminSecret}
                <div class="generated-secret" aria-label="Segredo gerado">
                  <code>{generatedAdminSecret}</code>
                  <button class="btn" type="button" onclick={copyGeneratedAdminSecret}>Copiar</button>
                </div>
              {/if}
            </section>

            <p class="hint">
              O modo web usa conexão de saída do agent para o servidor. Comandos destrutivos devem continuar auditados e confirmados.
            </p>
          </section>
        {:else if activeTab === "maintenance"}
          <section class="panel section">
            <SectionTitle icon={ClockCounterClockwise} title="Manutenção" text="Retenção, limpeza, otimização e backup do banco." />

            <div class="field-row three">
              <NumberField bind:value={settings.maintenance.logRetentionDays} label="Retenção de logs (dias)" min={0} />
              <NumberField bind:value={settings.maintenance.backupIntervalHours} label="Intervalo de backup (horas)" min={1} />
              <NumberField bind:value={settings.maintenance.backupRetentionCount} label="Backups mantidos" min={1} />
            </div>

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.maintenance.autoCompactDatabase} label="Compactar banco automaticamente" />
              <ToggleField bind:checked={settings.maintenance.optimizeAfterCleanup} label="Otimizar após limpeza" />
              <ToggleField bind:checked={settings.maintenance.backupEnabled} label="Ativar backup automático" />
            </div>

            <label class="field">
              <span class="field-label">Diretório de backup</span>
              <input class="field-input" bind:value={settings.maintenance.backupDirectory} placeholder="D:\Backups\AutoFlow" />
            </label>
          </section>
        {:else if activeTab === "support"}
          <section class="panel section">
            <SectionTitle icon={Coffee} title="Suporte e doações" text="Canais para projeto open source, Pix, depósito e Buy Me a Coffee." />

            <ToggleField bind:checked={settings.support.donationsEnabled} label="Mostrar seção de doações" />
            <div class="field-row">
              <label class="field">
                <span class="field-label">E-mail de suporte</span>
                <input class="field-input" bind:value={settings.support.supportEmail} />
              </label>
              <label class="field">
                <span class="field-label">Chave Pix</span>
                <input class="field-input" bind:value={settings.support.pixKey} />
              </label>
            </div>
            <label class="field">
              <span class="field-label">Dados para depósito</span>
              <textarea class="field-input textarea" rows="4" bind:value={settings.support.bankDepositInfo}></textarea>
            </label>
            <label class="field">
              <span class="field-label">Buy Me a Coffee</span>
              <input class="field-input" bind:value={settings.support.buyMeCoffeeUrl} placeholder="https://www.buymeacoffee.com/..." />
            </label>
          </section>
        {:else if activeTab === "about"}
          <section class="panel section">
            <SectionTitle icon={Info} title="Sobre" text="Descrição do projeto, versão e links do criador." />

            <div class="about-card">
              <strong>{settings.about.projectName}</strong>
              <span>Versão {health?.version ?? "0.2.0"}</span>
              <p>{settings.about.projectDescription}</p>
            </div>

            <div class="field-row">
              <label class="field">
                <span class="field-label">Nome do projeto</span>
                <input class="field-input" bind:value={settings.about.projectName} />
              </label>
              <label class="field">
                <span class="field-label">Criador</span>
                <input class="field-input" bind:value={settings.about.creatorName} />
              </label>
            </div>
            <label class="field">
              <span class="field-label">Descrição</span>
              <textarea class="field-input textarea" rows="3" bind:value={settings.about.projectDescription}></textarea>
            </label>
            <label class="field">
              <span class="field-label">Bio curta</span>
              <textarea class="field-input textarea" rows="3" bind:value={settings.about.creatorBio}></textarea>
            </label>
            <div class="field-row three">
              <label class="field">
                <span class="field-label">Site</span>
                <input class="field-input" bind:value={settings.about.websiteUrl} />
              </label>
              <label class="field">
                <span class="field-label">GitHub</span>
                <input class="field-input" bind:value={settings.about.githubUrl} />
              </label>
              <label class="field">
                <span class="field-label">LinkedIn</span>
                <input class="field-input" bind:value={settings.about.linkedinUrl} />
              </label>
            </div>
          </section>
        {:else}
          <section class="panel section">
            <SectionTitle icon={FloppyDisk} title="Changelog" text="Atualizações por versão, com busca e filtros." />

            <div class="field-row">
              <label class="field">
                <span class="field-label">Pesquisar</span>
                <input class="field-input" bind:value={changelogSearch} placeholder="feature, bug, versão..." />
              </label>
              <label class="field">
                <span class="field-label">Tipo</span>
                <select class="field-input" bind:value={changelogFilter}>
                  <option value="all">Todos</option>
                  <option value="feature">Novas features</option>
                  <option value="fix">Bug fix</option>
                  <option value="security">Segurança</option>
                  <option value="maintenance">Manutenção</option>
                </select>
              </label>
            </div>

            <div class="changelog-list">
              {#each filteredChangelog as entry}
                <article class="changelog-entry">
                  <button type="button" onclick={() => toggleVersion(entry.version)}>
                    <strong>{entry.version}</strong>
                    <span>{entry.date}</span>
                  </button>
                  {#if expandedVersions.includes(entry.version)}
                    <div class="changelog-body">
                      {#each entry.groups as group}
                        <section>
                          <h3>{group.label}</h3>
                          <ul>
                            {#each group.items as item}
                              <li>{item}</li>
                            {/each}
                          </ul>
                        </section>
                      {/each}
                    </div>
                  {/if}
                </article>
              {:else}
                <p class="muted">Nenhuma entrada corresponde ao filtro.</p>
              {/each}
            </div>
          </section>
        {/if}

        {#if message}
          <p class:success={message === "Configurações salvas."} class="feedback" aria-live="polite">{message}</p>
        {/if}
      </form>
    </div>
  {:else if message}
    <p class="banner error" role="alert">{message}</p>
  {:else}
    <p class="muted" aria-live="polite">Carregando configurações…</p>
  {/if}
</section>

<style>
  .settings-layout {
    display: grid;
    grid-template-columns: 14rem minmax(0, 1fr);
    gap: var(--space-4);
    align-items: start;
  }

  .settings-nav {
    position: sticky;
    top: var(--space-4);
    display: grid;
    gap: var(--space-1);
    padding: var(--space-3);
  }

  .settings-nav button {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    padding: 0.55rem 0.65rem;
    text-align: left;
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .settings-nav button:hover {
    background: var(--surface-muted);
    color: var(--text-primary);
  }

  .settings-nav button.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 650;
  }

  .settings-content,
  .section {
    display: grid;
    gap: var(--space-4);
  }

  .section-line h3,
  .subsection h3 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-primary);
  }

  .subsection {
    display: grid;
    gap: var(--space-3);
  }

  .section-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .choice-grid,
  .accent-grid,
  .field-row,
  .toggle-grid {
    display: grid;
    gap: var(--space-3);
  }

  .choice-grid.three,
  .field-row.three {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .field-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .toggle-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .accent-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .choice,
  .accent-swatch {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.55rem 0.65rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-muted);
    cursor: pointer;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .choice.selected,
  .accent-swatch.selected {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 650;
  }

  .accent-swatch.selected {
    border-color: var(--swatch);
    background: color-mix(in srgb, var(--swatch) 10%, var(--surface-muted));
    color: var(--text-primary);
  }

  .dot {
    width: 0.85rem;
    height: 0.85rem;
    border-radius: 999px;
    background: var(--swatch);
    flex-shrink: 0;
  }

  .textarea {
    resize: vertical;
  }

  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .nested-card,
  .about-card,
  .changelog-entry {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    padding: var(--space-3);
  }

  .nested-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: var(--space-3);
    gap: var(--space-3);
  }

  .secret-actions,
  .generated-secret {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    align-items: center;
  }

  .generated-secret {
    justify-content: space-between;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .generated-secret code {
    max-width: 100%;
    overflow-wrap: anywhere;
    color: var(--text-primary);
    font-size: var(--text-xs);
  }

  .status-pill {
    border-radius: 999px;
    padding: 0.2rem 0.55rem;
    background: var(--surface);
    color: var(--text-muted);
    font-size: var(--text-xs);
    font-weight: 650;
  }

  .status-pill.enabled {
    background: color-mix(in srgb, var(--success) 12%, transparent);
    color: var(--success);
  }

  .about-card {
    display: grid;
    gap: var(--space-1);
  }

  .about-card strong {
    font-size: var(--text-lg);
  }

  .about-card span,
  .about-card p {
    color: var(--text-muted);
    margin: 0;
    font-size: var(--text-sm);
  }

  .changelog-list {
    display: grid;
    gap: var(--space-3);
  }

  .changelog-entry {
    padding: 0;
    overflow: hidden;
  }

  .changelog-entry > button {
    width: 100%;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3);
    cursor: pointer;
  }

  .changelog-entry > button span {
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .changelog-body {
    display: grid;
    gap: var(--space-3);
    padding: 0 var(--space-3) var(--space-3);
  }

  .changelog-body ul {
    margin: var(--space-2) 0 0;
    padding-left: 1.2rem;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .feedback {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--danger);
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 8%, transparent);
  }

  .feedback.success {
    border-color: var(--success);
    color: var(--success);
    background: color-mix(in srgb, var(--success) 8%, transparent);
  }

  @media (max-width: 960px) {
    .settings-layout,
    .field-row,
    .field-row.three,
    .toggle-grid,
    .accent-grid,
    .choice-grid.three {
      grid-template-columns: 1fr;
    }

    .settings-nav {
      position: static;
    }
  }
</style>
