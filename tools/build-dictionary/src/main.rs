use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::Connection;

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 4 {
    eprintln!("Usage: {} <JMdict_e.xml> <kanjidic2.xml> <output_dir>", args[0]);
    std::process::exit(1);
  }

  let jmdict_path = &args[1];
  let kanjidic_path = &args[2];
  let output_dir = PathBuf::from(&args[3]);
  std::fs::create_dir_all(&output_dir)?;

  eprintln!("Building JMdict database...");
  build_jmdict(jmdict_path, &output_dir.join("jmdict.sqlite"))?;

  eprintln!("Building KANJIDIC2 database...");
  build_kanjidic(kanjidic_path, &output_dir.join("kanjidic2.sqlite"))?;

  eprintln!("Done.");
  Ok(())
}

fn build_jmdict(xml_path: &str, db_path: &PathBuf) -> Result<()> {
  let entity_map = parse_jmdict_entities(xml_path)?;

  if db_path.exists() {
    std::fs::remove_file(db_path)?;
  }
  let mut conn = Connection::open(db_path)?;
  conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=OFF;")?;

  conn.execute_batch(
    "CREATE TABLE entries (ent_seq INTEGER PRIMARY KEY);
     CREATE TABLE kanji (ent_seq INTEGER, keb TEXT);
     CREATE TABLE readings (ent_seq INTEGER, reb TEXT);
     CREATE TABLE senses (ent_seq INTEGER, sense_id INTEGER, pos TEXT, misc TEXT);
     CREATE TABLE glosses (ent_seq INTEGER, sense_id INTEGER, gloss TEXT);
     CREATE VIRTUAL TABLE glosses_fts USING fts5(gloss, ent_seq UNINDEXED, content=glosses);
     CREATE INDEX idx_kanji_seq ON kanji(ent_seq);
     CREATE INDEX idx_kanji_keb ON kanji(keb);
     CREATE INDEX idx_readings_seq ON readings(ent_seq);
     CREATE INDEX idx_readings_reb ON readings(reb);
     CREATE INDEX idx_senses_seq ON senses(ent_seq);
     CREATE INDEX idx_glosses_seq ON glosses(ent_seq, sense_id);",
  )?;

  let tx = conn.transaction()?;

  let file = std::fs::File::open(xml_path).with_context(|| format!("failed to open {}", xml_path))?;
  let buf_reader = std::io::BufReader::new(file);
  let mut reader = Reader::from_reader(buf_reader);
  reader.config_mut().trim_text(true);

  let mut buf = Vec::new();
  let mut entry_count = 0u64;

  let mut in_entry = false;
  let mut in_k_ele = false;
  let mut in_r_ele = false;
  let mut in_sense = false;
  let mut current_tag = String::new();

  let mut ent_seq: i64 = 0;
  let mut kanji_list: Vec<String> = Vec::new();
  let mut reading_list: Vec<String> = Vec::new();
  let mut sense_id: i64 = 0;
  let mut pos_list: Vec<String> = Vec::new();
  let mut misc_list: Vec<String> = Vec::new();
  let mut gloss_list: Vec<String> = Vec::new();

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(e)) => {
        let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
        current_tag = tag.clone();

        match tag.as_str() {
          "entry" => {
            in_entry = true;
            ent_seq = 0;
            kanji_list.clear();
            reading_list.clear();
            sense_id = 0;
          }
          "k_ele" => in_k_ele = true,
          "r_ele" => in_r_ele = true,
          "sense" => {
            in_sense = true;
            sense_id += 1;
            pos_list.clear();
            misc_list.clear();
            gloss_list.clear();
          }
          _ => {}
        }
      }
      Ok(Event::End(e)) => {
        let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
        match tag.as_str() {
          "entry" => {
            if in_entry && ent_seq > 0 {
              tx.execute("INSERT INTO entries VALUES (?1)", [ent_seq])?;
              for k in &kanji_list {
                tx.execute("INSERT INTO kanji VALUES (?1, ?2)", rusqlite::params![ent_seq, k])?;
              }
              for r in &reading_list {
                tx.execute("INSERT INTO readings VALUES (?1, ?2)",rusqlite::params![ent_seq, r])?;
              }
              entry_count += 1;
              if entry_count % 1000 == 0 {
                eprintln!("  {} entries...", entry_count);
              }
            }
            in_entry = false;
          }
          "k_ele" => in_k_ele = false,
          "r_ele" => in_r_ele = false,
          "sense" => {
            if in_sense && ent_seq > 0 {
              let pos_str = pos_list.join(";");
              let misc_str = misc_list.join(";");
              tx.execute(
                "INSERT INTO senses VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![ent_seq, sense_id, pos_str, misc_str],
              )?;
              for g in &gloss_list {
                tx.execute("INSERT INTO glosses VALUES (?1, ?2, ?3)",
                  rusqlite::params![ent_seq, sense_id, g],
                )?;
              }
            }
            in_sense = false;
          }
          _ => {}
        }
        current_tag.clear();
      }
      Ok(Event::Text(e)) => {
        if !in_entry {
          continue;
        }
        let text = String::from_utf8_lossy(e.as_ref()).to_string();
        match current_tag.as_str() {
          "ent_seq" => ent_seq = text.parse().unwrap_or(0),
          "keb" if in_k_ele => kanji_list.push(text),
          "reb" if in_r_ele => reading_list.push(text),
          "gloss" if in_sense => gloss_list.push(text),
          _ => {}
        }
      }
      Ok(Event::GeneralRef(e)) => {
        if !in_entry || !in_sense {
          continue; // Entity references like &n;, &v1;, etc.
        }
        let name = String::from_utf8_lossy(e.as_ref()).to_string();
        let resolved = entity_map
          .get(&name)
          .cloned()
          .unwrap_or_else(|| name.clone());
        match current_tag.as_str() {
          "pos" => pos_list.push(resolved),
          "misc" => misc_list.push(resolved),
          _ => {}
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => {
        eprintln!("XML error at position {}: {:?}", reader.error_position(), e);
        break;
      }
      _ => {}
    }
    buf.clear();
  }

  tx.execute("INSERT INTO glosses_fts(glosses_fts) VALUES('rebuild')", [])?;
  tx.commit()?;
  conn.execute_batch("PRAGMA journal_mode=DELETE;")?;

  eprintln!("  {} entries total", entry_count);
  Ok(())
}

