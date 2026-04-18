<script lang="ts">
  import type { Snippet } from 'svelte';

  import Dialog from './Dialog.svelte';

  let {
    visible = $bindable(false),
    title,
    icon,
    message,
    actions,
  }: {
    visible?: boolean;
    title: string;
    icon: Snippet;
    message: Snippet;
    actions: Snippet;
  } = $props();
</script>

<Dialog bind:visible {title} {actions}>
  <div class="ask-message">
    <span class="ask-icon">{@render icon()}</span>
    <div class="ask-body">{@render message()}</div>
  </div>
</Dialog>

<style>
  .ask-message {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    font-size: 14px;
    color: var(--color-text);

    :global(p) {
      margin: 0;
    }
  }

  .ask-icon {
    flex-shrink: 0;
  }
</style>
