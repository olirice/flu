//! Outside-in integration tests for the lob CLI.
//!
//! Every test invokes `lob` as a subprocess and asserts on
//! stdin / stdout / stderr / exit-code.  Internals are a black box.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// ── helpers ──────────────────────────────────────────────────────

fn lob() -> Command {
    cargo_bin_cmd!("lob")
}

/// Create a uniquely-named temp file that is cleaned up on drop.
fn temp(ext: &str, content: &str) -> TempFile {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("lob_test_{}_{}.{}", std::process::id(), id, ext));
    fs::write(&path, content).unwrap();
    TempFile(path)
}

struct TempFile(PathBuf);

impl TempFile {
    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

// ── Selection ────────────────────────────────────────────────────

#[test]
fn filter() -> Result<()> {
    lob()
        .arg("_.filter(|x| x.contains(\"ERROR\"))")
        .write_stdin("INFO: ok\nERROR: fail\nWARN: meh\nERROR: again\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ERROR: fail\""))
        .stdout(predicate::str::contains("\"ERROR: again\""))
        .stdout(predicate::str::contains("INFO").not());
    Ok(())
}

#[test]
fn take() -> Result<()> {
    lob()
        .arg("_.take(2)")
        .write_stdin("a\nb\nc\nd\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""))
        .stdout(predicate::str::contains("\"c\"").not());
    Ok(())
}

#[test]
fn skip() -> Result<()> {
    lob()
        .arg("_.skip(2)")
        .write_stdin("a\nb\nc\nd\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"c\""))
        .stdout(predicate::str::contains("\"d\""))
        .stdout(predicate::str::contains("\"a\"").not());
    Ok(())
}

#[test]
fn take_while() -> Result<()> {
    lob()
        .arg("_.take_while(|x| x != \"stop\")")
        .write_stdin("a\nb\nstop\nc\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""))
        .stdout(predicate::str::contains("stop").not());
    Ok(())
}

#[test]
fn drop_while() -> Result<()> {
    lob()
        .arg("_.drop_while(|x| x != \"go\")")
        .write_stdin("a\nb\ngo\nc\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"go\""))
        .stdout(predicate::str::contains("\"c\""));
    Ok(())
}

#[test]
fn unique() -> Result<()> {
    lob()
        .arg("_.unique()")
        .write_stdin("a\nb\na\nc\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""))
        .stdout(predicate::str::contains("\"c\""));
    Ok(())
}

// ── Transformation ───────────────────────────────────────────────

#[test]
fn map() -> Result<()> {
    lob()
        .arg("_.map(|x| x.to_uppercase())")
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"HELLO\""))
        .stdout(predicate::str::contains("\"WORLD\""));
    Ok(())
}

#[test]
fn enumerate() -> Result<()> {
    lob()
        .arg("_.enumerate().take(2)")
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("[0,"))
        .stdout(predicate::str::contains("[1,"));
    Ok(())
}

#[test]
fn flatten() -> Result<()> {
    lob()
        .arg("lob(vec![vec![1,2], vec![3,4]]).flatten().to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[1,2,3,4]"));
    Ok(())
}

#[test]
fn chained_operations() -> Result<()> {
    lob()
        .arg("_.filter(|x| x.len() > 3).map(|x| x.to_uppercase()).take(2)")
        .write_stdin("hi\nhello\nworld\ntest\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"HELLO\""))
        .stdout(predicate::str::contains("\"WORLD\""));
    Ok(())
}

// ── Grouping ─────────────────────────────────────────────────────

#[test]
fn chunk() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4,5]).chunk(2).to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[1,2]"))
        .stdout(predicate::str::contains("[3,4]"))
        .stdout(predicate::str::contains("[5]"));
    Ok(())
}

#[test]
fn window() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4]).window(3).to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[1,2,3]"))
        .stdout(predicate::str::contains("[2,3,4]"));
    Ok(())
}

#[test]
fn group_by() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4,5,6]).group_by(|x| x % 2).count()")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
    Ok(())
}

// ── Joins ────────────────────────────────────────────────────────

#[test]
fn join_inner() -> Result<()> {
    lob()
        .arg("lob(vec![(1,\"a\"),(2,\"b\"),(3,\"c\")]).join_inner(vec![(1,\"x\"),(2,\"y\"),(4,\"z\")], |x| x.0, |x| x.0).count()")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
    Ok(())
}

#[test]
fn join_left() -> Result<()> {
    lob()
        .arg("lob(vec![(1,\"a\"),(2,\"b\"),(3,\"c\")]).join_left(vec![(1,\"x\"),(2,\"y\")], |x| x.0, |x| x.0).count()")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
    Ok(())
}

