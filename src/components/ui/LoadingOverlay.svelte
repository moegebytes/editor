<script lang="ts">
  import { LoaderCircleIcon } from "@lucide/svelte";

  let { visible }: { visible: boolean } = $props();
</script>

{#if visible}
  <div class="overlay">
    <LoaderCircleIcon size={64} class="spinner" />
  </div>
{/if}

<style>
  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 100;
  }

  .overlay :global(.spinner) {
    color: var(--color-accent);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
