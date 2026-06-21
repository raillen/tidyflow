<script lang="ts">
  import type { Component } from "svelte";

  type Props = {
    icon: Component;
    title: string;
    text: string;
  };

  let { icon: Icon, title, text }: Props = $props();
</script>

<div class="section-head">
  <Icon size={18} weight="duotone" aria-hidden="true" />
  <div>
    <h2>{title}</h2>
    <p>{text}</p>
  </div>
</div>

<style>
  .section-head {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    color: var(--accent);
  }

  h2 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-primary);
  }

  p {
    margin: 2px 0 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
