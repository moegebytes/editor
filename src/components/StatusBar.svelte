<script lang="ts">
  let {
    modified,
    saving,
    stats,
  }: {
    modified: boolean;
    saving: boolean;
    stats: { totalText: number; translated: number; confirmed: number };
  } = $props();

  let confirmedPct = $derived(stats.totalText > 0 ? Math.round((stats.confirmed / stats.totalText) * 100) : 0);
  let translatedPct = $derived(stats.totalText > 0 ? Math.round((stats.translated / stats.totalText) * 100) : 0);
</script>

<div class="status-bar">
  <div class="status-left">
    {#if saving}
      <span class="status-saving">Saving...</span>
    {:else if modified}
      <span class="status-modified">Modified</span>
    {:else}
      <span class="status-saved">Saved</span>
    {/if}
  </div>

  {#if stats.totalText > 0}
    <div class="status-right">
      <span class="stat-count">
        {stats.confirmed}/{stats.translated}/{stats.totalText}
      </span>
      <span class="stat-pct">
        ({confirmedPct}% confirmed, {translatedPct}% translated)
      </span>
      <div class="progress-bar">
        <div class="progress-confirmed" style:width="{confirmedPct}%"></div>
        <div class="progress-translated" style:width="{translatedPct}%"></div>
      </div>
    </div>
  {/if}
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-text-muted);

    .status-left {
      display: flex;
      align-items: center;
      gap: 6px;

      .status-modified {
        color: var(--color-warning);
        font-weight: 600;
      }

      .status-saving {
        color: var(--color-accent);
        font-weight: 600;
      }

      .status-saved {
        color: var(--color-text-muted);
      }
    }

    .status-right {
      display: flex;
      align-items: center;
      gap: 8px;

      .stat-count {
        font-variant-numeric: tabular-nums;
      }

      .progress-bar {
        width: 120px;
        height: 6px;
        background: var(--color-border);
        border-radius: var(--radius-sm);
        overflow: hidden;
        position: relative;

        .progress-translated {
          position: absolute;
          top: 0;
          left: 0;
          height: 100%;
          background: var(--color-translated-progress);
          border-radius: var(--radius-sm);
        }

        .progress-confirmed {
          position: absolute;
          top: 0;
          left: 0;
          height: 100%;
          background: var(--color-accent);
          border-radius: var(--radius-sm);
          z-index: 1;
        }
      }
    }
  }
</style>
