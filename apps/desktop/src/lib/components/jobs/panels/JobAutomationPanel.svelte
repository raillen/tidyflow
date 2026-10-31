<script lang="ts">
  /**
   * "Automation" tab: the exclusive choice between a schedule and a watch, plus
   * the rule editor for each schedule kind.
   */
  import type { Job, ScheduleConfig, ScheduleRule } from "$lib/contracts/job";
  import {
    defaultSchedule,
    ruleKindOptions,
    selectedWeekdayLabels,
    weekdays,
  } from "$lib/contracts/job-options";
  import { t } from "$lib/i18n";

  type Props = {
    job: Job;
  };

  let { job = $bindable() }: Props = $props();

  /** The schedule currently being edited, or null when disabled/absent. */
  const activeSchedule = $derived(job.schedule?.enabled ? job.schedule : null);

  // Each variant gets its own narrowed derived so the template does not have to
  // re-check the discriminant after every read of mutable state.
  const intervalRule = $derived(
    activeSchedule?.rule.kind === "interval" ? activeSchedule.rule : null,
  );
  const dailyRule = $derived(
    activeSchedule?.rule.kind === "daily" ? activeSchedule.rule : null,
  );
  const weeklyRule = $derived(
    activeSchedule?.rule.kind === "weekly" ? activeSchedule.rule : null,
  );

  function patchSchedule(partial: Partial<ScheduleConfig>) {
    const base = job.schedule ?? defaultSchedule();
    job = { ...job, schedule: { ...base, ...partial } };
  }

  function patchRule(rule: ScheduleRule) {
    patchSchedule({ rule });
  }

  function setScheduleEnabled(enabled: boolean) {
    if (!enabled) {
      job = { ...job, schedule: job.schedule ? { ...job.schedule, enabled: false } : null };
      return;
    }

    job = {
      ...job,
      // A watch and a schedule are mutually exclusive: enabling one disables
      // the other so the backend never receives two competing triggers.
      watch: job.watch ? { ...job.watch, enabled: false } : job.watch,
      schedule: job.schedule ? { ...job.schedule, enabled: true } : defaultSchedule(),
    };
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
    if (!weeklyRule) return;
    const days = weeklyRule.days.includes(day)
      ? weeklyRule.days.filter((value) => value !== day)
      : [...weeklyRule.days, day].sort((a, b) => a - b);
    // A weekly rule with no days could never fire, so the last day sticks.
    if (days.length === 0) return;
    patchRule({
      kind: "weekly",
      days,
      hour: weeklyRule.hour,
      minute: weeklyRule.minute,
    });
  }

  function clampNumber(value: string, min: number, max: number, fallback: number): number {
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.min(max, Math.max(min, Math.trunc(parsed)));
  }
</script>

<div class="panel-header">
  <h3>{t("jobs.nav.automation")}</h3>
  <p>{t("jobs.auto.desc")}</p>
</div>

<details class="collapsible" open>
  <summary>{t("jobs.auto.schedule")}</summary>
  <div class="collapsible-body">
    <label class="toggle">
      <input
        type="checkbox"
        checked={job.schedule?.enabled ?? false}
        onchange={(event) => setScheduleEnabled(event.currentTarget.checked)}
      />
      <span>{t("jobs.auto.enabled")}</span>
    </label>

    {#if activeSchedule}
      <p class="field-hint">{t("jobs.auto.exclusiveHint")}</p>

      <label class="field">
        <span class="field-label">{t("jobs.auto.timezone")}</span>
        <select
          class="field-input"
          value={activeSchedule.timezone}
          onchange={(event) => patchSchedule({ timezone: event.currentTarget.value })}
        >
          <option value="local">{t("jobs.auto.tzLocal")}</option>
          <option value="UTC">UTC</option>
          <option value="America/Sao_Paulo">America/Sao_Paulo</option>
          <option value="Europe/Lisbon">Europe/Lisbon</option>
        </select>
      </label>

      <label class="field">
        <span class="field-label">{t("jobs.auto.ruleKind")}</span>
        <select
          class="field-input"
          value={activeSchedule.rule.kind}
          onchange={(event) => setRuleKind(event.currentTarget.value as ScheduleRule["kind"])}
        >
          {#each ruleKindOptions() as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </label>

      {#if intervalRule}
        <label class="field">
          <span class="field-label">{t("jobs.auto.intervalMinutes")}</span>
          <input
            class="field-input"
            type="number"
            min="1"
            value={intervalRule.minutes}
            oninput={(event) =>
              patchRule({
                kind: "interval",
                minutes: clampNumber(event.currentTarget.value, 1, 100_000, 60),
              })}
          />
        </label>
      {:else if dailyRule}
        <div class="field-row">
          <label class="field">
            <span class="field-label">{t("jobs.auto.hour")}</span>
            <input
              class="field-input"
              type="number"
              min="0"
              max="23"
              value={dailyRule.hour}
              oninput={(event) =>
                patchRule({
                  kind: "daily",
                  hour: clampNumber(event.currentTarget.value, 0, 23, 9),
                  minute: dailyRule.minute,
                })}
            />
          </label>
          <label class="field">
            <span class="field-label">{t("jobs.auto.minute")}</span>
            <input
              class="field-input"
              type="number"
              min="0"
              max="59"
              value={dailyRule.minute}
              oninput={(event) =>
                patchRule({
                  kind: "daily",
                  hour: dailyRule.hour,
                  minute: clampNumber(event.currentTarget.value, 0, 59, 0),
                })}
            />
          </label>
        </div>
      {:else if weeklyRule}
        <fieldset class="preset-group">
          <legend class="field-label">{t("jobs.auto.selectedDays")}</legend>
          <div class="weekday-grid" role="group" aria-label={t("jobs.auto.weekdaysAria")}>
            {#each weekdays() as day}
              <button
                type="button"
                class="weekday-btn"
                class:selected={weeklyRule.days.includes(day.value)}
                title={day.fullLabel}
                aria-pressed={weeklyRule.days.includes(day.value)}
                onclick={() => toggleWeekday(day.value)}
              >
                {day.label}
              </button>
            {/each}
          </div>
          <p class="weekday-summary muted">{selectedWeekdayLabels(weeklyRule.days)}</p>
        </fieldset>

        <div class="field-row">
          <label class="field">
            <span class="field-label">{t("jobs.auto.hour")}</span>
            <input
              class="field-input"
              type="number"
              min="0"
              max="23"
              value={weeklyRule.hour}
              oninput={(event) =>
                patchRule({
                  kind: "weekly",
                  days: weeklyRule.days,
                  hour: clampNumber(event.currentTarget.value, 0, 23, 9),
                  minute: weeklyRule.minute,
                })}
            />
          </label>
          <label class="field">
            <span class="field-label">{t("jobs.auto.minute")}</span>
            <input
              class="field-input"
              type="number"
              min="0"
              max="59"
              value={weeklyRule.minute}
              oninput={(event) =>
                patchRule({
                  kind: "weekly",
                  days: weeklyRule.days,
                  hour: weeklyRule.hour,
                  minute: clampNumber(event.currentTarget.value, 0, 59, 0),
                })}
            />
          </label>
        </div>
      {/if}
    {/if}
  </div>
</details>
