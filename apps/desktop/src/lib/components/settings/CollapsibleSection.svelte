<script lang="ts">
  /**
   * Collapsible section used by the job and blueprint editors.
   *
   * Each panel repeats `<details class="collapsible">` with the same chrome, so
   * the markup lives here and callers pass only the summary and body.
   */
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    open?: boolean;
    children: Snippet;
  };

  let { title, open = true, children }: Props = $props();
</script>

<details class="collapsible" {open}>
  <summary>{title}</summary>
  <div class="collapsible-body">
    {@render children()}
  </div>
</details>
