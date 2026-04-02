<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import { TriangleAlertIcon } from '@lucide/svelte';

  let {
    visible = $bindable(false),
    onSave,
    onDiscard,
  }: {
    visible?: boolean;
    onSave: () => void;
    onDiscard: () => void;
  } = $props();
</script>

<Dialog bind:visible title="Unsaved Changes">
  <div class="message">
    <TriangleAlertIcon size={24} class="warning-icon" />
    <p>You have unsaved changes. Would you like to save before continuing?</p>
  </div>

  {#snippet actions()}
    <button onclick={() => (visible = false)}>Cancel</button>
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
        onSave();
      }}>Save</button
    >
  {/snippet}
</Dialog>

<style>
  .message {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    font-size: 14px;
    color: var(--color-text);

    p {
      margin: 0;
    }
  }

  :global(.warning-icon) {
    color: var(--color-warning);
    flex-shrink: 0;
  }
</style>
