# lob

**Rust Pipeline Tool** - Run Rust data pipeline one-liners with native performance.

[![CI](https://github.com/olirice/lob/workflows/CI/badge.svg)](https://github.com/olirice/lob/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **Inspired by [flupy](https://github.com/olirice/flupy)** - Python's fluent pipeline library

## Features

- **Native Rust Performance** - Compiled pipelines run at native speed
- **Smart Caching** - Compiled expressions cached for instant re-execution
- **Lazy Evaluation** - Memory-efficient streaming operations
- **Fluent API** - Chainable operations for readable data transformations
- **Rich Operations** - Filter, map, group, join, and 20+ operations
- **Embedded Toolchain** - Self-contained binary with no system dependencies

## Quick Start

### Installation

```bash
cargo install --path crates/lob-cli
```

Or build from source:

```bash
cargo build --release
cp target/release/lob ~/.local/bin/  # or /usr/local/bin/
```

### Basic Usage

```bash
# From stdin
seq 1 100 | lob '_.filter(|x| x.parse::<i32>().unwrap() % 2 == 0).count()'
# Output: 50

# From file
lob data.txt '_.filter(|x| x.len() > 5).take(10)'
# Output: First 10 lines longer than 5 characters

# Multiple files
lob file1.txt file2.txt '_.unique().count()'
# Output: Number of unique lines across all files

# Parse CSV
lob users.csv --parse-csv '_.filter(|r| r["age"].parse::<i32>().unwrap() > 18)'
# Output: CSV rows where age > 18
```

## Examples

### Log Processing

```bash
# Extract error messages from file
lob app.log '_.filter(|x| x.contains("ERROR")).take(10)'
# Output: First 10 ERROR lines

# Count errors by type
cat app.log | lob '
  _.filter(|x| x.contains("ERROR"))
   .map(|x| x.split(":").nth(1).unwrap())
   .group_by(|x| x.clone())
   .map(|(k, v)| format!("{}: {}", k, v.len()))
'
```

### CSV Processing

```bash
# Filter CSV by column value
lob users.csv --parse-csv '_.filter(|r| r["status"] == "active")'
# Output: All active users

# Convert CSV to JSON
lob data.csv --parse-csv '_.take(100)' --format json > output.json

# Group by column and count
lob sales.csv --parse-csv '
  _.group_by(|r| r["category"].clone())
   .map(|(k, v)| format!("{}: {} items", k, v.len()))
'
# Output:
# "Electronics: 45 items"
# "Clothing: 32 items"
```

### Data Transformation

```bash
# Unique lines
cat data.txt | lob '_.unique()'

# Chunk into groups of 5
seq 1 20 | lob '_.chunk(5).map(|chunk| chunk.len()).sum::<usize>()'
# Output: 20

# Window analysis (sliding window of 3)
seq 1 10 | lob '_.window(3).take(3)'
# Output:
# ["1", "2", "3"]
# ["2", "3", "4"]
# ["3", "4", "5"]
```

## How It Works

1. **Generate** - Your expression is converted to a complete Rust program
2. **Compile** - The program is compiled with full optimizations (`-C opt-level=3`)
3. **Cache** - Compiled binary is cached (SHA256-based) for instant reuse
4. **Execute** - Native binary processes your data at full speed

```
Input Data -> lob Expression -> Generated Rust Code -> Compiled Binary (cached) -> Output
```

### Performance

```bash
# First run: ~1-2s (compilation)
seq 1 1000000 | lob '_.filter(|x| x.parse::<i32>().unwrap() % 2 == 0).count()'
# Output: 500000

# Subsequent runs: <10ms (cached)
seq 1 1000000 | lob '_.filter(|x| x.parse::<i32>().unwrap() % 2 == 0).count()'
# Output: 500000 (instant)
```

## Operations

### Selection
- `filter(predicate)` - Keep items matching condition
- `take(n)` - Take first n items
- `skip(n)` - Skip first n items
- `take_while(predicate)` - Take while condition holds
- `drop_while(predicate)` - Skip while condition holds
- `unique()` - Remove duplicates

### Transformation
- `map(f)` - Transform each item
- `enumerate()` - Add indices
- `zip(other)` - Pair with another iterator
- `flatten()` - Flatten nested iterators

### Grouping
- `chunk(n)` - Group into chunks of size n
- `window(n)` - Sliding window of size n
- `group_by(key_fn)` - Group by key function

### Joins
- `join_inner(other, left_key, right_key)` - Inner join
- `join_left(other, left_key, right_key)` - Left join

### Terminal
- `collect()` / `to_list()` - Collect to vector
- `count()` - Count items
- `sum()` - Sum items
- `min()` / `max()` - Find extrema
- `first()` / `last()` - Get first/last
- `reduce(f)` - Reduce with function
- `fold(init, f)` - Fold with initial value

## Input Formats

```bash
# CSV with headers (each row becomes HashMap<String, String>)
lob data.csv --parse-csv '_.filter(|r| r["age"].parse::<i32>().unwrap() > 18)'

# TSV (tab-separated)
lob data.tsv --parse-tsv '_.filter(|r| r["status"] == "active")'

# JSON Lines (newline-delimited JSON)
lob logs.jsonl --parse-json '_.filter(|obj| obj["level"] == "ERROR")'
```

## Output Formats

```bash
# JSON array
lob data.csv --parse-csv '_.take(5)' --format json

# JSON Lines (one object per line, great for piping to jq)
lob data.csv --parse-csv '_.filter(...)' --format jsonl | jq '.name'

# CSV
lob data.csv --parse-csv '_.filter(|r| r["age"].parse::<i32>().unwrap() > 25)' --format csv

# Formatted table
lob users.csv --parse-csv '_.take(5)' --format table
```

## CLI Reference

```bash
lob [OPTIONS] <EXPRESSION> [FILE...]

Options:
  --parse-csv         Parse input as CSV with headers
  --parse-tsv         Parse input as TSV with headers
  --parse-json        Parse input as JSON lines
  -f, --format FMT    Output format: debug, json, jsonl, csv, table
  -s, --show-source   Show generated source code without executing
  --stats             Show performance statistics after execution
  --clear-cache       Clear the compilation cache
  --cache-stats       Show cache statistics
  -v, --verbose       Verbose output
  -h, --help          Print help
  -V, --version       Print version
```

## Development

```bash
# Build
cargo build --release

# Test
cargo test --all

# Set up pre-commit hooks
git config core.hooksPath .githooks
```

## Inspiration

lob is inspired by [flupy](https://github.com/olirice/flupy), bringing Python's fluent pipeline style to Rust with native performance.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
