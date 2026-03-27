# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

Desktop app for translating Visual Novel script files. Reads paired JP/EN script files in a custom strings text format, displays side-by-side, supports inline editing of English translations with offline Japanese dictionary lookup.

## Tech Stack

- **Rust + Tauri 2** backend, **Svelte 5 + TypeScript** frontend.
- **vibrato** (pure Rust, IPADIC MeCab 2.7.0) for Japanese morphological analysis.
- **JMdict + KANJIDIC2** via SQLite (FTS5) for offline dictionary lookup. `rusqlite` with `bundled` feature.

## Project Structure

```
editor/
├── CLAUDE.md
├── package.json
├── index.html
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── src/                         -- Svelte 5 + TypeScript frontend
│   ├── App.svelte
│   ├── main.ts
│   ├── app.css
│   ├── lib/
│   │   ├── types.ts             -- Types for IPC
│   │   ├── ipc.ts               -- Tauri invoke wrappers
│   │   └── utils.ts             -- Shared utilities (entry predicates, set helpers, kanji detection)
│   └── components/
│       ├── ui/
│       │   ├── ContextMenu.svelte
│       │   ├── Dialog.svelte
│       │   ├── DropdownMenu.svelte
│       │   └── LoadingOverlay.svelte
│       ├── DictionaryPanel.svelte
│       ├── EditorTable.svelte
│       ├── FindReplaceBar.svelte
│       ├── GoToLineDialog.svelte
│       ├── ProjectHome.svelte
│       ├── SettingsView.svelte
│       ├── StatusBar.svelte
│       ├── Toolbar.svelte
│       └── UnsavedChangesDialog.svelte
├── src-tauri/                   -- Tauri 2 backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── src/
│       ├── main.rs              -- Tauri app setup, dictionary loading
│       ├── commands.rs          -- IPC command handlers
│       ├── project.rs           -- Project config persistence
│       ├── strings.rs           -- Strings format parser/writer
│       ├── core.rs              -- File pairing, FlatEntry type, notes, reconstruction
│       └── dictionary.rs        -- JMdict/KANJIDIC2 SQLite queries + Vibrato tokenizer
├── tools/
│   ├── Cargo.toml
│   └── build-dictionary/        -- Offline tool: JMdict XML + KANJIDIC2 XML -> SQLite
└── resources/
    ├── JMdict_e.xml             -- Dictionary XML source (git-lfs)
    ├── kanjidic2.xml            -- Kanji XML source (git-lfs)
    ├── ipadic-mecab-v270.dict   -- Pre-compiled IPADIC for vibrato (git-lfs, 46 MB)
    ├── gen/                     -- Generated files (gitignored)
    │   ├── jmdict.sqlite
    │   └── kanjidic2.sqlite
```

**Architecture:** Single Tauri crate, modules for logic. `commands.rs` stays thin (deserialize -> call -> serialize). `tools/` is a separate Cargo workspace.

## Strings file format

| Line type | Syntax | Parsed as |
|---|---|---|
| Plain text | `Hello world` | `StringsEntry::Text(String)` |
| Comment | `; comment` | `StringsEntry::Comment(String)` |
| Include | `#include <path.txt>` | `StringsEntry::Include { path, entries }` — recursive |
| Emit | `#emit identifier` | `StringsEntry::Emit(String)` |
| Reference | `#reference <path.txt>` | `StringsEntry::Reference(String)` |
| Blank line | (empty) | `StringsEntry::Blank` |

- Includes resolved relative to parent dir. Circular includes rejected.
- Parse -> write round-trips produce identical output (including included files).
- Comments preceding a text line are absorbed into its `notes` field. Comments before non-text entries stay as structural rows.
- References rendered as clickable jump links in the editor.

## UI Layout

**Toolbar** -> **Dictionary panel** (left, resizable) + **Editor table** (main) -> **Find/Replace bar** (bottom) -> **Status bar**.

