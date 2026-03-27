<script lang="ts">
  export interface MenuItem {
    label: string;
    action: () => void;
    shortcut?: string;
    disabled?: boolean;
    danger?: boolean;
  }

  export interface MenuSeparator {
    separator: true;
  }

  export type MenuEntry = MenuItem | MenuSeparator;

  export function isSeparator(entry: MenuEntry): entry is MenuSeparator {
    return "separator" in entry;
  }

  let {items, onClose}: {
    items: MenuEntry[];
    onClose: () => void;
  } = $props();

  function handleClick(item: MenuItem) {
    onClose();
    item.action();
  }
</script>

<div class="dropdown-menu">
  {#each items as entry}
    {#if isSeparator(entry)}
      <div class="menu-separator"></div>
    {:else}
      <button
        class="menu-item"
        class:menu-item-danger={entry.danger}
        disabled={entry.disabled}
        onclick={() => handleClick(entry)}
      >
        {entry.label}
        {#if entry.shortcut}
          <span class="menu-shortcut">{entry.shortcut}</span>
        {/if}
      </button>
    {/if}
  {/each}
</div>

<style>
  .dropdown-menu {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    box-shadow: var(--shadow-dropdown);
    padding: 4px 0;
    min-width: 140px;
    font-size: 13px;
  }

  .menu-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 6px 14px;
    background: none;
    border: none;
    border-radius: 0;
    text-align: left;
    font-size: 13px;
    color: var(--color-text);
    cursor: pointer;
    font-family: inherit;

    &:hover:not(:disabled) {
      background: var(--color-surface-alt);
    }

    &:disabled {
      color: var(--color-text-disabled);
      cursor: default;
    }

    &.menu-item-danger {
      color: var(--color-danger);
    }
  }

  .menu-shortcut {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .menu-separator {
    height: 1px;
    background: var(--color-border);
    margin: 4px 0;
  }
</style>
