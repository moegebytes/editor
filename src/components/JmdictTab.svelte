<script lang="ts">
  import type { DictEntry, Inflection } from '../lib/types';
  import { lookupJmdict } from '../lib/ipc';
  import { isKanji } from '../lib/utils';
  import KanjiDetail from './KanjiDetail.svelte';

  let {
    onNavigate,
  }: {
    onNavigate?: (term: string) => void;
  } = $props();

  let results: DictEntry[] = $state([]);
  let inflections: Inflection[] = $state([]);
  let loading = $state(false);
  let error: string | null = $state(null);
  let hasSearched = $state(false);
  let kanjiDetailRef: KanjiDetail | undefined = $state();

  export function lookup(q: string) {
    doLookup(q);
  }

  export function clear() {
    results = [];
    inflections = [];
    loading = false;
    error = null;
    hasSearched = false;
    kanjiDetailRef?.clear();
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
    kanjiDetailRef?.clear();
    try {
      const result = await lookupJmdict(trimmed);
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
  }

  function entryNotes(entry: DictEntry): string[] {
    const notes: string[] = [];
    for (const k of entry.kanji) {
      if (k.info) notes.push(`${k.text}: ${k.info}.`);
    }
    for (const r of entry.readings) {
      if (r.info) notes.push(`${r.text}: ${r.info}.`);
    }
    return notes;
  }

  function otherForms(entry: DictEntry): { kanji?: string; reading: string }[] {
    if (entry.kanji.length <= 1 && entry.readings.length <= 1) return [];
    const forms: { kanji?: string; reading: string }[] = [];
    if (entry.kanji.length > 1) {
      for (let i = 1; i < entry.kanji.length; i++) {
        const reading = entry.readings[i]?.text ?? entry.readings[0]?.text;
        if (reading) forms.push({ kanji: entry.kanji[i].text, reading });
      }
    }
    if (entry.readings.length > entry.kanji.length) {
      for (let i = Math.max(1, entry.kanji.length); i < entry.readings.length; i++) {
        forms.push({ reading: entry.readings[i].text });
      }
    }
    return forms;
  }
</script>

{#if loading}
  <div class="status">Searching...</div>
{:else if error}
  <div class="error">{error}</div>
{:else if hasSearched && results.length === 0 && inflections.length === 0}
  <div class="status">No results</div>
{:else if !hasSearched}
  <div class="status">Search for a word to look up.</div>
{/if}

{#each inflections as inf}
  <div class="inflection-hint">
    <div class="inflection-text">
      <span class="inflection-surface">{inf.surface}</span>
      could be an inflection of
      <button class="btn-icon inflection-base" onclick={() => navigateToBaseForm(inf.baseForm)}>{inf.baseForm}</button>
    </div>
    <div class="inflection-form">
      <span class="inflection-form-name">{inf.formName}.</span>
      {inf.description}
    </div>
  </div>
{/each}

<div class="dict-results">
  {#each results as entry (entry.entSeq)}
    {@const notes = entryNotes(entry)}
    {@const forms = otherForms(entry)}
    <div class="dict-entry">
      <div class="entry-headword">
        {#if entry.kanji.length > 0}
          <span class="kanji-text">
            {#each entry.kanji[0].text.split('') as ch}
              {#if isKanji(ch)}
                <button class="btn-icon kanji-link" onclick={() => kanjiDetailRef?.lookup(ch)}>{ch}</button>
              {:else}
                {ch}
              {/if}
            {/each}
          </span>
        {/if}
        <span class="reading-text">
          {entry.readings.map((r) => r.text).join('\u3001')}
        </span>
      </div>

      {#each entry.senses as sense, i}
        <div class="sense">
          {#if sense.pos.length > 0}
            <span class="pos-tags">
              {sense.pos.join(', ')}
            </span>
          {/if}
          <span class="sense-num">{i + 1}.</span>
          {sense.glosses.join('; ')}
          {#if sense.misc.length > 0}
            <span class="misc-tags">
              ({sense.misc.join(', ')})
            </span>
          {/if}
          {#if sense.xrefs.length > 0}
            <div class="xrefs">
              See also:
              {#each sense.xrefs as xref, xi}
                {@const term = xref.split('\u30FB')[0]}
                {#if xi > 0},
                {/if}
                <button class="btn-icon xref-link" onclick={() => onNavigate?.(term)}>{xref}</button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}

      {#if forms.length > 0}
        <div class="other-forms">
          <span class="other-forms-label">Other forms</span>
          {#each forms as form, fi}
            {#if fi > 0}、{/if}
            {#if form.kanji}
              {form.kanji}【{form.reading}】
            {:else}
              {form.reading}
            {/if}
          {/each}
        </div>
      {/if}

      {#if notes.length > 0}
        <div class="entry-notes">
          <span class="entry-notes-label">Notes</span>
          {notes.join(' ')}
        </div>
      {/if}
    </div>
  {/each}
</div>

<KanjiDetail bind:this={kanjiDetailRef} />

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

    .other-forms {
      margin-top: 6px;
      font-size: 13px;
      color: var(--color-text-muted);

      .other-forms-label {
        display: block;
        font-size: 11px;
        font-weight: 600;
        margin-bottom: 2px;
      }
    }

    .entry-notes {
      font-size: 12px;
      color: var(--color-text-muted);
      font-style: italic;
      margin-top: 4px;

      .entry-notes-label {
        display: block;
        font-size: 11px;
        font-weight: 600;
        font-style: normal;
        margin-bottom: 2px;
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

      .xrefs {
        font-size: 12px;
        color: var(--color-text-muted);
        margin-top: 2px;

        .xref-link {
          color: var(--color-accent);
          text-decoration: underline;
          text-decoration-style: dotted;
          text-underline-offset: 2px;

          &:hover {
            text-decoration-style: solid;
          }
        }
      }
    }
  }
</style>
