<script lang="ts">
  import Dialog from './ui/Dialog.svelte';

  let {
    visible = $bindable(false),
    maxLine = 0,
    onGo,
  }: {
    visible?: boolean;
    maxLine: number;
    onGo: (line: number) => void;
  } = $props();

  let input = $state('');
  let inputEl: HTMLInputElement | undefined = $state();
  let error = $state('');

  $effect(() => {
    if (visible) {
      input = '';
      error = '';
      requestAnimationFrame(() => inputEl?.focus());
    }
  });

  function submit() {
    const n = parseInt(input, 10);
    if (isNaN(n) || n < 1 || n > maxLine) {
      error = `Enter a number between 1 and ${maxLine}`;
      return;
    }
    visible = false;
    onGo(n);
  }
</script>

<Dialog bind:visible title="Go to Line">
  <div class="go-input">
    <input
      type="text"
      bind:this={inputEl}
      bind:value={input}
      placeholder="Line number (1–{maxLine})"
      onkeydown={(e) => {
        if (e.key === 'Enter') submit();
      }}
    />
  </div>
  {#if error}
    <div class="form-error">{error}</div>
  {/if}

  {#snippet actions()}
    <button class="btn-primary" onclick={submit}>Go</button>
    <button onclick={() => (visible = false)}>Cancel</button>
  {/snippet}
</Dialog>

<style>
  .go-input input {
    width: 100%;
    padding: 8px 10px;
  }

  .form-error {
    color: var(--color-danger-light);
    font-size: 13px;
    margin-top: 8px;
  }
</style>
