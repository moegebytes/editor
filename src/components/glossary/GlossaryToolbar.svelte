<script lang="ts">
  import { PlusIcon, SearchIcon, XIcon } from '@lucide/svelte';

  let {
    searchQuery = $bindable(),
    filterMode = $bindable(),
    total,
    issueCount,
    disableNew,
    onNew,
  }: {
    searchQuery: string;
    filterMode: 'all' | 'issues';
    total: number;
    issueCount: number;
    disableNew: boolean;
    onNew: () => void;
  } = $props();

  let inputEl: HTMLInputElement | undefined = $state();

  function clearSearch() {
    searchQuery = '';
    inputEl?.focus();
  }
</script>

<div class="toolbar">
  <div class="search-box">
    <SearchIcon size={14} />
    <input
      type="search"
      placeholder="Search terms, translations, notes..."
      bind:value={searchQuery}
      bind:this={inputEl}
      spellcheck="false"
    />
    {#if searchQuery}
      <button class="btn-icon clear-btn" onclick={clearSearch}>
        <XIcon size={12} />
      </button>
    {/if}
  </div>

  <div class="filter-chips">
    <button class="chip" class:active={filterMode === 'all'} onclick={() => (filterMode = 'all')}>
      All <span class="chip-count">{total}</span>
    </button>
    <button class="chip" class:active={filterMode === 'issues'} onclick={() => (filterMode = 'issues')}>
      Issues <span class="chip-count">{issueCount}</span>
    </button>
  </div>

  <div class="toolbar-actions">
    <button class="btn-primary" onclick={onNew} disabled={disableNew}>
      <PlusIcon size={13} />
      New term
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .search-box {
    position: relative;
    width: 300px;

    :global(svg:first-child) {
      position: absolute;
      left: 8px;
      top: 50%;
      transform: translateY(-50%);
      color: var(--color-text-muted);
      pointer-events: none;
    }

    input {
      width: 100%;
      padding: 5px 8px 5px 28px;
      font-size: 13px;
    }

    .clear-btn {
      position: absolute;
      right: 4px;
      top: 50%;
      transform: translateY(-50%);
      padding: 2px;
    }
  }

  .filter-chips {
    display: flex;
    gap: 4px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    font-size: 12px;
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);

    &:hover {
      color: var(--color-text);
      border-color: var(--color-text-disabled);
    }

    &.active {
      background: color-mix(in srgb, var(--color-accent) 18%, transparent);
      border-color: color-mix(in srgb, var(--color-accent) 55%, var(--color-border));
      color: var(--color-text);
    }

    .chip-count {
      font-size: 10px;
      font-variant-numeric: tabular-nums;
    }
  }

  .toolbar-actions {
    margin-left: auto;
    display: flex;
    gap: 8px;

    button {
      display: flex;
      align-items: center;
      gap: 4px;
    }
  }
</style>
