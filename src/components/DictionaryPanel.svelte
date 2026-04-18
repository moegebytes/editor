<script lang="ts">
  import { XIcon, SearchIcon, ArrowLeftIcon, ArrowRightIcon } from '@lucide/svelte';

  import JmdictTab from './dictionary/JmdictTab.svelte';
  import WiktTab from './dictionary/WiktTab.svelte';

  const MIN_WIDTH = 360;
  const MAX_WIDTH = 600;
  const MAX_HISTORY = 50;

  let {
    query = '',
    querySeq = 0,
    visible = $bindable(true),
    width = $bindable(MIN_WIDTH),
  }: {
    query?: string;
    querySeq?: number;
    visible?: boolean;
    width?: number;
  } = $props();

  let searchInput = $state('');
  let activeTab: 'dict' | 'wikt' = $state('dict');
  let dictTab: JmdictTab | undefined = $state();
  let wiktTab: WiktTab | undefined = $state();
  let history: string[] = $state([]);
  let historyIndex = $state(-1);
  let navigating = false;
  let lastSeq = -1;
  let resizeCleanup: (() => void) | null = null;

  let navCanGoBack = $derived(historyIndex > 0);
  let navCanGoForward = $derived(historyIndex < history.length - 1);

  $effect(() => {
    if (!visible) clearState();
  });

  $effect(() => {
    if (query && querySeq !== lastSeq) {
      lastSeq = querySeq;
      searchInput = query;
      activeTab = 'dict';
      pushHistory(query);
      dictTab?.lookup(query);
      wiktTab?.lookup(query);
    }
  });

  $effect(() => {
    return () => resizeCleanup?.();
  });

  function pushHistory(q: string) {
    const trimmed = q.trim();
    if (!trimmed) return;
    if (navigating) {
      navigating = false;
      return;
    }
    if (historyIndex >= 0 && history[historyIndex] === trimmed) return;
    history = history.slice(0, historyIndex + 1);
    history.push(trimmed);
    if (history.length > MAX_HISTORY) {
      history = history.slice(history.length - MAX_HISTORY);
    }
    historyIndex = history.length - 1;
  }

  function goBack() {
    if (!navCanGoBack) return;
    historyIndex--;
    searchInput = history[historyIndex];
    navigating = true;
    triggerSearch();
  }

  function goForward() {
    if (!navCanGoForward) return;
    historyIndex++;
    searchInput = history[historyIndex];
    navigating = true;
    triggerSearch();
  }

  function triggerSearch() {
    pushHistory(searchInput);
    dictTab?.lookup(searchInput);
    wiktTab?.lookup(searchInput);
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      triggerSearch();
    }
  }

  function handleNavigate(term: string) {
    searchInput = term;
    pushHistory(term);
    dictTab?.lookup(term);
    wiktTab?.lookup(term);
  }

  function clearState() {
    searchInput = '';
    activeTab = 'dict';
    dictTab?.clear();
    wiktTab?.clear();
    history = [];
    historyIndex = -1;
  }

  function close() {
    clearState();
    visible = false;
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();
    resizeCleanup?.();

    const startX = e.clientX;
    const startWidth = width;

    function onMove(e: MouseEvent) {
      const delta = e.clientX - startX;
      width = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
    }

    function onUp() {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      resizeCleanup = null;
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    resizeCleanup = onUp;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && visible && !e.defaultPrevented) {
      e.preventDefault();
      close();
    }
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if visible}
  <div class="dict-panel" style:width="{width}px">
    <div class="dict-content">
      <div class="dict-header">
        <button class="btn-icon dict-nav-btn" onclick={goBack} disabled={!navCanGoBack} title="Back">
          <ArrowLeftIcon size={14} />
        </button>
        <button class="btn-icon dict-nav-btn" onclick={goForward} disabled={!navCanGoForward} title="Forward">
          <ArrowRightIcon size={14} />
        </button>
        <input
          type="text"
          placeholder="Search..."
          bind:value={searchInput}
          onkeydown={handleSearchKeydown}
          class="dict-search-input"
        />
        <button class="btn-icon dict-search-btn" onclick={triggerSearch} disabled={!searchInput.trim()} title="Search">
          <SearchIcon size={14} />
        </button>
        <button class="btn-icon dict-close" onclick={close}>
          <XIcon size={14} />
        </button>
      </div>

      <div class="tab-bar">
        <button class="tab-btn" class:active={activeTab === 'dict'} onclick={() => (activeTab = 'dict')}>JMdict</button>
        <button class="tab-btn" class:active={activeTab === 'wikt'} onclick={() => (activeTab = 'wikt')}
          >Wiktionary</button
        >
      </div>

      <div class="tab-content" class:hidden={activeTab !== 'dict'}>
        <JmdictTab bind:this={dictTab} onNavigate={handleNavigate} />
      </div>
      <div class="tab-content" class:hidden={activeTab !== 'wikt'}>
        <WiktTab bind:this={wiktTab} onNavigate={handleNavigate} />
      </div>
    </div>
    <div class="resize-handle" onmousedown={startResize}></div>
  </div>
{/if}

<style>
  .dict-panel {
    display: flex;
    flex-shrink: 0;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    height: 100%;
    position: relative;
  }

  .resize-handle {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
    transition: background 0.15s;

    &:hover {
      background: var(--color-accent);
    }
  }

  .dict-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .dict-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--color-border);
  }

  .dict-search-input {
    flex: 1;
  }

  .dict-close,
  .dict-search-btn,
  .dict-nav-btn {
    padding: 4px 6px;
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
  }

  .dict-nav-btn {
    &:disabled {
      opacity: 0.3;
      cursor: default;
    }
  }

  .dict-search-btn {
    color: var(--color-accent);

    &:disabled {
      opacity: 0.3;
      cursor: default;
    }
  }

  .tab-bar {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .tab-btn {
    flex: 1;
    padding: 6px 12px;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: none;
    color: var(--color-text-muted);
    font-size: 12px;
    cursor: pointer;
    transition:
      color 0.15s,
      border-color 0.15s;

    &:hover {
      color: var(--color-text);
      background: none;
    }

    &.active {
      color: var(--color-accent);
      border-bottom-color: var(--color-accent);
    }
  }

  .tab-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .hidden {
    display: none;
  }
</style>
