<script lang="ts">
  import type { SvelteSet } from 'svelte/reactivity';
  import { SearchIcon, TrashIcon } from '@lucide/svelte';

  import type { GlossaryEntry } from '../../lib/types';

  let {
    entries,
    selectedIndices,
    activeIndex,
    issueIndices,
    searchQuery,
    filterMode,
    tableBody = $bindable(),
    onSelect,
    onRemove,
    onBulkDelete,
    onBulkClearNotes,
  }: {
    entries: Array<{ entry: GlossaryEntry; originalIndex: number }>;
    selectedIndices: SvelteSet<number>;
    activeIndex: number | null;
    issueIndices: SvelteSet<number>;
    searchQuery: string;
    filterMode: 'all' | 'issues';
    tableBody: HTMLDivElement | undefined;
    onSelect: (index: number) => void;
    onRemove: (index: number) => void;
    onBulkDelete: () => void;
    onBulkClearNotes: () => void;
  } = $props();

  let selectAllCheckbox: HTMLInputElement | undefined = $state();
  let someVisibleSelected = $derived(entries.some(({ originalIndex }) => selectedIndices.has(originalIndex)));
  let allVisibleSelected = $derived(
    entries.length > 0 && entries.every(({ originalIndex }) => selectedIndices.has(originalIndex)),
  );

  $effect(() => {
    if (selectAllCheckbox) {
      selectAllCheckbox.indeterminate = someVisibleSelected && !allVisibleSelected;
    }
  });

  function toggleSelectAll() {
    if (allVisibleSelected) {
      for (const { originalIndex } of entries) {
        selectedIndices.delete(originalIndex);
      }
    } else {
      for (const { originalIndex } of entries) {
        selectedIndices.add(originalIndex);
      }
    }
  }

  function toggleSelect(index: number) {
    if (selectedIndices.has(index)) {
      selectedIndices.delete(index);
    } else {
      selectedIndices.add(index);
    }
  }
</script>

