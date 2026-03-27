<script lang="ts">
  import type { DictEntry, Inflection, KanjiEntry } from "../lib/types";
  import { lookupWord, lookupKanji } from "../lib/ipc";
  import { isKanji } from "../lib/utils";
  import { XIcon } from "@lucide/svelte";

  let {
    query = "",
    querySeq = 0,
    visible = $bindable(true),
    width = $bindable(320),
  }: {
    query?: string;
    querySeq?: number;
    visible?: boolean;
    width?: number;
  } = $props();

  let results: DictEntry[] = $state([]);
  let inflections: Inflection[] = $state([]);
  let kanjiDetail: KanjiEntry | null = $state(null);
  let searchInput = $state("");
  let loading = $state(false);

  const MIN_WIDTH = 240;
  const MAX_WIDTH = 600;

  function clearState() {
    results = [];
    inflections = [];
    kanjiDetail = null;
    searchInput = "";
    loading = false;
  }

  function close() {
    clearState();
    visible = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && visible && !e.defaultPrevented) {
      e.preventDefault();
      close();
    }
  }

  // Clear state when panel is closed
  $effect(() => {
    if (!visible) clearState();
  });

  // React to external query changes
  let lastSeq = -1;
  $effect(() => {
    if (query && querySeq !== lastSeq) {
      lastSeq = querySeq;
      searchInput = query;
      doLookup(query);
    }
  });

  async function doLookup(q: string) {
    const trimmed = q.trim();
    if (!trimmed) {
      results = [];
      inflections = [];
      return;
    }
    loading = true;
    try {
      const result = await lookupWord(trimmed);
      results = result.entries;
      inflections = result.inflections;
    } catch {
      results = [];
      inflections = [];
    }
    loading = false;
  }

  function navigateToBaseForm(baseForm: string) {
    searchInput = baseForm;
    doLookup(baseForm);
  }

  async function handleKanjiClick(ch: string) {
    try {
      kanjiDetail = await lookupKanji(ch);
    } catch {
      kanjiDetail = null;
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      doLookup(searchInput);
    }
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();

    const startX = e.clientX;
    const startWidth = width;

    function onMove(e: MouseEvent) {
      const delta = e.clientX - startX;
      width = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
    }

    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

<svelte:window onkeydowncapture={handleKeydown} />

{#if visible}
  <div class="dict-panel" style:width="{width}px">
    <div class="dict-content">
      <div class="dict-header">
        <input
          type="text"
          placeholder="Search dictionary..."
          bind:value={searchInput}
          onkeydown={handleSearchKeydown}
          class="dict-search-input"
        />
        <button class="btn-icon dict-close" onclick={close}>
          <XIcon size={14} />
        </button>
      </div>

      {#if loading}
        <div class="dict-status">Searching...</div>
      {:else if results.length === 0 && inflections.length === 0 && searchInput}
        <div class="dict-status">No results</div>
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
                {entry.readings.join("、")}
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
              {kanjiDetail.onReadings.join("、")}
            </div>
          {/if}
          {#if kanjiDetail.kunReadings.length > 0}
            <div class="kanji-readings">
              <span class="reading-label">Kun:</span>
              {kanjiDetail.kunReadings.join("、")}
            </div>
          {/if}
          <div class="kanji-meanings">
            {kanjiDetail.meanings.join(", ")}
          </div>
        </div>
      {/if}
    </div>
    <div class="resize-handle" onmousedown={startResize}></div>
  </div>
{/if}

<style>
  .dict-panel {
    display: flex;
    flex-shrink: 0;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    height: 100%;
    position: relative;
  }

  .resize-handle {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
    transition: background 0.15s;

    &:hover {
      background: var(--color-accent);
    }
  }

  .dict-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .dict-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--color-border);
  }

  .dict-search-input {
    flex: 1;
  }

  .dict-close {
    padding: 4px 6px;
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
  }

  .dict-status {
    padding: 12px;
    color: var(--color-text-muted);
    font-size: 13px;
    text-align: center;
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