fn parse_jmdict_entities(xml_path: &str) -> Result<HashMap<String, String>> {
  let file = std::fs::File::open(xml_path)?;
  let reader = std::io::BufReader::new(file);
  let mut entities = HashMap::new();

  for line in reader.lines() {
    let line = line?;
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("<!ENTITY ") {
      if let Some(name_end) = rest.find(' ') {
        let name = &rest[..name_end];
        let value_part = &rest[name_end + 1..];
        if let (Some(start), Some(end)) = (value_part.find('"'), value_part.rfind('"')) {
          if start < end {
            let value = &value_part[start + 1..end];
            entities.insert(name.to_string(), value.to_string());
          }
        }
      }
    }
    if trimmed == "]>" {
      break;
    }
  }

  eprintln!("  Parsed {} entity definitions", entities.len());
  Ok(entities)
}

fn build_kanjidic(xml_path: &str, db_path: &PathBuf) -> Result<()> {
  if db_path.exists() {
    std::fs::remove_file(db_path)?;
  }
  let mut conn = Connection::open(db_path)?;
  conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=OFF;")?;

  conn.execute_batch(
    "CREATE TABLE kanji (
       literal TEXT PRIMARY KEY,
       grade INTEGER,
       stroke_count INTEGER NOT NULL,
       jlpt INTEGER,
       freq INTEGER
     );
     CREATE TABLE readings (literal TEXT, reading TEXT, r_type TEXT);
     CREATE TABLE meanings (literal TEXT, meaning TEXT, lang TEXT);
     CREATE INDEX idx_readings_literal ON readings(literal);
     CREATE INDEX idx_meanings_literal ON meanings(literal);",
  )?;

  let tx = conn.transaction()?;

  let file = std::fs::File::open(xml_path).with_context(|| format!("failed to open {}", xml_path))?;
  let buf_reader = std::io::BufReader::new(file);
  let mut reader = Reader::from_reader(buf_reader);
  reader.config_mut().trim_text(true);

  let mut buf = Vec::new();
  let mut char_count = 0u64;

  let mut in_character = false;
  let mut in_rmgroup = false;
  let mut in_misc = false;
  let mut current_tag = String::new();
  let mut current_r_type = String::new();
  let mut current_m_lang = String::new();

  let mut literal = String::new();
  let mut grade: Option<i32> = None;
  let mut stroke_count: i32 = 0;
  let mut jlpt: Option<i32> = None;
  let mut freq: Option<i32> = None;
  let mut readings: Vec<(String, String)> = Vec::new();
  let mut meanings: Vec<(String, Option<String>)> = Vec::new();

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(e)) => {
        let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
        current_tag = tag.clone();

        match tag.as_str() {
          "character" => {
            in_character = true;
            literal.clear();
            grade = None;
            stroke_count = 0;
            jlpt = None;
            freq = None;
            readings.clear();
            meanings.clear();
          }
          "misc" => in_misc = true,
          "rmgroup" => in_rmgroup = true,
          "reading" if in_rmgroup => {
            current_r_type.clear();
            for attr in e.attributes().flatten() {
              if attr.key.as_ref() == b"r_type" {
                current_r_type =
                  String::from_utf8_lossy(&attr.value).to_string();
              }
            }
          }
          "meaning" if in_rmgroup => {
            current_m_lang.clear();
            for attr in e.attributes().flatten() {
              if attr.key.as_ref() == b"m_lang" {
                current_m_lang =
                  String::from_utf8_lossy(&attr.value).to_string();
              }
            }
          }
          _ => {}
        }
      }
      Ok(Event::End(e)) => {
        let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
        match tag.as_str() {
          "character" => {
            if in_character && !literal.is_empty() {
              tx.execute(
                "INSERT INTO kanji VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![literal, grade, stroke_count, jlpt, freq],
              )?;
              for (reading, r_type) in &readings {
                tx.execute(
                  "INSERT INTO readings VALUES (?1, ?2, ?3)",
                  rusqlite::params![literal, reading, r_type],
                )?;
              }
              for (meaning, lang) in &meanings {
                tx.execute(
                  "INSERT INTO meanings VALUES (?1, ?2, ?3)",
                  rusqlite::params![literal, meaning, lang.as_deref()],
                )?;
              }
              char_count += 1;
            }
            in_character = false;
          }
          "misc" => in_misc = false,
          "rmgroup" => in_rmgroup = false,
          _ => {}
        }
        current_tag.clear();
      }
      Ok(Event::Text(e)) => {
        if !in_character {
          continue;
        }
        let text = String::from_utf8_lossy(e.as_ref()).to_string();
        match current_tag.as_str() {
          "literal" => literal = text,
          "grade" if in_misc => grade = text.parse().ok(),
          "stroke_count" if in_misc => stroke_count = text.parse().unwrap_or(0),
          "jlpt" if in_misc => jlpt = text.parse().ok(),
          "freq" if in_misc => freq = text.parse().ok(),
          "reading" if in_rmgroup => {
            readings.push((text, current_r_type.clone()));
          }
          "meaning" if in_rmgroup => {
            let lang = if current_m_lang.is_empty() {
              None
            } else {
              Some(current_m_lang.clone())
            };
            meanings.push((text, lang));
          }
          _ => {}
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => {
        eprintln!("XML error at position {}: {:?}", reader.error_position(), e);
        break;
      }
      _ => {}
    }
    buf.clear();
  }

  tx.commit()?;
  conn.execute_batch("PRAGMA journal_mode=DELETE;")?;
  eprintln!("  {} characters total", char_count);
  Ok(())
}
