use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Deserialize;

const SKIP_POS: &[&str] = &["soft-redirect", "romanization", "syllable", "character"];

#[derive(Deserialize)]
struct RawForm {
  form: Option<String>,
  ruby: Option<Vec<(String, String)>>,
  tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawSound {
  ipa: Option<String>,
}

#[derive(Deserialize)]
struct RawRelation {
  word: Option<String>,
}

#[derive(Deserialize)]
struct RawExample {
  text: Option<String>,
  english: Option<String>,
  translation: Option<String>,
  roman: Option<String>,
}

#[derive(Deserialize)]
struct RawSense {
  glosses: Option<Vec<String>>,
  tags: Option<Vec<String>>,
  examples: Option<Vec<RawExample>>,
  synonyms: Option<Vec<RawRelation>>,
  antonyms: Option<Vec<RawRelation>>,
  coordinate_terms: Option<Vec<RawRelation>>,
  related: Option<Vec<RawRelation>>,
  derived: Option<Vec<RawRelation>>,
  hypernyms: Option<Vec<RawRelation>>,
  hyponyms: Option<Vec<RawRelation>>,
}

#[derive(Deserialize)]
struct RawEntry {
  word: String,
  pos: String,
  lang_code: Option<String>,
  etymology_number: Option<i64>,
  forms: Option<Vec<RawForm>>,
  sounds: Option<Vec<RawSound>>,
  senses: Option<Vec<RawSense>>,
  synonyms: Option<Vec<RawRelation>>,
  antonyms: Option<Vec<RawRelation>>,
  coordinate_terms: Option<Vec<RawRelation>>,
  related: Option<Vec<RawRelation>>,
  derived: Option<Vec<RawRelation>>,
  hyponyms: Option<Vec<RawRelation>>,
}

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("Usage: {} <wiktionary.jsonl> <output_dir>", args[0]);
    std::process::exit(1);
  }

  let jsonl_path = &args[1];
  let output_dir = PathBuf::from(&args[2]);
  std::fs::create_dir_all(&output_dir)?;

  let db_path = output_dir.join("wiktionary.sqlite");
  eprintln!("Building Wiktionary database...");
  build_wiktionary(jsonl_path, &db_path)?;

  eprintln!("Done.");
  Ok(())
}

fn build_wiktionary(jsonl_path: &str, db_path: &PathBuf) -> Result<()> {
  if db_path.exists() {
    std::fs::remove_file(db_path)?;
  }
  let mut conn = Connection::open(db_path)?;
  conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=OFF;")?;

  conn.execute_batch(
    "CREATE TABLE words (
       id               INTEGER PRIMARY KEY,
       word             TEXT NOT NULL,
       pos              TEXT NOT NULL,
       lang_code        TEXT,
       sort_group       INTEGER,
       reading          TEXT,
       romaji           TEXT,
       ipa              TEXT
     );
     CREATE TABLE senses (
       id         INTEGER PRIMARY KEY,
       word_id    INTEGER NOT NULL REFERENCES words(id),
       sort_order INTEGER NOT NULL,
       gloss      TEXT NOT NULL,
       tags       TEXT
     );
     CREATE TABLE examples (
       id       INTEGER PRIMARY KEY,
       sense_id INTEGER NOT NULL REFERENCES senses(id),
       text     TEXT NOT NULL,
       english  TEXT,
       romaji   TEXT
     );
     CREATE TABLE relations (
       id       INTEGER PRIMARY KEY,
       word_id  INTEGER NOT NULL REFERENCES words(id),
       sense_id INTEGER,
       kind     TEXT NOT NULL,
       term     TEXT NOT NULL
     );
     CREATE INDEX idx_words_word ON words(word);
     CREATE INDEX idx_senses_word_id ON senses(word_id);
     CREATE INDEX idx_examples_sense_id ON examples(sense_id);
     CREATE INDEX idx_relations_word_id ON relations(word_id);
     CREATE INDEX idx_relations_sense_id ON relations(sense_id);",
  )?;

  let tx = conn.transaction()?;

  let file = std::fs::File::open(jsonl_path).with_context(|| format!("failed to open '{}'", jsonl_path))?;
  let reader = std::io::BufReader::new(file);

  let mut word_id: i64 = 0;
  let mut sense_id: i64 = 0;
  let mut entry_count: u64 = 0;
  let mut skip_count: u64 = 0;

  for (line_num, line) in reader.lines().enumerate() {
    let line = line.with_context(|| format!("failed to read line {}", line_num + 1))?;
    if line.is_empty() {
      continue;
    }

    let entry: RawEntry = match serde_json::from_str(&line) {
      Ok(e) => e,
      Err(err) => {
        eprintln!("  Warning: skipping line {} (parse error: {})", line_num + 1, err);
        continue;
      }
    };

    if SKIP_POS.contains(&entry.pos.as_str()) {
      skip_count += 1;
      continue;
    }

    let is_en = entry.lang_code.as_deref() == Some("en");

    // Check if entry has at least one sense with a gloss
    let has_senses = entry
      .senses
      .as_ref()
      .is_some_and(|senses| senses.iter().any(|s| s.glosses.as_ref().is_some_and(|g| !g.is_empty())));
    if !has_senses {
      skip_count += 1;
      continue;
    }

    word_id += 1;

    let reading = extract_reading(&entry);
    let romaji = extract_romaji(&entry);
    let ipa = entry
      .sounds
      .as_ref()
      .and_then(|s| s.iter().find_map(|s| s.ipa.as_deref()));

    tx.execute(
      "INSERT INTO words (id, word, pos, lang_code, sort_group, reading, romaji, ipa) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
      rusqlite::params![
        word_id,
        entry.word,
        entry.pos,
        entry.lang_code,
        entry.etymology_number,
        reading,
        romaji,
        ipa,
      ],
    )?;

    insert_entry_relations(&tx, word_id, None, &entry, is_en)?;

    if let Some(senses) = &entry.senses {
      for (i, sense) in senses.iter().enumerate() {
        let gloss = match sense.glosses.as_ref().and_then(|g| g.first()) {
          Some(g) => g,
          None => continue,
        };

        sense_id += 1;

        let tags = sense.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        tx.execute(
          "INSERT INTO senses (id, word_id, sort_order, gloss, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
          rusqlite::params![sense_id, word_id, i as i64, gloss, tags],
        )?;

        if !is_en {
          if let Some(examples) = &sense.examples {
            for ex in examples {
              let text = match ex.text.as_deref() {
                Some(t) if !t.is_empty() => t,
                _ => continue,
              };
              let english = ex.english.as_deref().or(ex.translation.as_deref());
              tx.execute(
                "INSERT INTO examples (sense_id, text, english, romaji) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![sense_id, text, english, ex.roman],
              )?;
            }
          }
        }

        insert_sense_relations(&tx, word_id, sense_id, sense, is_en)?;
      }
    }

    entry_count += 1;
    if entry_count.is_multiple_of(10000) {
      eprintln!("  {} entries...", entry_count);
    }
  }

  eprintln!("  {} entries inserted, {} skipped", entry_count, skip_count);

  eprintln!("  Pruning dangling relations...");
  let pruned = tx.execute(
    "DELETE FROM relations WHERE term NOT IN (SELECT DISTINCT word FROM words)",
    [],
  )?;
  eprintln!("  {} dangling relations removed", pruned);

  tx.commit()?;
  conn.execute_batch("PRAGMA journal_mode=DELETE; VACUUM;")?;

  Ok(())
}

