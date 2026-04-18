<script lang="ts">
  import { HistoryIcon } from '@lucide/svelte';

  import AskDialog from './ui/AskDialog.svelte';

  let {
    visible = $bindable(false),
    timestamp,
    entryCount,
    confirmedDelta,
    onRestore,
    onDiscard,
  }: {
    visible?: boolean;
    timestamp: number;
    entryCount: number;
    confirmedDelta: number;
    onRestore: () => void;
    onDiscard: () => void;
  } = $props();

  let formattedTime = $derived(
    timestamp ? new Date(timestamp).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' }) : '',
  );

  let summary = $derived.by(() => {
    const parts: string[] = [];
    if (entryCount > 0) parts.push(`${entryCount} ${entryCount === 1 ? 'entry' : 'entries'} modified`);
    if (confirmedDelta > 0) parts.push(`${confirmedDelta} ${confirmedDelta === 1 ? 'line' : 'lines'} newly confirmed`);
    if (confirmedDelta < 0) {
      const n = -confirmedDelta;
      parts.push(`${n} ${n === 1 ? 'line' : 'lines'} unconfirmed`);
    }
    return parts.join(', ');
  });
</script>

<AskDialog bind:visible title="Recovery Data Found">
  {#snippet icon()}
    <HistoryIcon size={24} color="var(--color-accent)" />
  {/snippet}

  {#snippet message()}
    <p>Unsaved changes from <strong>{formattedTime}</strong> were recovered. Would you like to restore them?</p>
    {#if summary}
      <p class="summary">{summary}</p>
    {/if}
  {/snippet}

  {#snippet actions()}
    <button
      class="btn btn-danger"
      onclick={() => {
        visible = false;
        onDiscard();
      }}>Discard</button
    >
    <button
      class="btn btn-primary"
      onclick={() => {
        visible = false;
        onRestore();
      }}>Restore</button
    >
  {/snippet}
</AskDialog>

<style>
  .summary {
    padding-top: 10px;
    font-size: 13px;
    color: var(--color-text-muted);
  }
</style>
