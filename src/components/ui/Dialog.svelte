<script lang="ts">
  import { XIcon } from '@lucide/svelte';
  import type { Snippet } from 'svelte';

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

  let dialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialogEl) return;
    if (visible && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!visible && dialogEl.open) {
      dialogEl.close();
    }
  });

  function close() {
    visible = false;
    onClose?.();
  }

  function handleCancel(e: Event) {
    e.preventDefault();
    close();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === dialogEl) close();
  }
</script>

<dialog bind:this={dialogEl} oncancel={handleCancel} onclick={handleBackdropClick} aria-label={title}>
  <div class="dialog-inner">
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
</dialog>

<style>
  dialog {
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    max-width: none;
    max-height: none;
    overflow: visible;
    margin: auto;
  }

  dialog::backdrop {
    background: rgba(0, 0, 0, 0.5);
  }

  .dialog-inner {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 20px;
    min-width: 320px;
    max-width: 520px;
    box-shadow: var(--shadow-dialog);
    user-select: none;
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
