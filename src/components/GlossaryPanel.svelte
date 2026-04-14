<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import { ArrowLeftIcon, PlusIcon, TrashIcon } from '@lucide/svelte';

  import type { GlossaryEntry } from '../lib/types';

  let {
    glossary,
    onBack,
    onSave,
  }: {
    glossary: GlossaryEntry[];
    onBack: () => void;
    onSave: (glossary: GlossaryEntry[]) => void;
  } = $props();

  // eslint-disable-next-line svelte/prefer-writable-derived
  let draft: GlossaryEntry[] = $state([]);
  $effect(() => {
    draft = glossary.map((e) => ({ ...e }));
  });

  let duplicateIndices = $derived.by(() => {
    const seen: Record<string, number> = {};
    const dupes = new SvelteSet<number>();
    for (let i = 0; i < draft.length; i++) {
      const jp = draft[i].jp.trim();
      if (!jp) continue;
      if (jp in seen) {
        dupes.add(seen[jp]);
        dupes.add(i);
      } else {
        seen[jp] = i;
      }
    }
    return dupes;
  });

  let hasChanges = $derived.by(() => {
    if (draft.length !== glossary.length) return true;
    for (let i = 0; i < draft.length; i++) {
      if (draft[i].jp !== glossary[i].jp) return true;
      if (draft[i].en !== glossary[i].en) return true;
      if ((draft[i].note ?? '') !== (glossary[i].note ?? '')) return true;
    }
    return false;
  });

  let newJp = $state('');
  let newEn = $state('');
  let newNote = $state('');

  let newRowPending = $derived(newJp.trim() !== '' || newEn.trim() !== '' || newNote.trim() !== '');
  let newRowComplete = $derived(newJp.trim() !== '' && newEn.trim() !== '');
  let newRowValid = $derived(!newRowPending || newRowComplete);

  let canSave = $derived(
    (hasChanges || newRowComplete) &&
      newRowValid &&
      duplicateIndices.size === 0 &&
      draft.every((e) => e.jp.trim() && e.en.trim()),
  );

  function commitNewRow() {
    draft.push({ jp: newJp.trim(), en: newEn.trim(), note: newNote.trim() || undefined });
    newJp = newEn = newNote = '';
  }

  function handleNewRowKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && newRowComplete) {
      e.preventDefault();
      commitNewRow();
    }
  }

  function removeEntry(index: number) {
    draft.splice(index, 1);
  }

  function save() {
    if (newRowComplete) commitNewRow();
    if (!canSave) return;
    const cleaned = draft.map((e) => {
      const entry: GlossaryEntry = { jp: e.jp.trim(), en: e.en.trim() };
      if (e.note?.trim()) entry.note = e.note.trim();
      return entry;
    });
    onSave(cleaned);
  }
</script>

<div class="glossary-view">
  <div class="glossary-header">
    <button class="back-btn" onclick={onBack}>
      <ArrowLeftIcon size={16} />
      <span>Back</span>
    </button>
    <div class="header-spacer"></div>
    <button class="btn-primary save-btn" onclick={save} disabled={!canSave}>Save</button>
  </div>

  <div class="glossary-body">
    <h1>Glossary</h1>
    <p class="glossary-hint">
      Japanese terms and their English translations. Matching terms are highlighted in the editor.
    </p>

    <div class="glossary-table">
      <div class="table-head">
        <div class="th col-jp">Japanese</div>
        <div class="th col-en">English</div>
        <div class="th col-note">Note</div>
        <div class="th col-actions"></div>
      </div>
      <div class="table-body">
        {#each draft as entry, i}
          <div class="table-row" class:row-error={duplicateIndices.has(i)}>
            <div class="td col-jp">
              <input type="text" bind:value={entry.jp} />
            </div>
            <div class="td col-en">
              <input type="text" bind:value={entry.en} />
            </div>
            <div class="td col-note">
              <input type="text" bind:value={entry.note} />
            </div>
            <div class="td col-actions">
              <button class="btn-icon delete-btn" onclick={() => removeEntry(i)} title="Remove entry">
                <TrashIcon size={14} />
              </button>
            </div>
          </div>
        {/each}
        <div class="table-row">
          <div class="td col-jp">
            <input type="text" bind:value={newJp} onkeydown={handleNewRowKeydown} />
          </div>
          <div class="td col-en">
            <input type="text" bind:value={newEn} onkeydown={handleNewRowKeydown} />
          </div>
          <div class="td col-note">
            <input type="text" bind:value={newNote} onkeydown={handleNewRowKeydown} />
          </div>
          <div class="td col-actions">
            <button class="btn-icon add-btn" onclick={commitNewRow} disabled={!newRowComplete} title="Add entry">
              <PlusIcon size={14} />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .glossary-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
  }

  .glossary-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);

    button {
      font-weight: 600;
    }
  }

  .header-spacer {
    flex: 1;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    font-size: 13px;
  }

  .save-btn {
    padding: 4px 12px;
    font-size: 13px;
  }

  .glossary-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 28px 32px;

    h1 {
      font-size: 20px;
      font-weight: 600;
      color: var(--color-text);
      margin-bottom: 8px;
    }
  }

  .glossary-hint {
    font-size: 13px;
    color: var(--color-text-muted);
    margin-bottom: 20px;
    line-height: 1.4;
  }

  .glossary-table {
    border: 1px solid var(--color-border);
    border-radius: 4px;
    overflow: hidden;
  }

  .table-head {
    display: flex;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }

  .th {
    padding: 8px 10px;
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .table-body {
    max-height: 600px;
    overflow-y: auto;
  }

  .table-row {
    display: flex;
    border-bottom: 1px solid var(--color-border);

    &:last-child {
      border-bottom: none;
    }

    &.row-error {
      background: var(--color-error-bg);
    }
  }

  .td {
    padding: 4px 6px;

    input {
      width: 100%;
      padding: 4px 6px;
      font-size: 13px;
    }
  }

  .col-jp {
    flex: 2;
    min-width: 0;
  }

  .col-en {
    flex: 3;
    min-width: 0;
  }

  .col-note {
    flex: 3;
    min-width: 0;
  }

  .col-actions {
    width: 40px;
    display: flex;
    align-items: stretch;
    flex-shrink: 0;

    .btn-icon {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }

  .add-btn {
    color: var(--color-text-muted);

    &:hover:not(:disabled) {
      color: var(--color-accent);
    }
  }

  .delete-btn {
    color: var(--color-text-muted);

    &:hover {
      color: var(--color-danger);
    }
  }
</style>
