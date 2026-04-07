use std::path::Path;

use log::trace;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KanjidicError {
  #[error("Database error: {0}.")]
  Db(#[from] rusqlite::Error),

  #[error("Database '{path}' not found.")]
  NotFound { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanjiEntry {
  pub literal: String,
  pub grade: Option<i32>,
  pub stroke_count: i32,
  pub jlpt: Option<i32>,
  pub freq: Option<i32>,
  pub on_readings: Vec<String>,
  pub kun_readings: Vec<String>,
  pub meanings: Vec<String>,
}

pub struct KanjiDb {
  conn: Connection,
}

impl KanjiDb {
  pub fn open(path: &Path) -> Result<Self, KanjidicError> {
    if !path.exists() {
      return Err(KanjidicError::NotFound {
        path: path.display().to_string(),
      });
    }

    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(KanjiDb { conn })
  }

  pub fn lookup(&self, ch: char) -> Result<Option<KanjiEntry>, KanjidicError> {
    trace!("KANJIDIC2 lookup for '{}'", ch);

    let literal = ch.to_string();
    let mut stmt = self
      .conn
      .prepare("SELECT literal, grade, stroke_count, jlpt, freq FROM kanji WHERE literal = ?1")?;

    let entry = stmt
      .query_row([&literal], |row| {
        Ok(KanjiEntry {
          literal: row.get(0)?,
          grade: row.get(1)?,
          stroke_count: row.get(2)?,
          jlpt: row.get(3)?,
          freq: row.get(4)?,
          on_readings: vec![],
          kun_readings: vec![],
          meanings: vec![],
        })
      })
      .ok();

    let Some(mut entry) = entry else {
      return Ok(None);
    };

    let mut read_stmt = self
      .conn
      .prepare("SELECT reading, r_type FROM readings WHERE literal = ?1")?;
    let readings: Vec<(String, String)> = read_stmt
      .query_map([&literal], |row| Ok((row.get(0)?, row.get(1)?)))?
      .filter_map(|r| r.ok())
      .collect();

    for (reading, r_type) in readings {
      match r_type.as_str() {
        "ja_on" => entry.on_readings.push(reading),
        "ja_kun" => entry.kun_readings.push(reading),
        _ => {}
      }
    }

    let mut meaning_stmt = self
      .conn
      .prepare("SELECT meaning FROM meanings WHERE literal = ?1 AND lang IS NULL")?;
    entry.meanings = meaning_stmt
      .query_map([&literal], |row| row.get(0))?
      .filter_map(|r| r.ok())
      .collect();

    Ok(Some(entry))
  }
}
