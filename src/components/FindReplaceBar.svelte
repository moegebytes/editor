<script lang="ts">
  import { ChevronUpIcon, ChevronDownIcon, XIcon } from '@lucide/svelte';
  import { useDebouncedValue } from '../lib/debounced.svelte';

  let {
    visible = $bindable(false),
    onFind,
    onFindNext,
    onFindPrev,
    onReplace,
    onReplaceAll,
    matchCount = 0,
    currentMatch = -1,
  }: {
    visible?: boolean;
    onFind: (query: string) => void;
    onFindNext: () => void;
    onFindPrev: () => void;
    onReplace: (replacement: string) => void;
    onReplaceAll: (query: string, replacement: string) => void;
    matchCount?: number;
    currentMatch?: number;
  } = $props();

  let findText = $state('');
  let replaceText = $state('');
  let findInput: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (visible && findInput) {
      findInput.focus();
    }
  });

  const debouncedFindText = useDebouncedValue(() => findText, 150);
  $effect(() => onFind(debouncedFindText.value));

  function handleFindKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (e.shiftKey) {
        onFindPrev();
      } else {
        onFindNext();
      }
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      visible = false;
    }
  }

  function handleReplaceKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      visible = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && visible && !e.defaultPrevented) {
      e.preventDefault();
      visible = false;
    }
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if visible}
  <div class="find-replace-bar">
    <input
      type="text"
      placeholder="Find..."
      bind:value={findText}
      bind:this={findInput}
      onkeydown={handleFindKeydown}
      class="find-input"
    />
    <input
      type="text"
      placeholder="Replace with..."
      bind:value={replaceText}
      onkeydown={handleReplaceKeydown}
      class="find-input"
    />
    <span class="match-info">
      {#if findText && matchCount > 0}
        {currentMatch + 1}/{matchCount}
      {:else if findText}
        0 results
      {/if}
    </span>
    <button onclick={onFindPrev} disabled={matchCount === 0} class="nav-btn">
      <ChevronUpIcon size={14} />
    </button>
    <button onclick={onFindNext} disabled={matchCount === 0} class="nav-btn">
      <ChevronDownIcon size={14} />
    </button>
    <button onclick={() => onReplace(replaceText)} disabled={matchCount === 0}> Replace </button>
    <button onclick={() => onReplaceAll(findText, replaceText)} disabled={matchCount === 0}> Replace All </button>
    <button onclick={() => (visible = false)} class="btn-icon close-btn">
      <XIcon size={14} />
    </button>
  </div>
{/if}

<style>
  .find-replace-bar {
    padding: 6px 12px;
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .find-input {
    flex: 1;
    min-width: 120px;
    padding: 4px 8px;
  }

  .match-info {
    font-size: 12px;
    color: var(--color-text-muted);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .nav-btn {
    padding: 4px 8px;
    font-size: 10px;
    line-height: 1;
  }

  .close-btn {
    padding: 4px 8px;
    font-size: 12px;
    line-height: 1;
  }
</style>
