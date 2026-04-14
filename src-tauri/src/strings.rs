use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StringsError {
  #[error("{}", crate::util::friendly_io_msg("", path, source))]
  Io { path: PathBuf, source: std::io::Error },

  #[error("{}", crate::util::friendly_io_msg("", path, source))]
  ResolvePath { path: PathBuf, source: std::io::Error },

  #[error("circular include detected: {from} includes {to}")]
  CircularInclude { from: PathBuf, to: PathBuf },

  #[error("include path escapes project directory: '{path}'")]
  PathTraversal { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringsEntry {
  Comment(String),
  Include { path: String, entries: Vec<StringsEntry> },
  Reference(String),
  Emit(String),
  Blank,
  Text(String),
}

pub fn parse_strings(path: &Path) -> Result<Vec<StringsEntry>, StringsError> {
  let mut visited = HashSet::new();
  let canonical = path.canonicalize().map_err(|e| StringsError::ResolvePath {
    path: path.to_path_buf(),
    source: e,
  })?;
  let root = canonical.parent().unwrap_or(Path::new(".")).to_path_buf();
  visited.insert(canonical.clone());
  parse_strings_inner(&canonical, &root, &mut visited)
}

fn parse_strings_inner(
  path: &Path,
  root: &Path,
  visited: &mut HashSet<PathBuf>,
) -> Result<Vec<StringsEntry>, StringsError> {
  let content = std::fs::read_to_string(path).map_err(|e| StringsError::Io {
    path: path.to_path_buf(),
    source: e,
  })?;

  let parent = path.parent().unwrap_or(Path::new("."));
  let mut entries = Vec::new();

  for line in content.lines() {
    if line.is_empty() {
      entries.push(StringsEntry::Blank);
    } else if line.starts_with(';') {
      entries.push(StringsEntry::Comment(line.to_string()));
    } else if let Some(rest) = line.strip_prefix("#include") {
      let rest = rest.trim();
      let include_path = if rest.starts_with('<') && rest.ends_with('>') {
        &rest[1..rest.len() - 1]
      } else {
        rest
      };

      let resolved = parent.join(include_path);
      let canonical = resolved.canonicalize().map_err(|e| StringsError::ResolvePath {
        path: resolved.clone(),
        source: e,
      })?;

      if !canonical.starts_with(root) {
        return Err(StringsError::PathTraversal { path: canonical });
      }

      if !visited.insert(canonical.clone()) {
        return Err(StringsError::CircularInclude {
          from: path.to_path_buf(),
          to: canonical,
        });
      }

      let sub_entries = parse_strings_inner(&canonical, root, visited)?;
      visited.remove(&canonical);

      entries.push(StringsEntry::Include {
        path: include_path.to_string(),
        entries: sub_entries,
      });
    } else if let Some(rest) = line.strip_prefix("#reference") {
      let rest = rest.trim();
      let reference_path = if rest.starts_with('<') && rest.ends_with('>') {
        &rest[1..rest.len() - 1]
      } else {
        rest
      };
      entries.push(StringsEntry::Reference(reference_path.to_string()));
    } else if let Some(rest) = line.strip_prefix("#emit") {
      entries.push(StringsEntry::Emit(rest.trim().to_string()));
    } else {
      entries.push(StringsEntry::Text(line.to_string()));
    }
  }

  Ok(entries)
}

pub fn collect_file_paths(entries: &[StringsEntry], parent: &Path) -> Vec<PathBuf> {
  let mut paths = Vec::new();
  for entry in entries {
    if let StringsEntry::Include {
      path: inc_path,
      entries: sub_entries,
    } = entry
    {
      let resolved = parent.join(inc_path);
      let sub_parent = resolved.parent().unwrap_or(parent).to_path_buf();
      paths.push(resolved);
      paths.extend(collect_file_paths(sub_entries, &sub_parent));
    }
  }
  paths
}

pub fn write_strings(entries: &[StringsEntry], path: &Path) -> Result<(), StringsError> {
  let parent = path.parent().unwrap_or(Path::new("."));
  let mut lines = Vec::new();

  for entry in entries {
    match entry {
      StringsEntry::Comment(s) => lines.push(s.clone()),
      StringsEntry::Include {
        path: inc_path,
        entries: sub_entries,
      } => {
        lines.push(format!("#include <{}>", inc_path));
        let resolved = parent.join(inc_path);
        write_strings(sub_entries, &resolved)?;
      }
      StringsEntry::Emit(arg) => lines.push(format!("#emit {}", arg)),
      StringsEntry::Reference(path) => lines.push(format!("#reference <{}>", path)),
      StringsEntry::Blank => lines.push(String::new()),
      StringsEntry::Text(s) => lines.push(s.clone()),
    }
  }

  std::fs::write(path, lines.join("\n")).map_err(|e| StringsError::Io {
    path: path.to_path_buf(),
    source: e,
  })?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  fn write_temp_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
  }

  #[test]
  fn parse_basic_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(dir.path(), "basic.txt", "Hello world\nSecond line");
    let entries = parse_strings(&path).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], StringsEntry::Text("Hello world".to_string()));
    assert_eq!(entries[1], StringsEntry::Text("Second line".to_string()));
  }

  #[test]
  fn parse_comments_and_blanks() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(dir.path(), "comments.txt", "; This is a comment\n\nHello");
    let entries = parse_strings(&path).unwrap();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], StringsEntry::Comment("; This is a comment".to_string()));
    assert_eq!(entries[1], StringsEntry::Blank);
    assert_eq!(entries[2], StringsEntry::Text("Hello".to_string()));
  }

  #[test]
  fn parse_emit_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(dir.path(), "emit.txt", "First\n#emit empty\nThird");
    let entries = parse_strings(&path).unwrap();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1], StringsEntry::Emit("empty".to_string()));
  }

  #[test]
  fn parse_include() {
    let dir = tempfile::tempdir().unwrap();
    write_temp_file(dir.path(), "included.txt", "Included line");
    let path = write_temp_file(dir.path(), "main.txt", "Before\n#include <included.txt>\nAfter");
    let entries = parse_strings(&path).unwrap();

    assert_eq!(entries.len(), 3);
    if let StringsEntry::Include {
      path: inc_path,
      entries: sub,
    } = &entries[1]
    {
      assert_eq!(inc_path, "included.txt");
      assert_eq!(sub.len(), 1);
      assert_eq!(sub[0], StringsEntry::Text("Included line".to_string()));
    } else {
      panic!("expected Include entry");
    }
  }

  #[test]
  fn detect_circular_include() {
    let dir = tempfile::tempdir().unwrap();
    write_temp_file(dir.path(), "a.txt", "#include <b.txt>");
    write_temp_file(dir.path(), "b.txt", "#include <a.txt>");
    let result = parse_strings(&dir.path().join("a.txt"));

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
      err_msg.contains("circular"),
      "error should mention circular: {}",
      err_msg
    );
  }

  #[test]
  fn round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let content = "; Comment\n\n#reference <reference.txt>\nHello world\n#emit empty\nGoodbye";
    let path = write_temp_file(dir.path(), "original.txt", content);
    let entries = parse_strings(&path).unwrap();

    let out_path = dir.path().join("roundtrip.txt");
    write_strings(&entries, &out_path).unwrap();

    let written = std::fs::read_to_string(&out_path).unwrap();
    assert_eq!(written, content);
  }

  #[test]
  fn round_trip_with_include() {
    let dir = tempfile::tempdir().unwrap();
    write_temp_file(dir.path(), "sub.txt", "Middle");
    let content = "Top\n#include <sub.txt>\nBottom";
    let path = write_temp_file(dir.path(), "main.txt", content);
    let entries = parse_strings(&path).unwrap();

    let out_dir = dir.path().join("output");
    std::fs::create_dir_all(&out_dir).unwrap();

    let out_path = out_dir.join("main.txt");
    write_strings(&entries, &out_path).unwrap();

    let written_main = std::fs::read_to_string(&out_path).unwrap();
    assert_eq!(written_main, content);

    let written_sub = std::fs::read_to_string(out_dir.join("sub.txt")).unwrap();
    assert_eq!(written_sub, "Middle");
  }

  #[test]
  fn parse_reference() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(dir.path(), "references.txt", "First\n#reference <referenced.txt>\nLast");
    let entries = parse_strings(&path).unwrap();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1], StringsEntry::Reference("referenced.txt".to_string()));
  }

  #[test]
  fn reject_path_traversal() {
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().parent().unwrap();
    write_temp_file(parent, "secret.txt", "sensitive data");
    let path = write_temp_file(dir.path(), "evil.txt", "#include <../secret.txt>");
    let result = parse_strings(&path);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
      err_msg.contains("escapes project directory"),
      "error should mention path traversal: {}",
      err_msg
    );
  }

  #[test]
  fn allow_subdirectory_include() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    write_temp_file(&sub, "child.txt", "Nested content");
    let path = write_temp_file(dir.path(), "main.txt", "#include <sub/child.txt>");
    let entries = parse_strings(&path).unwrap();

    assert_eq!(entries.len(), 1);
    if let StringsEntry::Include { entries: sub, .. } = &entries[0] {
      assert_eq!(sub.len(), 1);
      assert_eq!(sub[0], StringsEntry::Text("Nested content".to_string()));
    } else {
      panic!("expected Include entry");
    }
  }
}
