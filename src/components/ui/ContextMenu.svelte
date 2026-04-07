<script lang="ts">
  import DropdownMenu from './DropdownMenu.svelte';
  import type { MenuEntry } from './DropdownMenu.svelte';

  let visible = $state(false);
  let x = $state(0);
  let y = $state(0);
  let items: MenuEntry[] = $state([]);
  let menuEl: HTMLDivElement | undefined = $state();

  function isEditable(el: EventTarget | null): el is HTMLInputElement | HTMLTextAreaElement {
    return el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement;
  }

  function hasSelection(el: HTMLInputElement | HTMLTextAreaElement): boolean {
    return (el.selectionStart ?? 0) !== (el.selectionEnd ?? 0);
  }

  function buildItems(target: EventTarget | null): MenuEntry[] {
    const result: MenuEntry[] = [];

    if (isEditable(target)) {
      const el = target;
      const selected = hasSelection(el);

      const canCut = document.queryCommandSupported?.('cut') ?? false;
      const canCopy = document.queryCommandSupported?.('copy') ?? false;
      const canInsert = document.queryCommandSupported?.('insertText') ?? false;

      result.push(
        {
          label: 'Cut',
          disabled: !selected || !canCut,
          action: () => {
            el.focus();
            document.execCommand('cut');
          },
        },
        {
          label: 'Copy',
          disabled: !selected || !canCopy,
          action: () => {
            el.focus();
            document.execCommand('copy');
          },
        },
        {
          label: 'Paste',
          disabled: !canInsert,
          action: () => {
            el.focus();
            navigator.clipboard.readText().then((text) => {
              document.execCommand('insertText', false, text);
            });
          },
        },
        {
          label: 'Select All',
          action: () => {
            el.focus();
            el.select();
          },
        },
      );
    }

    return result;
  }

  function handleContextMenu(e: MouseEvent) {
    if (import.meta.env.DEV && e.shiftKey) return;

    e.preventDefault();
    const built = buildItems(e.target);
    if (built.length === 0) return;

    items = built;
    x = e.clientX;
    y = e.clientY;
    visible = true;

    requestAnimationFrame(() => {
      if (!menuEl) return;
      const rect = menuEl.getBoundingClientRect();
      if (rect.right > window.innerWidth) {
        x = window.innerWidth - rect.width - 4;
      }
      if (rect.bottom > window.innerHeight) {
        y = window.innerHeight - rect.height - 4;
      }
    });
  }

  function hide() {
    visible = false;
  }
</script>

<svelte:document
  oncontextmenu={handleContextMenu}
  onclick={hide}
  onkeydown={(e) => {
    if (e.key === 'Escape') hide();
  }}
/>

{#if visible}
  <div class="context-menu-pos" bind:this={menuEl} style:left="{x}px" style:top="{y}px">
    <DropdownMenu {items} onClose={hide} />
  </div>
{/if}

<style>
  .context-menu-pos {
    position: fixed;
    z-index: 9999;
  }
</style>
