# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

Desktop app for translating Visual Novel script files. Reads paired JP/EN script files in a custom strings text format,
displays side-by-side, supports inline editing of English translations with offline Japanese dictionary lookup and
offline Wiktionary lookup.

## Tech Stack

- **Rust + Tauri 2** backend, **Svelte 5 + TypeScript** frontend.
- **vibrato** (pure Rust, IPADIC MeCab 2.7.0) for Japanese morphological analysis.
- **JMdict** via SQLite (FTS5) for offline dictionary lookup.
- **KANJIDIC2** via SQLite for offline kanji lookup.
- **Wiktionary** via SQLite (FTS5) for offline lookup.

## Project Structure

```
editor/
├── CLAUDE.md
├── package.json
├── index.html
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── public/
│   └── fonts/                   -- Bundled fonts
│       └── NotoSansJP-Variable.woff2
├── src/                         -- Svelte 5 + TypeScript frontend
│   ├── App.svelte
│   ├── main.ts
│   ├── app.css
│   ├── lib/
│   │   ├── types.ts             -- Types for IPC
│   │   ├── ipc.ts               -- Tauri invoke wrappers
│   │   ├── utils.ts             -- Shared utilities (entry predicates, set helpers, kanji detection)
│   │   └── toast.svelte.ts      -- Toast notification store (Svelte 5 runes module)
│   └── components/
│       ├── ui/
│       │   ├── ContextMenu.svelte
│       │   ├── Dialog.svelte
│       │   ├── DropdownMenu.svelte
│       │   ├── LoadingOverlay.svelte
│       │   └── ToastContainer.svelte
│       ├── AboutDialog.svelte
│       ├── JmdictTab.svelte
│       ├── DictionaryPanel.svelte
│       ├── KanjiDetail.svelte
│       ├── EditorTable.svelte
│       ├── FindReplaceBar.svelte
│       ├── GoToLineDialog.svelte
│       ├── ProjectHome.svelte
│       ├── SettingsView.svelte
│       ├── StatusBar.svelte
│       ├── Toolbar.svelte
│       ├── WiktTab.svelte
│       └── UnsavedChangesDialog.svelte
├── src-tauri/                   -- Tauri 2 backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── src/
│       ├── main.rs              -- Tauri app setup, dictionary loading
│       ├── commands.rs          -- IPC command handlers
│       ├── logging.rs           -- Per-session file logger setup
│       ├── project.rs           -- Project config persistence
│       ├── strings.rs           -- Strings format parser/writer
│       ├── core.rs              -- File pairing, FlatEntry type, notes, reconstruction
│       ├── jmdict.rs            -- JMdict SQLite queries + Vibrato tokenizer
│       ├── kanjidic.rs          -- KANJIDIC2 SQLite queries (kanji lookup)
│       ├── util.rs              -- Shared utilities (friendly IO error messages)
│       └── wiktionary.rs        -- Wiktionary offline SQLite queries
├── tools/
│   ├── Cargo.toml
│   ├── build-jmdict/            -- Offline tool: JMdict XML -> SQLite
│   ├── build-kanjidic/          -- Offline tool: KANJIDIC2 XML -> SQLite
│   └── build-wiktionary/        -- Offline tool: wiktextract JSONL -> SQLite
└── resources/
    ├── ipadic-mecab-v270.dict   -- Pre-compiled IPADIC for vibrato
    ├── src/                     -- Compressed source data (zstd)
    │   ├── JMdict_e.xml.zst
    │   ├── kanjidic2.xml.zst
    │   └── wiktionary-ja_en.jsonl.zst
    ├── gen/                     -- Generated files (gitignored)
    │   ├── jmdict.sqlite
    │   ├── kanjidic2.sqlite
    │   └── wiktionary.sqlite
```

**Architecture:** Single Tauri crate, modules for logic. `commands.rs` stays thin (deserialize -> call -> serialize). `tools/` is
a separate Cargo workspace.

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
- Comments preceding a text line are absorbed into its `notes` field. Comments before non-text entries stay as
structural rows.
- References rendered as clickable jump links in the editor.

## Layout

**Project workflow:** Starts at Project Home (recent + all projects, new project).

**Table:** Rows matched by position. Virtualized (`@tanstack/svelte-virtual`). Three-state: amber (untranslated),
yellow (translated), green (confirmed). Double-click row number to confirm/unconfirm. Includes have collapse/expand.
Emit/blank rows hidden. Notes indicator: `+` to add, `[N]` when present.

**Toolbar menus:** Project (Save/Export/Settings/Close), Line (Next Untranslated/Unconfirmed, Confirm/Unconfirm),
Tools (Dictionary/Go to Line/Find & Replace), Help (About). Filter input (CTRL+F) right-aligned.

**Dictionary panel:** Two tabs: **JMdict** and **Wiktionary**. Shared search input with search button.
Back/forward navigation (shared history, 100 entries max). Arrow buttons in header. JMdict tab: Vibrato tokenizes JP
text, looks up tokens, inflection detection with base form navigation. Wiktionary tab: Entries grouped by etymology
with senses, examples, synonyms, antonyms, coordinate terms, and other relations. External query changes auto-search
both tabs and reset to JMdict tab.

