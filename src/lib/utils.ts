import { platform } from '@tauri-apps/plugin-os';
import type { FlatEntry } from './types';

export function isText(entry: FlatEntry): boolean {
  return entry.entryType === 'text';
}

export function isUntranslated(entry: FlatEntry): boolean {
  return entry.entryType === 'text' && !!entry.jpText && (!entry.enText || entry.enText === '');
}

export function isTranslated(entry: FlatEntry): boolean {
  return entry.entryType === 'text' && !!entry.jpText && !!entry.enText;
}

export function getFileName(path: string): string {
  return path.replace(/^.*[/\\]/, '');
}

export function modKey(e: KeyboardEvent): boolean {
  return platform() === 'macos' ? e.metaKey : e.ctrlKey;
}

export function isWordBoundary(prev: string | null, next: string | null): boolean {
  if (!prev || !next) return true;
  const diff = next.length - prev.length;
  if (diff > 0) {
    for (let i = 0; i < next.length; i++) {
      if (i >= prev.length || next[i] !== prev[i]) return /\W/.test(next[i]);
    }
  } else if (diff < 0) {
    for (let i = 0; i < prev.length; i++) {
      if (i >= next.length || prev[i] !== next[i]) return /\W/.test(prev[i]);
    }
  }
  return true;
}

export function isKanji(ch: string): boolean {
  const code = ch.codePointAt(0) ?? 0;
  return (
    (code >= 0x4e00 && code <= 0x9fff) || (code >= 0x3400 && code <= 0x4dbf) || (code >= 0x20000 && code <= 0x2a6df)
  );
}