**Project workflow:** Starts at Project Home (recent + all projects, new project). Projects stored as `{uuid}.json` in `~/.config/com.moegebytes.yona/projects/`. `recent.json` stores UUIDs only.

**Table:** Rows matched by position (JP line N <-> EN line N). Virtualized (`@tanstack/svelte-virtual`). Three-state: amber (untranslated), yellow (translated), green (confirmed). Double-click row number to confirm/unconfirm. Includes have collapse/expand. Emit/blank rows hidden. Notes indicator: `+` to add, `[N]` when present.

**Toolbar menus:** Project (Save/Export/Settings/Close), Line (Next Untranslated/Unconfirmed, Confirm/Unconfirm), Tools (Dictionary), Filter input (CTRL+F) right-aligned).

**Dictionary panel:** Vibrato tokenizes JP text, looks up tokens. Inflection detection with base form navigation. Kanji detail via KANJIDIC2. Manual search box.

**Find & Replace (Ctrl+H):** Searches EN text in filtered entries. Real-time match count, navigate Enter/Shift+Enter, replace current or all.

**Context menu:** Custom (replaces WebView default). Cut/Copy/Paste/Select All on editable elements.

**Status bar:** Left: modified/saved. Right: confirmed/translated/total with dual progress bar.

## Tauri IPC Commands

```
# Project management
create_project(name, files: {jp, en}) -> ProjectWithEntries
open_project(id) -> ProjectWithEntries
save_project() / save_en_file(entries)
confirm_line(index) / unconfirm_line(index)
rename_project(name) / delete_project(id)
list_recent_projects() / list_all_projects() -> Vec<RecentProject>
remove_recent_project(id)
export_project(dest_path) / update_project_settings(settings)

# Import
preview_import(source_path) -> ImportPreview { name, confirmedCount }
import_project(source_path, name, files: {jp, en}) -> ProjectWithEntries

# Dictionary
lookup_word(query) -> LookupResult { entries, inflections }
lookup_kanji(ch) -> Option<KanjiEntry>
```

**ProjectWithEntries:** Project fields (name, files, confirmedLines, settings) via `#[serde(flatten)]` + `entries: Vec<FlatEntry>`.

**FlatEntry** (`#[serde(rename_all = "camelCase")]`): `index`, `entry_type` (Text/Comment/Include/Emit/Blank), `jp_text`, `en_text`, `source_file`, `depth` (0 = top level), `notes: Vec<String>` (without `;` prefix).

## Build & development

```bash
# install frontend deps
pnpm install

# build dictionary databases (one-time)
cd tools && cargo run -p build-dictionary -- \
  ../resources/JMdict_e.xml ../resources/kanjidic2.xml ../resources/gen/

# dev mode with hot reload
pnpm tauri dev

# run Rust tests
cd src-tauri && cargo test

# production build
pnpm tauri build
```

- Dictionaries + IPADIC bundled as Tauri resources. DevTools only in debug builds.
- Single-instance via `tauri-plugin-single-instance`. Linux needs WebKitGTK.

## Future

- Font bundling (Noto Sans JP) for consistent CJK rendering.
- Wikidata lookup for proper nouns (optional, requires network).

## Coding Conventions

- **Rust:** `anyhow` for app errors, `thiserror` for module error types. `rustfmt` + `clippy`.
- **TypeScript:** Strict mode. Svelte 5 runes (`$state`, `$derived`, `$props`). No legacy stores.
- **IPC boundary:** JSON via `serde` with `#[serde(rename_all = "camelCase")]`. Keep types flat.
- **Package management:** pnpm via corepack.
- **Testing:** Unit tests in `src-tauri/src/` modules.
- **Svelte a11y:** Warnings suppressed globally (desktop app).
- **@tanstack/svelte-virtual:** Svelte 5 requires `$derived(createVirtualizer(...))` with scroll element read at top level (not inside closure).
