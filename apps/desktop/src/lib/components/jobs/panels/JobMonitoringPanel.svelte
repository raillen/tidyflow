<script lang="ts">
  /**
   * "Monitoring" tab: the filesystem watch configuration. Enabling a watch
   * disables the schedule, since both would otherwise compete for the same job.
   */
  import type { Job, WatchConfig, WatchDetectionMode, WatchEventKind } from "$lib/contracts/job";
  import { defaultWatch, detectionOptions, watchEvents } from "$lib/contracts/job-options";
  import { t } from "$lib/i18n";

  type Props = {
    job: Job;
  };

  let { job = $bindable() }: Props = $props();

  const activeWatch = $derived(job.watch ?? null);

  function patchWatch(partial: Partial<WatchConfig>) {
    const base = job.watch ?? defaultWatch();
    job = { ...job, watch: { ...base, ...partial } };
  }

  function setWatchEnabled(enabled: boolean) {
    if (!enabled) {
      job = { ...job, watch: job.watch ? { ...job.watch, enabled: false } : null };
      return;
    }

    job = {
      ...job,
      watch: job.watch ? { ...job.watch, enabled: true } : defaultWatch(),
      schedule: job.schedule ? { ...job.schedule, enabled: false } : job.schedule,
    };
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
    // At least one event must stay selected or the watch never fires.
    if (events.length === 0) return;
    patchWatch({ events });
  }

  function clampNumber(value: string, min: number, max: number, fallback: number): number {
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.min(max, Math.max(min, Math.trunc(parsed)));
  }
</script>

<div class="panel-header">
  <h3>{t("jobs.nav.monitoring")}</h3>
  <p>{t("jobs.watch.desc")}</p>
</div>

<details class="collapsible" open>
  <summary>{t("jobs.watch.section")}</summary>
  <div class="collapsible-body">
    <label class="toggle">
      <input
        type="checkbox"
        checked={activeWatch?.enabled ?? false}
        onchange={(event) => setWatchEnabled(event.currentTarget.checked)}
      />
      <span>{t("jobs.watch.enabled")}</span>
    </label>

    {#if activeWatch?.enabled}
      <p class="field-hint">
        {t("jobs.watch.hint", { path: job.sourcePath || "—" })}
      </p>

      <label class="field">
        <span class="field-label">{t("jobs.watch.settle")}</span>
        <input
          class="field-input"
          type="number"
          min="1"
          max="60"
          value={activeWatch.settleSeconds}
          oninput={(event) =>
            patchWatch({
              settleSeconds: clampNumber(event.currentTarget.value, 1, 60, 2),
            })}
        />
      </label>

      <label class="field">
        <span class="field-label">{t("jobs.watch.detection")}</span>
        <select
          class="field-input"
          value={activeWatch.detection.kind}
          onchange={(event) =>
            setDetectionKind(event.currentTarget.value as WatchDetectionMode["kind"])}
        >
          {#each detectionOptions() as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </label>

      <p class="field-hint">
        {detectionOptions().find((option) => option.value === activeWatch.detection.kind)?.hint}
      </p>

      {#if activeWatch.detection.kind === "polling"}
        <label class="field">
          <span class="field-label">{t("jobs.watch.pollInterval")}</span>
          <input
            class="field-input"
            type="number"
            min="5"
            max="3600"
            value={activeWatch.detection.intervalSecs}
            oninput={(event) =>
              patchWatch({
                detection: {
                  kind: "polling",
                  intervalSecs: clampNumber(event.currentTarget.value, 5, 3600, 30),
                },
              })}
          />
        </label>
      {:else if activeWatch.detection.kind === "hybrid"}
        <label class="field">
          <span class="field-label">{t("jobs.watch.backupPoll")}</span>
          <input
            class="field-input"
            type="number"
            min="5"
            max="3600"
            value={activeWatch.detection.pollIntervalSecs}
            oninput={(event) =>
              patchWatch({
                detection: {
                  kind: "hybrid",
                  pollIntervalSecs: clampNumber(event.currentTarget.value, 5, 3600, 30),
                },
              })}
          />
        </label>
      {/if}

      <fieldset class="preset-group">
        <legend class="field-label">{t("jobs.watch.events")}</legend>
        <div class="checkbox-grid">
          {#each watchEvents() as event}
            {@const isSelected = activeWatch.events.includes(event.id)}
            <!-- The last selected event cannot be removed: a watch with no
                 events would never fire, and a blocked state change would
                 leave the checkbox visually unchecked but logically on. -->
            <label class="toggle">
              <input
                type="checkbox"
                checked={isSelected}
                disabled={isSelected && activeWatch.events.length === 1}
                onchange={() => toggleWatchEvent(event.id)}
              />
              <span>{event.label}</span>
            </label>
          {/each}
        </div>
      </fieldset>

      <label class="toggle">
        <input
          type="checkbox"
          checked={activeWatch.autoCorrect}
          onchange={(event) => patchWatch({ autoCorrect: event.currentTarget.checked })}
        />
        <span>{t("watch.autoCorrect")}</span>
      </label>
    {/if}
  </div>
</details>
