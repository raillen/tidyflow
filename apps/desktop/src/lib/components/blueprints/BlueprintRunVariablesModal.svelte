<script lang="ts">
  import Modal from "$lib/components/ui/Modal.svelte";
  import type { Blueprint } from "$lib/contracts/blueprint";
  import { t } from "$lib/i18n";

  type Props = {
    open: boolean;
    blueprint: Blueprint | null;
    action: "apply" | "simulate";
    loading: boolean;
    onclose: () => void;
    onrun: (variables: Record<string, string>) => void;
  };

  let { open, blueprint, action, loading, onclose, onrun }: Props = $props();

  let values = $state<Record<string, string>>({});

  const variables = $derived(blueprint?.folderPlan?.variables ?? []);
  const actionLabel = $derived(action === "apply" ? t("common.apply") : t("common.simulate"));
  const title = $derived(action === "apply" ? t("blueprints.apply") + " blueprint" : t("blueprints.simulate") + " blueprint");
  const missingRequired = $derived(
    variables
      .filter((variable) => variable.required && !(values[variable.name] ?? "").trim())
      .map((variable) => variable.label || variable.name),
  );

  $effect(() => {
    if (!open || !blueprint) return;
    const next: Record<string, string> = {};
    for (const variable of blueprint.folderPlan?.variables ?? []) {
      next[variable.name] = variable.defaultValue ?? "";
    }
    values = next;
  });

  function setValue(name: string, value: string) {
    values = { ...values, [name]: value };
  }

  function submit() {
    if (loading || missingRequired.length > 0) return;
    onrun({ ...values });
  }
</script>

<Modal {open} {title} description={blueprint?.name} size="md" {onclose}>
  {#if variables.length === 0}
    <p class="muted">{t("run.variablesEmpty")}</p>
  {:else}
    <p class="intro">{t("run.variablesIntro")}</p>
    <div class="fields">
      {#each variables as variable}
        <label class="field">
          <span class="field-label">
            {variable.label || variable.name}
            {#if variable.required}<span class="required" aria-hidden="true">*</span>{/if}
          </span>
          <input
            class="field-input mono"
            value={values[variable.name] ?? ""}
            placeholder={variable.defaultValue || variable.name}
            aria-required={variable.required}
            oninput={(event) => setValue(variable.name, event.currentTarget.value)}
          />
          <span class="field-hint">{t("run.tokenLabel")} <code>{`{${variable.name}}`}</code></span>
        </label>
      {/each}
    </div>

    {#if missingRequired.length > 0}
      <p class="warning" role="alert">
        {t("run.requiredWarning", { fields: missingRequired.join(", ") })}
      </p>
    {/if}
  {/if}

  {#snippet footer()}
    <button type="button" class="btn" onclick={onclose} disabled={loading}>{t("common.cancel")}</button>
    <button
      type="button"
      class="btn primary"
      onclick={submit}
      disabled={loading || missingRequired.length > 0}
    >
      {loading ? t("run.running") : actionLabel}
    </button>
  {/snippet}
</Modal>

<style>
  .intro {
    margin: 0 0 var(--space-4);
    color: var(--text-muted);
  }

  .fields {
    display: grid;
    gap: var(--space-4);
  }

  .field {
    display: grid;
    gap: 0.35rem;
  }

  .field-label {
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .required {
    margin-left: 0.15rem;
    color: var(--danger);
  }

  .field-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .field-hint code {
    font-family: var(--font-mono);
  }

  .warning {
    margin: var(--space-4) 0 0;
    color: var(--danger);
    font-size: var(--text-sm);
  }

  .muted {
    margin: 0;
    color: var(--text-muted);
  }
</style>
