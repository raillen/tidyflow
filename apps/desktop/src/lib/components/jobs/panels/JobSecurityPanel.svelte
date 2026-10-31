<script lang="ts">
  /**
   * "Security" tab: sync strategy, post-copy integrity checks and the
   * encrypted-pack options.
   */
  import type { Job, TransferOptions } from "$lib/contracts/job";
  import { t } from "$lib/i18n";

  type Props = {
    job: Job;
  };

  let { job = $bindable() }: Props = $props();

  function patchOptions(partial: Partial<TransferOptions>) {
    job = { ...job, options: { ...job.options, ...partial } };
  }
</script>

<div class="panel-header">
  <h3>{t("jobs.nav.security")}</h3>
  <p>{t("jobs.security.desc")}</p>
</div>

<details class="collapsible" open>
  <summary>{t("jobs.security.sync")}</summary>
  <div class="collapsible-body">
    <label class="toggle">
      <input
        type="checkbox"
        checked={job.options.smartSync}
        onchange={(event) => patchOptions({ smartSync: event.currentTarget.checked })}
      />
      <span>{t("jobs.security.smartSync")}</span>
    </label>

    <label class="toggle">
      <input
        type="checkbox"
        checked={job.options.strictHashSync}
        onchange={(event) => patchOptions({ strictHashSync: event.currentTarget.checked })}
      />
      <span>{t("jobs.security.strictHash")}</span>
    </label>
  </div>
</details>

<details class="collapsible" open>
  <summary>{t("jobs.security.integrity")}</summary>
  <div class="collapsible-body">
    <label class="toggle">
      <input
        type="checkbox"
        checked={job.options.verifyAfterCopy}
        onchange={(event) => patchOptions({ verifyAfterCopy: event.currentTarget.checked })}
      />
      <span>{t("jobs.security.verify")}</span>
    </label>

    <label class="toggle">
      <input
        type="checkbox"
        checked={job.options.stopOnIntegrityError}
        onchange={(event) =>
          patchOptions({ stopOnIntegrityError: event.currentTarget.checked })}
      />
      <span>{t("jobs.security.stopOnError")}</span>
    </label>
  </div>
</details>

<details class="collapsible">
  <summary>{t("jobs.security.packEncryption")}</summary>
  <div class="collapsible-body">
    <label class="toggle">
      <input
        type="checkbox"
        checked={job.options.encryptOutput}
        onchange={(event) => patchOptions({ encryptOutput: event.currentTarget.checked })}
      />
      <span>{t("jobs.security.encrypt")}</span>
    </label>

    {#if job.options.encryptOutput}
      <label class="field">
        <span class="field-label">{t("jobs.security.password")}</span>
        <input
          class="field-input"
          type="password"
          value={job.options.encryptPassword ?? ""}
          oninput={(event) =>
            patchOptions({ encryptPassword: event.currentTarget.value || null })}
          autocomplete="new-password"
        />
      </label>

      <label class="toggle">
        <input
          type="checkbox"
          checked={job.options.rememberEncryptPassword}
          onchange={(event) =>
            patchOptions({ rememberEncryptPassword: event.currentTarget.checked })}
        />
        <span>{t("jobs.security.rememberPassword")}</span>
      </label>

      <label class="field">
        <span class="field-label">{t("jobs.security.packName")}</span>
        <input
          type="text"
          class="field-input"
          value={job.options.packFilename ?? ""}
          oninput={(event) => patchOptions({ packFilename: event.currentTarget.value || null })}
          placeholder={t("jobs.security.packNamePlaceholder")}
        />
      </label>

      <label class="toggle">
        <input
          type="checkbox"
          checked={job.options.removeFilesAfterPack}
          onchange={(event) =>
            patchOptions({ removeFilesAfterPack: event.currentTarget.checked })}
        />
        <span>{t("jobs.security.removeAfterPack")}</span>
      </label>
    {/if}
  </div>
</details>
