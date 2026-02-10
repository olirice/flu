# flu

**Embedded Rust Pipeline Tool** - A CLI for running Rust data pipeline one-liners with native performance.

[![CI](https://github.com/your-org/flu/workflows/CI/badge.svg)](https://github.com/your-org/flu/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Features

- 🦀 **Native Rust Performance** - Compiled pipelines run at native speed
- ⚡ **Smart Caching** - Compiled expressions cached for instant re-execution
- 🔄 **Lazy Evaluation** - Memory-efficient streaming operations
- 🎯 **Fluent API** - Chainable operations inspired by Python's flupy
- 📊 **Rich Operations** - Filter, map, group, join, and 20+ operations

## Quick Start

### Installation

**Prerequisites:** Rust toolchain (for now - embedded toolchain coming soon)

```bash
cargo install --path crates/flu-cli
```

Or build from source:

```bash
cargo build --release
cp target/release/flu ~/.local/bin/  # or /usr/local/bin/
```

### Basic Examples

```bash
# Filter log lines containing ERROR
cat app.log | flu '_.filter(|x| x.contains("ERROR"))'

# Take first 10 lines, map to uppercase
seq 1 100 | flu '_.take(10).map(|x| x.to_uppercase())'

# Sum numbers
seq 1 100 | flu '_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()'

# Count lines matching a pattern
cat file.txt | flu '_.filter(|x| x.starts_with("ERROR")).count()'

# Keep only unique lines
cat file.txt | flu '_.unique()'

# Get first 5 unique items
cat file.txt | flu '_.unique().take(5)'
```

### Advanced Examples

```bash
# Group by length, count each group
cat words.txt | flu '_.group_by(|w| w.len()).map(|(k,v)| (k, v.len()))'

# Chunk into groups of 3
seq 1 10 | flu '_.chunk(3)'

# Sliding window of size 2
seq 1 5 | flu '_.window(2)'

# Enumerate with indices
cat file.txt | flu '_.enumerate().take(5)'
```

## Available Operations

### Selection
- `filter(predicate)` - Keep elements matching predicate
- `take(n)` - Take first n elements
- `skip(n)` - Skip first n elements
- `take_while(predicate)` - Take while predicate is true
- `drop_while(predicate)` - Drop while predicate is true
- `unique()` - Keep only unique elements

### Transformation
- `map(fn)` - Transform each element
- `enumerate()` - Add indices (0, item), (1, item), ...
- `zip(other)` - Zip with another iterator
- `flatten()` - Flatten nested iterators

### Grouping
- `chunk(n)` - Group into chunks of size n
- `window(n)` - Sliding windows of size n
- `group_by(key_fn)` - Group by key function

### Joins
- `join_inner(other, left_key, right_key)` - Inner join
- `join_left(other, left_key, right_key)` - Left join

### Terminal (consume iterator)
- `collect()` - Collect into collection
- `to_list()` - Collect into Vec
- `count()` - Count elements
- `sum()` - Sum elements
- `min()` / `max()` - Find min/max
- `first()` / `last()` - Get first/last
- `reduce(fn)` - Reduce to single value
- `fold(init, fn)` - Fold with initial value
- `any(predicate)` / `all(predicate)` - Check conditions

## CLI Usage

```bash
# Execute expression
flu 'EXPRESSION'

# Show generated source without executing
flu --show-source 'EXPRESSION'

# View cache statistics
flu --cache-stats

# Clear cache
flu --clear-cache

# Verbose output
flu -v 'EXPRESSION'
```

## How It Works

1. **Code Generation**: Flu expressions are transformed into Rust programs
2. **Compilation**: Programs are compiled using rustc (with optimization)
3. **Caching**: Compiled binaries are cached by content hash
4. **Execution**: Cached binaries run instantly on repeated expressions

### Example Generated Code

Expression: `_.filter(|x| x.len() > 5).take(3)`

```rust
use flu_prelude::*;

fn main() {
    let stdin_data = input();
    let result = stdin_data.filter(|x| x.len() > 5).take(3);
    for item in result {
        println!("{:?}", item);
    }
}
```

## Architecture

```
flu/
├── crates/
│   ├── flu-cli/      # CLI with code generation & caching
│   ├── flu-core/     # Core iterator operations
│   └── flu-prelude/  # User-facing API
└── tests/            # Integration tests
```

## Performance

- **First run**: ~1-2s (compilation overhead)
- **Cached runs**: <10ms (instant binary execution)
- **Runtime**: Native Rust performance (zero-cost abstractions)

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run all tests including integration
cargo test --all

# Run specific integration test
cargo test -p flu --test integration_test

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-targets --all-features
```

## Roadmap

- [x] Core fluent API with lazy evaluation
- [x] Code generation and caching system
- [x] Basic operations (filter, map, take, etc.)
- [x] Advanced operations (group_by, joins, window)
- [x] Integration tests
- [ ] Embedded Rust toolchain (true zero dependencies)
- [ ] Property-based tests
- [ ] 99%+ test coverage
- [ ] Pre-commit hooks
- [ ] CI/CD with coverage enforcement
- [ ] Comprehensive documentation (MkDocs)
- [ ] Performance benchmarks

## Inspiration

Inspired by [flupy](https://github.com/olirice/flupy) - bringing Python's fluent iteration API to Rust with native performance.

## Contributing

Contributions welcome! Please ensure:
- Tests pass: `cargo test --all`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy --all-targets`

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
