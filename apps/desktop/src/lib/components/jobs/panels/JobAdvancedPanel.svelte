<script lang="ts">
  /**
   * "Advanced" tab: pre/post scripts and the notification channels.
   */
  import { Bell, Plus, Trash } from "phosphor-svelte";
  import type { Job, NotifyChannel, NotifyConfig, ScriptsConfig } from "$lib/contracts/job";
  import { notifyEvents, type NotifyEvent } from "$lib/contracts/job-options";
  import { t } from "$lib/i18n";

  type Props = {
    job: Job;
  };

  let { job = $bindable() }: Props = $props();

  function patchScripts(partial: Partial<ScriptsConfig>) {
    job = { ...job, scripts: { ...job.scripts, ...partial } };
  }

  function patchNotify(partial: Partial<NotifyConfig>) {
    job = { ...job, notify: { ...job.notify, ...partial } };
  }

  function toggleNotifyEvent(event: NotifyEvent) {
    const events = job.notify.events;
    patchNotify({
      events: events.includes(event) ? events.filter((item) => item !== event) : [...events, event],
    });
  }

  function addChannel(type: NotifyChannel["type"]) {
    const channel: NotifyChannel =
      type === "discord"
        ? { type: "discord", webhookUrl: "https://discord.com/api/webhooks/" }
        : type === "telegram"
          ? { type: "telegram", botToken: "", chatId: "", rememberToken: false }
          : { type: "generic", url: "https://", headers: {} };
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
      .map(([key, value]) => `${key}: ${value}`)
      .join("\n");
  }

  function textToHeaders(text: string): Record<string, string> {
    const headers: Record<string, string> = {};
    for (const line of text.split("\n")) {
      const separator = line.indexOf(":");
      if (separator <= 0) continue;
      const key = line.slice(0, separator).trim();
      const value = line.slice(separator + 1).trim();
      if (key) headers[key] = value;
    }
    return headers;
  }
</script>

<div class="panel-header">
  <h3>{t("jobs.nav.advanced")}</h3>
  <p>{t("jobs.advanced.desc")}</p>
</div>

<details class="collapsible" open>
  <summary>{t("jobs.advanced.scripts")}</summary>
  <div class="collapsible-body">
    <label class="field">
      <span class="field-label">{t("jobs.advanced.preScript")}</span>
      <input
        type="text"
        class="field-input"
        value={job.scripts.preScript ?? ""}
        oninput={(event) => patchScripts({ preScript: event.currentTarget.value || null })}
        placeholder={t("jobs.advanced.preScriptPlaceholder")}
      />
    </label>

    <label class="field">
      <span class="field-label">{t("jobs.advanced.postScript")}</span>
      <input
        type="text"
        class="field-input"
        value={job.scripts.postScript ?? ""}
        oninput={(event) => patchScripts({ postScript: event.currentTarget.value || null })}
        placeholder={t("jobs.advanced.postScriptPlaceholder")}
      />
    </label>

    <label class="field">
      <span class="field-label">{t("jobs.advanced.timeout")}</span>
      <input
        class="field-input"
        type="number"
        min="5"
        max="600"
        value={job.scripts.timeoutSecs}
        oninput={(event) =>
          patchScripts({
            timeoutSecs: Math.min(600, Math.max(5, Number(event.currentTarget.value) || 60)),
          })}
      />
    </label>
  </div>
</details>

<details class="collapsible" open>
  <summary>
    <Bell size={14} weight="duotone" />
    {t("jobs.advanced.notifications")}
  </summary>
  <div class="collapsible-body">
    <label class="toggle">
      <input
        type="checkbox"
        checked={job.notify.enabled}
        onchange={(event) => patchNotify({ enabled: event.currentTarget.checked })}
      />
      <span>{t("jobs.advanced.notifyEnabled")}</span>
    </label>

    {#if job.notify.enabled}
      <fieldset class="preset-group">
        <legend class="field-label">{t("jobs.advanced.events")}</legend>
        <div class="checkbox-grid">
          {#each notifyEvents() as event}
            <label class="toggle">
              <input
                type="checkbox"
                checked={job.notify.events.includes(event.id)}
                onchange={() => toggleNotifyEvent(event.id)}
              />
              <span>{event.label}</span>
            </label>
          {/each}
        </div>
      </fieldset>

      <div class="channel-list">
        {#each job.notify.channels as channel, index (index)}
          <div class="channel-card panel">
            <div class="channel-head">
              <span class="badge">{channel.type}</span>
              <button
                type="button"
                class="btn ghost danger"
                aria-label={t("jobs.advanced.removeChannel")}
                onclick={() => removeChannel(index)}
              >
                <Trash size={14} />
              </button>
            </div>

            {#if channel.type === "discord"}
              <label class="field">
                <span class="field-label">{t("jobs.advanced.webhookUrl")}</span>
                <input
                  type="text"
                  class="field-input"
                  value={channel.webhookUrl}
                  oninput={(event) =>
                    updateChannel(index, {
                      type: "discord",
                      webhookUrl: event.currentTarget.value,
                    })}
                />
              </label>
            {:else if channel.type === "telegram"}
              <label class="field">
                <span class="field-label">{t("jobs.advanced.botToken")}</span>
                <input
                  class="field-input"
                  type="password"
                  autocomplete="off"
                  value={channel.botToken}
                  oninput={(event) =>
                    updateChannel(index, {
                      ...channel,
                      botToken: event.currentTarget.value,
                    })}
                />
              </label>
              <label class="field">
                <span class="field-label">{t("jobs.advanced.chatId")}</span>
                <input
                  type="text"
                  class="field-input"
                  value={channel.chatId}
                  oninput={(event) =>
                    updateChannel(index, {
                      ...channel,
                      chatId: event.currentTarget.value,
                    })}
                />
              </label>
              <label class="toggle">
                <input
                  type="checkbox"
                  checked={channel.rememberToken}
                  onchange={(event) =>
                    updateChannel(index, {
                      ...channel,
                      rememberToken: event.currentTarget.checked,
                    })}
                />
                <span>{t("jobs.advanced.rememberToken")}</span>
              </label>
            {:else if channel.type === "generic"}
              <label class="field">
                <span class="field-label">{t("jobs.advanced.url")}</span>
                <input
                  type="text"
                  class="field-input"
                  value={channel.url}
                  oninput={(event) =>
                    updateChannel(index, {
                      type: "generic",
                      url: event.currentTarget.value,
                      headers: channel.headers,
                    })}
                />
              </label>
              <label class="field">
                <span class="field-label">{t("jobs.advanced.headers")}</span>
                <textarea
                  class="field-input textarea"
                  rows="3"
                  value={headersToText(channel.headers)}
                  oninput={(event) =>
                    updateChannel(index, {
                      type: "generic",
                      url: channel.url,
                      headers: textToHeaders(event.currentTarget.value),
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
          {t("jobs.advanced.generic")}
        </button>
      </div>
    {/if}
  </div>
</details>
