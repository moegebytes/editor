# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project overview

Desktop app for translating Visual Novel script files. Reads paired JP/EN script files in a custom strings text format,
displays side-by-side, supports inline editing of English translations with offline Japanese dictionary lookup and
offline Wiktionary lookup.

## Tech stack

- **Rust + Tauri 2** backend, **Svelte 5 + TypeScript** frontend.
- **vibrato** with IPADIC MeCab 2.7.0 dictionary for Japanese morphological analysis.
- **JMdict** via SQLite (FTS5) for offline dictionary lookup.
- **KANJIDIC2** via SQLite for offline kanji lookup.
- **Wiktionary** via SQLite for offline lookup.

## Project Structure

```
editor/
├── CLAUDE.md
├── README.md
├── RESOURCES.md
├── LICENSE
├── package.json
├── index.html
├── vite.config.js
├── svelte.config.js
├── tsconfig.json
├── public/
│   └── fonts/                       -- Bundled fonts
│       └── NotoSansJP-Variable.woff2
├── src/                             -- Svelte 5 + TypeScript frontend
│   ├── App.svelte
│   ├── main.ts
│   ├── app.css
│   ├── lib/
│   │   ├── types.ts                 -- Types for IPC
│   │   ├── ipc.ts                   -- Tauri invoke wrappers
│   │   ├── dialogs.ts               -- Tauri dialog helpers (file/export/import pickers)
│   │   ├── utils.ts                 -- Shared utilities (entry predicates, set helpers, kanji detection)
│   │   ├── segment.ts               -- Generic text splitter for match highlighting
│   │   ├── debounce.svelte.ts       -- useDebouncedValue rune helper
│   │   ├── search.svelte.ts         -- Find-in-entries search state (match computation, navigation)
│   │   ├── toast.svelte.ts          -- Toast notification store (Svelte 5 runes module)
│   │   └── undo.svelte.ts           -- Undo/redo stack (command pattern)
│   └── components/
│       ├── dictionary/
│       │   ├── JmdictTab.svelte
│       │   ├── KanjiDetail.svelte
│       │   ├── KanjiText.svelte
│       │   └── WiktTab.svelte
│       ├── glossary/
│       │   ├── EntryDetail.svelte
│       │   ├── GlossaryList.svelte
│       │   └── GlossaryToolbar.svelte
│       ├── ui/
│       │   ├── AskDialog.svelte
│       │   ├── ContextMenu.svelte
│       │   ├── CopyButton.svelte
│       │   ├── Dialog.svelte
│       │   ├── DropdownMenu.svelte
│       │   ├── LoadingOverlay.svelte
│       │   ├── PathText.svelte
│       │   ├── ToastContainer.svelte
│       │   └── ViewHeader.svelte
│       ├── AboutDialog.svelte
│       ├── DictionaryPanel.svelte
│       ├── EditorTable.svelte
│       ├── FindReplaceBar.svelte
│       ├── GlossaryView.svelte
│       ├── GoToLineDialog.svelte
│       ├── ProjectHome.svelte
│       ├── RecoveryDialog.svelte
│       ├── SettingsView.svelte
│       ├── StatusBar.svelte
│       ├── Toolbar.svelte
│       └── UnsavedChangesDialog.svelte
├── src-tauri/                       -- Tauri 2 backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── src/
│       ├── main.rs                  -- Tauri app setup, dictionary loading
│       ├── commands.rs              -- IPC command handlers
│       ├── logging.rs               -- Per-session file logger setup
│       ├── settings.rs              -- App-wide settings persistence (settings.json)
│       ├── project.rs               -- Project config persistence
│       ├── strings.rs               -- Strings format parser/writer
│       ├── core.rs                  -- File pairing, FlatEntry type, notes, reconstruction
│       ├── jmdict.rs                -- JMdict SQLite queries + Vibrato tokenizer
│       ├── kanjidic.rs              -- KANJIDIC2 SQLite queries (kanji lookup)
│       ├── recovery.rs              -- Auto-save recovery file I/O
│       ├── util.rs                  -- Shared utilities (friendly IO error messages)
│       └── wiktionary.rs            -- Wiktionary offline SQLite queries
├── flatpak/                         -- Flatpak packaging
│   ├── com.moegebytes.yona.yml
│   └── com.moegebytes.yona.metainfo.xml
├── tools/
│   ├── Cargo.toml
│   ├── build-jmdict/                -- Offline tool: JMdict XML -> SQLite
│   ├── build-kanjidic/              -- Offline tool: KANJIDIC2 XML -> SQLite
│   └── build-wiktionary/            -- Offline tool: wiktextract JSONL -> SQLite
└── resources/
    ├── ipadic-mecab-v270.dict.zstd  -- Pre-compiled IPADIC for vibrato (zstd)
    ├── ipadic-mecab-v270.dict       -- Pre-compiled IPADIC for vibrato (decompressed, gitignored)
    ├── src/                         -- Compressed source data (zstd)
    │   ├── JMdict_e.xml.zst
    │   ├── kanjidic2.xml.zst
    │   └── enwiktionary-ja_en.jsonl.zst
    ├── gen/                         -- Generated files (gitignored)
    │   ├── jmdict.sqlite
    │   ├── kanjidic2.sqlite
    │   └── wiktionary.sqlite
```

