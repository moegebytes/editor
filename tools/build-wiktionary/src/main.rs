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
struct RawHeadTemplate {
  expansion: Option<String>,
}

#[derive(Deserialize)]
struct RawRelation {
  word: Option<String>,
  source: Option<String>,
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
  raw_glosses: Option<Vec<String>>,
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
  etymology_number: Option<i64>,
  etymology_text: Option<String>,
  forms: Option<Vec<RawForm>>,
  sounds: Option<Vec<RawSound>>,
  head_templates: Option<Vec<RawHeadTemplate>>,
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
       etymology_number INTEGER,
       etymology_text   TEXT,
       reading          TEXT,
       romaji           TEXT,
       display          TEXT,
       ipa              TEXT
     );
     CREATE TABLE senses (
       id         INTEGER PRIMARY KEY,
       word_id    INTEGER NOT NULL REFERENCES words(id),
       sort_order INTEGER NOT NULL,
       gloss      TEXT NOT NULL,
       raw_gloss  TEXT,
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
       term     TEXT NOT NULL,
       thesaurus INTEGER NOT NULL DEFAULT 0
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

    word_id += 1;

    let reading = extract_reading(&entry);
    let romaji = extract_romaji(&entry);
    let display = entry
      .head_templates
      .as_ref()
      .and_then(|t| t.first())
      .and_then(|t| t.expansion.as_deref());
    let ipa = entry
      .sounds
      .as_ref()
      .and_then(|s| s.iter().find_map(|s| s.ipa.as_deref()));

    let etymology_text = entry.etymology_text.as_deref().map(clean_etymology);

    tx.execute(
      "INSERT INTO words (id, word, pos, etymology_number, etymology_text, \
       reading, romaji, display, ipa) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
      rusqlite::params![
        word_id,
        entry.word,
        entry.pos,
        entry.etymology_number,
        etymology_text,
        reading,
        romaji,
        display,
        ipa,
      ],
    )?;

    insert_entry_relations(&tx, word_id, None, &entry)?;

    if let Some(senses) = &entry.senses {
      for (i, sense) in senses.iter().enumerate() {
        let gloss = match sense.glosses.as_ref().and_then(|g| g.first()) {
          Some(g) => g,
          None => continue,
        };

        sense_id += 1;

        let raw_gloss = sense.raw_glosses.as_ref().and_then(|g| g.first());
        let tags = sense.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        tx.execute(
          "INSERT INTO senses (id, word_id, sort_order, gloss, raw_gloss, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
          rusqlite::params![sense_id, word_id, i as i64, gloss, raw_gloss, tags],
        )?;

        if let Some(examples) = &sense.examples {
          for ex in examples {
            let text = match ex.text.as_deref() {
              Some(t) if !t.is_empty() => t,
              _ => continue,
            };
            let english = ex
              .english
              .as_deref()
              .or(ex.translation.as_deref());
            tx.execute(
              "INSERT INTO examples (sense_id, text, english, romaji) VALUES (?1, ?2, ?3, ?4)",
              rusqlite::params![sense_id, text, english, ex.roman],
            )?;
          }
        }

        insert_sense_relations(&tx, word_id, sense_id, sense)?;
      }
    }

    entry_count += 1;
    if entry_count % 10000 == 0 {
      eprintln!("  {} entries...", entry_count);
    }
  }

  eprintln!("  Pruning dangling relations...");
  let pruned = tx.execute(
    "DELETE FROM relations WHERE term NOT IN (SELECT DISTINCT word FROM words) AND thesaurus = 0",
    [],
  )?;
  eprintln!("  {} dangling relations removed", pruned);

  tx.commit()?;
  conn.execute_batch("PRAGMA journal_mode=DELETE;")?;

  eprintln!("  {} entries inserted, {} skipped", entry_count, skip_count);
  Ok(())
}

fn clean_etymology(text: &str) -> String {
  text
    .lines()
    .filter(|line| !line.contains("see WT:"))
    .collect::<Vec<_>>()
    .join("\n")
    .trim()
    .to_string()
}

fn extract_reading(entry: &RawEntry) -> Option<String> {
  let forms = entry.forms.as_ref()?;
  let canonical = forms.iter().find(|f| {
    f.tags.as_ref().map_or(false, |t| t.iter().any(|tag| tag == "canonical"))
  })?;
  let ruby = canonical.ruby.as_ref()?;
  if ruby.is_empty() {
    return None;
  }
  let reading: String = ruby.iter().map(|pair| pair.1.as_str()).collect();
  if reading.is_empty() { None } else { Some(reading) }
}

fn extract_romaji(entry: &RawEntry) -> Option<String> {
  let forms = entry.forms.as_ref()?;
  let rom = forms.iter().find(|f| {
    f.tags.as_ref().map_or(false, |t| t.iter().any(|tag| tag == "romanization"))
  })?;
  rom.form.clone()
}

fn insert_relations(
  tx: &rusqlite::Transaction,
  word_id: i64,
  sense_id: Option<i64>,
  pairs: Vec<(&str, &Option<Vec<RawRelation>>)>,
) -> Result<()> {
  use std::collections::HashSet;

  for (kind, relations) in pairs {
    if let Some(rels) = relations {
      let mut thesaurus_seen = HashSet::new();
      for rel in rels {
        let source = rel.source.as_deref().unwrap_or("");
        if let Some(thesaurus_term) = source.strip_prefix("Thesaurus:") {
          if thesaurus_seen.insert(thesaurus_term.to_string()) {
            tx.execute(
              "INSERT INTO relations (word_id, sense_id, kind, term, thesaurus) VALUES (?1, ?2, ?3, ?4, 1)",
              rusqlite::params![word_id, sense_id, kind, thesaurus_term],
            )?;
          }
        } else if let Some(term) = rel.word.as_deref() {
          if !term.is_empty() {
            tx.execute(
              "INSERT INTO relations (word_id, sense_id, kind, term, thesaurus) VALUES (?1, ?2, ?3, ?4, 0)",
              rusqlite::params![word_id, sense_id, kind, term],
            )?;
          }
        }
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
) -> Result<()> {
  insert_relations(tx, word_id, sense_id, vec![
    ("synonym", &entry.synonyms),
    ("antonym", &entry.antonyms),
    ("coordinate_term", &entry.coordinate_terms),
    ("related", &entry.related),
    ("derived", &entry.derived),
    ("hyponym", &entry.hyponyms),
  ])
}

fn insert_sense_relations(
  tx: &rusqlite::Transaction,
  word_id: i64,
  sense_id: i64,
  sense: &RawSense,
) -> Result<()> {
  insert_relations(tx, word_id, Some(sense_id), vec![
    ("synonym", &sense.synonyms),
    ("antonym", &sense.antonyms),
    ("coordinate_term", &sense.coordinate_terms),
    ("related", &sense.related),
    ("derived", &sense.derived),
    ("hypernym", &sense.hypernyms),
    ("hyponym", &sense.hyponyms),
  ])
}
