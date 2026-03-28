<script lang="ts">
  import { toast } from "../../lib/toast.svelte";
  import {
    XIcon,
    CircleCheckIcon,
    CircleXIcon,
    InfoIcon,
  } from "@lucide/svelte";

  const iconMap = {
    error: CircleXIcon,
    success: CircleCheckIcon,
    info: InfoIcon,
  };

  // Insert zero-width spaces after path separators and colons so the browser
  // breaks long file paths at natural boundaries instead of mid-word.
  function softBreak(text: string): string {
    return text.replace(/([\\/:])(?=\S)/g, "$1\u200B");
  }
</script>

{#if toast.all.length > 0}
  <div class="toast-container">
    {#each toast.all as t (t.id)}
      {@const Icon = iconMap[t.type]}
      <div class="toast toast-{t.type}">
        <Icon size={16} />
        <span class="toast-message">{softBreak(t.message)}</span>
        <button class="toast-dismiss btn-icon" onclick={() => toast.dismiss(t.id)}>
          <XIcon size={14} />
        </button>
        {#if t.duration > 0}
          <div
            class="toast-timer"
            style="animation-duration: {t.duration}ms"
          ></div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 36px;
    right: 12px;
    z-index: 500;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    max-width: 520px;
    pointer-events: none;
  }

  .toast {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 12px 14px;
    border-radius: 6px;
    border: 1px solid;
    font-size: 14px;
    line-height: 1.4;
    min-width: 240px;
    box-shadow: var(--shadow-dropdown);
    pointer-events: auto;
    animation: toast-slide-in 0.2s ease-out;
    overflow: hidden;
  }

  /*noinspection CssUnusedSymbol*/
  .toast-error {
    background: var(--color-error-bg);
    color: var(--color-error-text);
    border-color: var(--color-error-border);
  }

  /*noinspection CssUnusedSymbol*/
  .toast-success {
    background: var(--color-success-bg);
    color: var(--color-success-text);
    border-color: var(--color-success-border);
  }

  /*noinspection CssUnusedSymbol*/
  .toast-info {
    background: var(--color-surface-alt);
    color: var(--color-text);
    border-color: var(--color-accent);
  }

  .toast-message {
    flex: 1;
    min-width: 0;
    word-wrap: break-word;
  }

  .toast-dismiss {
    flex-shrink: 0;
    color: inherit;
    opacity: 0.6;
    padding: 0 2px;

    &:hover {
      opacity: 1;
    }
  }

  .toast-timer {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: currentColor;
    opacity: 0.3;
    animation: toast-timer-shrink linear forwards;
    transform-origin: left;
  }

  @keyframes toast-timer-shrink {
    from { transform: scaleX(1); }
    to { transform: scaleX(0); }
  }

  @keyframes toast-slide-in {
    from {
      opacity: 0;
      transform: translateX(16px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
</style>