**Architecture:** Single Tauri crate, modules for logic. `commands.rs` stays thin (deserialize -> call -> serialize). `tools/` is
a separate Cargo workspace.

**Logging:** Per-session log files in `config_dir/logs/` (10 max, auto-pruned). File logger at `Debug` level, console at
`Info` (release) or `Debug` (dev). Custom panic hook captures panic info + backtrace to log. All IPC commands log entry
at `debug!` level. Frontend `window.onerror` and `unhandledrejection` are forwarded to the backend via `log_error` and
logged with `(WebView)` prefix.

## Strings file format

| Line type  | Syntax                  | Parsed as                                             |
|------------|-------------------------|-------------------------------------------------------|
| Plain text | `Hello world`           | `StringsEntry::Text(String)`                          |
| Comment    | `; comment`             | `StringsEntry::Comment(String)`                       |
| Include    | `#include <path.txt>`   | `StringsEntry::Include { path, entries }` - recursive |
| Emit       | `#emit identifier`      | `StringsEntry::Emit(String)`                          |
| Reference  | `#reference <path.txt>` | `StringsEntry::Reference(String)`                     |
| Blank line | (empty)                 | `StringsEntry::Blank`                                 |

- Includes resolved relative to parent dir. Circular includes rejected.
- Parse -> write round-trips produce identical output (including included files).
- Comments preceding a text line are absorbed into its `notes` field. Comments before non-text entries stay as
structural rows.
- References rendered as clickable jump links in the editor.

## Layout

**Project workflow:** Starts at Project Home (recent + all projects, new project).

**Table:** Rows matched by position. Virtualized (`@tanstack/svelte-virtual`). Three-state: amber (untranslated),
yellow (translated), green (confirmed). Double-click row number to confirm/unconfirm. Includes have collapse/expand.
Emit/blank rows hidden. Notes indicator: `+` to add, `[N]` when present.

**Toolbar menus:** Project (Save/Export/Settings/Close), Line (Undo/Redo, Next Untranslated/Unconfirmed,
Confirm/Unconfirm), Tools (Dictionary/Go to Line/Find & Replace), Help (About). Filter input (CTRL+F) right-aligned.

**Undo/redo (Ctrl+Z / Ctrl+Y):** Command pattern with coalescing (consecutive text edits to same cell merge into one
entry) and compound actions (auto-confirm on Enter groups with preceding text edit, Replace All is one entry). Stack
bounded at 200, survives saves, resets on project close/open. Lives in frontend (`lib/undo.svelte.ts`).

**Dictionary panel:** Two tabs: **JMdict** and **Wiktionary**. Shared search input with search button.
Back/forward navigation (shared history, 50 entries max). Arrow buttons in header. JMdict tab: Vibrato tokenizes JP
text, looks up tokens, inflection detection with base form navigation with results ordered by priority.
Wiktionary tab: Entries grouped by etymology with senses, examples, synonyms, antonyms, coordinate terms, and other
relations. External query changes auto-search both tabs and reset to JMdict tab. Optional partial (prefix) search
includes prefix results alongside exact matches, with exact matches always ordered first (configurable in settings).

**Settings:** Three tabs: **Project**, **Editor**, **Dictionary**.
Project tab controls per-project settings (`ProjectSettings`). Other tabs control app-wide settings (`AppSettings`,
persisted in `settings.json`).

**Auto-save:** Periodic recovery file written to `{config_dir}/recovery/{project_id}.recovery.json` when there are
unsaved changes (default: every 120s, configurable in Editor settings, 0 = disabled). On project open, if a recovery
file exists, a dialog offers to restore or discard. Recovery files are cleaned up on explicit save or clean close.

**Find & Replace (Ctrl+H):** Searches EN text in filtered entries. Real-time match count, navigate Enter/Shift+Enter,
replace current or all.

**Glossary:** Per-project list of JP<->EN term translations (stored in project JSON as `glossary` array of
`{ jp, en, note? }`). Editable via Tools -> Glossary panel. Glossary JP terms are highlighted inline in the Japanese
text column with a dotted underline; hovering shows the expected EN translation and optional note.

