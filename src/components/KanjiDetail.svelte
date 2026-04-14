<script lang="ts">
  import { XIcon } from '@lucide/svelte';

  import type { KanjiEntry } from '../lib/types';
  import { lookupKanji } from '../lib/ipc';

  let kanjiDetail: KanjiEntry | null = $state(null);
  let error: string | null = $state(null);

  export async function lookup(ch: string) {
    try {
      kanjiDetail = await lookupKanji(ch);
    } catch (e) {
      kanjiDetail = null;
      error = `${e}`;
    }
  }

  export function clear() {
    kanjiDetail = null;
    error = null;
  }
</script>

{#if error}
  <div class="error">{error}</div>
{/if}

{#if kanjiDetail}
  <div class="kanji-detail">
    <div class="kanji-detail-header">
      <span class="kanji-large">{kanjiDetail.literal}</span>
      <button class="btn-icon kanji-close" onclick={() => (kanjiDetail = null)}><XIcon size={16} /></button>
    </div>
    <div class="kanji-meta">
      <span>{kanjiDetail.strokeCount} strokes</span>
      {#if kanjiDetail.grade}
        <span>Grade {kanjiDetail.grade}</span>
      {/if}
      {#if kanjiDetail.jlpt}
        <span>JLPT N{kanjiDetail.jlpt}</span>
      {/if}
      {#if kanjiDetail.freq}
        <span>Freq #{kanjiDetail.freq}</span>
      {/if}
    </div>
    {#if kanjiDetail.onReadings.length > 0}
      <div class="kanji-readings">
        <span class="reading-label">On:</span>
        {kanjiDetail.onReadings.join('\u3001')}
      </div>
    {/if}
    {#if kanjiDetail.kunReadings.length > 0}
      <div class="kanji-readings">
        <span class="reading-label">Kun:</span>
        {kanjiDetail.kunReadings.join('\u3001')}
      </div>
    {/if}
    <div class="kanji-meanings">
      {kanjiDetail.meanings.join(', ')}
    </div>
  </div>
{/if}

<style>
  .error {
    margin: 8px;
    padding: 8px 10px;
    font-size: 13px;
    color: var(--color-error-text);
    background: var(--color-error-bg);
    border: 1px solid var(--color-error-border);
    border-radius: 4px;
  }

  .kanji-detail {
    border-top: 2px solid var(--color-accent);
    padding: 12px;
    background: var(--color-bg);

    .kanji-detail-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;

      .kanji-large {
        font-size: 48px;
        line-height: 1;
      }

      .kanji-close {
        font-size: 16px;
        padding: 4px 8px;
      }
    }

    .kanji-meta {
      display: flex;
      gap: 12px;
      font-size: 12px;
      color: var(--color-text-muted);
      margin: 8px 0;
    }

    .kanji-readings {
      font-size: 14px;
      margin: 4px 0;

      .reading-label {
        font-weight: 600;
        color: var(--color-text-muted);
        font-size: 12px;
        margin-right: 4px;
      }
    }

    .kanji-meanings {
      margin-top: 8px;
      font-size: 14px;
    }
  }
</style>
