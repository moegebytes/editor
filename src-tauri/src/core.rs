use crate::strings::StringsEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryType {
  Text,
  Comment,
  Include,
  Emit,
  Reference,
  Blank,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatEntry {
  pub index: usize,
  pub entry_type: EntryType,
  pub jp_text: Option<String>,
  pub en_text: Option<String>,
  pub source_file: Option<String>,
  pub depth: u32,
  #[serde(default)]
  pub notes: Vec<String>,
}

struct FlatRow {
  entry_type: EntryType,
  text: Option<String>,
  source_file: Option<String>,
  depth: u32,
  notes: Vec<String>,
}

fn flatten_entries(entries: &[StringsEntry], source_file: Option<&str>, depth: u32) -> Vec<FlatRow> {
  let mut rows = Vec::new();

  for entry in entries {
    match entry {
      StringsEntry::Text(s) => rows.push(FlatRow {
        entry_type: EntryType::Text,
        text: Some(s.clone()),
        source_file: source_file.map(String::from),
        depth,
        notes: vec![],
      }),
      StringsEntry::Comment(s) => rows.push(FlatRow {
        entry_type: EntryType::Comment,
        text: Some(s.clone()),
        source_file: source_file.map(String::from),
        depth,
        notes: vec![],
      }),
      StringsEntry::Include {
        path,
        entries: sub_entries,
      } => {
        rows.push(FlatRow {
          entry_type: EntryType::Include,
          text: Some(path.clone()),
          source_file: source_file.map(String::from),
          depth,
          notes: vec![],
        });
        rows.extend(flatten_entries(sub_entries, Some(path), depth + 1));
      }
      StringsEntry::Emit(s) => rows.push(FlatRow {
        entry_type: EntryType::Emit,
        text: Some(s.clone()),
        source_file: source_file.map(String::from),
        depth,
        notes: vec![],
      }),
      StringsEntry::Reference(s) => rows.push(FlatRow {
        entry_type: EntryType::Reference,
        text: Some(s.clone()),
        source_file: source_file.map(String::from),
        depth,
        notes: vec![],
      }),
      StringsEntry::Blank => rows.push(FlatRow {
        entry_type: EntryType::Blank,
        text: None,
        source_file: source_file.map(String::from),
        depth,
        notes: vec![],
      }),
    }
  }

  rows
}

fn strip_comment_prefix(s: &str) -> String {
  s.strip_prefix("; ")
    .or_else(|| s.strip_prefix(';'))
    .unwrap_or(s)
    .to_string()
}

// Absorb comment rows that directly precede a text row into the text row's notes.
// Comments preceding non-text entries are kept as separate rows.
// This operates on FlatRows (single side) before pairing.
fn absorb_notes_in_rows(rows: &mut Vec<FlatRow>) {
  let mut to_remove = Vec::new();
  let mut i = 0;

  while i < rows.len() {
    let comment_start = i;
    while i < rows.len() && rows[i].entry_type == EntryType::Comment {
      i += 1;
    }

    if i > comment_start && i < rows.len() && rows[i].entry_type == EntryType::Text {
      let notes: Vec<String> = rows[comment_start..i]
        .iter()
        .filter_map(|r| r.text.as_ref().map(|s| strip_comment_prefix(s)))
        .collect();
      rows[i].notes = notes;

      for j in comment_start..i {
        to_remove.push(j);
      }
    }

    if i == comment_start {
      i += 1;
    }
  }

  let remove_set: std::collections::HashSet<usize> = to_remove.into_iter().collect();
  let mut idx = 0;
  rows.retain(|_| {
    let keep = !remove_set.contains(&idx);
    idx += 1;
    keep
  });
}

pub fn pair_files(jp_entries: &[StringsEntry], en_entries: &[StringsEntry]) -> Vec<FlatEntry> {
  let mut jp_rows = flatten_entries(jp_entries, None, 0);
  let mut en_rows = flatten_entries(en_entries, None, 0);

  // Absorb comments into following text rows _before_ pairing so that added/removed notes don't
  // shift positional alignment
  absorb_notes_in_rows(&mut jp_rows);
  absorb_notes_in_rows(&mut en_rows);

  let max_len = jp_rows.len().max(en_rows.len());
  let mut result = Vec::with_capacity(max_len);

  for i in 0..max_len {
    let jp = jp_rows.get(i);
    let en = en_rows.get(i);

    let entry_type = jp
      .map(|r| r.entry_type)
      .or_else(|| en.map(|r| r.entry_type))
      .unwrap_or(EntryType::Blank);

    let source_file = jp
      .and_then(|r| r.source_file.clone())
      .or_else(|| en.and_then(|r| r.source_file.clone()));

    let depth = jp.map(|r| r.depth).unwrap_or_else(|| en.map(|r| r.depth).unwrap_or(0));

    // Prefer EN side, fall back to JP side
    let notes = en
      .map(|r| r.notes.clone())
      .filter(|n| !n.is_empty())
      .or_else(|| jp.map(|r| r.notes.clone()))
      .unwrap_or_default();

    result.push(FlatEntry {
      index: i,
      entry_type,
      jp_text: jp.and_then(|r| r.text.clone()),
      en_text: en.and_then(|r| r.text.clone()),
      source_file,
      depth,
      notes,
    });
  }

  result
}

pub fn reconstruct_entries(flat: &[FlatEntry]) -> Vec<StringsEntry> {
  reconstruct_at_depth(flat, 0)
}

fn en_text(entry: &FlatEntry) -> String {
  entry.en_text.clone().unwrap_or_default()
}

fn emit_notes(result: &mut Vec<StringsEntry>, entry: &FlatEntry) {
  for note in &entry.notes {
    if note.is_empty() {
      result.push(StringsEntry::Comment(";".to_string()));
    } else {
      result.push(StringsEntry::Comment(format!("; {}", note)));
    }
  }
}

fn reconstruct_at_depth(flat: &[FlatEntry], base_depth: u32) -> Vec<StringsEntry> {
  let mut result = Vec::new();
  let mut i = 0;

  while i < flat.len() {
    let entry = &flat[i];
    if entry.depth != base_depth {
      i += 1;
      continue;
    }

    match entry.entry_type {
      EntryType::Text => {
        emit_notes(&mut result, entry);
        result.push(StringsEntry::Text(en_text(entry)));
      }
      EntryType::Comment => {
        result.push(StringsEntry::Comment(en_text(entry)));
      }
      EntryType::Include => {
        let inc_path = en_text(entry);
        let target_depth = base_depth + 1;
        let sub_start = i + 1;
        let mut sub_end = sub_start;
        while sub_end < flat.len() && flat[sub_end].depth >= target_depth {
          sub_end += 1;
        }
        let sub_entries = reconstruct_at_depth(&flat[sub_start..sub_end], target_depth);
        result.push(StringsEntry::Include {
          path: inc_path,
          entries: sub_entries,
        });
        i = sub_end;
        continue;
      }
      EntryType::Emit => {
        result.push(StringsEntry::Emit(en_text(entry)));
      }
      EntryType::Reference => {
        result.push(StringsEntry::Reference(en_text(entry)));
      }
      EntryType::Blank => {
        result.push(StringsEntry::Blank);
      }
    }
    i += 1;
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  fn text(s: &str) -> StringsEntry {
    StringsEntry::Text(s.to_string())
  }

  fn comment(s: &str) -> StringsEntry {
    StringsEntry::Comment(s.to_string())
  }

  #[test]
  fn pair_equal_length() {
    let jp = vec![text("こんにちは"), text("さようなら")];
    let en = vec![text("Hello"), text("Goodbye")];
    let paired = pair_files(&jp, &en);

    assert_eq!(paired.len(), 2);
    assert_eq!(paired[0].jp_text.as_deref(), Some("こんにちは"));
    assert_eq!(paired[0].en_text.as_deref(), Some("Hello"));
    assert_eq!(paired[0].entry_type, EntryType::Text);
  }

  #[test]
  fn pair_unequal_length() {
    let jp = vec![text("一"), text("二"), text("三")];
    let en = vec![text("One")];
    let paired = pair_files(&jp, &en);

    assert_eq!(paired.len(), 3);
    assert_eq!(paired[0].en_text.as_deref(), Some("One"));
    assert_eq!(paired[1].en_text, None);
    assert_eq!(paired[2].en_text, None);
  }

  #[test]
  fn notes_absorbed_from_comments() {
    let jp = vec![
      comment("; Translation note"),
      comment("; Second note"),
      text("テスト"),
      StringsEntry::Blank,
    ];
    let en = vec![
      comment("; Translation note"),
      comment("; Second note"),
      text("Test"),
      StringsEntry::Blank,
    ];
    let paired = pair_files(&jp, &en);

    assert_eq!(paired.len(), 2);
    assert_eq!(paired[0].entry_type, EntryType::Text);
    assert_eq!(paired[0].notes, vec!["Translation note", "Second note"]);
    assert_eq!(paired[0].en_text.as_deref(), Some("Test"));
    assert_eq!(paired[1].entry_type, EntryType::Blank);
  }

  #[test]
  fn comments_before_non_text_kept() {
    let jp = vec![
      comment("; File header"),
      StringsEntry::Include {
        path: "sub.txt".to_string(),
        entries: vec![text("中身")],
      },
    ];
    let en = vec![
      comment("; File header"),
      StringsEntry::Include {
        path: "sub.txt".to_string(),
        entries: vec![text("Content")],
      },
    ];
    let paired = pair_files(&jp, &en);

    assert_eq!(paired[0].entry_type, EntryType::Comment);
    assert_eq!(paired[1].entry_type, EntryType::Include);
  }

  #[test]
  fn notes_round_trip() {
    let en = vec![comment("; Note about choice"), text("Test"), StringsEntry::Blank];
    let jp = vec![comment("; Note about choice"), text("テスト"), StringsEntry::Blank];
    let paired = pair_files(&jp, &en);

    assert_eq!(paired[0].entry_type, EntryType::Text);
    assert_eq!(paired[0].notes, vec!["Note about choice"]);

    let reconstructed = reconstruct_entries(&paired);
    assert_eq!(reconstructed, en);
  }

  #[test]
  fn notes_added_then_reopened() {
    let jp_original = vec![text("テスト"), text("二行目")];
    let en_original = vec![text("Test"), text("Second line")];

    let mut paired = pair_files(&jp_original, &en_original);
    assert_eq!(paired.len(), 2);
    assert!(paired[0].notes.is_empty());

    paired[0].notes = vec!["Editor note".to_string()];

    let en_saved = reconstruct_entries(&paired);
    assert_eq!(en_saved.len(), 3); // comment + text + text
    assert_eq!(en_saved[0], StringsEntry::Comment("; Editor note".to_string()));
    assert_eq!(en_saved[1], StringsEntry::Text("Test".to_string()));
    assert_eq!(en_saved[2], StringsEntry::Text("Second line".to_string()));

    let re_paired = pair_files(&jp_original, &en_saved);
    assert_eq!(re_paired.len(), 2);
    assert_eq!(re_paired[0].jp_text.as_deref(), Some("テスト"));
    assert_eq!(re_paired[0].en_text.as_deref(), Some("Test"));
    assert_eq!(re_paired[0].notes, vec!["Editor note"]);
    assert_eq!(re_paired[1].jp_text.as_deref(), Some("二行目"));
    assert_eq!(re_paired[1].en_text.as_deref(), Some("Second line"));
  }

  #[test]
  fn notes_added_by_user_round_trip() {
    let jp = vec![text("テスト")];
    let en = vec![text("Test")];
    let mut paired = pair_files(&jp, &en);

    paired[0].notes = vec!["New note".to_string(), "Another note".to_string()];

    let reconstructed = reconstruct_entries(&paired);
    assert_eq!(reconstructed.len(), 3);
    assert_eq!(reconstructed[0], StringsEntry::Comment("; New note".to_string()));
    assert_eq!(reconstructed[1], StringsEntry::Comment("; Another note".to_string()));
    assert_eq!(reconstructed[2], StringsEntry::Text("Test".to_string()));
  }

  #[test]
  fn pair_with_includes() {
    let jp = vec![
      text("Top"),
      StringsEntry::Include {
        path: "sub.txt".to_string(),
        entries: vec![text("中身")],
      },
      text("Bottom"),
    ];
    let en = vec![
      text("Top"),
      StringsEntry::Include {
        path: "sub.txt".to_string(),
        entries: vec![text("Content")],
      },
      text("Bottom"),
    ];
    let paired = pair_files(&jp, &en);

    assert_eq!(paired.len(), 4);
    assert_eq!(paired[0].entry_type, EntryType::Text);
    assert_eq!(paired[1].entry_type, EntryType::Include);
    assert_eq!(paired[2].entry_type, EntryType::Text);
    assert_eq!(paired[2].depth, 1);
    assert_eq!(paired[3].entry_type, EntryType::Text);
    assert_eq!(paired[3].depth, 0);
  }

  #[test]
  fn reconstruct_with_includes() {
    let en = vec![
      text("Top"),
      StringsEntry::Include {
        path: "sub.txt".to_string(),
        entries: vec![text("Content")],
      },
      text("Bottom"),
    ];
    let jp = vec![
      text("上"),
      StringsEntry::Include {
        path: "sub.txt".to_string(),
        entries: vec![text("中身")],
      },
      text("下"),
    ];
    let paired = pair_files(&jp, &en);
    let reconstructed = reconstruct_entries(&paired);
    assert_eq!(reconstructed, en);
  }
}
