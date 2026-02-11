//! User-facing prelude for lob data pipelines
//!
//! This crate provides the public API that users interact with in their
//! generated code. It re-exports the core functionality and adds convenient
//! helpers like `input()` for reading from stdin.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

// Re-export core types and traits
pub use lob_core::{HashSet, Lob, LobExt};

// Re-export serde_json for JSON output
pub use serde_json;

// Re-export tabled for table output
pub use tabled;

/// Creates a Lob iterator from stdin lines
///
/// This function reads lines from stdin and returns a `Lob` iterator over them.
/// Lines are trimmed and empty lines are filtered out by default.
///
/// # Examples
///
/// ```no_run
/// use lob_prelude::*;
///
/// // Read lines from stdin and filter
/// let result: Vec<_> = input()
///     .filter(|line| line.contains("ERROR"))
///     .collect();
/// ```
#[must_use]
pub fn input() -> Lob<impl Iterator<Item = String>> {
    let stdin = io::stdin();
    Lob::new(
        stdin
            .lock()
            .lines()
            .map_while(Result::ok)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    )
}

/// Creates a Lob iterator from any iterable
///
/// This is a convenience function to convert any type that implements
/// `IntoIterator` into a `Lob` iterator.
///
/// # Examples
///
/// ```
/// use lob_prelude::*;
///
/// let result: Vec<_> = lob(vec![1, 2, 3, 4, 5])
///     .filter(|x| x % 2 == 0)
///     .collect();
///
/// assert_eq!(result, vec![2, 4]);
/// ```
#[must_use]
pub fn lob<I: IntoIterator>(iterable: I) -> Lob<I::IntoIter> {
    Lob::new(iterable.into_iter())
}

/// Creates a Lob iterator from a range
///
/// # Examples
///
/// ```
/// use lob_prelude::*;
///
/// let result: Vec<_> = range(0, 5)
///     .map(|x| x * 2)
///     .collect();
///
/// assert_eq!(result, vec![0, 2, 4, 6, 8]);
/// ```
#[must_use]
pub fn range(start: i64, end: i64) -> Lob<impl Iterator<Item = i64>> {
    Lob::new(start..end)
}

// File input helpers

