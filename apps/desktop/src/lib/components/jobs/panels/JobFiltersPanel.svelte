<script lang="ts">
  /**
   * "Filters" tab: extension allow-list, exclude presets/patterns and the
   * regex/size/content predicates applied by the backend filter engine.
   */
  import { EXCLUDE_PRESETS, type FileFilter, type Job } from "$lib/contracts/job";
  import { t } from "$lib/i18n";

  type Props = {
    job: Job;
    filterCount: number;
  };

  let { job = $bindable(), filterCount }: Props = $props();

  // Free-text fields keep their raw text so a trailing comma or a newline does
  // not get swallowed while the user is still typing.
  let extensionsText = $state(job.filters.includeExtensions.join(", "));
  let contentExtensionsText = $state(job.filters.contentExtensions.join(", "));
  let excludePatternsText = $state(job.filters.excludePatterns.join("\n"));

  function splitList(value: string): string[] {
    return value
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }

  function splitLines(value: string): string[] {
    return value
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);
  }

  function sameList(a: string[], b: string[]): boolean {
    return a.length === b.length && a.every((value, index) => value === b[index]);
  }

  // Resync the raw text only when the job changed from the outside (a different
  // job was loaded, or the value was reset). Re-syncing after our own edit would
  // strip a trailing comma the user is still typing.
  $effect(() => {
    const next = job.filters.includeExtensions;
    if (!sameList(splitList(extensionsText), next)) {
      extensionsText = next.join(", ");
    }
  });

  $effect(() => {
    const next = job.filters.contentExtensions;
    if (!sameList(splitList(contentExtensionsText), next)) {
      contentExtensionsText = next.join(", ");
    }
  });

  $effect(() => {
    const next = job.filters.excludePatterns;
    if (!sameList(splitLines(excludePatternsText), next)) {
      excludePatternsText = next.join("\n");
    }
  });

  function patchFilters(partial: Partial<FileFilter>) {
    job = { ...job, filters: { ...job.filters, ...partial } };
  }

  function handleExtensionsInput(value: string) {
    extensionsText = value;
    patchFilters({ includeExtensions: splitList(value) });
  }

  function handleContentExtensionsInput(value: string) {
    contentExtensionsText = value;
    patchFilters({ contentExtensions: splitList(value) });
  }

  function handleExcludePatternsInput(value: string) {
    excludePatternsText = value;
    patchFilters({ excludePatterns: splitLines(value) });
  }

  function togglePreset(id: string) {
    const ids = job.filters.excludePresetIds;
    patchFilters({
      excludePresetIds: ids.includes(id) ? ids.filter((item) => item !== id) : [...ids, id],
    });
  }

  function isoToLocal(iso: string | null | undefined): string {
    if (!iso) return "";
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return "";
    const pad = (value: number) => String(value).padStart(2, "0");
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
  }

  function localToIso(local: string): string | null {
    if (!local) return null;
    const date = new Date(local);
    return Number.isNaN(date.getTime()) ? null : date.toISOString();
  }
</script>

