<script lang="ts">
  import type { Snippet } from 'svelte';
  import { ArrowLeftIcon } from '@lucide/svelte';

  let {
    onBack,
    children,
    actions,
  }: {
    onBack: () => void;
    children: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<header class="view-header">
  <button class="back-btn" onclick={onBack}>
    <ArrowLeftIcon size={14} />
    <span>Back</span>
  </button>
  <span class="breadcrumb">
    {@render children()}
  </span>
  {#if actions}
    <div class="header-actions">
      {@render actions()}
    </div>
  {/if}
</header>

<style>
  .view-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 600;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 12px;
    border-left: 1px solid var(--color-border);
    font-size: 13px;
    color: var(--color-text-muted);

    :global(strong) {
      color: var(--color-text);
      font-weight: 500;
    }
  }

  .header-actions {
    margin-left: auto;
  }
</style>
