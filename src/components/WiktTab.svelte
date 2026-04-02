<script lang="ts">
  import type { WiktResult, WiktRelation } from '../lib/types';
  import { lookupWiktionary } from '../lib/ipc';
  import { isKanji } from '../lib/utils';
  import KanjiDetail from './KanjiDetail.svelte';

  let {
    onNavigate,
  }: {
    onNavigate?: (term: string) => void;
  } = $props();

  let wiktResult: WiktResult | null = $state(null);
  let wiktLoading = $state(false);
  let wiktError: string | null = $state(null);
  let hasSearched = $state(false);
  let kanjiDetailRef: KanjiDetail | undefined = $state();

  export function lookup(q: string) {
    doWiktLookup(q);
  }

  export function clear() {
    wiktResult = null;
    wiktLoading = false;
    wiktError = null;
    hasSearched = false;
    kanjiDetailRef?.clear();
  }

  async function doWiktLookup(q: string) {
    const trimmed = q.trim();
    if (!trimmed) {
      wiktResult = null;
      wiktError = null;
      return;
    }
    wiktLoading = true;
    wiktError = null;
    hasSearched = true;
    kanjiDetailRef?.clear();
    try {
      wiktResult = await lookupWiktionary(trimmed);
      if (wiktResult.entries.length === 0) {
        wiktResult = null;
      }
    } catch (e) {
      wiktResult = null;
      wiktError = `${e}`;
    }
    wiktLoading = false;
  }

  function navigateTo(term: string) {
    onNavigate?.(term);
  }

  interface RelationGroup {
    kind: string;
    label: string;
    items: WiktRelation[];
  }

  function groupRelations(rels: WiktRelation[]): RelationGroup[] {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const map = new Map<string, WiktRelation[]>();
    for (const r of rels) {
      const list = map.get(r.kind);
      if (list) {
        list.push(r);
      } else {
        map.set(r.kind, [r]);
      }
    }
    const groups: RelationGroup[] = [];
    for (const [kind, items] of map) {
      if (kind === 'derived') continue;
      groups.push({ kind, label: kind.replace(/_/g, ' '), items });
    }
    // Derived terms last, if any
    const derived = map.get('derived');
    if (derived) {
      groups.push({ kind: 'derived', label: 'derived', items: derived });
    }
    return groups;
  }
</script>

{#if wiktLoading}
  <div class="status">Searching...</div>
{:else if wiktError}
  <div class="error">{wiktError}</div>
{:else if hasSearched && !wiktResult}
  <div class="status">No results</div>
{:else if !hasSearched}
  <div class="status">Search for a word to look up.</div>
{/if}

{#if wiktResult}
  <div class="wikt-results">
    {#each wiktResult.entries as entry (entry.id)}
      <div class="wikt-entry">
        <div class="entry-headword">
          <span class="headword-text">
            {#each entry.word.split('') as ch}
              {#if isKanji(ch)}
                <button class="btn-icon kanji-link" onclick={() => kanjiDetailRef?.lookup(ch)}>{ch}</button>
              {:else}
                {ch}
              {/if}
            {/each}
          </span>
          {#if entry.reading}
            <span class="reading-text">{entry.reading}</span>
          {/if}
          {#if entry.romaji}
            <span class="romaji-text">{entry.romaji}</span>
          {/if}
          {#if entry.ipa}
            <span class="ipa-text">{entry.ipa}</span>
          {/if}
        </div>

        <span class="pos-tags">{entry.pos}</span>

        {#each entry.senses as sense, i}
          <div class="sense">
            <div class="sense-line">
              <span class="sense-num">{i + 1}.</span>
              <span class="sense-gloss">{sense.gloss}</span>
              {#if sense.tags.length > 0}
                <span class="misc-tags">({sense.tags.join(', ')})</span>
              {/if}
            </div>

            {#each sense.examples as ex}
              <div class="wikt-example">
                <div class="wikt-example-jp">{ex.text}</div>
                {#if ex.romaji}
                  <div class="wikt-example-romaji">{ex.romaji}</div>
                {/if}
                {#if ex.english}
                  <div class="wikt-example-en">{ex.english}</div>
                {/if}
              </div>
            {/each}

            {#if sense.relations.length > 0}
              {@const groups = groupRelations(sense.relations)}
              {#each groups as group}
                <div class="relation-group" class:derived-group={group.kind === 'derived'}>
                  <span class="relation-label">{group.label}:</span>
                  {#each group.items as rel}
                    <button class="relation-link" onclick={() => navigateTo(rel.term)}>{rel.term}</button>
                  {/each}
                </div>
              {/each}
            {/if}
          </div>
        {/each}

        {#if entry.relations.length > 0}
          {@const groups = groupRelations(entry.relations)}
          <div class="entry-relations">
            {#each groups as group}
              <div class="relation-group" class:derived-group={group.kind === 'derived'}>
                <span class="relation-label">{group.label}:</span>
                {#each group.items as rel}
                  <button class="relation-link" onclick={() => navigateTo(rel.term)}>{rel.term}</button>
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

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

  .wikt-results {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .wikt-entry {
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);

    .entry-headword {
      margin-bottom: 4px;

      .headword-text {
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

      .romaji-text {
        color: var(--color-text-muted);
        font-size: 13px;
        font-style: italic;
        margin-left: 6px;
      }

      .ipa-text {
        color: var(--color-text-muted);
        font-size: 11px;
        margin-left: 6px;
      }
    }

    .pos-tags {
      display: block;
      color: var(--color-accent);
      font-size: 11px;
      font-style: italic;
      margin-bottom: 2px;
    }

    .sense {
      font-size: 13px;
      line-height: 1.5;
      margin-left: 8px;
      margin-bottom: 4px;

      .sense-num {
        color: var(--color-text-muted);
        margin-right: 4px;
      }

      .misc-tags {
        color: var(--color-text-muted);
        font-size: 11px;
        font-style: italic;
      }
    }

    .wikt-example {
      margin: 4px 0 6px 16px;
      padding-left: 8px;
      border-left: 2px solid var(--color-border);
      font-size: 12px;
      line-height: 1.5;

      .wikt-example-jp {
        font-size: 14px;
      }

      .wikt-example-romaji {
        color: var(--color-text-muted);
        font-style: italic;
      }

      .wikt-example-en {
        color: var(--color-text-muted);
      }
    }

    .relation-group {
      font-size: 12px;
      margin: 2px 0 2px 16px;
      line-height: 1.5;

      .relation-label {
        color: var(--color-accent);
        font-style: italic;
        margin-right: 4px;
      }

      .relation-link {
        color: var(--color-accent);
        background: none;
        border: none;
        padding: 0;
        font: inherit;
        cursor: pointer;
        text-decoration: underline;
        text-decoration-style: dotted;
        text-underline-offset: 2px;

        &::after {
          content: ',\a0';
          text-decoration: none;
          color: var(--color-text);
        }

        &:last-child::after {
          content: none;
        }

        &:hover {
          text-decoration-style: solid;
        }
      }
    }

    .derived-group {
      color: var(--color-text-muted);

      .relation-link {
        color: var(--color-text-muted);
      }
    }

    .entry-relations {
      margin-top: 4px;
      padding-top: 4px;
      border-top: 1px dashed var(--color-border);
    }
  }
</style>
