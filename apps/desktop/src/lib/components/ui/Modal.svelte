<script lang="ts">
  import { onMount } from "svelte";
  import { X } from "phosphor-svelte";
  import type { Snippet } from "svelte";
  import { loadModalState, saveModalState, type ModalUiState } from "$lib/core/ipc/ui-state";

  type Props = {
    open: boolean;
    title: string;
    description?: string;
    size?: "sm" | "md" | "lg" | "xl" | "large";
    persistKey?: string;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  };

  let {
    open,
    title,
    description,
    size = "lg",
    persistKey,
    onclose,
    children,
    footer,
  }: Props = $props();

  let dialogEl = $state<HTMLDialogElement | null>(null);
  let panelEl = $state<HTMLDivElement | null>(null);
  let width = $state(0);
  let height = $state(0);
  let resizing = $state<string | null>(null);

  const MIN_W = 520;
  const MIN_H = 360;
  const MAX_RATIO = 0.92;

  onMount(async () => {
    const fallback = {
      width: Math.round(window.innerWidth * 0.78),
      height: Math.round(window.innerHeight * 0.82),
    };
    if (persistKey) {
      const saved = await loadModalState(persistKey);
      width = clamp(saved.width, MIN_W, window.innerWidth * MAX_RATIO);
      height = clamp(saved.height, MIN_H, window.innerHeight * MAX_RATIO);
    } else {
      width = fallback.width;
      height = fallback.height;
    }
  });

  function clamp(v: number, min: number, max: number) {
    return Math.min(Math.max(v, min), max);
  }

  $effect(() => {
    if (!dialogEl) return;
    if (open && !dialogEl.open) dialogEl.showModal();
    else if (!open && dialogEl.open) dialogEl.close();
  });

  function handleDialogClose() {
    onclose();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === dialogEl) onclose();
  }

  function startResize(edge: string, event: MouseEvent) {
    event.preventDefault();
    resizing = edge;
    const startX = event.clientX;
    const startY = event.clientY;
    const startW = width;
    const startH = height;

    function onMove(e: MouseEvent) {
      if (!resizing) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      let w = startW;
      let h = startH;
      if (resizing.includes("e")) w = startW + dx;
      if (resizing.includes("w")) w = startW - dx;
      if (resizing.includes("s")) h = startH + dy;
      if (resizing.includes("n")) h = startH - dy;
      width = clamp(w, MIN_W, window.innerWidth * MAX_RATIO);
      height = clamp(h, MIN_H, window.innerHeight * MAX_RATIO);
    }

    function onUp() {
      resizing = null;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      if (persistKey) {
        void saveModalState(persistKey, { width, height });
      }
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  const panelStyle = $derived(
    size === "large" || size === "xl"
      ? `width:${width}px;height:${height}px;max-width:92vw;max-height:92vh;`
      : undefined,
  );
</script>

<dialog
  bind:this={dialogEl}
  class="modal {size}"
  aria-labelledby="modal-title"
  aria-describedby={description ? "modal-desc" : undefined}
  onclose={handleDialogClose}
  onclick={handleBackdropClick}
>
  <div bind:this={panelEl} class="modal-panel" style={panelStyle}>
    <header class="modal-header">
      <div>
        <h2 id="modal-title">{title}</h2>
        {#if description}
          <p id="modal-desc" class="modal-desc">{description}</p>
        {/if}
      </div>
      <button type="button" class="icon-btn" aria-label="Fechar" onclick={onclose}>
        <X size={16} weight="bold" />
      </button>
    </header>

    <div class="modal-body">
      {@render children()}
    </div>

    {#if footer}
      <footer class="modal-footer">
        {@render footer()}
      </footer>
    {/if}

    {#if size === "large" || size === "xl"}
      {#each ["n", "s", "e", "w", "ne", "nw", "se", "sw"] as edge}
        <button
          type="button"
          class="resize-handle {edge}"
          aria-label="Redimensionar"
          onmousedown={(e) => startResize(edge, e)}
        ></button>
      {/each}
    {/if}
  </div>
</dialog>

<style>
  .modal {
    border: none;
    padding: 0;
    margin: auto;
    background: transparent;
    color: inherit;
  }

  .modal::backdrop {
    background: rgba(10, 12, 16, 0.45);
    backdrop-filter: blur(2px);
  }

  :global(.dark) .modal::backdrop {
    background: rgba(0, 0, 0, 0.62);
  }

  .modal-panel {
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.18);
    overflow: hidden;
  }

  .modal.sm .modal-panel { width: min(24rem, 92vw); max-height: 92vh; }
  .modal.md .modal-panel { width: min(32rem, 92vw); max-height: 92vh; }
  .modal.lg .modal-panel { width: min(42rem, 92vw); max-height: 92vh; }
  .modal.xl .modal-panel,
  .modal.large .modal-panel { min-width: 520px; min-height: 360px; }

  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .modal-header h2 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 650;
  }

  .modal-desc {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .modal-body {
    padding: 0;
    overflow: hidden;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .icon-btn {
    display: grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .resize-handle {
    position: absolute;
    background: transparent;
    border: none;
    padding: 0;
    z-index: 2;
  }

  .n, .s { left: 8px; right: 8px; height: 6px; cursor: ns-resize; }
  .e, .w { top: 8px; bottom: 8px; width: 6px; cursor: ew-resize; }
  .n { top: 0; }
  .s { bottom: 0; }
  .e { right: 0; }
  .w { left: 0; }
  .ne, .nw, .se, .sw { width: 12px; height: 12px; }
  .ne { top: 0; right: 0; cursor: nesw-resize; }
  .nw { top: 0; left: 0; cursor: nwse-resize; }
  .se { bottom: 0; right: 0; cursor: nwse-resize; }
  .sw { bottom: 0; left: 0; cursor: nesw-resize; }
</style>
