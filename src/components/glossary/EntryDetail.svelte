<script lang="ts">
  import { TrashIcon } from '@lucide/svelte';

  import type { GlossaryEntry } from '../../lib/types';

  let {
    mode,
    entry = $bindable(null),
    canCommit = false,
    onCommit = () => {},
    onCancel = () => {},
    onRemove = () => {},
  }: {
    mode: 'new' | 'edit' | 'empty';
    entry?: GlossaryEntry | null;
    canCommit?: boolean;
    onCommit?: () => void;
    onCancel?: () => void;
    onRemove?: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (mode !== 'new') return;
    if (e.key === 'Enter' && (!(e.target instanceof HTMLTextAreaElement) || e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      onCommit();
    }
  }
</script>

<aside class="detail-pane">
  {#if mode === 'empty' || !entry}
    <div class="detail-empty">
      <p>Select an entry to edit</p>
    </div>
  {:else}
    <div class="detail-header">
      <span class="detail-title" class:placeholder={mode === 'new' && !entry.jp.trim()}>
        {#if mode === 'new'}
          {entry.jp.trim() || 'New term'}
        {:else}
          {entry.jp || '—'}
        {/if}
      </span>
      {#if mode === 'edit'}
        <button class="btn-icon delete-btn" onclick={onRemove} title="Delete">
          <TrashIcon size={14} />
        </button>
      {/if}
    </div>

    <div class="detail-form">
      <label class="detail-field">
        <span class="field-label">Japanese</span>
        <input type="text" bind:value={entry.jp} onkeydown={handleKeydown} />
      </label>
      <label class="detail-field">
        <span class="field-label">English</span>
        <input type="text" bind:value={entry.en} onkeydown={handleKeydown} />
      </label>
      <label class="detail-field">
        <span class="field-label">Note</span>
        <textarea rows="3" bind:value={entry.note} onkeydown={handleKeydown}></textarea>
      </label>
    </div>

    {#if mode === 'new'}
      <div class="detail-footer">
        <button onclick={onCancel}>Cancel</button>
        <button class="btn-primary" onclick={onCommit} disabled={!canCommit}>Add term</button>
      </div>
    {/if}
  {/if}
</aside>

<style>
  .detail-pane {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    overflow: hidden;
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: color-mix(in srgb, var(--color-bg) 40%, var(--color-surface));
    border-bottom: 1px solid var(--color-border);

    .detail-title {
      flex: 1;
      font-size: 22px;
      font-weight: 600;

      &.placeholder {
        color: var(--color-text-muted);
      }
    }
  }

  .delete-btn:hover {
    color: var(--color-danger);
  }

  .detail-form {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 14px;
    overflow-y: auto;
  }

  .detail-field {
    display: flex;
    flex-direction: column;
    gap: 6px;

    .field-label {
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--color-text-muted);
    }

    input {
      padding: 8px 10px;
      font-size: 14px;
    }

    textarea {
      padding: 8px 10px;
      font-size: 13px;
      resize: vertical;
      font-family: inherit;
    }
  }

  .detail-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px 14px;
    border-top: 1px solid var(--color-border);
  }

  .detail-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    font-size: 13px;
  }
</style>