<div class="list-pane">
  {#if selectedIndices.size > 0}
    <div class="bulk-bar">
      <input type="checkbox" checked={allVisibleSelected} bind:this={selectAllCheckbox} onchange={toggleSelectAll} />
      <span class="sel-count"><strong>{selectedIndices.size}</strong> selected</span>
      <span class="spacer"></span>
      <button onclick={onBulkClearNotes}>Clear notes</button>
      <button class="bulk-delete" onclick={onBulkDelete}>Delete</button>
    </div>
  {:else}
    <div class="table-head">
      <div class="th col-sel">
        <input type="checkbox" checked={allVisibleSelected} bind:this={selectAllCheckbox} onchange={toggleSelectAll} />
      </div>
      <div class="th col-term">Terms</div>
      <div class="th col-actions"></div>
    </div>
  {/if}

  <div class="table-body" bind:this={tableBody}>
    {#if entries.length === 0}
      <div class="empty-state">
        <SearchIcon size={32} />
        {#if searchQuery}
          <p>No matches for "{searchQuery}"</p>
        {:else if filterMode === 'issues'}
          <p>No issues found.</p>
        {:else}
          <p>No entries yet. Click <strong>New term</strong> to add one.</p>
        {/if}
      </div>
    {:else}
      {#each entries as { entry, originalIndex }}
        <div
          class="table-row"
          class:active={activeIndex === originalIndex}
          class:row-error={issueIndices.has(originalIndex)}
          onclick={() => onSelect(originalIndex)}
        >
          <label class="td col-sel" onclick={(e) => e.stopPropagation()}>
            <input
              type="checkbox"
              checked={selectedIndices.has(originalIndex)}
              onclick={(e) => e.stopPropagation()}
              onchange={() => toggleSelect(originalIndex)}
            />
          </label>
          <div class="td col-term">
            <div class="term-line">
              <span class="term-jp">{entry.jp}</span>
              <span class="term-arrow">→</span>
              <span class="term-en">{entry.en}</span>
            </div>
            {#if entry.note}<div class="term-note">{entry.note}</div>{/if}
          </div>
          <div class="td col-actions">
            <button
              class="btn-icon delete-btn"
              onclick={(e) => {
                e.stopPropagation();
                onRemove(originalIndex);
              }}
              title="Delete"
            >
              <TrashIcon size={14} />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .list-pane {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    overflow: hidden;
  }

  .bulk-bar,
  .table-head {
    min-height: 34px;
    flex-shrink: 0;
  }

  .bulk-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px;
    background: color-mix(in srgb, var(--color-accent) 14%, var(--color-surface));
    border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 40%, var(--color-border));
    font-size: 12px;

    .sel-count {
      color: var(--color-text);
    }

    .spacer {
      flex: 1;
    }

    button {
      padding: 3px 10px;
      font-size: 12px;
    }

    .bulk-delete {
      color: var(--color-error-text);
    }
  }

  .table-head {
    display: grid;
    grid-template-columns: 32px 1fr 40px;
    background: color-mix(in srgb, var(--color-bg) 40%, var(--color-surface));
    border-bottom: 1px solid var(--color-border);
  }

  .th {
    padding: 8px 10px;
    font-weight: 600;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-muted);
    display: flex;
    align-items: center;

    &.col-sel {
      justify-content: center;
    }
  }

  .table-body {
    flex: 1;
    overflow-y: auto;
  }

  .table-row {
    display: grid;
    grid-template-columns: 32px 1fr 40px;
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background 0.1s;
    position: relative;

    &:last-child {
      border-bottom: none;
    }

    &:hover {
      background: color-mix(in srgb, var(--color-accent) 5%, transparent);
    }

    &.active {
      background: color-mix(in srgb, var(--color-accent) 16%, transparent);
      box-shadow: inset 2px 0 0 var(--color-accent);
    }

    &.row-error {
      background: color-mix(in srgb, var(--color-danger) 12%, transparent);

      &::before {
        content: '';
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        width: 2px;
        background: var(--color-danger);
      }
    }
  }

  .td {
    padding: 6px;
    display: flex;
    align-items: center;
    min-width: 0;

    &.col-sel {
      padding-left: 10px;
      justify-content: center;
      cursor: pointer;
      align-self: stretch;
    }

    &.col-term {
      flex-direction: column;
      align-items: flex-start;
      justify-content: center;
      gap: 2px;
      padding: 10px;
    }

    &.col-actions {
      justify-content: center;
    }
  }

  .delete-btn:hover {
    color: var(--color-danger);
  }

  .term-line {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .term-jp {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text);
    flex-shrink: 0;
  }

  .term-arrow {
    font-size: 11px;
    color: var(--color-text-disabled);
    flex-shrink: 0;
  }

  .term-en {
    font-size: 13px;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .term-note {
    font-size: 11px;
    color: var(--color-text-disabled);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 20px;
    color: var(--color-text-muted);
    font-size: 13px;
    text-align: center;

    :global(svg) {
      color: var(--color-text-disabled);
      margin-bottom: 8px;
    }

    strong {
      color: var(--color-text);
    }
  }

  input[type='checkbox'] {
    appearance: none;
    width: 14px;
    height: 14px;
    border: 1px solid var(--color-text-disabled);
    background: var(--color-input-bg);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;

    &:checked {
      background: var(--color-accent);
      border-color: var(--color-accent);

      &::after {
        content: '';
        width: 3px;
        height: 7px;
        border: solid #fff;
        border-width: 0 1px 1px 0;
        transform: rotate(45deg) translate(-1px, -1px);
      }
    }

    &:indeterminate {
      background:
        linear-gradient(#fff, #fff) center / 8px 2px no-repeat,
        var(--color-accent);
      border-color: var(--color-accent);
    }
  }
</style>
