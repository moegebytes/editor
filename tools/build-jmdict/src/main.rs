use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::Connection;

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("Usage: {} <JMdict_e.xml> <output_dir>", args[0]);
    std::process::exit(1);
  }

  let jmdict_path = &args[1];
  let output_dir = PathBuf::from(&args[2]);
  std::fs::create_dir_all(&output_dir)?;

  eprintln!("Building JMdict database...");
  build_jmdict(jmdict_path, &output_dir.join("jmdict.sqlite"))?;

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

  eprintln!("  Rebuilding FTS index...");
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

