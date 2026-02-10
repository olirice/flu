//! Integration tests for flu CLI

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_basic_filter() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.filter(|x| x.contains(\"ERROR\"))")
        .write_stdin("INFO: message\nERROR: failed\nWARN: something\nERROR: again\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ERROR: failed\""))
        .stdout(predicate::str::contains("\"ERROR: again\""));
    Ok(())
}

#[test]
fn test_map_to_uppercase() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.map(|x| x.to_uppercase())")
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"HELLO\""))
        .stdout(predicate::str::contains("\"WORLD\""));
    Ok(())
}

#[test]
fn test_sum_terminal() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()")
        .write_stdin("1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
    Ok(())
}

#[test]
fn test_count_terminal() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.count()")
        .write_stdin("line1\nline2\nline3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
    Ok(())
}

#[test]
fn test_take() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.take(2)")
        .write_stdin("1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"1\""))
        .stdout(predicate::str::contains("\"2\""))
        .stdout(predicate::str::contains("\"3\"").not());
    Ok(())
}

#[test]
fn test_chained_operations() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.filter(|x| x.len() > 3).map(|x| x.to_uppercase()).take(2)")
        .write_stdin("hi\nhello\nworld\ntest\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"HELLO\""))
        .stdout(predicate::str::contains("\"WORLD\""));
    Ok(())
}

#[test]
fn test_cache_stats() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("--cache-stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache statistics:"))
        .stdout(predicate::str::contains("Cached binaries:"));
    Ok(())
}

#[test]
fn test_show_source() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("--show-source")
        .arg("_.take(3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("use flu_prelude::*;"))
        .stdout(predicate::str::contains("fn main()"));
    Ok(())
}

#[test]
fn test_unique() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.unique()")
        .write_stdin("a\nb\na\nc\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""))
        .stdout(predicate::str::contains("\"c\""));
    Ok(())
}

#[test]
fn test_enumerate() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("_.enumerate().take(2)")
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("(0,"))
        .stdout(predicate::str::contains("(1,"));
    Ok(())
}

#[test]
fn test_without_stdin() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("flu");
    cmd.arg("flu(vec![1, 2, 3]).map(|x| x * 2).to_list()")
        .assert()
        .success()
        .stdout(predicate::str::contains("[2, 4, 6]"));
    Ok(())
}
