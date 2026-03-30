use std::path::PathBuf;

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::Connection;

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("Usage: {} <kanjidic2.xml> <output_dir>", args[0]);
    std::process::exit(1);
  }

  let kanjidic_path = &args[1];
  let output_dir = PathBuf::from(&args[2]);
  std::fs::create_dir_all(&output_dir)?;

  eprintln!("Building KANJIDIC2 database...");
  build_kanjidic(kanjidic_path, &output_dir.join("kanjidic2.sqlite"))?;

  eprintln!("Done.");
  Ok(())
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

  let file =
    std::fs::File::open(xml_path).with_context(|| format!("failed to open {}", xml_path))?;
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
                current_r_type = String::from_utf8_lossy(&attr.value).to_string();
              }
            }
          }
          "meaning" if in_rmgroup => {
            current_m_lang.clear();
            for attr in e.attributes().flatten() {
              if attr.key.as_ref() == b"m_lang" {
                current_m_lang = String::from_utf8_lossy(&attr.value).to_string();
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
  conn.execute_batch("PRAGMA journal_mode=DELETE; VACUUM;")?;
  eprintln!("  {} characters total", char_count);
  Ok(())
}