**Context menu:** Custom (replaces WebView default). Cut/Copy/Paste/Select All on editable elements.

**Status bar:** Left: modified/saved. Right: confirmed/translated/total with dual progress bar.

## IPC

**ProjectWithEntries:** `id` + Project fields (name, files, confirmedLines, settings, glossary) via
`#[serde(flatten)]` + `entries: Vec<FlatEntry>`.

**FlatEntry** (`#[serde(rename_all = "camelCase")]`): `index`, `entry_type` (Text/Comment/Include/Emit/Blank),
`jp_text`, `en_text`, `source_file`, `depth` (0 = top level), `notes: Vec<String>` (without `;` prefix).

### Project management
```
create_project(name, files: {jp, en}) -> ProjectWithEntries
open_project(id) -> ProjectWithEntries
save_project() -> ()
close_project() -> ()
save_translation(entries) -> ()
confirm_line(index) -> ()
unconfirm_line(index) -> ()
get_project_info(id) -> ProjectInfo { name, files, settings }
update_project(id, name, files, settings) -> ()
delete_project(id) -> ()
list_recent_projects() -> Vec<RecentProject>
list_all_projects() -> Vec<RecentProject>
remove_recent_project(id) -> ()
export_project(dest_path) -> ()
open_app_dir() -> ()
```

### Import
```
preview_import(source_path) -> ImportPreview { name, confirmedCount }
import_project(source_path, name, files: {jp, en}) -> ProjectWithEntries
```

### Glossary
```
update_glossary(glossary: Vec<GlossaryEntry { jp, en, note? }>) -> ()
```

### Dictionary
```
lookup_jmdict(query) -> LookupResult { entries, inflections }
lookup_kanji(ch) -> Option<KanjiEntry>
```

### Wiktionary
```
lookup_wiktionary(term) -> WiktResult { term, entries: Vec<WiktWordEntry> }
```

### Recovery
```
write_recovery(entries: Map<index, RecoveryEntry { enText, notes }>) -> ()
check_recovery(id) -> Option<RecoveryInfo { timestamp }>
load_recovery(id) -> RecoveryData { entries: Map<index, RecoveryEntry>, confirmedLines, timestamp }
delete_recovery(id) -> ()
```

### App settings
```
get_app_settings() -> AppSettings { autoConfirmOnEnter, partialSearch, autoSaveIntervalSecs }
update_app_settings(settings) -> ()
```

### Logging
```
log_error(message, stack?) -> ()
```

### Environment
```
get_environment_info() -> EnvironmentInfo { appName, appVersion, tauriVersion, os, arch, debug }
```

## Build & development

```bash
# decompress resources (one-time, requires zstd)
zstd -d resources/ipadic-mecab-v270.dict.zst

# build JMdict database (one-time)
zstd -d resources/src/JMdict_e.xml.zst
cd tools && cargo run -p build-jmdict -- \
  ../resources/src/JMdict_e.xml ../resources/gen/

# build KANJIDIC2 database (one-time)
zstd -d resources/src/kanjidic2.xml.zst
cd tools && cargo run -p build-kanjidic -- \
  ../resources/src/kanjidic2.xml ../resources/gen/

# build Wiktionary database (one-time)
zstd -d resources/src/enwiktionary-ja_en.jsonl.zst
cd tools && cargo run -p build-wiktionary -- \
  ../resources/src/enwiktionary-ja_en.jsonl ../resources/gen/
```

```bash
pnpm install # install frontend deps
pnpm lint # lint (all linters in parallel)
pnpm format # format frontend code
pnpm check && pnpm test # run tests
pnpm tauri dev # dev mode with hot reload
pnpm tauri build # production build
pnpm clean # clean build artifacts
```

Verify that `pnpm lint` and `pnpm test` produce no warnings/errors before committing.

## Conventions

- **Rust:** `anyhow` for app errors, `thiserror` for module error types.
- **Linting:** `pnpm lint` runs ESLint, Prettier, clippy, and rustfmt in parallel. `printWidth`/`max_width` = 120.
- **TypeScript:** Strict mode. Svelte 5 runes (`$state`, `$derived`, `$props`). No legacy stores. Imports ordered in
groups separated by blank lines: external, internal libs (`./lib/*`), components (`./components/*`).
- **IPC boundary:** JSON via `serde` with `#[serde(rename_all = "camelCase")]`. Keep types flat.
- **Package management:** pnpm via corepack.
- **Testing:** Unit tests in `src-tauri/src/` modules.
- **@tanstack/svelte-virtual:** Svelte 5 requires `$derived(createVirtualizer(...))` with scroll element read at top
level (not inside closure).
