<script lang="ts">
  import DropdownMenu from "./ui/DropdownMenu.svelte";
  import type { MenuEntry } from "./ui/DropdownMenu.svelte";

  let {
    onSave,
    onExport,
    onCloseProject,
    onOpenDict,
    onToggleFindReplace,
    onJumpUntranslated,
    onJumpUnconfirmed,
    onConfirmToggle,
    onGoToLine,
    onOpenSettings,
    onAbout,
    projectName,
    saveDisabled,
    filterText = $bindable(""),
  }: {
    onSave: () => void;
    onExport: () => void;
    onCloseProject: () => void;
    onOpenDict: () => void;
    onToggleFindReplace: () => void;
    onJumpUntranslated: () => void;
    onJumpUnconfirmed: () => void;
    onConfirmToggle: () => void;
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
    { label: "Save", action: onSave, disabled: saveDisabled, shortcut: "Ctrl+S" },
    { label: "Export", action: onExport },
    { separator: true },
    { label: "Settings", action: onOpenSettings },
    { separator: true },
    { label: "Close Project", action: onCloseProject, danger: true },
  ]);

  let lineItems: MenuEntry[] = $derived([
    { label: "Next Untranslated", action: onJumpUntranslated, shortcut: "Ctrl+\u2193" },
    { label: "Next Unconfirmed", action: onJumpUnconfirmed, shortcut: "Ctrl+Alt+\u2193" },
    { separator: true },
    { label: "Confirm / Unconfirm", action: onConfirmToggle, shortcut: "Ctrl+Enter" },
  ]);

  let toolsItems: MenuEntry[] = $derived([
    { label: "Go to Line", action: onGoToLine, shortcut: "Ctrl+G" },
    { label: "Open Dictionary", action: onOpenDict, shortcut: "Ctrl+D" },
    { label: "Find & Replace", action: onToggleFindReplace, shortcut: "Ctrl+H" },
  ]);

  let helpItems: MenuEntry[] = $derived([
    { label: "About", action: onAbout },
  ]);

</script>

{#if openMenu}
  <div class="menu-backdrop" onclick={closeMenu}></div>
{/if}

<div class="toolbar">
  <div class="toolbar-left">
    <div class="menu-container">
      <button
        class:btn-active={openMenu === "project"}
        onclick={() => toggleMenu("project")}
      >
        {projectName ?? "Project"}
      </button>
      {#if openMenu === "project"}
        <div class="menu-pos">
          <DropdownMenu items={projectItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

    <div class="menu-container">
      <button
        class:btn-active={openMenu === "line"}
        onclick={() => toggleMenu("line")}
      >
        Line
      </button>
      {#if openMenu === "line"}
        <div class="menu-pos">
          <DropdownMenu items={lineItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

    <div class="menu-container">
      <button
        class:btn-active={openMenu === "tools"}
        onclick={() => toggleMenu("tools")}
      >
        Tools
      </button>
      {#if openMenu === "tools"}
        <div class="menu-pos">
          <DropdownMenu items={toolsItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

    <div class="menu-container">
      <button
        class:btn-active={openMenu === "help"}
        onclick={() => toggleMenu("help")}
      >
        Help
      </button>
      {#if openMenu === "help"}
        <div class="menu-pos">
          <DropdownMenu items={helpItems} onClose={closeMenu} />
        </div>
      {/if}
    </div>

  </div>

  <input
    type="text"
    placeholder="Filter..."
    bind:value={filterText}
    bind:this={filterInput}
    class="filter-input"
  />
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

  .filter-input {
    width: 260px;
    padding: 4px 8px;
    font-size: 13px;
    flex-shrink: 0;
  }

  .btn-active {
    background: var(--color-surface-alt);
    border-color: var(--color-accent);
  }
</style>
