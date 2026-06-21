<script lang="ts">
  type Props = {
    value: number;
    label: string;
    min?: number;
    max?: number;
  };

  let { value = $bindable(), label, min = 0, max }: Props = $props();
</script>

<label class="field">
  <span class="field-label">{label}</span>
  <input class="field-input" type="number" {min} {max} bind:value />
</label>
