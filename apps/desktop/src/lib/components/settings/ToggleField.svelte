<script lang="ts">
  type Props = {
    checked: boolean;
    label: string;
  };

  let { checked = $bindable(), label }: Props = $props();
</script>

<label class="toggle">
  <input type="checkbox" bind:checked />
  <span>{label}</span>
</label>

<style>
  .toggle {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }
</style>
