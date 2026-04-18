<script lang="ts">
  import { isKanji } from '../../lib/utils';

  let { text, onKanjiClick }: { text: string; onKanjiClick: (ch: string) => void } = $props();
</script>

{#each text.split('') as ch}{#if isKanji(ch)}<button class="btn-icon kanji-link" onclick={() => onKanjiClick(ch)}
      >{ch}</button
    >{:else}{ch}{/if}{/each}

<style>
  .kanji-link {
    color: var(--color-accent);

    &:hover {
      text-decoration: underline;
    }
  }
</style>
