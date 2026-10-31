<script lang="ts">
  /**
   * "General" tab of the job editor: identity, folders and the copy/move mode.
   */
  import { ArrowsLeftRight, Copy, FolderOpen } from "phosphor-svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Job } from "$lib/contracts/job";
  import { t } from "$lib/i18n";

  type Props = {
    job: Job;
  };

  let { job = $bindable() }: Props = $props();

  async function pickFolder(field: "sourcePath" | "targetPath") {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      job = { ...job, [field]: selected };
    }
  }
</script>

<div class="panel-header">
  <h3>{t("jobs.nav.general")}</h3>
  <p>{t("jobs.general.desc")}</p>
</div>

<details class="collapsible" open>
  <summary>{t("jobs.general.identification")}</summary>
  <div class="collapsible-body">
    <label class="field">
      <span class="field-label">{t("jobs.general.name")}</span>
      <input class="field-input" type="text" bind:value={job.name} maxlength="120" required />
    </label>

    <label class="toggle">
      <input type="checkbox" bind:checked={job.enabled} />
      <span>{t("jobs.general.active")}</span>
    </label>
  </div>
</details>

<details class="collapsible" open>
  <summary>{t("jobs.general.folders")}</summary>
  <div class="collapsible-body">
    <div class="path-row">
      <label class="field">
        <span class="field-label">{t("jobs.general.source")}</span>
        <input
          class="field-input"
          type="text"
          bind:value={job.sourcePath}
          placeholder={t("jobs.general.sourcePlaceholder")}
          required
        />
      </label>
      <button type="button" class="btn" onclick={() => pickFolder("sourcePath")}>
        <FolderOpen size={14} />
        {t("jobs.general.choose")}
      </button>
    </div>

    <div class="path-row">
      <label class="field">
        <span class="field-label">{t("jobs.general.target")}</span>
        <input
          class="field-input"
          type="text"
          bind:value={job.targetPath}
          placeholder={t("jobs.general.targetPlaceholder")}
          required
        />
      </label>
      <button type="button" class="btn" onclick={() => pickFolder("targetPath")}>
        <FolderOpen size={14} />
        {t("jobs.general.choose")}
      </button>
    </div>
  </div>
</details>

<details class="collapsible" open>
  <summary>{t("jobs.general.operation")}</summary>
  <div class="collapsible-body">
    <div class="subsection">
      <span class="field-label">{t("jobs.general.mode")}</span>
      <div class="option-grid">
        <button
          type="button"
          class="option-btn"
          class:selected={job.mode === "copy"}
          aria-pressed={job.mode === "copy"}
          onclick={() => (job = { ...job, mode: "copy" })}
        >
          <Copy size={16} weight={job.mode === "copy" ? "fill" : "regular"} />
          {t("jobs.general.copy")}
        </button>
        <button
          type="button"
          class="option-btn"
          class:selected={job.mode === "move"}
          aria-pressed={job.mode === "move"}
          onclick={() => (job = { ...job, mode: "move" })}
        >
          <ArrowsLeftRight size={16} weight={job.mode === "move" ? "fill" : "regular"} />
          {t("jobs.general.move")}
        </button>
      </div>
    </div>

    <label class="field">
      <span class="field-label">{t("jobs.general.conflict")}</span>
      <select
        class="field-input"
        value={job.conflict}
        onchange={(e) => (job = { ...job, conflict: e.currentTarget.value as Job["conflict"] })}
      >
        <option value="skip">{t("jobs.general.conflictSkip")}</option>
        <option value="overwrite">{t("jobs.general.conflictOverwrite")}</option>
        <option value="rename">{t("jobs.general.conflictRename")}</option>
      </select>
    </label>
  </div>
</details>
