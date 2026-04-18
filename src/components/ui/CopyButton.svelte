<script lang="ts">
  import { CheckIcon, CopyIcon } from '@lucide/svelte';

  let { text, title = 'Copy' }: { text: string; title?: string } = $props();

  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    return () => clearTimeout(timer);
  });

  function copy() {
    navigator.clipboard.writeText(text);
    copied = true;
    clearTimeout(timer);
    timer = setTimeout(() => (copied = false), 2000);
  }
</script>

<button class="copy-btn" class:copied {title} onclick={copy}>
  {#if copied}<CheckIcon size={13} />{:else}<CopyIcon size={13} />{/if}
</button>

<style>
  .copy-btn {
    padding: 4px;
    background: none;
    border: none;
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    cursor: pointer;

    &:hover {
      color: var(--color-text);
    }

    &.copied {
      color: var(--color-success-text);
    }
  }
</style>