**Find & Replace (Ctrl+H):** Searches EN text in filtered entries. Real-time match count, navigate Enter/Shift+Enter,
replace current or all.

**Context menu:** Custom (replaces WebView default). Cut/Copy/Paste/Select All on editable elements.

**Status bar:** Left: modified/saved. Right: confirmed/translated/total with dual progress bar.

## Tauri IPC Commands

**ProjectWithEntries:** `id` + Project fields (name, files, confirmedLines, settings) via
`#[serde(flatten)]` + `entries: Vec<FlatEntry>`.

**FlatEntry** (`#[serde(rename_all = "camelCase")]`): `index`, `entry_type` (Text/Comment/Include/Emit/Blank),
`jp_text`, `en_text`, `source_file`, `depth` (0 = top level), `notes: Vec<String>` (without `;` prefix).

### Project management
```
create_project(name, files: {jp, en}) -> ProjectWithEntries
open_project(id) -> ProjectWithEntries
save_project() -> ()
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

### Dictionary
```
lookup_word(query) -> LookupResult { entries, inflections }
lookup_kanji(ch) -> Option<KanjiEntry>
```

### Wiktionary
```
lookup_wiktionary(term) -> WiktResult { term, entries: Vec<WiktWordEntry> }
```

### Environment
```
get_environment_info() -> EnvironmentInfo { appName, appVersion, tauriVersion, os, arch, debug }
```

## Build & development

```bash
# decompress resources (one-time, requires zstd)
zstd -d resources/src/JMdict_e.xml.zst
zstd -d resources/src/kanjidic2.xml.zst
zstd -d resources/src/wiktionary-ja_en.jsonl.zst

# build JMdict database (one-time)
cd tools && cargo run -p build-jmdict -- \
  ../resources/src/JMdict_e.xml ../resources/gen/

# build KANJIDIC2 database (one-time)
cd tools && cargo run -p build-kanjidic -- \
  ../resources/src/kanjidic2.xml ../resources/gen/

# build wiktionary database (one-time)
cd tools && cargo run -p build-wiktionary -- \
  ../resources/src/wiktionary-ja_en.jsonl ../resources/gen/
```

```bash
# install frontend deps
pnpm install

# run tests
pnpm svelte-check
cd src-tauri && cargo test

# dev mode with hot reload
pnpm tauri dev

# production build
pnpm tauri build
```

### Preparing fresh Wiktionary JSON

```bash
# clone repository
git clone https://github.com/tatuylonen/wiktextract.git && \
  git checkout f47b8fc87a0e17f4dcca68f534e73f4c6fa8e8e7

# apply patch for romaji (https://github.com/tatuylonen/wiktextract/issues/1620)
git apply <<EOF
diff --git a/src/wiktextract/extractor/en/example.py b/src/wiktextract/extractor/en/example.py
index a224f3c2..fce55506 100644
--- a/src/wiktextract/extractor/en/example.py
+++ b/src/wiktextract/extractor/en/example.py
@@ -167,7 +167,7 @@ def extract_template_ja_usex(
         )
         example_data["ruby"] = ruby_data
     for span_tag in expanded_node.find_html_recursively(
-        "span", attr_name="class", attr_value="tr"
+        "span", attr_name="class", attr_value="e-transliteration"
     ):
         example_data["roman"] = clean_node(wxr, None, span_tag)
         calculate_bold_offsets(
@@ -177,6 +177,7 @@ def extract_template_ja_usex(
             example_data,
             "bold_roman_offsets",
         )
+        break
     tr_arg = wxr.wtp.parse(
         wxr.wtp.node_to_wikitext(node.template_parameters.get(3, "")),
         expand_all=True,
EOF

# set up venv
python3 -m venv .venv
pip install -U pip && pip install -e .

# download xml dump
wget https://dumps.wikimedia.org/enwiktionary/latest/enwiktionary-latest-pages-articles.xml.bz2

# prepare database
wiktwords --db-path enwiktionary-latest.db --edition en \
  --skip-extraction enwiktionary-latest-pages-articles.xml.bz2

# extract entries
wiktwords --db-path enwiktionary-latest.db --edition en --language-code ja \
  --examples --etymologies --linkages --pronunciations --out wiktionary-pre-ja_en.jsonl

# clean-up remaining stale elements such as redirects or non-ja thesaurus
jq -c 'select(.lang_code == "ja")' wiktionary-pre-ja_en.jsonl > wiktionary-ja_en.jsonl
```

## Coding Conventions

- **Rust:** `anyhow` for app errors, `thiserror` for module error types. `rustfmt` + `clippy`.
- **TypeScript:** Strict mode. Svelte 5 runes (`$state`, `$derived`, `$props`). No legacy stores.
- **IPC boundary:** JSON via `serde` with `#[serde(rename_all = "camelCase")]`. Keep types flat.
- **Package management:** pnpm via corepack.
- **Testing:** Unit tests in `src-tauri/src/` modules.
- **Svelte a11y:** Warnings suppressed globally (desktop app).
- **@tanstack/svelte-virtual:** Svelte 5 requires `$derived(createVirtualizer(...))` with scroll element read at top
level (not inside closure).
