export type EntryType = "text" | "comment" | "include" | "emit" | "reference" | "blank";

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

export interface DictEntry {
  entSeq: number;
  kanji: string[];
  readings: string[];
  senses: Sense[];
}

export interface Sense {
  pos: string[];
  glosses: string[];
  misc: string[];
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
  example: string;
  transliteration?: string;
  translation?: string;
}

export interface WiktDefinition {
  definition: string;
  parsedExamples: WiktExample[];
}

export interface WiktEntry {
  partOfSpeech: string;
  language: string;
  definitions: WiktDefinition[];
}

export interface WiktLanguageSection {
  code: string;
  language: string;
  entries: WiktEntry[];
}

export interface WiktResult {
  term: string;
  sections: WiktLanguageSection[];
  cached: boolean;
}