// ── Terminal operations ──────────────────────────────────────────

#[test]
fn count() -> Result<()> {
    lob()
        .arg("_.count()")
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
    Ok(())
}

#[test]
fn sum() -> Result<()> {
    lob()
        .arg("_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()")
        .write_stdin("1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
    Ok(())
}

#[test]
fn min() -> Result<()> {
    lob()
        .arg("_.map(|x| x.parse::<i32>().unwrap()).min()")
        .write_stdin("3\n1\n4\n1\n5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
    Ok(())
}

#[test]
fn max() -> Result<()> {
    lob()
        .arg("_.map(|x| x.parse::<i32>().unwrap()).max()")
        .write_stdin("3\n1\n4\n1\n5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
    Ok(())
}

#[test]
fn first() -> Result<()> {
    lob()
        .arg("_.first()")
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
    Ok(())
}

#[test]
fn last() -> Result<()> {
    lob()
        .arg("_.last()")
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("world"));
    Ok(())
}

#[test]
fn collect() -> Result<()> {
    lob()
        .arg("_.collect::<Vec<_>>()")
        .write_stdin("a\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("b"));
    Ok(())
}

#[test]
fn to_list() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3]).to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[1,2,3]"));
    Ok(())
}

#[test]
fn reduce() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4,5]).reduce(|a, b| a + b)")
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
    Ok(())
}

#[test]
fn fold() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4,5]).fold(0, |a, b| a + b)")
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
    Ok(())
}

#[test]
fn any() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4,5]).any(|x| x > 3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
    Ok(())
}

#[test]
fn all() -> Result<()> {
    lob()
        .arg("lob(vec![1,2,3,4,5]).all(|x| x > 0)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
    Ok(())
}

// ── Input formats ────────────────────────────────────────────────

#[test]
fn stdin_input() -> Result<()> {
    lob()
        .arg("_.count()")
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
    Ok(())
}

#[test]
fn file_input() -> Result<()> {
    let f = temp("txt", "hello\nworld\ntest\n");
    lob()
        .arg("_.filter(|x| x.len() > 4)")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"hello\""))
        .stdout(predicate::str::contains("\"world\""));
    Ok(())
}

#[test]
fn multi_file_input() -> Result<()> {
    let f1 = temp("txt", "a\nb\nc\n");
    let f2 = temp("txt", "c\nd\ne\n");
    lob()
        .arg("_.unique().count()")
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
    Ok(())
}

#[test]
fn parse_csv_stdin() -> Result<()> {
    lob()
        .arg("--parse-csv")
        .arg("_.count()")
        .write_stdin("name,age\nAlice,30\nBob,25\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
    Ok(())
}

#[test]
fn parse_csv_file() -> Result<()> {
    let f = temp(
        "csv",
        "name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,SF\n",
    );
    lob()
        .arg("--parse-csv")
        .arg("_.filter(|r| r[\"age\"].parse::<i32>().unwrap() > 26)")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Bob").not());
    Ok(())
}

#[test]
fn parse_tsv() -> Result<()> {
    lob()
        .arg("--parse-tsv")
        .arg("_.count()")
        .write_stdin("name\tage\nAlice\t30\nBob\t25\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
    Ok(())
}

#[test]
fn parse_json() -> Result<()> {
    lob()
        .arg("--parse-json")
        .arg("_.count()")
        .write_stdin("{\"a\":1}\n{\"b\":2}\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
    Ok(())
}

// ── Output formats ───────────────────────────────────────────────

#[test]
fn output_default_jsonl_when_piped() -> Result<()> {
    // When piped (not a terminal) default format is jsonl
    lob()
        .arg("_.take(2)")
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""))
        .stdout(predicate::str::contains("\"c\"").not());
    Ok(())
}

#[test]
fn output_json() -> Result<()> {
    let f = temp("txt", "1\n2\n3\n");
    lob()
        .arg("--format")
        .arg("json")
        .arg("_.take(2)")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"))
        .stdout(predicate::str::contains("\"1\""))
        .stdout(predicate::str::contains("\"2\""));
    Ok(())
}

#[test]
fn output_jsonl() -> Result<()> {
    lob()
        .arg("--format")
        .arg("jsonl")
        .arg("_.take(2)")
        .write_stdin("1\n2\n3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"1\""))
        .stdout(predicate::str::contains("\"2\""))
        .stdout(predicate::str::contains("\"3\"").not());
    Ok(())
}

#[test]
fn output_csv() -> Result<()> {
    lob()
        .arg("--format")
        .arg("csv")
        .arg("_.take(2)")
        .write_stdin("hello\nworld\nfoo\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"))
        .stdout(predicate::str::contains("foo").not());
    Ok(())
}