<div class="panel-header">
  <h3>{t("jobs.nav.filters")}</h3>
  <p>
    {t("jobs.filters.desc")}
    {#if filterCount > 0}
      <span class="badge">{t("jobs.filters.activeBadge", { count: filterCount })}</span>
    {/if}
  </p>
</div>

<details class="collapsible" open>
  <summary>{t("jobs.filters.extensions")}</summary>
  <div class="collapsible-body">
    <label class="field">
      <span class="field-label">{t("jobs.filters.includeExtensions")}</span>
      <input
        type="text"
        class="field-input"
        value={extensionsText}
        oninput={(event) => handleExtensionsInput(event.currentTarget.value)}
        placeholder=".pdf, .docx"
      />
    </label>

    <fieldset class="preset-group">
      <legend class="field-label">{t("jobs.filters.excludePresets")}</legend>
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
      <span class="field-label">{t("jobs.filters.excludePatterns")}</span>
      <textarea
        class="field-input textarea"
        rows="4"
        value={excludePatternsText}
        oninput={(event) => handleExcludePatternsInput(event.currentTarget.value)}
        placeholder={"*.tmp\nThumbs.db"}
      ></textarea>
    </label>
  </div>
</details>

<details class="collapsible">
  <summary>{t("jobs.filters.regexSize")}</summary>
  <div class="collapsible-body">
    <label class="field">
      <span class="field-label">{t("jobs.filters.nameRegex")}</span>
      <input
        type="text"
        class="field-input"
        value={job.filters.nameRegex ?? ""}
        oninput={(event) =>
          patchFilters({ nameRegex: event.currentTarget.value || null })}
        placeholder="^backup_.*"
      />
    </label>

    <label class="field">
      <span class="field-label">{t("jobs.filters.pathRegex")}</span>
      <input
        type="text"
        class="field-input"
        value={job.filters.pathRegex ?? ""}
        oninput={(event) =>
          patchFilters({ pathRegex: event.currentTarget.value || null })}
        placeholder={t("jobs.filters.pathRegexPlaceholder")}
      />
    </label>

    <div class="field-row">
      <label class="field">
        <span class="field-label">{t("jobs.filters.minSize")}</span>
        <input
          class="field-input"
          type="number"
          min="0"
          value={job.filters.minSizeBytes ?? ""}
          oninput={(event) =>
            patchFilters({
              minSizeBytes: event.currentTarget.value ? Number(event.currentTarget.value) : null,
            })}
        />
      </label>
      <label class="field">
        <span class="field-label">{t("jobs.filters.maxSize")}</span>
        <input
          class="field-input"
          type="number"
          min="0"
          value={job.filters.maxSizeBytes ?? ""}
          oninput={(event) =>
            patchFilters({
              maxSizeBytes: event.currentTarget.value ? Number(event.currentTarget.value) : null,
            })}
        />
      </label>
    </div>

    <div class="field-row">
      <label class="field">
        <span class="field-label">{t("jobs.filters.maxDepth")}</span>
        <input
          class="field-input"
          type="number"
          min="0"
          value={job.filters.maxDepth ?? ""}
          oninput={(event) =>
            patchFilters({
              maxDepth: event.currentTarget.value ? Number(event.currentTarget.value) : null,
            })}
        />
      </label>
      <label class="field">
        <span class="field-label">{t("jobs.filters.olderThan")}</span>
        <input
          class="field-input"
          type="number"
          min="0"
          value={job.filters.olderThanDays ?? ""}
          oninput={(event) =>
            patchFilters({
              olderThanDays: event.currentTarget.value ? Number(event.currentTarget.value) : null,
            })}
        />
      </label>
    </div>

    <div class="field-row">
      <label class="field">
        <span class="field-label">{t("jobs.filters.modifiedAfter")}</span>
        <input
          class="field-input"
          type="datetime-local"
          value={isoToLocal(job.filters.modifiedAfter)}
          oninput={(event) =>
            patchFilters({ modifiedAfter: localToIso(event.currentTarget.value) })}
        />
      </label>
      <label class="field">
        <span class="field-label">{t("jobs.filters.modifiedBefore")}</span>
        <input
          class="field-input"
          type="datetime-local"
          value={isoToLocal(job.filters.modifiedBefore)}
          oninput={(event) =>
            patchFilters({ modifiedBefore: localToIso(event.currentTarget.value) })}
        />
      </label>
    </div>

    <label class="toggle">
      <input
        type="checkbox"
        checked={job.filters.recursive}
        onchange={(event) => patchFilters({ recursive: event.currentTarget.checked })}
      />
      <span>{t("jobs.filters.recursive")}</span>
    </label>

    <label class="toggle">
      <input
        type="checkbox"
        checked={job.filters.includeHidden}
        onchange={(event) => patchFilters({ includeHidden: event.currentTarget.checked })}
      />
      <span>{t("jobs.filters.includeHidden")}</span>
    </label>

    <label class="toggle">
      <input
        type="checkbox"
        checked={job.filters.skipEmptyFiles}
        onchange={(event) => patchFilters({ skipEmptyFiles: event.currentTarget.checked })}
      />
      <span>{t("jobs.filters.skipEmpty")}</span>
    </label>

    <label class="field">
      <span class="field-label">{t("jobs.filters.symlinkMode")}</span>
      <select
        class="field-input"
        value={job.filters.symlinkMode}
        onchange={(event) =>
          patchFilters({
            symlinkMode: event.currentTarget.value as FileFilter["symlinkMode"],
          })}
      >
        <option value="follow">{t("jobs.filters.symlinkFollow")}</option>
        <option value="copyLink">{t("jobs.filters.symlinkCopy")}</option>
        <option value="skip">{t("jobs.filters.symlinkSkip")}</option>
      </select>
    </label>
  </div>
</details>

<details class="collapsible">
  <summary>{t("jobs.filters.content")}</summary>
  <div class="collapsible-body">
    <label class="field">
      <span class="field-label">{t("jobs.filters.contentContains")}</span>
      <input
        type="text"
        class="field-input"
        value={job.filters.contentContains ?? ""}
        oninput={(event) =>
          patchFilters({ contentContains: event.currentTarget.value || null })}
        placeholder="ERROR"
      />
    </label>

    <label class="field">
      <span class="field-label">{t("jobs.filters.contentExtensions")}</span>
      <input
        type="text"
        class="field-input"
        value={contentExtensionsText}
        oninput={(event) => handleContentExtensionsInput(event.currentTarget.value)}
        placeholder=".txt, .log, .json"
      />
    </label>

    <label class="field">
      <span class="field-label">{t("jobs.filters.contentMaxBytes")}</span>
      <input
        class="field-input"
        type="number"
        min="1"
        value={job.filters.contentMaxBytes}
        oninput={(event) =>
          patchFilters({
            contentMaxBytes: Math.max(1, Number(event.currentTarget.value) || 1),
          })}
      />
    </label>
  </div>
</details>
