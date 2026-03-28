<script lang="ts">
  import { XIcon, SearchIcon } from "@lucide/svelte";
  import DictTab from "./DictTab.svelte";
  import WiktTab from "./WiktTab.svelte";

  let {
    query = "",
    querySeq = 0,
    visible = $bindable(true),
    width = $bindable(320),
  }: {
    query?: string;
    querySeq?: number;
    visible?: boolean;
    width?: number;
  } = $props();

  let searchInput = $state("");
  let activeTab: "dict" | "wikt" = $state("dict");
  let wiktCached = $state(false);

  let dictTab: DictTab | undefined = $state();
  let wiktTab: WiktTab | undefined = $state();

  const MIN_WIDTH = 240;
  const MAX_WIDTH = 600;

  function clearState() {
    searchInput = "";
    activeTab = "dict";
    wiktCached = false;
    dictTab?.clear();
    wiktTab?.clear();
  }

  function close() {
    clearState();
    visible = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && visible && !e.defaultPrevented) {
      e.preventDefault();
      close();
    }
  }

  $effect(() => {
    if (!visible) clearState();
  });

  let lastSeq = -1;
  $effect(() => {
    if (query && querySeq !== lastSeq) {
      lastSeq = querySeq;
      searchInput = query;
      activeTab = "dict";
      wiktTab?.clear();
      wiktCached = false;
      dictTab?.lookup(query);
    }
  });

  function triggerSearch() {
    if (activeTab === "wikt") {
      wiktTab?.lookup(searchInput);
    } else {
      dictTab?.lookup(searchInput);
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      triggerSearch();
    }
  }

  function handleNavigate(term: string) {
    searchInput = term;
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();

    const startX = e.clientX;
    const startWidth = width;

    function onMove(e: MouseEvent) {
      const delta = e.clientX - startX;
      width = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
    }

    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if visible}
  <div class="dict-panel" style:width="{width}px">
    <div class="dict-content">
      <div class="dict-header">
        <input
          type="text"
          placeholder="Search..."
          bind:value={searchInput}
          onkeydown={handleSearchKeydown}
          class="dict-search-input"
        />
        <button
          class="btn-icon dict-search-btn"
          onclick={triggerSearch}
          disabled={!searchInput.trim()}
          title="Search"
        >
          <SearchIcon size={14} />
        </button>
        <button class="btn-icon dict-close" onclick={close}>
          <XIcon size={14} />
        </button>
      </div>

      <div class="tab-bar">
        <button
          class="tab-btn"
          class:active={activeTab === "dict"}
          onclick={() => (activeTab = "dict")}
        >Dictionary</button>
        <button
          class="tab-btn"
          class:active={activeTab === "wikt"}
          onclick={() => (activeTab = "wikt")}
        >Wiktionary{#if wiktCached}<span class="tab-cached">cached</span>{/if}</button>
      </div>

      <div class="tab-content" class:hidden={activeTab !== "dict"}>
        <DictTab bind:this={dictTab} onNavigate={handleNavigate} />
      </div>
      <div class="tab-content" class:hidden={activeTab !== "wikt"}>
        <WiktTab bind:this={wiktTab} bind:cached={wiktCached} />
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

  .dict-close, .dict-search-btn {
    padding: 4px 6px;
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
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
    transition: color 0.15s, border-color 0.15s;

    &:hover {
      color: var(--color-text);
      background: none;
    }

    &.active {
      color: var(--color-accent);
      border-bottom-color: var(--color-accent);
    }

    .tab-cached {
      font-size: 10px;
      color: var(--color-text-muted);
      margin-left: 4px;
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
