# flu - Embedded Rust Pipeline Tool

A CLI for running Rust data pipeline one-liners with native performance.

## Features

🦀 **Native Rust Performance** - Compiled pipelines run at native speed

⚡ **Smart Caching** - Compiled expressions cached for instant re-execution

🔄 **Lazy Evaluation** - Memory-efficient streaming operations

🎯 **Fluent API** - Chainable operations inspired by Python's flupy

📊 **Rich Operations** - Filter, map, group, join, and 20+ operations

## Quick Example

```bash
# Filter log lines containing ERROR
cat app.log | flu '_.filter(|x| x.contains("ERROR"))'

# Sum numbers from 1 to 100
seq 1 100 | flu '_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()'

# Take first 5 unique items
cat file.txt | flu '_.unique().take(5)'
```

## How It Works

1. **Code Generation**: Your flu expression is transformed into a complete Rust program
2. **Compilation**: The program is compiled using rustc with full optimization
3. **Caching**: Compiled binaries are cached by content hash for instant reuse
4. **Execution**: Cached binaries execute with native performance

```rust
// Expression: _.filter(|x| x.len() > 5).take(3)
// Generates:
use flu_prelude::*;

fn main() {
    let stdin_data = input();
    let result = stdin_data.filter(|x| x.len() > 5).take(3);
    for item in result {
        println!("{:?}", item);
    }
}
```

## Performance

| Run Type | Time | Notes |
|----------|------|-------|
| First run | ~1-2s | Includes compilation |
| Cached runs | <10ms | Instant binary execution |
| Runtime | Native | Zero-cost abstractions |

## Getting Started

See the [Installation Guide](guide/installation.md) to get started, or jump to the [Quick Start](guide/quickstart.md) for examples.
