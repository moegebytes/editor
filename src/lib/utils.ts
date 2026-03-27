import type { FlatEntry } from "./types";

export function isText(entry: FlatEntry): boolean {
  return entry.entryType === "text";
}

export function isUntranslated(entry: FlatEntry): boolean {
  return entry.entryType === "text" && !!entry.jpText && (!entry.enText || entry.enText === "");
}

export function isTranslated(entry: FlatEntry): boolean {
  return entry.entryType === "text" && !!entry.jpText && !!entry.enText;
}

export function toggleSetMember<T>(set: Set<T>, item: T): Set<T> {
  const next = new Set(set);
  if (next.has(item)) {
    next.delete(item);
  } else {
    next.add(item);
  }
  return next;
}

export function getFileName(path: string): string {
  return path.replace(/^.*[/\\]/, "");
}

export function isKanji(ch: string): boolean {
  const code = ch.codePointAt(0) ?? 0;
  return (
    (code >= 0x4e00 && code <= 0x9fff) ||
    (code >= 0x3400 && code <= 0x4dbf) ||
    (code >= 0x20000 && code <= 0x2a6df)
  );
}
