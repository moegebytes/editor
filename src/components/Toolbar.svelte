<script lang="ts">
  import DropdownMenu from './ui/DropdownMenu.svelte';
  import type { MenuEntry } from './ui/DropdownMenu.svelte';
  import { XIcon } from '@lucide/svelte';

  let {
    onSave,
    onExport,
    onCloseProject,
    onOpenDict,
    onToggleFindReplace,
    onJumpUntranslated,
    onJumpUnconfirmed,
    onConfirmToggle,
    onUndo,
    onRedo,
    undoDisabled,
    redoDisabled,
    onGoToLine,
    onOpenSettings,
    onAbout,
    projectName,
    saveDisabled,
    filterText = $bindable(''),
  }: {
    onSave: () => void;
    onExport: () => void;
    onCloseProject: () => void;
    onOpenDict: () => void;
    onToggleFindReplace: () => void;
    onJumpUntranslated: () => void;
    onJumpUnconfirmed: () => void;
    onConfirmToggle: () => void;
    onUndo: () => void;
    onRedo: () => void;
    undoDisabled: boolean;
    redoDisabled: boolean;
    onGoToLine: () => void;
    onOpenSettings: () => void;
    onAbout: () => void;
    projectName: string | null;
    saveDisabled: boolean;
    filterText?: string;
  } = $props();

  let openMenu: string | null = $state(null);
  let filterInput: HTMLInputElement;

  export function focusFilter() {
    filterInput?.focus();
    filterInput?.select();
  }

  function toggleMenu(name: string) {
    openMenu = openMenu === name ? null : name;
  }

  function closeMenu() {
    openMenu = null;
  }

  let projectItems: MenuEntry[] = $derived([
    { label: 'Save', action: onSave, disabled: saveDisabled, shortcut: 'Ctrl+S' },
    { label: 'Export', action: onExport },
    { separator: true },
    { label: 'Settings', action: onOpenSettings },
    { separator: true },
    { label: 'Close Project', action: onCloseProject, danger: true },
  ]);

  let lineItems: MenuEntry[] = $derived([
    { label: 'Undo', action: onUndo, disabled: undoDisabled, shortcut: 'Ctrl+Z' },
    { label: 'Redo', action: onRedo, disabled: redoDisabled, shortcut: 'Ctrl+Y' },
    { separator: true },
    { label: 'Next Untranslated', action: onJumpUntranslated, shortcut: 'Ctrl+\u2193' },
    { label: 'Next Unconfirmed', action: onJumpUnconfirmed, shortcut: 'Ctrl+Alt+\u2193' },
    { separator: true },
    { label: 'Confirm / Unconfirm', action: onConfirmToggle, shortcut: 'Ctrl+Enter' },
  ]);

  let toolsItems: MenuEntry[] = $derived([
    { label: 'Go to Line', action: onGoToLine, shortcut: 'Ctrl+G' },
    { label: 'Open Dictionary', action: onOpenDict, shortcut: 'Ctrl+D' },
    { label: 'Find & Replace', action: onToggleFindReplace, shortcut: 'Ctrl+H' },
  ]);

  let helpItems: MenuEntry[] = $derived([{ label: 'About', action: onAbout }]);
</script>

{#if openMenu}
  <div class="menu-backdrop" onclick={closeMenu} role="none"></div>
{/if}

<div class="toolbar">
  <div class="toolbar-left">
    <div class="menu-container">
      <button class:btn-active={openMenu === 'project'} onclick={() => toggleMenu('project')}>
        {projectName ?? 'Project'}
      </button>
      {#if openMenu === 'project'}
        <div class="menu-pos">
          <DropdownMenu items={projectItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

    <div class="menu-container">
      <button class:btn-active={openMenu === 'line'} onclick={() => toggleMenu('line')}> Line </button>
      {#if openMenu === 'line'}
        <div class="menu-pos">
          <DropdownMenu items={lineItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

    <div class="menu-container">
      <button class:btn-active={openMenu === 'tools'} onclick={() => toggleMenu('tools')}> Tools </button>
      {#if openMenu === 'tools'}
        <div class="menu-pos">
          <DropdownMenu items={toolsItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

    <div class="menu-container">
      <button class:btn-active={openMenu === 'help'} onclick={() => toggleMenu('help')}> Help </button>
      {#if openMenu === 'help'}
        <div class="menu-pos">
          <DropdownMenu items={helpItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>
  </div>

  <div class="filter-wrapper">
    <input type="text" placeholder="Filter..." bind:value={filterText} bind:this={filterInput} class="filter-input" />
    {#if filterText}
      <button class="filter-clear" onclick={() => (filterText = '')}>
        <XIcon size={14} />
      </button>
    {/if}
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    gap: 12px;

    button {
      font-weight: 600;
    }
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .menu-container {
    position: relative;
  }

  .menu-pos {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    z-index: 100;
    min-width: 220px;
  }

  .menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .filter-wrapper {
    position: relative;
    flex-shrink: 0;
  }

  .filter-input {
    width: 260px;
    padding: 4px 24px 4px 8px;
    font-size: 13px;
  }

  .filter-clear {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    padding: 2px;
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;

    &:hover {
      color: var(--color-text);
    }
  }

  .btn-active {
    background: var(--color-surface-alt);
    border-color: var(--color-accent);
  }
</style>
