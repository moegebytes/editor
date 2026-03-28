use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use log::{error, warn};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum WiktError {
  #[error("Could not reach Wiktionary: {0}")]
  Network(String),
  #[error("No Wiktionary entry found for '{term}'.")]
  NotFound { term: String },
  #[error("Cache error: {0}")]
  Cache(#[from] rusqlite::Error),
  #[error("Failed to parse response: {0}")]
  Parse(String),
  #[error("Failed to create cache directory: {0}")]
  Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktExample {
  #[serde(default)]
  pub example: String,
  #[serde(default)]
  pub transliteration: Option<String>,
  #[serde(default)]
  pub translation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktDefinition {
  pub definition: String,
  #[serde(default)]
  pub parsed_examples: Vec<WiktExample>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktEntry {
  pub part_of_speech: String,
  pub language: String,
  pub definitions: Vec<WiktDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktLanguageSection {
  pub code: String,
  pub language: String,
  pub entries: Vec<WiktEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WiktResult {
  pub term: String,
  pub sections: Vec<WiktLanguageSection>,
  pub cached: bool,
}

pub struct WiktCache {
  db: Connection,
  client: reqwest::blocking::Client,
}

impl WiktCache {
  pub fn open(cache_dir: &Path) -> Result<Self, WiktError> {
    std::fs::create_dir_all(cache_dir)?;
    let db_path = cache_dir.join("wiktionary.sqlite");
    let db = Connection::open(&db_path)?;
    db.execute_batch(
      "CREATE TABLE IF NOT EXISTS cache (
        term       TEXT PRIMARY KEY,
        response   TEXT NOT NULL,
        fetched_at INTEGER NOT NULL
      );"
    )?;

    let client = reqwest::blocking::Client::builder()
      .user_agent("Yona/1.0 (Visual Novel Translation Editor)")
      .build()
      .map_err(|e| WiktError::Network(e.to_string()))?;

    Ok(Self { db, client })
  }

  pub fn lookup(&self, term: &str) -> Result<WiktResult, WiktError> {
    match self.get_cached(term) {
      Ok(Some(sections)) => {
        return Ok(WiktResult { term: term.to_string(), sections, cached: true });
      }
      Ok(None) => {}
      Err(e) => {
        warn!("Wiktionary cache read failed for '{}': {}", term, e);
      }
    }

    let sections = self.fetch_from_api(term)?;

    if let Err(e) = self.store_cache(term, &sections) {
      warn!("Wiktionary cache write failed for '{}': {}", term, e);
    }

    Ok(WiktResult { term: term.to_string(), sections, cached: false })
  }

  fn fetch_from_api(&self, term: &str) -> Result<Vec<WiktLanguageSection>, WiktError> {
    let encoded = percent_encode(term.as_bytes(), NON_ALPHANUMERIC);
    let url = format!("https://en.wiktionary.org/api/rest_v1/page/definition/{}", encoded);

    let response = self.client.get(&url).send().map_err(|e| {
      error!("Wiktionary request failed for '{}': {}", term, e);
      WiktError::Network(e.to_string())
    })?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
      return Err(WiktError::NotFound { term: term.to_string() });
    }

    if !response.status().is_success() {
      return Err(WiktError::Network(format!("HTTP {}", response.status())));
    }

    let body = response.text().map_err(|e| WiktError::Parse(e.to_string()))?;
    parse_definition_response(&body)
  }

  fn get_cached(&self, term: &str) -> Result<Option<Vec<WiktLanguageSection>>, WiktError> {
    let mut stmt = self.db.prepare("SELECT response FROM cache WHERE term = ?1")?;
    let mut rows = stmt.query(rusqlite::params![term])?;
    match rows.next()? {
      Some(row) => {
        let json: String = row.get(0)?;
        let sections: Vec<WiktLanguageSection> = serde_json::from_str(&json)
          .map_err(|e| WiktError::Parse(e.to_string()))?;
        Ok(Some(sections))
      }
      None => Ok(None),
    }
  }

  fn store_cache(&self, term: &str, sections: &[WiktLanguageSection]) -> Result<(), WiktError> {
    let json = serde_json::to_string(sections).map_err(|e| WiktError::Parse(e.to_string()))?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    self.db.execute(
      "INSERT OR REPLACE INTO cache (term, response, fetched_at) VALUES (?1, ?2, ?3)",
      rusqlite::params![term, json, now],
    )?;
    Ok(())
  }

  pub fn clear_cache(&self) -> Result<(), WiktError> {
    self.db.execute("DELETE FROM cache", [])?;
    Ok(())
  }
}

fn parse_definition_response(body: &str) -> Result<Vec<WiktLanguageSection>, WiktError> {
  use std::collections::HashMap;

  let raw: HashMap<String, Vec<WiktEntry>> = serde_json::from_str(body).map_err(|e| WiktError::Parse(e.to_string()))?;
  let mut sections = Vec::new();

  for (code, entries) in raw {
    if entries.is_empty() {
      continue;
    }
    let language = entries[0].language.clone();
    sections.push(WiktLanguageSection { code, language, entries });
  }

  // Sort: Japanese first, then alphabetical
  sections.sort_by(|a, b| {
    let a_jp = a.code == "ja";
    let b_jp = b.code == "ja";
    b_jp.cmp(&a_jp).then_with(|| a.language.cmp(&b.language))
  });

  Ok(sections)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_definition_basic() {
    let json = r#"{
      "ja": [{
        "partOfSpeech": "Verb",
        "language": "Japanese",
        "definitions": [{
          "definition": "to eat",
          "parsedExamples": [{
            "example": "ご飯を食べる",
            "transliteration": "gohan o taberu",
            "translation": "to eat a meal"
          }]
        }]
      }]
    }"#;

    let sections = parse_definition_response(json).unwrap();
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].code, "ja");
    assert_eq!(sections[0].language, "Japanese");
    assert_eq!(sections[0].entries.len(), 1);
    assert_eq!(sections[0].entries[0].part_of_speech, "Verb");
    assert_eq!(sections[0].entries[0].definitions[0].definition, "to eat");
    assert_eq!(
      sections[0].entries[0].definitions[0].parsed_examples[0].translation,
      Some("to eat a meal".to_string())
    );
  }

  #[test]
  fn parse_definition_multi_language() {
    let json = r#"{
      "en": [{"partOfSpeech": "Noun", "language": "English", "definitions": [{"definition": "water"}]}],
      "ja": [{"partOfSpeech": "Noun", "language": "Japanese", "definitions": [{"definition": "water"}]}],
      "fr": [{"partOfSpeech": "Noun", "language": "French", "definitions": [{"definition": "eau"}]}]
    }"#;

    let sections = parse_definition_response(json).unwrap();
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].language, "Japanese");
    assert_eq!(sections[1].language, "English");
    assert_eq!(sections[2].language, "French");
  }
}
