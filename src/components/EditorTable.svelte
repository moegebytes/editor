<script lang="ts">
  import { tick } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import { StickyNoteIcon, PlusIcon, ChevronRightIcon, ChevronDownIcon, ArrowRightIcon } from '@lucide/svelte';

  import type { FlatEntry, GlossaryEntry } from '../lib/types';
  import { splitByMatches } from '../lib/segment';
  import { isText, isUntranslated, getFileName, modKey } from '../lib/utils';

  import UnsavedChangesDialog from './UnsavedChangesDialog.svelte';

  let {
    entries,
    onEnTextChange,
    selectedIndex = $bindable(-1),
    onSave,
    onJpSelect,
    onToggleConfirm,
    onNotesChange,
    onJumpNextUnconfirmed,
    glossary = [],
    confirmedLines = new Set(),
    dirtyIndices = new Map(),
    autoConfirmOnEnter = false,
    filterText = '',
    findQuery = '',
    findMatchIndices = [],
    currentFindMatch = -1,
  }: {
    entries: FlatEntry[];
    onEnTextChange: (index: number, newText: string) => void;
    selectedIndex?: number;
    onSave?: () => void;
    onJpSelect?: (text: string) => void;
    onToggleConfirm?: (index: number, grouped?: boolean) => void;
    onNotesChange?: (index: number, notes: string[]) => void;
    onJumpNextUnconfirmed?: () => void;
    glossary?: GlossaryEntry[];
    confirmedLines?: Set<number>;
    dirtyIndices?: Map<number, number>;
    autoConfirmOnEnter?: boolean;
    filterText?: string;
    findQuery?: string;
    findMatchIndices?: number[];
    currentFindMatch?: number;
  } = $props();

  const ROW_HEIGHT = 34;

  let scrollElement: HTMLDivElement | undefined = $state();
  let collapsedIncludes = new SvelteSet<number>();
  let lastScrolledTo = -1;
  let skipAutoFocus = false;

  // Notes
  let expandedNotes: number | null = $state(null);
  let notesDraft: string = $state('');
  let discardNotesVisible = $state(false);
  let pendingNotesTarget: number | null = $state(null);

  let filterLower = $derived(filterText.toLowerCase());

  let includeNames = $derived(
    new Set(entries.filter((e) => e.entryType === 'include').map((e) => getFileName(e.jpText ?? e.enText ?? ''))),
  );

  let visibleEntries = $derived.by(() => {
    const result: FlatEntry[] = [];
    let skipUntilDepth = -1;

    for (const entry of entries) {
      if (skipUntilDepth >= 0) {
        if (entry.depth > skipUntilDepth) continue;
        skipUntilDepth = -1;
      }

      if (entry.entryType === 'emit' || entry.entryType === 'blank') continue;
      if (entry.entryType === 'reference' && filterLower) continue;

      if (filterLower && isText(entry)) {
        const jp = (entry.jpText ?? '').toLowerCase();
        const en = (entry.enText ?? '').toLowerCase();
        if (!jp.includes(filterLower) && !en.includes(filterLower)) continue;
      }

      result.push(entry);

      if (entry.entryType === 'include' && collapsedIncludes.has(entry.index)) {
        skipUntilDepth = entry.depth;
      }
    }

    return result;
  });

  let virtualizer = $derived(
    createVirtualizer({
      count: visibleEntries.length,
      getScrollElement: scrollElement ? () => scrollElement as Element : () => null,
      estimateSize: () => ROW_HEIGHT,
      overscan: 20,
    }),
  );

  $effect(() => {
    if (selectedIndex >= 0 && selectedIndex !== lastScrolledTo) {
      lastScrolledTo = selectedIndex;
      const visIdx = visibleEntries.findIndex((e) => e.index === selectedIndex);
      if (visIdx >= 0) {
        $virtualizer.scrollToIndex(visIdx, { align: 'auto' });
        if (!skipAutoFocus) {
          requestAnimationFrame(() => {
            const row = scrollElement?.querySelector(`[data-entry-index="${selectedIndex}"] input`);
            if (row instanceof HTMLInputElement) row.focus();
          });
        }
      }
      skipAutoFocus = false;
    }
  });

  type MeasureParams = { virt: { measureElement: (el: HTMLElement) => void }; measure: boolean };
  function measureRow(node: HTMLElement, params: MeasureParams) {
    if (params.measure) params.virt.measureElement(node);
    return {
      update(params: MeasureParams) {
        if (params.measure) params.virt.measureElement(node);
      },
    };
  }

  function childCount(includeIndex: number): number {
    const includeEntry = entries[includeIndex];
    let count = 0;
    for (let i = includeIndex + 1; i < entries.length; i++) {
      if (entries[i].depth <= includeEntry.depth) break;
      if (isText(entries[i])) count++;
    }
    return count;
  }

  function toggleCollapse(entryIndex: number) {
    if (collapsedIncludes.has(entryIndex)) {
      collapsedIncludes.delete(entryIndex);
    } else {
      collapsedIncludes.add(entryIndex);
    }
  }

  function isRefBroken(refPath: string): boolean {
    return !includeNames.has(getFileName(refPath));
  }

  async function jumpToInclude(refPath: string) {
    const refName = getFileName(refPath);
    const target = entries.find((e) => {
      if (e.entryType !== 'include') return false;
      const incPath = e.jpText ?? e.enText ?? '';
      return getFileName(incPath) === refName || incPath === refPath;
    });
    if (!target) return;

    // Walk backwards from the target to find all ancestor includes that are collapsed.
    // An ancestor include is one that appears before the target at a strictly lower depth
    // and whose child range spans the target.
    let needsUncollapse = false;
    for (let i = target.index - 1; i >= 0; i--) {
      const entry = entries[i];
      if (entry.entryType !== 'include') continue;
      if (entry.depth >= target.depth) continue;
      if (collapsedIncludes.has(entry.index)) {
        collapsedIncludes.delete(entry.index);
        needsUncollapse = true;
      }
      if (entry.depth === 0) break;
    }

    if (needsUncollapse) await tick();

    const visIdx = visibleEntries.findIndex((e) => e.index === target.index);
    if (visIdx >= 0) {
      selectedIndex = target.index;
      $virtualizer.scrollToIndex(visIdx, { align: 'center' });
    }
  }

  function hasUnsavedNotes(): boolean {
    if (expandedNotes === null) return false;
    const entry = entries.find((e) => e.index === expandedNotes);
    const saved = entry?.notes.join('\n') ?? '';
    return notesDraft !== saved;
  }

  function openNotes(entryIndex: number) {
    expandedNotes = entryIndex;
    const entry = entries.find((e) => e.index === entryIndex);
    notesDraft = entry?.notes.join('\n') ?? '';
    requestAnimationFrame(() => {
      const textarea = scrollElement?.querySelector(`[data-entry-index="${entryIndex}"] .notes-textarea`);
      if (textarea instanceof HTMLTextAreaElement) textarea.focus();
    });
  }

  function toggleNotes(entryIndex: number) {
    const closing = expandedNotes === entryIndex;
    if (hasUnsavedNotes()) {
      pendingNotesTarget = closing ? null : entryIndex;
      discardNotesVisible = true;
      return;
    }
    if (closing) {
      expandedNotes = null;
    } else {
      openNotes(entryIndex);
    }
  }

  function handleSaveAndSwitchNotes() {
    if (expandedNotes !== null) saveNotes(expandedNotes);
    if (pendingNotesTarget !== null) {
      openNotes(pendingNotesTarget);
    }
    pendingNotesTarget = null;
  }

  function handleDiscardNotes() {
    expandedNotes = null;
    if (pendingNotesTarget !== null) {
      openNotes(pendingNotesTarget);
    }
    pendingNotesTarget = null;
  }

  function saveNotes(entryIndex: number) {
    const lines = notesDraft.split('\n').filter((l) => l.trim() !== '');
    onNotesChange?.(entryIndex, lines);
    expandedNotes = null;
  }

  function rowClass(entry: FlatEntry): string {
    if (!isText(entry)) return 'row-structural';
    if (isUntranslated(entry)) return 'row-untranslated';
    if (confirmedLines.has(entry.index)) return 'row-confirmed';
    if (entry.jpText && entry.enText) return 'row-translated';
    return '';
  }

  function depthPadding(depth: number): string {
    return depth > 0 ? `${depth * 16}px` : '0';
  }

  function handleInput(index: number, event: Event & { currentTarget: HTMLInputElement }) {
    onEnTextChange(index, event.currentTarget.value);
  }

  function handleRowClick(entryIndex: number) {
    skipAutoFocus = true;
    selectedIndex = entryIndex;
  }

  function handleJpMouseUp(e: MouseEvent) {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !onJpSelect) return;
    const cell = e.currentTarget as HTMLElement;
    if (!cell.contains(sel.anchorNode) || !cell.contains(sel.focusNode)) return;
    const text = sel.toString().trim();
    if (text) onJpSelect(text);
  }

  function visibleIndexOf(entryIndex: number): number {
    return visibleEntries.findIndex((e) => e.index === entryIndex);
  }

  function findEditableRow(fromVisible: number, direction: 1 | -1): number {
    let i = fromVisible + direction;
    while (i >= 0 && i < visibleEntries.length) {
      if (isText(visibleEntries[i])) return i;
      i += direction;
    }
    return -1;
  }

  function findUntranslatedRow(fromVisible: number, direction: 1 | -1): number {
    let i = fromVisible + direction;
    while (i >= 0 && i < visibleEntries.length) {
      if (isUntranslated(visibleEntries[i])) return i;
      i += direction;
    }
    return -1;
  }

  function scrollToVisibleRow(visibleIdx: number) {
    if (visibleIdx < 0 || visibleIdx >= visibleEntries.length) return;
    selectedIndex = visibleEntries[visibleIdx].index;
    $virtualizer.scrollToIndex(visibleIdx, { align: 'auto' });
    requestAnimationFrame(() => {
      const row = scrollElement?.querySelector(`[data-entry-index="${selectedIndex}"] input`);
      if (row instanceof HTMLInputElement) row.focus();
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    const currentVisible = visibleIndexOf(selectedIndex);
    const isEnFocused =
      document.activeElement instanceof HTMLInputElement && scrollElement?.contains(document.activeElement);

    const mod = modKey(e);

    if (mod && e.key === 's') {
      e.preventDefault();
      onSave?.();
      return;
    }

    if (e.key === 'Escape' && !e.defaultPrevented) {
      if (expandedNotes !== null) {
        if (hasUnsavedNotes()) {
          pendingNotesTarget = null;
          discardNotesVisible = true;
          return;
        }
        expandedNotes = null;
      } else if (document.activeElement instanceof HTMLInputElement) {
        document.activeElement.blur();
      }
      return;
    }

    if (mod && e.key === 'Enter') {
      e.preventDefault();
      if (selectedIndex >= 0) onToggleConfirm?.(selectedIndex);
      const target = findEditableRow(currentVisible, 1);
      if (target >= 0) scrollToVisibleRow(target);
      return;
    }

    if (mod && e.altKey && e.key === 'ArrowDown') {
      e.preventDefault();
      onJumpNextUnconfirmed?.();
      return;
    }

    if (e.key === 'Enter' && isEnFocused) {
      e.preventDefault();
      if (autoConfirmOnEnter && selectedIndex >= 0 && !confirmedLines.has(selectedIndex)) {
        onToggleConfirm?.(selectedIndex, true);
      }
      const target = findEditableRow(currentVisible, 1);
      if (target >= 0) scrollToVisibleRow(target);
      return;
    }

    if ((e.key === 'ArrowDown' || e.key === 'ArrowUp') && isEnFocused) {
      e.preventDefault();
      const dir = e.key === 'ArrowDown' ? 1 : -1;
      const target = findEditableRow(currentVisible, dir as 1 | -1);
      if (target >= 0) scrollToVisibleRow(target);
      return;
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      const dir = e.shiftKey ? -1 : 1;
      const start = currentVisible >= 0 ? currentVisible : dir === 1 ? -1 : visibleEntries.length;
      const target = findEditableRow(start, dir as 1 | -1);
      if (target >= 0) scrollToVisibleRow(target);
      return;
    }

    if (mod && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
      e.preventDefault();
      const dir = e.key === 'ArrowDown' ? 1 : -1;
      const start = currentVisible >= 0 ? currentVisible : dir === 1 ? -1 : visibleEntries.length;
      const target = findUntranslatedRow(start, dir as 1 | -1);
      if (target >= 0) scrollToVisibleRow(target);
      return;
    }
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if entries.length === 0}
  <div class="empty-state">Loading...</div>
{:else}
  <div class="table-header">
    <div class="header-cell col-num">#</div>
    <div class="header-cell col-jp">Japanese</div>
    <div class="header-cell col-en">English</div>
  </div>
  <div class="scroll-container" bind:this={scrollElement}>
    <div style:height="{$virtualizer.getTotalSize()}px" style:position="relative">
      {#each $virtualizer.getVirtualItems() as row (row.index)}
        {@const entry = visibleEntries[row.index]}
        <div
          class="table-row {rowClass(entry)}"
          class:row-selected={entry.index === selectedIndex}
          class:row-find-match={findQuery && findMatchIndices.includes(entry.index)}
          class:row-find-current={findQuery &&
            currentFindMatch >= 0 &&
            findMatchIndices[currentFindMatch] === entry.index}
          style:position="absolute"
          style:top="{row.start}px"
          style:min-height="{ROW_HEIGHT}px"
          style:left="0"
          style:right="0"
          data-entry-index={entry.index}
          data-index={row.index}
          use:measureRow={{
            virt: $virtualizer,
            measure: expandedNotes === entry.index || (isText(entry) && entry.index === selectedIndex),
          }}
          onclick={() => handleRowClick(entry.index)}
        >
          <div
            class="cell col-num"
            class:col-num-confirmed={confirmedLines.has(entry.index)}
            class:col-num-dirty={(dirtyIndices.get(entry.index) ?? 0) > 0}
            ondblclick={() => {
              if (isText(entry) && onToggleConfirm) {
                onToggleConfirm(entry.index);
              }
            }}
            title={entry.entryType === 'text'
              ? confirmedLines.has(entry.index)
                ? 'Confirmed (double-click to unconfirm)'
                : 'Double-click to confirm'
              : ''}
          >
            {#if isText(entry)}{entry.index + 1}{/if}
          </div>

          {#if isText(entry)}
            <button
              class="btn-icon notes-btn"
              class:has-notes={entry.notes.length > 0}
              onclick={(e) => {
                e.stopPropagation();
                toggleNotes(entry.index);
              }}
              title={entry.notes.length > 0 ? `${entry.notes.length} note(s) — click to edit` : 'Add notes'}
            >
              {#if entry.notes.length > 0}
                <StickyNoteIcon size={12} />
              {:else}
                <PlusIcon size={12} />
              {/if}
            </button>
            {@const segments = splitByMatches(entry.jpText ?? '', glossary, (g) => g.jp)}
            <div
              class="cell col-jp"
              class:col-jp-wrap={entry.index === selectedIndex}
              style:padding-left={depthPadding(entry.depth)}
              onmouseup={handleJpMouseUp}
            >
              {#each segments as seg}
                {#if seg.match}
                  <span class="glossary-term" title="{seg.match.en}{seg.match.note ? ` — ${seg.match.note}` : ''}"
                    >{seg.text}</span
                  >
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </div>
            <div class="cell col-en">
              <input
                type="text"
                value={entry.enText ?? ''}
                oninput={(e) => handleInput(entry.index, e)}
                onfocus={() => handleRowClick(entry.index)}
              />
            </div>

            {#if expandedNotes === entry.index}
              <div class="notes-panel">
                <textarea
                  class="notes-textarea"
                  bind:value={notesDraft}
                  placeholder="Add notes (one per line)..."
                  rows="3"
                ></textarea>
                <div class="notes-actions">
                  <button class="btn-primary" disabled={!hasUnsavedNotes()} onclick={() => saveNotes(entry.index)}
                    >Save</button
                  >
                  <button onclick={() => toggleNotes(entry.index)}>Cancel</button>
                </div>
              </div>
            {/if}
          {:else if entry.entryType === 'comment'}
            <div class="cell col-jp structural-text" style:padding-left={depthPadding(entry.depth)}>
              {entry.jpText ?? ''}
            </div>
            <div class="cell col-en structural-text">
              {entry.enText ?? ''}
            </div>
          {:else if entry.entryType === 'include'}
            <div class="cell col-wide include-row" style:padding-left={depthPadding(entry.depth)}>
              <button
                class="btn-icon collapse-toggle"
                onclick={(e) => {
                  e.stopPropagation();
                  toggleCollapse(entry.index);
                }}
              >
                {#if collapsedIncludes.has(entry.index)}
                  <ChevronRightIcon size={12} />
                {:else}
                  <ChevronDownIcon size={12} />
                {/if}
              </button>
              <span class="include-path">
                {entry.jpText ?? entry.enText ?? ''}
              </span>
              <span class="include-count">
                ({childCount(entry.index)} lines)
              </span>
            </div>
          {:else if entry.entryType === 'reference'}
            {@const refPath = entry.jpText ?? entry.enText ?? ''}
            {@const broken = isRefBroken(refPath)}
            <div
              class="cell col-wide reference-row"
              class:reference-broken={broken}
              style:padding-left={depthPadding(entry.depth)}
            >
              <ArrowRightIcon size={12} />
              {#if broken}
                <span class="reference-path-broken">{refPath}</span>
                <span class="reference-error">— no matching include found</span>
              {:else}
                <button
                  class="btn-icon reference-link"
                  onclick={(e) => {
                    e.stopPropagation();
                    jumpToInclude(refPath);
                  }}
                  title="Jump to referenced file"
                >
                  {refPath}
                </button>
              {/if}
            </div>
          {:else}
            <div class="cell col-wide">&nbsp;</div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}

<UnsavedChangesDialog
  bind:visible={discardNotesVisible}
  onSave={handleSaveAndSwitchNotes}
  onDiscard={handleDiscardNotes}
/>

<style>
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 16px;
  }

  .table-header {
    display: flex;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
  }

  .header-cell {
    padding: 6px 10px;
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .scroll-container {
    flex: 1;
    overflow-y: auto;
    will-change: transform;
  }

  .table-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    box-shadow: inset 0 -1px 0 var(--color-border);
    cursor: default;
  }

  .cell {
    padding: 4px 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .col-num {
    width: 50px;
    padding-left: 4px;
    padding-right: 4px;
    text-align: right;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }

  .col-jp {
    flex: 4;
    min-width: 0;
    user-select: text;

    &.col-jp-wrap {
      white-space: normal;
      overflow: visible;
      text-overflow: clip;
      word-break: break-word;
    }
  }

  .col-en {
    flex: 5;
    min-width: 0;
    user-select: text;

    input {
      width: 100%;
      border: 1px solid transparent;
      background: transparent;
      padding: 2px 4px;
      border-radius: var(--radius-sm);
      color: var(--color-text);
      font-family: inherit;
      font-size: inherit;

      &:focus {
        background: var(--color-surface-alt);
        border-color: var(--color-accent);
        outline: none;
      }
    }
  }

  .col-wide {
    flex: 1;
  }

  .structural-text {
    color: var(--color-structural-text);
    font-style: italic;
    font-size: 13px;
  }

  .include-row {
    display: flex;
    align-items: center;
    gap: 6px;

    .collapse-toggle {
      padding: 0 4px;
      font-size: 10px;
      line-height: 1;
      flex-shrink: 0;
    }

    .include-path {
      color: var(--color-accent);
      font-size: 13px;
    }

    .include-count {
      color: var(--color-text-muted);
      font-size: 12px;
    }
  }

  .reference-row {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--color-text-muted);

    .reference-link {
      color: var(--color-accent);
      font-size: 13px;
      text-decoration: underline;
      text-decoration-style: dotted;
      text-underline-offset: 2px;

      &:hover {
        color: var(--color-text);
      }
    }

    &.reference-broken {
      color: var(--color-danger);
    }

    .reference-path-broken {
      font-size: 13px;
      color: var(--color-danger-light);
    }

    .reference-error {
      font-size: 12px;
      font-style: italic;
      color: var(--color-danger);
    }
  }

  .glossary-term {
    border-bottom: 1px dotted var(--color-glossary);
    cursor: help;
  }

  .notes-btn {
    width: 24px;
    margin: 0;
    align-self: stretch;
    color: var(--color-notes);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;

    &.has-notes {
      color: var(--color-accent);
    }
  }

  .notes-panel {
    width: 100%;
    padding: 8px 12px 8px 70px;
    background: var(--color-surface);

    .notes-textarea {
      width: 100%;
      min-height: 60px;
      resize: vertical;
      background: var(--color-bg);
      color: var(--color-text);
      border: 1px solid var(--color-border);
      border-radius: var(--radius-sm);
      padding: 6px 8px;
      font-family: inherit;
      font-size: 13px;

      &:focus {
        outline: none;
        border-color: var(--color-accent);
      }
    }

    .notes-actions {
      display: flex;
      gap: 6px;
      margin-top: 6px;
      justify-content: flex-end;
    }
  }

  .col-num-confirmed {
    color: var(--color-confirmed-text);
    font-weight: 600;
  }

  .col-num-dirty {
    border-left: 3px solid var(--color-accent);
  }

  /*noinspection CssUnusedSymbol*/
  .row-structural {
    background: var(--color-structural);
  }

  /*noinspection CssUnusedSymbol*/
  .row-untranslated {
    background: var(--color-untranslated);
  }

  /*noinspection CssUnusedSymbol*/
  .row-translated {
    background: var(--color-translated);
  }

  /*noinspection CssUnusedSymbol*/
  .row-confirmed {
    background: var(--color-confirmed);
  }

  .row-selected {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .row-find-match {
    background: var(--color-find-match) !important;
  }

  .row-find-current {
    background: var(--color-find-current) !important;
    outline: 2px solid var(--color-warning);
    outline-offset: -2px;
  }
</style>
