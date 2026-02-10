# Quick Start

This guide will get you up and running with flu in minutes.

## Basic Concepts

flu expressions start with `_`, which represents the input stream (stdin). You chain operations using the fluent API, and the result is printed to stdout.

### Your First Pipeline

```bash
# Read lines from stdin, keep those containing "ERROR"
echo -e "INFO: ok\nERROR: failed\nWARN: check" | flu '_.filter(|x| x.contains("ERROR"))'
# Output: "ERROR: failed"
```

### Common Patterns

#### Selection

Filter, take, skip, and more:

```bash
# Take first 5 lines
seq 1 100 | flu '_.take(5)'

# Skip first 10 lines
cat file.txt | flu '_.skip(10)'

# Filter by condition
cat file.txt | flu '_.filter(|x| x.len() > 10)'

# Keep unique lines
cat file.txt | flu '_.unique()'
```

#### Transformation

Map, enumerate, and transform:

```bash
# Convert to uppercase
cat file.txt | flu '_.map(|x| x.to_uppercase())'

# Add line numbers
cat file.txt | flu '_.enumerate()'

# Parse and double numbers
seq 1 5 | flu '_.map(|x| x.parse::<i32>().unwrap() * 2)'
```

#### Terminal Operations

Operations that produce a final result:

```bash
# Count lines
cat file.txt | flu '_.count()'

# Sum numbers
seq 1 100 | flu '_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()'

# Find max
seq 1 100 | flu '_.map(|x| x.parse::<i32>().unwrap()).max()'
```

## Chaining Operations

The power of flu comes from chaining operations:

```bash
# Complex pipeline
cat file.txt | flu '_.filter(|x| x.len() > 5).unique().take(10).map(|x| x.to_uppercase())'
```

This pipeline:
1. Filters lines longer than 5 characters
2. Keeps only unique lines
3. Takes the first 10
4. Converts to uppercase

## Working with Data

### Numbers

```bash
# Sum 1 to 100
seq 1 100 | flu '_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()'
# Output: 5050

# Average (manual)
seq 1 10 | flu '_.map(|x| x.parse::<i32>().unwrap()).fold((0, 0), |(sum, count), x| (sum + x, count + 1))' | awk '{print $1/$2}'
```

### Strings

```bash
# Count lines starting with "ERROR"
cat app.log | flu '_.filter(|x| x.starts_with("ERROR")).count()'

# Extract first word
cat file.txt | flu '_.map(|x| x.split_whitespace().next().unwrap_or(""))'
```

### Grouping

```bash
# Group into chunks of 3
seq 1 10 | flu '_.chunk(3)'
# Output: [1, 2, 3], [4, 5, 6], [7, 8, 9], [10]

# Sliding window
seq 1 5 | flu '_.window(2)'
# Output: [1, 2], [2, 3], [3, 4], [4, 5]
```

## CLI Features

### Show Generated Code

See what Rust code flu generates:

```bash
flu --show-source '_.take(3)'
```

### Cache Management

```bash
# View cache stats
flu --cache-stats

# Clear cache
flu --clear-cache
```

### Verbose Mode

```bash
# See compilation and execution details
flu -v '_.take(3)'
```

## Tips & Tricks

### Parsing

flu reads lines as strings. To work with numbers:

```bash
# Parse to i32
seq 1 5 | flu '_.map(|x| x.parse::<i32>().unwrap())'

# Parse to f64
echo -e "1.5\n2.3\n3.7" | flu '_.map(|x| x.parse::<f64>().unwrap()).sum::<f64>()'
```

### Error Handling

Use `unwrap()` for simplicity, or `unwrap_or()` for safety:

```bash
# Safe parsing with default
cat mixed.txt | flu '_.map(|x| x.parse::<i32>().unwrap_or(0))'
```

### Performance

- First run compiles (1-2s)
- Repeated runs are instant (<10ms)
- Clear cache if you're testing: `flu --clear-cache`

## Next Steps

- [CLI Usage](cli.md) - Explore all command-line options
- [API Reference](../api/overview.md) - Learn all available operations
- [Examples](../examples/logs.md) - See real-world use cases