#[test]
fn output_table() -> Result<()> {
    let f = temp("csv", "name,age\nAlice,30\nBob,25\n");
    lob()
        .arg("--parse-csv")
        .arg("--format")
        .arg("table")
        .arg("_.take(2)")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
    Ok(())
}

// ── CLI flags ────────────────────────────────────────────────────

#[test]
fn show_source() -> Result<()> {
    lob()
        .arg("--show-source")
        .arg("_.take(3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("use lob_prelude::*;"))
        .stdout(predicate::str::contains("fn main()"));
    Ok(())
}

#[test]
fn show_source_csv() -> Result<()> {
    lob()
        .arg("--show-source")
        .arg("--parse-csv")
        .arg("_.take(5)")
        .assert()
        .success()
        .stdout(predicate::str::contains("input_csv"))
        .stdout(predicate::str::contains("use lob_prelude::*"));
    Ok(())
}

#[test]
fn cache_stats() -> Result<()> {
    lob()
        .arg("--cache-stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache statistics:"))
        .stdout(predicate::str::contains("Cached binaries:"));
    Ok(())
}

#[test]
fn clear_cache() -> Result<()> {
    lob()
        .arg("--clear-cache")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache cleared"));
    Ok(())
}

#[test]
fn stats_flag() -> Result<()> {
    lob()
        .arg("--stats")
        .arg("lob(vec![1,2,3]).count()")
        .assert()
        .success()
        .stderr(predicate::str::contains("Statistics:"))
        .stderr(predicate::str::contains("Compilation time:"));
    Ok(())
}

#[test]
fn verbose_flag() -> Result<()> {
    lob()
        .arg("-v")
        .arg("lob(vec![1,2,3]).count()")
        .assert()
        .success()
        .stderr(predicate::str::contains("Compiling expression"))
        .stderr(predicate::str::contains("Cache hit:"));
    Ok(())
}

#[test]
fn version_flag() -> Result<()> {
    lob()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("lob"));
    Ok(())
}

// ── Error handling ───────────────────────────────────────────────

#[test]
fn error_syntax() -> Result<()> {
    lob()
        .arg("_.filter(|x|")
        .write_stdin("a\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Compilation Error"));
    Ok(())
}

#[test]
fn error_missing_file() -> Result<()> {
    lob()
        .arg("_.count()")
        .arg("/nonexistent/file/path.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
    Ok(())
}

#[test]
fn error_type_with_suggestion() -> Result<()> {
    lob()
        .arg("_.filter(|x| x > 1)")
        .write_stdin("1\n2\n3\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Problem:"))
        .stderr(predicate::str::contains("How to fix:"))
        .stderr(predicate::str::contains("parse"));
    Ok(())
}

#[test]
fn error_no_expression() -> Result<()> {
    // When piped (not a terminal) and no expression, should error
    lob().write_stdin("data\n").assert().failure();
    Ok(())
}

// ── Caching ──────────────────────────────────────────────────────

#[test]
fn cache_miss_then_hit() -> Result<()> {
    let expr = "lob(vec![42]).to_list()";

    // First run
    lob()
        .arg("-v")
        .arg(expr)
        .assert()
        .success()
        .stdout(predicate::str::contains("[42]"));

    // Second run should hit cache
    lob()
        .arg("-v")
        .arg(expr)
        .assert()
        .success()
        .stderr(predicate::str::contains("Cache hit: true"))
        .stdout(predicate::str::contains("[42]"));
    Ok(())
}

#[test]
fn different_exprs_different_results() -> Result<()> {
    let out1 = lob().arg("lob(vec![1,2,3]).count()").output()?;
    let out2 = lob().arg("lob(vec![1,2,3,4]).count()").output()?;

    let s1 = String::from_utf8(out1.stdout)?;
    let s2 = String::from_utf8(out2.stdout)?;
    assert!(s1.contains('3'));
    assert!(s2.contains('4'));
    Ok(())
}

// ── Expressions without stdin ────────────────────────────────────

#[test]
fn lob_vec_literal() -> Result<()> {
    lob()
        .arg("lob(vec![1, 2, 3]).map(|x| x * 2).to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[2,4,6]"));
    Ok(())
}

#[test]
fn range_expression() -> Result<()> {
    lob()
        .arg("range(1, 4).to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[1,2,3]"));
    Ok(())
}

// ── CSV with different output formats ────────────────────────────

#[test]
fn csv_with_json_output() -> Result<()> {
    let f = temp("csv", "name,age\nAlice,30\nBob,25\n");
    lob()
        .arg("--parse-csv")
        .arg("--format")
        .arg("json")
        .arg("_.take(2)")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
    Ok(())
}
