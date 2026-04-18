<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import { BookOpenIcon, TriangleAlertIcon } from '@lucide/svelte';

  import type { GlossaryEntry } from '../lib/types';

  import EntryDetail from './glossary/EntryDetail.svelte';
  import GlossaryList from './glossary/GlossaryList.svelte';
  import GlossaryToolbar from './glossary/GlossaryToolbar.svelte';
  import AskDialog from './ui/AskDialog.svelte';
  import ViewHeader from './ui/ViewHeader.svelte';

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

  let searchQuery = $state('');
  let filterMode: 'all' | 'issues' = $state('all');
  let selectedIndices = new SvelteSet<number>();
  let activeIndex: number | null = $state(null);
  let isNewMode = $state(false);
  let newDraft: GlossaryEntry = $state({ jp: '', en: '', note: '' });

  let tableBody: HTMLDivElement | undefined = $state();
  let showDiscardDialog = $state(false);

  function isValidEntry(e: { jp: string; en: string }): boolean {
    return e.jp.trim() !== '' && e.en.trim() !== '';
  }

  let duplicates = $derived.by(() => {
    const seen: Record<string, number> = {};
    const indices = new SvelteSet<number>();
    const groups = new SvelteSet<string>();
    for (let i = 0; i < draft.length; i++) {
      const jp = draft[i].jp.trim();
      if (!jp) continue;
      if (jp in seen) {
        indices.add(seen[jp]);
        indices.add(i);
        groups.add(jp);
      } else {
        seen[jp] = i;
      }
    }
    return { indices, conflictCount: groups.size };
  });

  let duplicateIndices = $derived(duplicates.indices);
  let conflictCount = $derived(duplicates.conflictCount);

  let invalidIndices = $derived.by(() => {
    const indices = new SvelteSet<number>();
    for (let i = 0; i < draft.length; i++) {
      if (!isValidEntry(draft[i])) indices.add(i);
    }
    return indices;
  });
  let invalidCount = $derived(invalidIndices.size);

  let issueIndices = $derived.by(() => {
    const combined = new SvelteSet<number>();
    for (const i of duplicateIndices) combined.add(i);
    for (const i of invalidIndices) combined.add(i);
    return combined;
  });
  let issueCount = $derived(issueIndices.size);

  let filteredEntries = $derived.by(() => {
    let entries = draft.map((e, i) => ({ entry: e, originalIndex: i }));

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      entries = entries.filter(
        ({ entry }) =>
          entry.jp.toLowerCase().includes(q) ||
          entry.en.toLowerCase().includes(q) ||
          (entry.note ?? '').toLowerCase().includes(q),
      );
    }

    if (filterMode === 'issues') {
      entries = entries.filter(({ originalIndex }) => issueIndices.has(originalIndex));
    }

    return entries;
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

  let canAddNew = $derived(isValidEntry(newDraft));

  let hasConflicts = $derived(conflictCount > 0);
  let hasInvalid = $derived(invalidCount > 0);
  let canSave = $derived(hasChanges && !hasConflicts && !hasInvalid);

  let discardReason = $derived(
    hasConflicts && hasInvalid
      ? 'unresolved conflicts and incomplete entries'
      : hasConflicts
        ? 'unresolved conflicts'
        : 'incomplete entries',
  );

  function startNewEntry() {
    isNewMode = true;
    activeIndex = null;
    newDraft = { jp: '', en: '', note: '' };
  }

  function cancelNewEntry() {
    isNewMode = false;
    newDraft = { jp: '', en: '', note: '' };
  }

  async function commitNewEntry() {
    if (!canAddNew) return;
    draft.push({ jp: newDraft.jp.trim(), en: newDraft.en.trim(), note: newDraft.note?.trim() || undefined });
    activeIndex = draft.length - 1;
    isNewMode = false;
    newDraft = { jp: '', en: '', note: '' };
    searchQuery = '';
    filterMode = 'all';
    await tick();
    tableBody?.querySelector('.table-row.active')?.scrollIntoView({ block: 'nearest' });
  }

  function selectEntry(index: number) {
    if (isNewMode) {
      isNewMode = false;
      newDraft = { jp: '', en: '', note: '' };
    }
    activeIndex = index;
  }

  function deleteEntries(indices: Set<number>) {
    const sorted = [...indices].sort((a, b) => a - b);
    const shift = (i: number) => i - sorted.filter((d) => d < i).length;
    draft = draft.filter((_, i) => !indices.has(i));
    if (activeIndex !== null) {
      activeIndex = indices.has(activeIndex) ? null : shift(activeIndex);
    }
    const remapped = [...selectedIndices].filter((i) => !indices.has(i)).map(shift);
    selectedIndices.clear();
    for (const i of remapped) selectedIndices.add(i);
  }

  function removeEntry(index: number) {
    deleteEntries(new Set([index]));
  }

  function bulkDelete() {
    deleteEntries(new Set(selectedIndices));
  }

  function bulkClearNotes() {
    for (const i of selectedIndices) {
      if (draft[i]) draft[i].note = undefined;
    }
  }

  function handleBack() {
    if (hasChanges && !canSave) {
      showDiscardDialog = true;
      return;
    }
    if (canSave) {
      const cleaned = draft.map((e) => {
        const entry: GlossaryEntry = { jp: e.jp.trim(), en: e.en.trim() };
        if (e.note?.trim()) entry.note = e.note.trim();
        return entry;
      });
      onSave(cleaned);
    }
    onBack();
  }

  function discardAndBack() {
    showDiscardDialog = false;
    onBack();
  }
</script>

<div class="glossary-view">
  <ViewHeader onBack={handleBack}>
    <BookOpenIcon size={14} />
    <strong>Glossary</strong>
    {#snippet actions()}
      <div class="count-pill">
        <span><strong>{draft.length}</strong> terms</span>
        {#if conflictCount > 0}
          <span class="issue">{conflictCount} conflict{conflictCount === 1 ? '' : 's'}</span>
        {/if}
        {#if invalidCount > 0}
          <span class="issue">{invalidCount} incomplete</span>
        {/if}
      </div>
    {/snippet}
  </ViewHeader>

  <div class="glossary-body">
    <GlossaryToolbar
      bind:searchQuery
      bind:filterMode
      total={draft.length}
      {issueCount}
      disableNew={isNewMode}
      onNew={startNewEntry}
    />

    <div class="content-panes">
      <GlossaryList
        entries={filteredEntries}
        {selectedIndices}
        {activeIndex}
        {issueIndices}
        {searchQuery}
        {filterMode}
        bind:tableBody
        onSelect={selectEntry}
        onRemove={removeEntry}
        onBulkDelete={bulkDelete}
        onBulkClearNotes={bulkClearNotes}
      />

      {#if isNewMode}
        <EntryDetail
          mode="new"
          bind:entry={newDraft}
          canCommit={canAddNew}
          onCommit={commitNewEntry}
          onCancel={cancelNewEntry}
        />
      {:else if activeIndex !== null}
        <EntryDetail
          mode="edit"
          bind:entry={draft[activeIndex]}
          onRemove={() => activeIndex !== null && removeEntry(activeIndex)}
        />
      {:else}
        <EntryDetail mode="empty" />
      {/if}
    </div>
  </div>
</div>

<AskDialog bind:visible={showDiscardDialog} title="Discard changes?">
  {#snippet icon()}
    <TriangleAlertIcon size={24} color="var(--color-warning)" />
  {/snippet}

  {#snippet message()}
    <p>Your glossary has {discardReason} that prevent saving. Going back will discard all your changes.</p>
  {/snippet}

  {#snippet actions()}
    <button onclick={() => (showDiscardDialog = false)}>Cancel</button>
    <button class="btn btn-danger" onclick={discardAndBack}>Discard</button>
  {/snippet}
</AskDialog>

<style>
  .glossary-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
  }

  .count-pill {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 11px;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;

    strong {
      color: var(--color-text);
    }

    .issue {
      color: var(--color-error-text);
    }
  }

  .glossary-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 14px 24px 16px;
    gap: 10px;
  }

  .content-panes {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(280px, 400px) 1fr;
    gap: 12px;
    min-height: 0;
  }
</style>