fn extract_reading(entry: &RawEntry) -> Option<String> {
  let forms = entry.forms.as_ref()?;
  let canonical = forms
    .iter()
    .find(|f| f.tags.as_ref().is_some_and(|t| t.iter().any(|tag| tag == "canonical")))?;
  let ruby = canonical.ruby.as_ref()?;
  if ruby.is_empty() {
    return None;
  }
  let reading: String = ruby.iter().map(|pair| pair.1.as_str()).collect();
  if reading.is_empty() {
    None
  } else {
    Some(reading)
  }
}

fn extract_romaji(entry: &RawEntry) -> Option<String> {
  let forms = entry.forms.as_ref()?;
  let rom = forms.iter().find(|f| {
    f.tags
      .as_ref()
      .is_some_and(|t| t.iter().any(|tag| tag == "romanization"))
  })?;
  rom.form.clone()
}

fn insert_relations(
  tx: &rusqlite::Transaction,
  word_id: i64,
  sense_id: Option<i64>,
  pairs: Vec<(&str, &Option<Vec<RawRelation>>)>,
) -> Result<()> {
  for (kind, relations) in pairs {
    if let Some(rels) = relations {
      for rel in rels {
        let Some(term) = rel.word.as_deref() else { continue };
        if term.is_empty() {
          continue;
        }
        tx.execute(
          "INSERT INTO relations (word_id, sense_id, kind, term) VALUES (?1, ?2, ?3, ?4)",
          rusqlite::params![word_id, sense_id, kind, term],
        )?;
      }
    }
  }
  Ok(())
}

fn insert_entry_relations(
  tx: &rusqlite::Transaction,
  word_id: i64,
  sense_id: Option<i64>,
  entry: &RawEntry,
  is_en: bool,
) -> Result<()> {
  let mut pairs: Vec<(&str, &Option<Vec<RawRelation>>)> = vec![
    ("synonym", &entry.synonyms),
    ("antonym", &entry.antonyms),
    ("coordinate_term", &entry.coordinate_terms),
    ("related", &entry.related),
    ("hyponym", &entry.hyponyms),
  ];
  if !is_en {
    pairs.push(("derived", &entry.derived));
  }
  insert_relations(tx, word_id, sense_id, pairs)
}

fn insert_sense_relations(
  tx: &rusqlite::Transaction,
  word_id: i64,
  sense_id: i64,
  sense: &RawSense,
  is_en: bool,
) -> Result<()> {
  let mut pairs: Vec<(&str, &Option<Vec<RawRelation>>)> = vec![
    ("synonym", &sense.synonyms),
    ("antonym", &sense.antonyms),
    ("coordinate_term", &sense.coordinate_terms),
    ("related", &sense.related),
    ("hypernym", &sense.hypernyms),
    ("hyponym", &sense.hyponyms),
  ];
  if !is_en {
    pairs.push(("derived", &sense.derived));
  }
  insert_relations(tx, word_id, Some(sense_id), pairs)
}
