<script lang="ts">
  import { Blueprint, Sparkle } from "phosphor-svelte";
</script>

<section class="page">
  <header class="page-header">
    <div>
      <h1 class="page-title">Blueprints</h1>
      <p class="page-desc">Organização e renomeação automática — Fase 3.</p>
    </div>
  </header>

  <div class="empty panel">
    <Blueprint size={32} weight="duotone" aria-hidden="true" />
    <p>Em breve</p>
    <span class="muted">
      Templates de pastas, tokenizer e scaffolding chegam na próxima fase.
    </span>
    <span class="chip">
      <Sparkle size={12} weight="fill" />
      Fase 3
    </span>
  </div>
</section>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6);
    max-width: 28rem;
    text-align: center;
    color: var(--accent);
  }

  .empty p {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-primary);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    margin-top: var(--space-2);
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: var(--text-xs);
    font-weight: 600;
  }
</style>
