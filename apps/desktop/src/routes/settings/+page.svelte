<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import QRCode from "qrcode";
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
    accentPresets,
    adminAgentModeOptions,
    changelogEntries,
    interfaceFontOptions,
    languageOptions,
    processPriorityOptions,
    themeOptions,
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
    setSecurityPin,
  } from "$lib/core/ipc/client";
  import { applyAppearance } from "$lib/core/stores/theme";
  import { setLocale, t, type Locale } from "$lib/i18n";
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

  const tabs = $derived<{ id: SettingsTab; label: string; icon: Component }[]>([
    { id: "general", label: t("settings.general"), icon: GearSix },
    { id: "performance", label: t("settings.performance"), icon: SlidersHorizontal },
    { id: "security", label: t("settings.security"), icon: LockKey },
    { id: "notifications", label: t("settings.notifications"), icon: Bell },
    { id: "admin", label: t("settings.admin"), icon: DesktopTower },
    { id: "maintenance", label: t("settings.maintenance"), icon: ClockCounterClockwise },
    { id: "support", label: t("settings.support"), icon: Coffee },
    { id: "about", label: t("settings.about"), icon: Info },
    { id: "changelog", label: t("settings.changelogTitle"), icon: FloppyDisk },
  ]);

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
  let expandedVersions = $state<string[]>([changelogEntries()[0]?.version ?? ""]);
  let initialLanguage = $state("pt-BR");

  const filteredChangelog = $derived(
    changelogEntries().map((entry) => ({
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
      initialLanguage = settings.language;
      health = await fetchHealth();
      applyAppearance(settings);
      updatePixQrCode();
    } catch (e) {
      message = e instanceof Error ? e.message : t("settings.loadError");
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
        if (newPin.length < 4) throw new Error(t("settings.pinMin"));
        if (newPin !== confirmPin) throw new Error(t("settings.pinMismatch"));
      }

      settings = await saveSettings(next);
      if (next.security.pinEnabled && (newPin || confirmPin)) {
        settings = await setSecurityPin(newPin);
      }
      newPin = "";
      confirmPin = "";
      applyAppearance(settings);
      if (next.language !== initialLanguage) {
        initialLanguage = next.language;
        setLocale(next.language as Locale);
        window.location.reload();
        return;
      }
      message = t("settings.saved");
    } catch (e) {
      message = e instanceof Error ? e.message : t("settings.saveError");
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
      message = t("settings.agentSecretMin");
      return;
    }

    saving = true;
    message = null;
    try {
      settings = await setAdminAgentSecret(adminSecretInput);
      adminSecretInput = "";
      generatedAdminSecret = null;
      message = t("settings.agentSecretSaved");
    } catch (e) {
      message = e instanceof Error ? e.message : t("settings.agentSecretSaveError");
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
      message = t("settings.agentSecretGenerated");
    } catch (e) {
      message = e instanceof Error ? e.message : t("settings.agentSecretGenerateError");
    } finally {
      saving = false;
    }
  }

  async function clearAdminSecret() {
    if (!window.confirm(t("settings.agentSecretRemoveConfirm"))) {
      return;
    }

    saving = true;
    message = null;
    try {
      settings = await clearAdminAgentSecret();
      adminSecretInput = "";
      generatedAdminSecret = null;
      message = t("settings.agentSecretRemoved");
    } catch (e) {
      message = e instanceof Error ? e.message : t("settings.agentSecretRemoveError");
    } finally {
      saving = false;
    }
  }

  async function copyGeneratedAdminSecret() {
    if (!generatedAdminSecret) return;
    try {
      await navigator.clipboard.writeText(generatedAdminSecret);
      message = t("settings.agentSecretCopyOk");
    } catch {
      message = t("settings.agentSecretCopyError");
    }
  }

  let pixQrCodeUrl = $state<string | null>(null);
  let pixPayload = $state<string>("");

  function generatePixPayload(key: string, name: string, city: string): string {
    const f = (id: string, value: string) => id + String(value.length).padStart(2, '0') + value;
    const gui = f('00', 'br.gov.bcb.pix');
    const keyInfo = f('01', key);
    const merchantAccountInfo = f('26', gui + keyInfo);
    
    const payloadFormat = '000201';
    const mcc = '52040000';
    const currency = '5303986';
    const country = '5802BR';
    const merchantName = f('59', name);
    const merchantCity = f('60', city);
    const additionalData = f('62', f('05', '***'));
    
    const partialPayload = payloadFormat + merchantAccountInfo + mcc + currency + country + merchantName + merchantCity + additionalData + '6304';
    
    // Calculate CRC16
    let crc = 0xFFFF;
    for (let i = 0; i < partialPayload.length; i++) {
      const charCode = partialPayload.charCodeAt(i);
      crc ^= (charCode << 8);
      for (let j = 0; j < 8; j++) {
        if ((crc & 0x8000) !== 0) {
          crc = (crc << 1) ^ 0x1021;
        } else {
          crc <<= 1;
        }
      }
    }
    const crcStr = (crc & 0xFFFF).toString(16).toUpperCase().padStart(4, '0');
    return partialPayload + crcStr;
  }

  function updatePixQrCode() {
    const key = "contato@raillen.site";
    const name = "RAILLEN SANTOS";
    const city = "SAO PAULO";
    try {
      const payload = generatePixPayload(key, name, city);
      pixPayload = payload;
      QRCode.toDataURL(payload, { margin: 1, width: 180 }, (err, url) => {
        if (!err) {
          pixQrCodeUrl = url;
        }
      });
    } catch (err) {
      console.error("Erro ao gerar QR Code Pix", err);
    }
  }

  async function copyText(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      message = t("settings.copied");
      setTimeout(() => {
        if (message === t("settings.copied")) {
          message = null;
        }
      }, 3000);
    } catch {
      message = t("settings.copyError");
    }
  }

  function handleLanguageChange(value: string) {
    if (!settings) return;
    settings = { ...settings, language: value };
    setLocale(value as Locale);
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
      <h1 class="page-title">{t("settings.title")}</h1>
      <p class="page-desc">{t("settings.subtitle")}</p>
    </div>
    <button class="btn primary" type="button" onclick={handleSave} disabled={!settings || saving}>
      <CheckCircle size={16} weight="bold" />
      {saving ? t("settings.saving") : t("common.save")}
    </button>
  </header>

  {#if settings}
    <div class="settings-layout">
      <nav class="settings-nav panel" aria-label={t("settings.navAria")}>
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
            <SectionTitle icon={Palette} title={t("settings.general")} text={t("settings.generalText")} />

            <div class="subsection">
              <span class="field-label">{t("settings.theme")}</span>
              <div class="choice-grid three">
                {#each themeOptions() as option}
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
              <span class="field-label">{t("settings.accentColor")}</span>
              <div class="accent-grid" role="radiogroup" aria-label={t("settings.accentColor")}>
                {#each accentPresets() as preset}
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
                <span class="field-label">{t("settings.language")}</span>
                <select class="field-input" bind:value={settings.language} onchange={(event) => handleLanguageChange(event.currentTarget.value)}>
                  {#each languageOptions() as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>

              <label class="field">
                <span class="field-label">{t("settings.interfaceFont")}</span>
                <select
                  class="field-input"
                  value={settings.interfaceFont}
                  onchange={(event) => setInterfaceFont(event.currentTarget.value)}
                >
                  {#each interfaceFontOptions() as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>
            </div>

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.autostart} label={t("settings.autostart")} />
              <ToggleField bind:checked={settings.closeToTray} label={t("settings.closeToTray")} />
              <ToggleField bind:checked={settings.startMinimized} label={t("settings.startMinimized")} />
            </div>
          </section>
        {:else if activeTab === "performance"}
          <section class="panel section">
            <SectionTitle icon={SlidersHorizontal} title={t("settings.performance")} text={t("settings.performanceText")} />

            <div class="field-row three">
              <NumberField bind:value={settings.maxParallelFiles} label={t("settings.maxParallelFiles")} min={1} max={64} />
              <NumberField bind:value={settings.performance.maxThreads} label={t("settings.maxThreads")} min={1} max={128} />
              <NumberField bind:value={settings.performance.memoryLimitMb} label={t("settings.memoryLimit")} min={0} />
            </div>

            <div class="field-row three">
              <label class="field">
                <span class="field-label">{t("settings.processPriority")}</span>
                <select class="field-input" bind:value={settings.performance.processPriority}>
                  {#each processPriorityOptions() as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>
              <NumberField bind:value={settings.performance.globalBandwidthLimitMbps} label={t("settings.bandwidth")} min={0} />
              <NumberField bind:value={settings.performance.queuePollIntervalMs} label={t("settings.queuePoll")} min={100} />
            </div>

            <ToggleField bind:checked={settings.performance.pauseWhenOnBattery} label={t("settings.pauseOnBattery")} />
          </section>
        {:else if activeTab === "security"}
          <section class="panel section">
            <SectionTitle icon={ShieldCheck} title={t("settings.security")} text={t("settings.securityText")} />

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.security.pinEnabled} label={t("settings.pinEnabled")} />
              <ToggleField bind:checked={settings.security.requirePinOnStartup} label={t("settings.pinOnStartup")} />
              <ToggleField bind:checked={settings.security.lockOnMinimize} label={t("settings.lockOnMinimize")} />
              <ToggleField bind:checked={settings.security.lockOnTray} label={t("settings.lockOnTray")} />
              <ToggleField bind:checked={settings.security.encryptionEnabled} label={t("settings.encryption")} />
              <ToggleField bind:checked={settings.security.maskSensitivePaths} label={t("settings.maskPaths")} />
            </div>

            {#if settings.security.pinEnabled}
              <div class="field-row">
                <label class="field">
                  <span class="field-label">{t("settings.newPin")}</span>
                  <input class="field-input" type="password" bind:value={newPin} autocomplete="new-password" />
                </label>
                <label class="field">
                  <span class="field-label">{t("settings.confirmPin")}</span>
                  <input class="field-input" type="password" bind:value={confirmPin} autocomplete="new-password" />
                </label>
              </div>
              <p class="hint">
                {t("settings.pinHashHint")}
              </p>
            {/if}

            <label class="field">
              <span class="field-label">{t("settings.masterKeyHint")}</span>
              <input class="field-input" bind:value={settings.security.masterKeyHint} placeholder={t("settings.masterKeyPlaceholder")} />
            </label>
          </section>
        {:else if activeTab === "notifications"}
          <section class="panel section">
            <SectionTitle icon={Bell} title={t("settings.notifications")} text={t("settings.notificationsText")} />

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.notifications.enabled} label={t("settings.notificationsEnabled")} />
              <ToggleField bind:checked={settings.notifications.desktopEnabled} label={t("settings.desktopEnabled")} />
              <ToggleField bind:checked={settings.notifications.notifyOnSuccess} label={t("settings.notifyOnSuccess")} />
              <ToggleField bind:checked={settings.notifications.notifyOnFailure} label={t("settings.notifyOnFailure")} />
              <ToggleField bind:checked={settings.notifications.webhookEnabled} label={t("settings.webhookEnabled")} />
              <ToggleField bind:checked={settings.notifications.adminPanelEnabled} label={t("settings.adminPanelEnabled")} />
            </div>

            <div class="subsection">
              <div class="section-line">
                <h3>{t("settings.webhooks")}</h3>
                <button class="btn" type="button" onclick={addWebhook}>
                  <Plus size={14} />
                  {t("settings.add")}
                </button>
              </div>
              {#each settings.notifications.webhooks as webhook, index}
                <div class="nested-card">
                  <div class="field-row">
                    <label class="field">
                      <span class="field-label">{t("common.name")}</span>
                      <input class="field-input" bind:value={webhook.name} />
                    </label>
                    <label class="field">
                      <span class="field-label">URL</span>
                      <input class="field-input" bind:value={webhook.url} placeholder="https://..." />
                    </label>
                  </div>
                  <div class="nested-actions">
                    <ToggleField bind:checked={webhook.enabled} label={t("settings.active")} />
                    <button class="btn ghost danger" type="button" onclick={() => removeWebhook(index)}>
                      <Trash size={14} />
                      {t("settings.remove")}
                    </button>
                  </div>
                </div>
              {:else}
                <p class="muted">{t("settings.noWebhooks")}</p>
              {/each}
            </div>

            <div class="subsection">
              <h3>{t("settings.smtp")}</h3>
              <ToggleField bind:checked={settings.notifications.smtp.enabled} label={t("settings.smtpEnabled")} />
              <div class="field-row three">
                <label class="field">
                  <span class="field-label">{t("settings.host")}</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.host} placeholder="smtp.exemplo.com" />
                </label>
                <NumberField bind:value={settings.notifications.smtp.port} label={t("settings.port")} min={1} max={65535} />
                <label class="field">
                  <span class="field-label">{t("settings.from")}</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.fromAddress} placeholder="tidyflow@exemplo.com" />
                </label>
              </div>
              <div class="field-row">
                <label class="field">
                  <span class="field-label">{t("settings.username")}</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.username} />
                </label>
                <label class="field">
                  <span class="field-label">{t("settings.password")}</span>
                  <input class="field-input" type="password" bind:value={settings.notifications.smtp.password} />
                </label>
                <label class="field">
                  <span class="field-label">{t("settings.to")}</span>
                  <input class="field-input" bind:value={settings.notifications.smtp.toAddress} placeholder="destino@exemplo.com" />
                </label>
                <ToggleField bind:checked={settings.notifications.smtp.useTls} label={t("settings.useTls")} />
              </div>
            </div>
          </section>
        {:else if activeTab === "admin"}
          <section class="panel section">
            <SectionTitle icon={DesktopTower} title={t("settings.admin")} text={t("settings.adminText")} />

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.admin.enabled} label={t("settings.adminEnabled")} />
              <ToggleField bind:checked={settings.admin.allowRemoteCommands} label={t("settings.allowRemote")} />
              <ToggleField bind:checked={settings.admin.allowBatchCommands} label={t("settings.allowBatch")} />
            </div>

            <div class="field-row">
              <label class="field">
                <span class="field-label">{t("settings.mode")}</span>
                <select class="field-input" bind:value={settings.admin.mode}>
                  {#each adminAgentModeOptions() as option}
                    <option value={option.id}>{option.label}</option>
                  {/each}
                </select>
              </label>
              <label class="field">
                <span class="field-label">{t("settings.displayName")}</span>
                <input class="field-input" bind:value={settings.admin.displayName} placeholder={t("settings.displayNamePlaceholder")} />
              </label>
            </div>

            <div class="field-row">
              <label class="field">
                <span class="field-label">{t("settings.instanceId")}</span>
                <input class="field-input" value={settings.admin.instanceId ?? t("settings.generatedOnStart")} readonly />
              </label>
              <label class="field">
                <span class="field-label">{t("settings.serverUrl")}</span>
                <input class="field-input" bind:value={settings.admin.serverUrl} placeholder="https://admin.tidyflow.local" />
              </label>
              <label class="field">
                <span class="field-label">{t("settings.enrollmentToken")}</span>
                <input class="field-input" type="password" bind:value={settings.admin.enrollmentToken} placeholder={t("settings.enrollmentTokenPlaceholder")} />
              </label>
            </div>

            <div class="field-row three">
              <NumberField bind:value={settings.admin.heartbeatIntervalSecs} label={t("settings.heartbeat")} min={10} max={3600} />
              <NumberField bind:value={settings.admin.inventoryIntervalSecs} label={t("settings.inventory")} min={60} max={86400} />
              <label class="field">
                <span class="field-label">{t("settings.enrollmentTokenStatus")}</span>
                <input
                  class="field-input"
                  value={settings.admin.enrollmentTokenConfigured ? t("settings.configured") : t("settings.notConfigured")}
                  readonly
                />
              </label>
            </div>

            <section class="subsection nested-card" aria-label={t("settings.agentSecret")}>
              <div class="section-line">
                <h3>{t("settings.agentSecret")}</h3>
                <span class="status-pill" class:enabled={settings.admin.enrollmentTokenConfigured}>
                  {settings.admin.enrollmentTokenConfigured ? t("settings.inVault") : t("settings.notConfigured")}
                </span>
              </div>

              <label class="field">
                <span class="field-label">{t("settings.secretHint")}</span>
                <input
                  class="field-input"
                  type="password"
                  bind:value={adminSecretInput}
                  autocomplete="new-password"
                  placeholder={t("settings.secretPlaceholder")}
                />
              </label>

              <div class="secret-actions">
                <button class="btn primary" type="button" onclick={saveAdminSecret} disabled={saving || adminSecretInput.trim().length < 32}>
                  {t("settings.saveToVault")}
                </button>
                <button class="btn" type="button" onclick={generateAdminSecret} disabled={saving}>
                  {t("settings.generateLocal")}
                </button>
                <button class="btn danger" type="button" onclick={clearAdminSecret} disabled={saving || !settings.admin.enrollmentTokenConfigured}>
                  {t("settings.clear")}
                </button>
              </div>

              {#if generatedAdminSecret}
                <div class="generated-secret" aria-label={t("settings.generatedSecretAria")}>
                  <code>{generatedAdminSecret}</code>
                  <button class="btn" type="button" onclick={copyGeneratedAdminSecret}>{t("settings.copy")}</button>
                </div>
              {/if}
            </section>

            <p class="hint">
              {t("settings.webModeHint")}
            </p>
          </section>
        {:else if activeTab === "maintenance"}
          <section class="panel section">
            <SectionTitle icon={ClockCounterClockwise} title={t("settings.maintenance")} text={t("settings.maintenanceText")} />

            <div class="field-row three">
              <NumberField bind:value={settings.maintenance.logRetentionDays} label={t("settings.logRetention")} min={0} />
              <NumberField bind:value={settings.maintenance.backupIntervalHours} label={t("settings.backupInterval")} min={1} />
              <NumberField bind:value={settings.maintenance.backupRetentionCount} label={t("settings.backupsKept")} min={1} />
            </div>

            <div class="toggle-grid">
              <ToggleField bind:checked={settings.maintenance.autoCompactDatabase} label={t("settings.autoCompact")} />
              <ToggleField bind:checked={settings.maintenance.optimizeAfterCleanup} label={t("settings.optimizeAfterCleanup")} />
              <ToggleField bind:checked={settings.maintenance.backupEnabled} label={t("settings.backupEnabled")} />
            </div>

            <label class="field">
              <span class="field-label">{t("settings.backupDirectory")}</span>
              <input class="field-input" bind:value={settings.maintenance.backupDirectory} placeholder="D:\Backups\TidyFlow" />
            </label>
          </section>
        {:else if activeTab === "support"}
          <section class="panel section">
            <SectionTitle icon={Coffee} title={t("settings.support")} text={t("settings.supportText")} />

            <ToggleField bind:checked={settings.support.donationsEnabled} label={t("settings.showDonations")} />
            
            <div class="support-cards-grid">
              <div class="support-card">
                <h4>{t("settings.supportContact")}</h4>
                <div class="copy-field">
                  <span class="value-text">contato@raillen.site</span>
                  <button type="button" class="btn small" onclick={() => copyText("contato@raillen.site")}>
                    {t("settings.copy")}
                  </button>
                </div>
              </div>

              <div class="support-card">
                <h4>{t("settings.buyMe")}</h4>
                <button type="button" class="btn primary" onclick={() => openUrl("https://www.buymeacoffee.com/raillen")}>
                  {t("settings.buyMe")}
                </button>
              </div>
            </div>

            <div class="pix-donation-container">
              <div class="pix-info">
                <h4>{t("settings.pixTitle")}</h4>
                <p class="muted text-xs">{t("settings.pixDesc")}</p>
                <div class="copy-field">
                  <span class="label-tiny">{t("settings.pixKey")}</span>
                  <span class="value-text">contato@raillen.site</span>
                  <button type="button" class="btn small" onclick={() => copyText("contato@raillen.site")}>
                    {t("settings.copy")}
                  </button>
                </div>
                {#if pixPayload}
                  <div class="copy-field margin-top-sm">
                    <span class="label-tiny">{t("settings.pixCopyPaste")}</span>
                    <input class="field-input text-xs" readonly value={pixPayload} />
                    <button type="button" class="btn small" onclick={() => copyText(pixPayload)}>
                      {t("settings.copy")}
                    </button>
                  </div>
                {/if}
              </div>
              {#if pixQrCodeUrl}
                <div class="pix-qrcode">
                  <img src={pixQrCodeUrl} alt={t("settings.pixQrAlt")} />
                </div>
              {/if}
            </div>

            <div class="deposit-info-container">
              <h4>{t("settings.depositTitle")}</h4>
              <pre class="deposit-details">
- BRAZIL
RAILLEN DOS SANTOS DE OLIVEIRA
Conta: 2228476-1 Agência: 0007 Banco 077 - Inter

- GLOBAL
RAILLEN DOS SANTOS DE OLIVEIRA
Acc Number: 8896373745
ACH Routing Number: 026073150
Wire Transfer Routing Number: 026073008
Bank Name: Community Federal Savings Bank
Bank Address: 5 Penn Plaza, New York, NY 10001
              </pre>
            </div>
          </section>
        {:else if activeTab === "about"}
          <section class="panel section">
            <SectionTitle icon={Info} title={t("settings.about")} text={t("settings.aboutText")} />

            <div class="about-container">
              <div class="about-app-card">
                <h3>TidyFlow</h3>
                <span class="version-badge">{t("settings.version")} {health?.version ?? "0.2.1-alpha"}</span>
                <p class="description">{t("settings.description")}</p>
              </div>

              <div class="about-creator-card">
                <h4>{t("settings.creator")}</h4>
                <strong class="creator-name">Raillen Santos</strong>
                <p class="bio">
                  {t("settings.creatorBio")}
                </p>
                
                <div class="creator-links">
                  <button type="button" class="btn" onclick={() => openUrl("https://raillen.site")}>
                    {t("settings.officialSite")}
                  </button>
                  <button type="button" class="btn" onclick={() => openUrl("https://github.com/raillen")}>
                    GitHub
                  </button>
                  <button type="button" class="btn" onclick={() => openUrl("https://br.linkedin.com/in/raillen")}>
                    LinkedIn
                  </button>
                </div>
              </div>
            </div>
          </section>
        {:else}
          <section class="panel section">
            <SectionTitle icon={FloppyDisk} title={t("settings.changelogTitle")} text={t("settings.changelogText")} />

            <div class="field-row">
              <label class="field">
                <span class="field-label">{t("settings.changelogSearch")}</span>
                <input class="field-input" bind:value={changelogSearch} placeholder={t("settings.changelogSearchPlaceholder")} />
              </label>
              <label class="field">
                <span class="field-label">{t("settings.changelogType")}</span>
                <select class="field-input" bind:value={changelogFilter}>
                  <option value="all">{t("settings.changelogAll")}</option>
                  <option value="feature">{t("settings.changelogFeatures")}</option>
                  <option value="fix">{t("settings.changelogFix")}</option>
                  <option value="security">{t("settings.changelogSecurity")}</option>
                  <option value="maintenance">{t("settings.changelogMaintenance")}</option>
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
                <p class="muted">{t("settings.changelogEmpty")}</p>
              {/each}
            </div>
          </section>
        {/if}

        {#if message}
          <p class:success={message === t("settings.saved")} class="feedback" aria-live="polite">{message}</p>
        {/if}
      </form>
    </div>
  {:else if message}
    <p class="banner error" role="alert">{message}</p>
  {:else}
    <p class="muted" aria-live="polite">{t("settings.loading")}</p>
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



  .hint {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .nested-card,
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

  /* Custom Premium Layout styles for read-only Support & About sections */
  .support-cards-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
  }

  .support-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-muted);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .support-card h4,
  .pix-info h4,
  .deposit-info-container h4,
  .about-creator-card h4 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-primary);
  }

  .copy-field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .value-text {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-primary);
    word-break: break-all;
  }

  .label-tiny {
    font-size: var(--text-xs);
    color: var(--text-muted);
    font-weight: 500;
  }

  .margin-top-sm {
    margin-top: var(--space-2);
  }

  .pix-donation-container {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-muted);
    padding: var(--space-4);
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-4);
    align-items: center;
  }

  .pix-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .pix-qrcode {
    background: var(--surface);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 196px;
    height: 196px;
  }

  .pix-qrcode img {
    width: 180px;
    height: 180px;
    display: block;
  }

  .deposit-info-container {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-muted);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .deposit-details {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.6;
    color: var(--text-secondary);
    background: var(--surface);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* About section styles */
  .about-container {
    display: grid;
    gap: var(--space-4);
  }

  .about-app-card,
  .about-creator-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface-muted);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .about-app-card h3 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 700;
  }

  .version-badge {
    align-self: flex-start;
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
    font-size: var(--text-xs);
  }

  .about-app-card .description {
    margin: var(--space-2) 0 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .about-creator-card .creator-name {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--text-primary);
  }

  .about-creator-card .bio {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .creator-links {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
</style>
