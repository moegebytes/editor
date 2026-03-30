use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum WiktError {
  #[error("Database error: {0}")]
  Db(#[from] rusqlite::Error),
  #[error("Wiktionary database not found at '{path}'.")]
  NotFound { path: String },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktExample {
  pub text: String,
  pub english: Option<String>,
  pub romaji: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktRelation {
  pub kind: String,
  pub term: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktSense {
  pub gloss: String,
  pub tags: Vec<String>,
  pub examples: Vec<WiktExample>,
  pub relations: Vec<WiktRelation>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktWordEntry {
  pub id: i64,
  pub word: String,
  pub pos: String,
  pub lang_code: Option<String>,
  pub sort_group: Option<i64>,
  pub reading: Option<String>,
  pub romaji: Option<String>,
  pub ipa: Option<String>,
  pub senses: Vec<WiktSense>,
  pub relations: Vec<WiktRelation>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktResult {
  pub term: String,
  pub entries: Vec<WiktWordEntry>,
}

pub struct WiktDb {
  db: Connection,
}

impl WiktDb {
  pub fn open(db_path: &Path) -> Result<Self, WiktError> {
    if !db_path.exists() {
      return Err(WiktError::NotFound {
        path: db_path.to_string_lossy().into_owned(),
      });
    }
    let db = Connection::open_with_flags(
      db_path,
      rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(Self { db })
  }

  pub fn lookup(&self, term: &str) -> Result<WiktResult, WiktError> {
    let mut stmt = self.db.prepare_cached(
      "SELECT id, word, pos, lang_code, sort_group, reading, romaji, ipa \
       FROM words WHERE word = ?1 ORDER BY sort_group, pos",
    )?;
    let rows = stmt.query_map([term], |row| self.row_to_word(row))?;
    let mut entries = Vec::new();
    for row in rows {
      let mut entry = row?;
      entry.senses = self.load_senses(entry.id)?;
      entry.relations = self.load_relations(entry.id, None)?;
      entries.push(entry);
    }
    Ok(WiktResult { term: term.to_string(), entries })
  }

  fn row_to_word(&self, row: &rusqlite::Row) -> rusqlite::Result<WiktWordEntry> {
    Ok(WiktWordEntry {
      id: row.get(0)?,
      word: row.get(1)?,
      pos: row.get(2)?,
      lang_code: row.get(3)?,
      sort_group: row.get(4)?,
      reading: row.get(5)?,
      romaji: row.get(6)?,
      ipa: row.get(7)?,
      senses: Vec::new(),
      relations: Vec::new(),
    })
  }

  fn load_senses(&self, word_id: i64) -> Result<Vec<WiktSense>, WiktError> {
    let mut stmt = self.db.prepare_cached(
      "SELECT id, gloss, tags FROM senses WHERE word_id = ?1 ORDER BY sort_order",
    )?;
    let rows = stmt.query_map([word_id], |row| {
      let sense_id: i64 = row.get(0)?;
      let gloss: String = row.get(1)?;
      let tags_json: Option<String> = row.get(2)?;
      Ok((sense_id, gloss, tags_json))
    })?;

    let mut senses = Vec::new();
    for row in rows {
      let (sense_id, gloss, tags_json) = row?;
      let tags: Vec<String> = tags_json.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default();
      let examples = self.load_examples(sense_id)?;
      let relations = self.load_relations(word_id, Some(sense_id))?;
      senses.push(WiktSense { gloss, tags, examples, relations });
    }
    Ok(senses)
  }

  fn load_examples(&self, sense_id: i64) -> Result<Vec<WiktExample>, WiktError> {
    let mut stmt = self.db.prepare_cached("SELECT text, english, romaji FROM examples WHERE sense_id = ?1")?;
    let rows = stmt.query_map([sense_id], |row| {
      Ok(WiktExample {
        text: row.get(0)?,
        english: row.get(1)?,
        romaji: row.get(2)?,
      })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(WiktError::from)
  }

  fn load_relations(&self, word_id: i64, sense_id: Option<i64>) -> Result<Vec<WiktRelation>, WiktError> {
    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<WiktRelation> {
      Ok(WiktRelation { kind: row.get(0)?, term: row.get(1)? })
    };
    match sense_id {
      Some(sid) => {
        let mut stmt = self.db.prepare_cached(
          "SELECT kind, term FROM relations WHERE word_id = ?1 AND sense_id = ?2",
        )?;
        let results = stmt.query_map(rusqlite::params![word_id, sid], map_row)?.collect::<Result<Vec<_>, _>>()?;
        Ok(results)
      }
      None => {
        let mut stmt = self.db.prepare_cached(
          "SELECT kind, term FROM relations WHERE word_id = ?1 AND sense_id IS NULL",
        )?;
        let results = stmt.query_map([word_id], map_row)?.collect::<Result<Vec<_>, _>>()?;
        Ok(results)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn test_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("resources/gen/wiktionary.sqlite")
  }

  #[test]
  fn lookup_exact_match() {
    let db_path = test_db_path();
    if !db_path.exists() {
      eprintln!("Skipping test: wiktionary.sqlite not found");
      return;
    }
    let db = WiktDb::open(&db_path).unwrap();
    let result = db.lookup("食べる").unwrap();
    assert!(!result.entries.is_empty());
    assert_eq!(result.entries[0].word, "食べる");
    assert_eq!(result.entries[0].pos, "verb");
    assert!(!result.entries[0].senses.is_empty());
    assert_eq!(result.entries[0].senses[0].gloss, "to eat");
  }

  #[test]
  fn lookup_with_relations() {
    let db_path = test_db_path();
    if !db_path.exists() {
      eprintln!("Skipping test: wiktionary.sqlite not found");
      return;
    }
    let db = WiktDb::open(&db_path).unwrap();
    let result = db.lookup("月").unwrap();
    assert!(result.entries.len() >= 9);

    let etym4 = result
      .entries
      .iter()
      .find(|e| e.sort_group == Some(4) && e.pos == "noun")
      .expect("sort_group 4 noun not found");
    let sense_rels = &etym4.senses[0].relations;
    assert!(sense_rels.iter().any(|r| r.kind == "coordinate_term" && r.term == "影"), "expected coordinate_term 影");
    assert!(sense_rels.iter().any(|r| r.kind == "synonym" && r.term == "パンパン"), "expected synonym パンパン");
  }

  #[test]
  fn lookup_not_found() {
    let db_path = test_db_path();
    if !db_path.exists() {
      eprintln!("Skipping test: wiktionary.sqlite not found");
      return;
    }
    let db = WiktDb::open(&db_path).unwrap();
    let result = db.lookup("zzzznonexistent").unwrap();
    assert!(result.entries.is_empty());
  }
}
