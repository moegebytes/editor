export interface Segment<T> {
  text: string;
  match?: T;
}

export function splitByMatches<T>(text: string, terms: readonly T[], getKey: (term: T) => string): Segment<T>[] {
  if (!text || terms.length === 0) return [{ text }];

  const hits: { start: number; end: number; term: T }[] = [];
  for (const term of terms) {
    const key = getKey(term);
    if (!key) continue;
    let pos = 0;
    while ((pos = text.indexOf(key, pos)) !== -1) {
      hits.push({ start: pos, end: pos + key.length, term });
      pos += key.length;
    }
  }

  if (hits.length === 0) return [{ text }];

  // Prefer longer matches when two start at the same position.
  hits.sort((a, b) => a.start - b.start || b.end - a.end);

  const filtered: typeof hits = [];
  let lastEnd = 0;
  for (const h of hits) {
    if (h.start >= lastEnd) {
      filtered.push(h);
      lastEnd = h.end;
    }
  }

  const segments: Segment<T>[] = [];
  let cursor = 0;
  for (const h of filtered) {
    if (h.start > cursor) segments.push({ text: text.slice(cursor, h.start) });
    segments.push({ text: text.slice(h.start, h.end), match: h.term });
    cursor = h.end;
  }
  if (cursor < text.length) segments.push({ text: text.slice(cursor) });
  return segments;
}
