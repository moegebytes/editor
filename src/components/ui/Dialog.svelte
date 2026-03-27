<script lang="ts">
  import { XIcon } from "@lucide/svelte";
  import type { Snippet } from "svelte";

  let {
    visible = $bindable(false),
    title,
    onClose,
    children,
    actions,
  }: {
    visible?: boolean;
    title: string;
    onClose?: () => void;
    children: Snippet;
    actions?: Snippet;
  } = $props();

  function close() {
    visible = false;
    onClose?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && visible && !e.defaultPrevented) {
      e.preventDefault();
      close();
    }
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if visible}
  <div class="overlay" onclick={close} role="none">
    <div class="dialog" onclick={(e) => e.stopPropagation()} role="none">
      <div class="dialog-header">
        <h2>{title}</h2>
        <button class="btn-icon dialog-close" onclick={close}>
          <XIcon size={16} />
        </button>
      </div>

      <div class="dialog-body">
        {@render children()}
      </div>

      {#if actions}
        <div class="dialog-actions">
          {@render actions()}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 20px;
    min-width: 320px;
    max-width: 520px;
    box-shadow: var(--shadow-dialog);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;

    h2 {
      font-size: 16px;
      color: var(--color-text);
    }

    .dialog-close {
      padding: 4px;
    }
  }

  .dialog-body {
    margin-bottom: 16px;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
</style>
