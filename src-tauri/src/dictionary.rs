use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use vibrato::{Dictionary, Tokenizer};

#[derive(Debug, Error)]
pub enum DictError {
  #[error("database error: {0}")]
  Db(#[from] rusqlite::Error),

  #[error("database not found: {path}")]
  NotFound { path: String },

  #[error("tokenizer error: {0}")]
  Tokenizer(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictEntry {
  pub ent_seq: i64,
  pub kanji: Vec<String>,
  pub readings: Vec<String>,
  pub senses: Vec<Sense>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sense {
  pub pos: Vec<String>,
  pub glosses: Vec<String>,
  pub misc: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inflection {
  pub surface: String,
  pub base_form: String,
  pub form_name: String,
  pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupResult {
  pub entries: Vec<DictEntry>,
  pub inflections: Vec<Inflection>,
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

fn identify_inflection(
  conj_form: &str,
  aux_chain: &str,
  is_adjective: bool,
) -> Option<(String, String)> {
  let kind = if is_adjective { "adjective" } else { "verb" };

  let (form, desc) = match aux_chain {
    // Polite forms (check before simpler patterns)
    "ませんでした" =>
      ("Masen deshita-form", format!("Polite negative past tense of the {}.", kind)),
    "ません" =>
      ("Masen-form", format!("Polite negative form of the {}.", kind)),
    "ました" =>
      ("Mashita-form", format!("Polite past tense of the {}.", kind)),
    "ます" =>
      ("Masu-form", format!("Polite form of the {}.", kind)),

    // Negative forms
    "なかった" =>
      ("Nakatta-form", format!("Negative past tense of the {}.", kind)),
    "ない" | "ぬ" | "ん" =>
      ("Nai-form", format!("Negative form of the {}.", kind)),

    // Past tense
    "た" | "だ" =>
      ("Ta-form", format!("Past tense of the {}.", kind)),

    // Te-form variations
    "ている" | "てる" | "でいる" | "でる" =>
      ("Te-iru form", format!("Progressive or resultative form of the {}.", kind)),
    "ていた" | "てた" | "でいた" | "でた" =>
      ("Te-ita form", format!("Past progressive form of the {}.", kind)),
    "て" | "で" =>
      ("Te-form", format!("Conjunctive form of the {}.", kind)),

    // Desire
    "たかった" =>
      ("Takatta-form", format!("Past tense of the desire form of the {}.", kind)),
    "たくない" =>
      ("Takunai-form", format!("Negative desire form of the {}.", kind)),
    "たい" =>
      ("Tai-form", format!("Expresses desire to perform the action of the {}.", kind)),

    // Conditional
    "たら" | "だら" =>
      ("Tara-form", format!("Conditional form of the {}.", kind)),
    "ば" =>
      ("Ba-form", format!("Conditional form of the {}.", kind)),

    // Voice
    "れる" | "られる" =>
      ("Passive/Potential", format!("Passive or potential form of the {}.", kind)),
    "せる" | "させる" =>
      ("Causative", format!("Causative form of the {}.", kind)),

    // Volitional
    "う" | "よう" =>
      ("Volitional", "Expresses intention or suggestion.".to_string()),

    // No auxiliary — check conjugation form directly
    "" => {
      if conj_form.starts_with("命令") {
        ("Imperative", format!("Command form of the {}.", kind))
      } else {
        return None;
      }
    }

    // Unknown auxiliary chain — generic fallback
    _ => ("Inflected form", format!("An inflected form of the {}.", kind)),
  };

  Some((form.to_string(), desc))
}

pub struct DictDb {
  jmdict: Connection,
  kanjidic: Connection,
  tokenizer: Tokenizer,
}

impl DictDb {
  pub fn open(
    jmdict_path: &Path,
    kanjidic_path: &Path,
    ipadic_path: &Path,
  ) -> Result<Self, DictError> {
    for path in [jmdict_path, kanjidic_path, ipadic_path] {
      if !path.exists() {
        return Err(DictError::NotFound {
          path: path.display().to_string(),
        });
      }
    }

    let jmdict = Connection::open_with_flags(jmdict_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let kanjidic = Connection::open_with_flags(kanjidic_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let dict_file = std::fs::File::open(ipadic_path)
      .map_err(|e| DictError::Tokenizer(format!("failed to open IPADIC: {}", e)))?;
    let reader = std::io::BufReader::new(dict_file);
    let dict = Dictionary::read(reader)
      .map_err(|e| DictError::Tokenizer(format!("dictionary load error: {}", e)))?;
    let tokenizer = Tokenizer::new(dict);

    Ok(DictDb {
      jmdict,
      kanjidic,
      tokenizer,
    })
  }

  pub fn lookup_word(&self, query: &str) -> Result<LookupResult, DictError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
      return Ok(LookupResult { entries: vec![], inflections: vec![] });
    }

    // Detect inflections (always runs vibrato on the query)
    let inflections = self.detect_inflections(trimmed);

    // First try exact match on the full query
    let exact = self.lookup_exact(trimmed)?;
    if !exact.is_empty() {
      return Ok(LookupResult { entries: exact, inflections });
    }

    // Tokenize with vibrato and look up each meaningful token
    let mut worker = self.tokenizer.new_worker();
    worker.reset_sentence(trimmed);
    worker.tokenize();

    let mut entries = Vec::new();
    let mut seen_seqs = HashSet::new();

    for i in 0..worker.num_tokens() {
      let token = worker.token(i);
      let surface = token.surface();
      let feature = token.feature();
      let fields: Vec<&str> = feature.split(',').collect();

      // Skip particles, auxiliary verbs, symbols
      if !fields.is_empty() {
        let pos = fields[0];
        if pos == "助詞" || pos == "助動詞" || pos == "記号" {
          continue;
        }
      }

      // Look up the base/dictionary form if available (field index 6 in IPADIC)
      let lookup_form = if fields.len() > 6 && fields[6] != "*" {
        fields[6]
      } else {
        surface
      };

      let token_entries = self.lookup_exact(lookup_form)?;
      for entry in token_entries {
        if seen_seqs.insert(entry.ent_seq) {
          entries.push(entry);
        }
      }

      // Also try the surface form if different from dictionary form
      if lookup_form != surface {
        let surface_entries = self.lookup_exact(surface)?;
        for entry in surface_entries {
          if seen_seqs.insert(entry.ent_seq) {
            entries.push(entry);
          }
        }
      }
    }

    // If tokenization yielded nothing, fall back to FTS5
    if entries.is_empty() {
      entries = self.lookup_fts(trimmed)?;
    }

    Ok(LookupResult { entries, inflections })
  }

  fn detect_inflections(&self, query: &str) -> Vec<Inflection> {
    let mut worker = self.tokenizer.new_worker();
    worker.reset_sentence(query);
    worker.tokenize();

    if worker.num_tokens() == 0 {
      return vec![];
    }

    // Collect token info
    struct TokenInfo {
      surface: String,
      pos: String,
      pos_sub1: String,
      conj_form: String,
      base_form: String,
    }

    let mut tokens = Vec::new();
    for i in 0..worker.num_tokens() {
      let token = worker.token(i);
      let fields: Vec<&str> = token.feature().split(',').collect();
      tokens.push(TokenInfo {
        surface: token.surface().to_string(),
        pos: fields.first().unwrap_or(&"*").to_string(),
        pos_sub1: fields.get(1).unwrap_or(&"*").to_string(),
        conj_form: fields.get(5).unwrap_or(&"*").to_string(),
        base_form: if fields.len() > 6 && fields[6] != "*" {
          fields[6].to_string()
        } else {
          token.surface().to_string()
        },
      });
    }

    // Count independent content words (verbs/adjectives tagged as 自立)
    let content_indices: Vec<usize> = tokens.iter().enumerate()
      .filter(|(_, t)| {
        (t.pos == "動詞" || t.pos == "形容詞") && t.pos_sub1 == "自立"
      })
      .map(|(i, _)| i)
      .collect();

    // Only detect inflection for single content word + auxiliaries
    if content_indices.len() != 1 {
      return vec![];
    }

    let idx = content_indices[0];
    let content = &tokens[idx];

    // If base form equals the query, no inflection to report
    if content.base_form == query {
      return vec![];
    }

    let is_adjective = content.pos == "形容詞";

    // Collect auxiliary/particle surfaces after the content word
    let aux_chain: String = tokens[idx + 1..]
      .iter()
      .map(|t| t.surface.as_str())
      .collect();

    if let Some((form_name, description)) =
      identify_inflection(&content.conj_form, &aux_chain, is_adjective)
    {
      vec![Inflection {
        surface: query.to_string(),
        base_form: content.base_form.clone(),
        form_name,
        description,
      }]
    } else {
      vec![]
    }
  }

  fn lookup_exact(&self, query: &str) -> Result<Vec<DictEntry>, DictError> {
    let mut stmt = self.jmdict.prepare(
      "SELECT DISTINCT e.ent_seq FROM entries e
       LEFT JOIN kanji k ON e.ent_seq = k.ent_seq
       LEFT JOIN readings r ON e.ent_seq = r.ent_seq
       WHERE k.keb = ?1 OR r.reb = ?1
       LIMIT 20",
    )?;

    let seq_ids: Vec<i64> = stmt
      .query_map([query], |row| row.get(0))?
      .filter_map(|r| r.ok())
      .collect();

    self.load_entries(&seq_ids)
  }

  fn lookup_fts(&self, query: &str) -> Result<Vec<DictEntry>, DictError> {
    let fts_query = format!("\"{}\"", query.replace('"', "\"\""));
    let mut stmt = self.jmdict.prepare(
      "SELECT DISTINCT ent_seq FROM glosses_fts WHERE glosses_fts MATCH ?1 LIMIT 20",
    )?;

    let seq_ids: Vec<i64> = stmt
      .query_map([&fts_query], |row| row.get(0))?
      .filter_map(|r| r.ok())
      .collect();

    self.load_entries(&seq_ids)
  }

  fn load_entries(&self, seq_ids: &[i64]) -> Result<Vec<DictEntry>, DictError> {
    let mut results = Vec::new();

    for &seq_id in seq_ids {
      let mut kanji_stmt = self
        .jmdict
        .prepare_cached("SELECT keb FROM kanji WHERE ent_seq = ?1")?;
      let kanji: Vec<String> = kanji_stmt
        .query_map([seq_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

      let mut read_stmt = self
        .jmdict
        .prepare_cached("SELECT reb FROM readings WHERE ent_seq = ?1")?;
      let readings: Vec<String> = read_stmt
        .query_map([seq_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

      let mut sense_stmt = self.jmdict.prepare_cached(
        "SELECT sense_id, pos, misc FROM senses WHERE ent_seq = ?1 ORDER BY sense_id",
      )?;
      let sense_rows: Vec<(i64, String, String)> = sense_stmt
        .query_map([seq_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();

      let mut senses = Vec::new();
      for (sense_id, pos_str, misc_str) in &sense_rows {
        let mut gloss_stmt = self.jmdict.prepare_cached(
          "SELECT gloss FROM glosses WHERE ent_seq = ?1 AND sense_id = ?2",
        )?;
        let glosses: Vec<String> = gloss_stmt
          .query_map(rusqlite::params![seq_id, sense_id], |row| row.get(0))?
          .filter_map(|r| r.ok())
          .collect();

        let pos: Vec<String> = pos_str
          .split(';')
          .filter(|s| !s.is_empty())
          .map(String::from)
          .collect();

        let misc: Vec<String> = misc_str
          .split(';')
          .filter(|s| !s.is_empty())
          .map(String::from)
          .collect();

        senses.push(Sense {
          pos,
          glosses,
          misc,
        });
      }

      results.push(DictEntry {
        ent_seq: seq_id,
        kanji,
        readings,
        senses,
      });
    }

    Ok(results)
  }

  pub fn lookup_kanji(&self, ch: char) -> Result<Option<KanjiEntry>, DictError> {
    let literal = ch.to_string();
    let mut stmt = self.kanjidic.prepare(
      "SELECT literal, grade, stroke_count, jlpt, freq FROM kanji WHERE literal = ?1",
    )?;

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
      .kanjidic
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
      .kanjidic
      .prepare("SELECT meaning FROM meanings WHERE literal = ?1 AND lang IS NULL")?;
    entry.meanings = meaning_stmt
      .query_map([&literal], |row| row.get(0))?
      .filter_map(|r| r.ok())
      .collect();

    Ok(Some(entry))
  }
}
