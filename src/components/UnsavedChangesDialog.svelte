<script lang="ts">
  import { TriangleAlertIcon } from '@lucide/svelte';

  import AskDialog from './ui/AskDialog.svelte';

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

<AskDialog bind:visible title="Unsaved Changes">
  {#snippet icon()}
    <TriangleAlertIcon size={24} color="var(--color-warning)" />
  {/snippet}

  {#snippet message()}
    <p>You have unsaved changes. Would you like to save before continuing?</p>
  {/snippet}

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
</AskDialog>
