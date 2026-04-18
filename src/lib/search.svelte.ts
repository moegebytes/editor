import type { FlatEntry } from './types';
import { isText } from './utils';

export class SearchState {
  query = $state('');
  caseSensitive = $state(false);
  matchIndices: number[] = $state([]);
  currentMatch = $state(-1);

  #entries: () => FlatEntry[];
  #filterText: () => string;

  constructor(entries: () => FlatEntry[], filterText: () => string) {
    this.#entries = entries;
    this.#filterText = filterText;
  }

  compute(query: string) {
    this.query = query;
    if (!query) {
      this.matchIndices = [];
      this.currentMatch = -1;
      return;
    }
    const needle = this.caseSensitive ? query : query.toLowerCase();
    const fLower = this.#filterText().toLowerCase();
    const matches: number[] = [];
    for (const entry of this.#entries()) {
      if (!isText(entry)) continue;
      const enText = entry.enText ?? '';
      if (fLower) {
        const jpLower = (entry.jpText ?? '').toLowerCase();
        const enLower = enText.toLowerCase();
        if (!jpLower.includes(fLower) && !enLower.includes(fLower)) continue;
      }
      const haystack = this.caseSensitive ? enText : enText.toLowerCase();
      if (haystack.includes(needle)) matches.push(entry.index);
    }
    this.matchIndices = matches;
    this.currentMatch = matches.length > 0 ? 0 : -1;
  }

  next(): number | undefined {
    if (this.matchIndices.length === 0) return;
    this.currentMatch = (this.currentMatch + 1) % this.matchIndices.length;
    return this.matchIndices[this.currentMatch];
  }

  prev(): number | undefined {
    if (this.matchIndices.length === 0) return;
    this.currentMatch = (this.currentMatch - 1 + this.matchIndices.length) % this.matchIndices.length;
    return this.matchIndices[this.currentMatch];
  }

  reset() {
    this.query = '';
    this.caseSensitive = false;
    this.matchIndices = [];
    this.currentMatch = -1;
  }

  normalize(s: string): string {
    return this.caseSensitive ? s : s.toLowerCase();
  }
}