/// Read lines from multiple files
#[must_use]
#[allow(clippy::needless_collect)]
pub fn input_from_files(paths: &[std::path::PathBuf]) -> Lob<impl Iterator<Item = String>> {
    let lines: Vec<String> = paths
        .iter()
        .flat_map(|path| {
            File::open(path)
                .ok()
                .map(|file| {
                    BufReader::new(file)
                        .lines()
                        .map_while(Result::ok)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    Lob::new(lines.into_iter())
}

// CSV input helpers

/// Parse CSV from stdin with headers
#[must_use]
pub fn input_csv() -> Lob<impl Iterator<Item = HashMap<String, String>>> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    parse_csv_reader(reader)
}

/// Parse CSV from files with headers
#[must_use]
#[allow(clippy::needless_collect)]
pub fn input_csv_from_files(
    paths: &[std::path::PathBuf],
) -> Lob<impl Iterator<Item = HashMap<String, String>>> {
    let rows: Vec<HashMap<String, String>> = paths
        .iter()
        .flat_map(|path| {
            File::open(path)
                .ok()
                .map(|file| {
                    let reader = BufReader::new(file);
                    parse_csv_reader(reader).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    Lob::new(rows.into_iter())
}

fn parse_csv_reader<R: io::Read>(reader: R) -> Lob<impl Iterator<Item = HashMap<String, String>>> {
    let mut csv_reader = csv::Reader::from_reader(reader);

    let headers: Vec<String> = csv_reader
        .headers()
        .ok()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, String>> = csv_reader
        .records()
        .filter_map(Result::ok)
        .map(|record| {
            let mut row = HashMap::new();
            for (header, value) in headers.iter().zip(record.iter()) {
                row.insert(header.clone(), value.to_string());
            }
            row
        })
        .collect();

    Lob::new(rows.into_iter())
}

// TSV input helpers

/// Parse TSV from stdin with headers
#[must_use]
pub fn input_tsv() -> Lob<impl Iterator<Item = HashMap<String, String>>> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    parse_tsv_reader(reader)
}

/// Parse TSV from files with headers
#[must_use]
#[allow(clippy::needless_collect)]
pub fn input_tsv_from_files(
    paths: &[std::path::PathBuf],
) -> Lob<impl Iterator<Item = HashMap<String, String>>> {
    let rows: Vec<HashMap<String, String>> = paths
        .iter()
        .flat_map(|path| {
            File::open(path)
                .ok()
                .map(|file| {
                    let reader = BufReader::new(file);
                    parse_tsv_reader(reader).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    Lob::new(rows.into_iter())
}

fn parse_tsv_reader<R: io::Read>(reader: R) -> Lob<impl Iterator<Item = HashMap<String, String>>> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(reader);

    let headers: Vec<String> = csv_reader
        .headers()
        .ok()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let rows: Vec<HashMap<String, String>> = csv_reader
        .records()
        .filter_map(Result::ok)
        .map(|record| {
            let mut row = HashMap::new();
            for (header, value) in headers.iter().zip(record.iter()) {
                row.insert(header.clone(), value.to_string());
            }
            row
        })
        .collect();

    Lob::new(rows.into_iter())
}

// JSON input helpers

/// Parse JSON lines from stdin
#[must_use]
pub fn input_json() -> Lob<impl Iterator<Item = serde_json::Value>> {
    let stdin = io::stdin();
    Lob::new(
        stdin
            .lock()
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str(&line).ok()),
    )
}

/// Parse JSON lines from files
#[must_use]
#[allow(clippy::needless_collect)]
pub fn input_json_from_files(
    paths: &[std::path::PathBuf],
) -> Lob<impl Iterator<Item = serde_json::Value>> {
    let values: Vec<serde_json::Value> = paths
        .iter()
        .flat_map(|path| {
            File::open(path)
                .ok()
                .map(|file| {
                    BufReader::new(file)
                        .lines()
                        .map_while(Result::ok)
                        .filter_map(|line| serde_json::from_str(&line).ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    Lob::new(values.into_iter())
}

// CSV output helper

/// Output data as CSV
pub fn output_csv<T: serde::Serialize>(items: &[T]) {
    if items.is_empty() {
        return;
    }

    let mut writer = csv::Writer::from_writer(io::stdout());

    for item in items {
        let _ = writer.serialize(item);
    }

    let _ = writer.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lob_from_vec() {
        let result: Vec<_> = lob(vec![1, 2, 3, 4, 5]).filter(|x| x % 2 == 0).collect();
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn range_basic() {
        let result: Vec<_> = range(0, 5).collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn chained_operations() {
        let result: Vec<_> = lob(vec![1, 2, 3, 4, 5])
            .filter(|x| x % 2 == 0)
            .map(|x| x * 2)
            .take(2)
            .collect();
        assert_eq!(result, vec![4, 8]);
    }

    #[test]
    fn test_parse_csv_from_string() {
        use std::io::Cursor;
        let data = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let cursor = Cursor::new(data);

        let result: Vec<_> = parse_csv_reader(cursor).collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get("name"), Some(&"Alice".to_string()));
        assert_eq!(result[0].get("age"), Some(&"30".to_string()));
        assert_eq!(result[1].get("name"), Some(&"Bob".to_string()));
    }

    #[test]
    fn test_parse_csv_empty() {
        use std::io::Cursor;
        let data = "name,age\n";
        let cursor = Cursor::new(data);

        let result: Vec<_> = parse_csv_reader(cursor).collect();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_tsv_from_string() {
        use std::io::Cursor;
        let data = "name\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA\n";
        let cursor = Cursor::new(data);

        let result: Vec<_> = parse_tsv_reader(cursor).collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get("name"), Some(&"Alice".to_string()));
        assert_eq!(result[1].get("age"), Some(&"25".to_string()));
    }

    #[test]
    fn test_input_from_files_basic() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("test_input1.txt");
        let file2 = temp_dir.join("test_input2.txt");

        fs::write(&file1, "line1\nline2\n").unwrap();
        fs::write(&file2, "line3\nline4\n").unwrap();

        let result: Vec<_> = input_from_files(&[file1.clone(), file2.clone()]).collect();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "line1");
        assert_eq!(result[3], "line4");

        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }

    #[test]
    fn test_input_csv_from_files() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let file = temp_dir.join("test_csv.csv");

        fs::write(&file, "name,value\ntest,42\n").unwrap();

        let result: Vec<_> = input_csv_from_files(std::slice::from_ref(&file)).collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("name"), Some(&"test".to_string()));
        assert_eq!(result[0].get("value"), Some(&"42".to_string()));

        let _ = fs::remove_file(&file);
    }

    #[test]
    fn test_input_tsv_from_files() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let file = temp_dir.join("test_tsv.tsv");

        fs::write(&file, "name\tvalue\ntest\t42\n").unwrap();

        let result: Vec<_> = input_tsv_from_files(std::slice::from_ref(&file)).collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("name"), Some(&"test".to_string()));

        let _ = fs::remove_file(&file);
    }

    #[test]
    fn test_input_json_from_files() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let file = temp_dir.join("test_json.jsonl");

        fs::write(&file, "{\"name\":\"test\",\"value\":42}\n").unwrap();

        let result: Vec<_> = input_json_from_files(std::slice::from_ref(&file)).collect();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("name").and_then(|v| v.as_str()), Some("test"));

        let _ = fs::remove_file(&file);
    }
}
