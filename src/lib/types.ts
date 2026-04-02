export type EntryType = 'text' | 'comment' | 'include' | 'emit' | 'reference' | 'blank';

export interface FlatEntry {
  index: number;
  entryType: EntryType;
  jpText: string | null;
  enText: string | null;
  sourceFile: string | null;
  depth: number;
  notes: string[];
}

export interface Inflection {
  surface: string;
  baseForm: string;
  formName: string;
  description: string;
}

export interface LookupResult {
  entries: DictEntry[];
  inflections: Inflection[];
}

export interface KanjiForm {
  text: string;
  info?: string;
}

export interface ReadingForm {
  text: string;
  info?: string;
}

export interface DictEntry {
  entSeq: number;
  kanji: KanjiForm[];
  readings: ReadingForm[];
  senses: Sense[];
}

export interface Sense {
  pos: string[];
  glosses: string[];
  misc: string[];
  xrefs: string[];
}

export interface ProjectSettings {
  autoConfirmOnEnter: boolean;
}

export interface ProjectFiles {
  jp: string;
  en: string;
}

export interface Project {
  id: string;
  name: string;
  files: ProjectFiles;
  confirmedLines: number[];
  settings: ProjectSettings;
  entries: FlatEntry[];
}

export interface ProjectInfo {
  name: string;
  files: ProjectFiles;
  settings: ProjectSettings;
}

export interface RecentProject {
  name: string;
  id: string;
}

export interface ImportPreview {
  name: string;
  confirmedCount: number;
}

export interface KanjiEntry {
  literal: string;
  grade: number | null;
  strokeCount: number;
  jlpt: number | null;
  freq: number | null;
  onReadings: string[];
  kunReadings: string[];
  meanings: string[];
}

export interface WiktExample {
  text: string;
  english?: string;
  romaji?: string;
}

export interface WiktRelation {
  kind: string;
  term: string;
}

export interface WiktSense {
  gloss: string;
  tags: string[];
  examples: WiktExample[];
  relations: WiktRelation[];
}

export interface WiktWordEntry {
  id: number;
  word: string;
  pos: string;
  langCode?: string;
  sortGroup?: number;
  reading?: string;
  romaji?: string;
  ipa?: string;
  senses: WiktSense[];
  relations: WiktRelation[];
}

export interface WiktResult {
  term: string;
  entries: WiktWordEntry[];
}

export interface EnvironmentInfo {
  appName: string;
  appVersion: string;
  tauriVersion: string;
  os: string;
  arch: string;
  debug: boolean;
}
