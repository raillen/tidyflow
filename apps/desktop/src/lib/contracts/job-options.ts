/**
 * Reactive option builders for the job editor.
 *
 * Every list is a function (not a constant) so labels are resolved against the
 * active locale on each render. Keeping them out of the component also lets the
 * individual panels import only the options they need.
 */
import type {
  NotifyConfig,
  ScheduleConfig,
  ScheduleRule,
  WatchConfig,
  WatchDetectionMode,
  WatchEventKind,
} from "$lib/contracts/job";
import { t } from "$lib/i18n";

export type NotifyEvent = NotifyConfig["events"][number];

export interface WeekdayOption {
  value: number;
  label: string;
  fullLabel: string;
}

export interface WatchEventOption {
  id: WatchEventKind;
  label: string;
}

export interface DetectionOption {
  value: WatchDetectionMode["kind"];
  label: string;
  hint: string;
}

export interface RuleKindOption {
  value: ScheduleRule["kind"];
  label: string;
}

export interface NotifyEventOption {
  id: NotifyEvent;
  label: string;
}

export function weekdays(): WeekdayOption[] {
  return [
    { value: 0, label: t("jobs.weekday.sun"), fullLabel: t("jobs.weekday.sunFull") },
    { value: 1, label: t("jobs.weekday.mon"), fullLabel: t("jobs.weekday.monFull") },
    { value: 2, label: t("jobs.weekday.tue"), fullLabel: t("jobs.weekday.tueFull") },
    { value: 3, label: t("jobs.weekday.wed"), fullLabel: t("jobs.weekday.wedFull") },
    { value: 4, label: t("jobs.weekday.thu"), fullLabel: t("jobs.weekday.thuFull") },
    { value: 5, label: t("jobs.weekday.fri"), fullLabel: t("jobs.weekday.friFull") },
    { value: 6, label: t("jobs.weekday.sat"), fullLabel: t("jobs.weekday.satFull") },
  ];
}

export function watchEvents(): WatchEventOption[] {
  return [
    { id: "create", label: t("jobs.event.create") },
    { id: "modify", label: t("jobs.event.modify") },
    { id: "remove", label: t("jobs.event.remove") },
    { id: "rename", label: t("jobs.event.rename") },
  ];
}

export function detectionOptions(): DetectionOption[] {
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

export function ruleKindOptions(): RuleKindOption[] {
  return [
    { value: "interval", label: t("jobs.rule.interval") },
    { value: "daily", label: t("jobs.rule.daily") },
    { value: "weekly", label: t("jobs.rule.weekly") },
  ];
}

export function notifyEvents(): NotifyEventOption[] {
  return [
    { id: "started", label: t("jobs.notifyEvent.started") },
    { id: "completed", label: t("jobs.notifyEvent.completed") },
    { id: "failed", label: t("jobs.notifyEvent.failed") },
  ];
}

/** Defaults applied when the user first enables a watch or a schedule. */
export function defaultWatch(): WatchConfig {
  return {
    enabled: true,
    settleSeconds: 2,
    detection: { kind: "realtime" },
    events: ["create"],
    autoCorrect: false,
    namingPattern: {},
  };
}

export function defaultSchedule(): ScheduleConfig {
  return {
    enabled: true,
    timezone: "local",
    rule: { kind: "interval", minutes: 60 },
  };
}

/** Weekday list rendered as a single human-readable sentence. */
export function selectedWeekdayLabels(days: number[]): string {
  return weekdays()
    .filter((day) => days.includes(day.value))
    .map((day) => day.fullLabel)
    .join(", ");
}
