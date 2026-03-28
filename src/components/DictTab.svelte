<script lang="ts">
  import type { DictEntry, Inflection, KanjiEntry } from "../lib/types";
  import { lookupWord, lookupKanji } from "../lib/ipc";
  import { isKanji } from "../lib/utils";
  import { XIcon } from "@lucide/svelte";

  let {
    onNavigate,
  }: {
    onNavigate?: (term: string) => void;
  } = $props();

  let results: DictEntry[] = $state([]);
  let inflections: Inflection[] = $state([]);
  let kanjiDetail: KanjiEntry | null = $state(null);
  let loading = $state(false);
  let error: string | null = $state(null);
  let hasSearched = $state(false);

  export function lookup(q: string) {
    doLookup(q);
  }

  export function clear() {
    results = [];
    inflections = [];
    kanjiDetail = null;
    loading = false;
    error = null;
    hasSearched = false;
  }

  async function doLookup(q: string) {
    const trimmed = q.trim();
    if (!trimmed) {
      results = [];
      inflections = [];
      error = null;
      return;
    }
    loading = true;
    error = null;
    hasSearched = true;
    try {
      const result = await lookupWord(trimmed);
      results = result.entries;
      inflections = result.inflections;
    } catch (e) {
      results = [];
      inflections = [];
      error = `${e}`;
    }
    loading = false;
  }

  function navigateToBaseForm(baseForm: string) {
    onNavigate?.(baseForm);
    doLookup(baseForm);
  }

  async function handleKanjiClick(ch: string) {
    try {
      kanjiDetail = await lookupKanji(ch);
    } catch (e) {
      kanjiDetail = null;
      error = `${e}`;
    }
  }
</script>

{#if loading}
  <div class="status">Searching...</div>
{:else if error}
  <div class="error">{error}</div>
{:else if hasSearched && results.length === 0 && inflections.length === 0}
  <div class="status">No results</div>
{/if}

{#each inflections as inf}
  <div class="inflection-hint">
    <div class="inflection-text">
      <span class="inflection-surface">{inf.surface}</span>
      could be an inflection of
      <button
        class="btn-icon inflection-base"
        onclick={() => navigateToBaseForm(inf.baseForm)}
      >{inf.baseForm}</button>
    </div>
    <div class="inflection-form">
      <span class="inflection-form-name">{inf.formName}.</span>
      {inf.description}
    </div>
  </div>
{/each}

<div class="dict-results">
  {#each results as entry (entry.entSeq)}
    <div class="dict-entry">
      <div class="entry-headword">
        {#if entry.kanji.length > 0}
          <span class="kanji-text">
            {#each entry.kanji[0].split("") as ch}
              {#if isKanji(ch)}
                <button
                  class="btn-icon kanji-link"
                  onclick={() => handleKanjiClick(ch)}
                >{ch}</button>
              {:else}
                {ch}
              {/if}
            {/each}
          </span>
        {/if}
        <span class="reading-text">
          {entry.readings.join("\u3001")}
        </span>
      </div>

      {#each entry.senses as sense, i}
        <div class="sense">
          {#if sense.pos.length > 0}
            <span class="pos-tags">
              {sense.pos.join(", ")}
            </span>
          {/if}
          <span class="sense-num">{i + 1}.</span>
          {sense.glosses.join("; ")}
          {#if sense.misc.length > 0}
            <span class="misc-tags">
              ({sense.misc.join(", ")})
            </span>
          {/if}
        </div>
      {/each}
    </div>
  {/each}
</div>

{#if kanjiDetail}
  <div class="kanji-detail">
    <div class="kanji-detail-header">
      <span class="kanji-large">{kanjiDetail.literal}</span>
      <button
        class="btn-icon kanji-close"
        onclick={() => (kanjiDetail = null)}
      ><XIcon size={16} /></button>
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
        {kanjiDetail.onReadings.join("\u3001")}
      </div>
    {/if}
    {#if kanjiDetail.kunReadings.length > 0}
      <div class="kanji-readings">
        <span class="reading-label">Kun:</span>
        {kanjiDetail.kunReadings.join("\u3001")}
      </div>
    {/if}
    <div class="kanji-meanings">
      {kanjiDetail.meanings.join(", ")}
    </div>
  </div>
{/if}

<style>
  .status {
    padding: 12px;
    color: var(--color-text-muted);
    font-size: 13px;
    text-align: center;
  }

  .error {
    margin: 8px;
    padding: 8px 10px;
    font-size: 13px;
    color: var(--color-error-text);
    background: var(--color-error-bg);
    border: 1px solid var(--color-error-border);
    border-radius: 4px;
  }

  .inflection-hint {
    margin: 8px;
    padding: 8px 10px;
    background: var(--color-surface-alt);
    border: 1px solid var(--color-border);
    border-left: 3px solid var(--color-accent);
    border-radius: 4px;
    font-size: 13px;
    line-height: 1.5;

    .inflection-surface {
      font-weight: 600;
    }

    .inflection-base {
      color: var(--color-accent);
      font-weight: 600;
      text-decoration: underline;
      text-decoration-style: dotted;
      text-underline-offset: 2px;

      &:hover {
        text-decoration-style: solid;
      }
    }

    .inflection-form {
      margin-top: 4px;
      color: var(--color-text-muted);
      font-size: 12px;

      .inflection-form-name {
        font-weight: 600;
        color: var(--color-text);
      }
    }
  }

  .dict-results {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .dict-entry {
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);

    .entry-headword {
      margin-bottom: 4px;

      .kanji-text {
        font-size: 20px;
        margin-right: 8px;

        .kanji-link {
          color: var(--color-accent);

          &:hover {
            text-decoration: underline;
          }
        }
      }

      .reading-text {
        color: var(--color-text-muted);
        font-size: 14px;
      }
    }

    .sense {
      font-size: 13px;
      line-height: 1.5;
      margin-left: 8px;

      .sense-num {
        color: var(--color-text-muted);
        margin-right: 4px;
      }

      .pos-tags {
        display: block;
        color: var(--color-accent);
        font-size: 11px;
        font-style: italic;
        margin-bottom: 2px;
      }

      .misc-tags {
        color: var(--color-text-muted);
        font-size: 11px;
        font-style: italic;
      }
    }
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
