<script lang="ts">
  import type { WiktResult } from "../lib/types";
  import { lookupWiktionary } from "../lib/ipc";

  let {
    cached = $bindable(false),
  }: {
    cached?: boolean;
  } = $props();

  let wiktResult: WiktResult | null = $state(null);
  let wiktLoading = $state(false);
  let wiktError: string | null = $state(null);

  export function lookup(q: string) {
    doWiktLookup(q);
  }

  export function clear() {
    wiktResult = null;
    wiktLoading = false;
    wiktError = null;
    cached = false;
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
    try {
      wiktResult = await lookupWiktionary(trimmed);
      if (wiktResult.sections.length === 0) {
        wiktResult = null;
        wiktError = `No Wiktionary entry found for '${trimmed}'.`;
      }
    } catch (e) {
      wiktResult = null;
      wiktError = `${e}`;
    }
    wiktLoading = false;
    cached = wiktResult?.cached ?? false;
  }
</script>

{#if wiktLoading}
  <div class="status">Looking up on Wiktionary...</div>
{:else if wiktError}
  <div class="error">{wiktError}</div>
{:else if !wiktResult}
  <div class="status">Press Enter or click search to look up</div>
{/if}

{#if wiktResult}
  <div class="wikt-results">
    {#each wiktResult.sections as section, i}
      <details open={section.code === "ja" || i === 0}>
        <summary class="wikt-lang-header">{section.language}</summary>
        <div class="wikt-section-content">
          {#each section.entries as entry}
            <div class="wikt-entry">
              <div class="wikt-entry-header">
                <span class="wikt-pos-tag">{entry.partOfSpeech}</span>
                {#if entry.language !== section.language}
                  <span class="wikt-entry-lang">{entry.language}</span>
                {/if}
              </div>
              {#each entry.definitions as def}
                {#if def.definition}
                  <div class="wikt-def">
                    <span class="wikt-def-bullet">&bull;</span>
                    <span class="wikt-def-text">{@html def.definition}</span>
                  </div>
                  {#each def.parsedExamples as ex}
                    <div class="wikt-example">
                      <div class="wikt-example-jp">{@html ex.example}</div>
                      {#if ex.transliteration}
                        <div class="wikt-example-romaji">{@html ex.transliteration}</div>
                      {/if}
                      {#if ex.translation}
                        <div class="wikt-example-en">{@html ex.translation}</div>
                      {/if}
                    </div>
                  {/each}
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      </details>
    {/each}
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

  .wikt-results {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 8px;
  }

  .wikt-lang-header {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-accent);
    padding: 8px 0 4px;
    cursor: pointer;
    user-select: none;
  }

  .wikt-section-content {
    padding: 0 4px 8px;
    border-bottom: 1px solid var(--color-border);
  }

  .wikt-entry {
    padding: 6px 0;
    border-bottom: 1px solid var(--color-border);

    &:last-child {
      border-bottom: none;
    }

    .wikt-entry-header {
      display: flex;
      align-items: baseline;
      gap: 6px;
      margin-bottom: 2px;

      .wikt-pos-tag {
        color: var(--color-accent);
        font-size: 11px;
        font-style: italic;
      }

      .wikt-entry-lang {
        color: var(--color-text-muted);
        font-size: 11px;
      }
    }

    .wikt-def {
      font-size: 13px;
      line-height: 1.5;
      margin-left: 8px;

      .wikt-def-bullet {
        color: var(--color-text-muted);
        margin-right: 4px;
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
  }

  /* Inline HTML within Wiktionary definition entry */
  .wikt-section-content {
    :global(a) {
      color: var(--color-accent);
      text-decoration: none;
      pointer-events: none;
    }

    :global(ol), :global(ul) {
      margin: 2px 0;
      padding-left: 18px;
      font-size: 12px;
      color: var(--color-text-muted);
    }

    :global(li) {
      color: var(--color-text);
    }

    :global(ol) {
      list-style: none;
      counter-reset: wikt-ol;
      padding-left: 12px;
    }

    :global(ol > li) {
      counter-increment: wikt-ol;

      &::before {
        content: counter(wikt-ol) ". ";
        color: var(--color-text-muted);
      }
    }

    :global(ruby) {
      ruby-align: center;
    }

    :global(rt) {
      font-size: 0.6em;
      color: var(--color-text-muted);
    }
  }
</style>
