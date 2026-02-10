# API Overview

flu provides a fluent API for data pipeline operations, inspired by Python's flupy with Rust's type safety and performance.

## Design Philosophy

### Lazy Evaluation

All non-terminal operations are lazy - they don't execute until a terminal operation is called:

```bash
# These operations build a pipeline but don't execute yet
_.filter(|x| x.len() > 5)
  .map(|x| x.to_uppercase())
  .take(10)
  .count()  # Terminal operation - pipeline executes here
```

### Type Safety

flu leverages Rust's type system for safety:

```rust
// Type checking at compile time
_.map(|x| x.parse::<i32>().unwrap())  // String -> i32
  .sum::<i32>()                        // Sum of i32 values
```

### Zero-Cost Abstractions

flu uses Rust's iterator protocol, providing native performance with no overhead.

## Operation Categories

### Selection Operations

Filter and slice the data stream:

- [`filter`](selection.md#filter) - Keep elements matching a predicate
- [`take`](selection.md#take) - Take first n elements
- [`skip`](selection.md#skip) - Skip first n elements
- [`take_while`](selection.md#take-while) - Take while condition is true
- [`drop_while`](selection.md#drop-while) - Drop while condition is true
- [`unique`](selection.md#unique) - Keep only unique elements

[Learn more →](selection.md)

### Transformation Operations

Transform elements one-by-one:

- [`map`](transformation.md#map) - Transform each element
- [`enumerate`](transformation.md#enumerate) - Add indices
- [`zip`](transformation.md#zip) - Combine with another iterator
- [`flatten`](transformation.md#flatten) - Flatten nested structures

[Learn more →](transformation.md)

### Grouping Operations

Group elements into chunks:

- [`chunk`](grouping.md#chunk) - Fixed-size chunks
- [`window`](grouping.md#window) - Sliding windows
- [`group_by`](grouping.md#group-by) - Group by key function

[Learn more →](grouping.md)

### Join Operations

Combine two data streams:

- [`join_inner`](joins.md#inner-join) - Inner join
- [`join_left`](joins.md#left-join) - Left join

[Learn more →](joins.md)

### Terminal Operations

Consume the iterator and produce a result:

- [`collect`](terminal.md#collect) - Collect into a collection
- [`to_list`](terminal.md#to-list) - Collect into a Vec
- [`count`](terminal.md#count) - Count elements
- [`sum`](terminal.md#sum) - Sum elements
- [`min`](terminal.md#min) / [`max`](terminal.md#max) - Find extrema
- [`first`](terminal.md#first) / [`last`](terminal.md#last) - Get boundary elements
- [`reduce`](terminal.md#reduce) - Reduce to single value
- [`fold`](terminal.md#fold) - Fold with initial value
- [`any`](terminal.md#any) / [`all`](terminal.md#all) - Check conditions

[Learn more →](terminal.md)

## Input Sources

### stdin (default)

Use `_` to read from stdin:

```bash
cat file.txt | flu '_.take(10)'
```

### In-memory Data

Use `flu()` helper:

```bash
flu 'flu(vec![1, 2, 3]).sum::<i32>()'
```

### Ranges

Use `range()` helper:

```bash
flu 'range(0, 100).filter(|x| x % 2 == 0).count()'
```

## Chaining Operations

Operations can be chained indefinitely:

```bash
_.filter(|x| x.len() > 5)      # Selection
  .map(|x| x.to_uppercase())   # Transformation
  .unique()                    # Selection
  .take(10)                    # Selection
  .enumerate()                 # Transformation
  .collect()                   # Terminal
```

## Type Conversions

### Parsing Strings

```bash
# Parse to integers
_.map(|x| x.parse::<i32>().unwrap())

# Parse to floats
_.map(|x| x.parse::<f64>().unwrap())

# Safe parsing
_.map(|x| x.parse::<i32>().unwrap_or(0))
```

### String Operations

```bash
# Case conversion
_.map(|x| x.to_uppercase())
_.map(|x| x.to_lowercase())

# Trimming
_.map(|x| x.trim().to_string())

# Splitting
_.map(|x| x.split(',').collect::<Vec<_>>())
```

## Common Patterns

### Count Matching Lines

```bash
_.filter(|x| x.contains("ERROR")).count()
```

### Top N Items

```bash
_.take(10).to_list()
```

### Sum Numbers

```bash
_.map(|x| x.parse::<i32>().unwrap()).sum::<i32>()
```

### Group and Count

```bash
_.group_by(|x| x.chars().next().unwrap())
  .map(|(k, v)| (k, v.len()))
```

### Deduplicate

```bash
_.unique().to_list()
```

## Performance Tips

1. **Use lazy operations** - They don't allocate until needed
2. **Minimize parsing** - Parse once, reuse
3. **Use `take` early** - Limits work done
4. **Cache binaries** - Reuse compiled code automatically

## Next Steps

Explore detailed documentation for each operation category:

- [Selection Operations](selection.md)
- [Transformation Operations](transformation.md)
- [Grouping Operations](grouping.md)
- [Join Operations](joins.md)
- [Terminal Operations](terminal.md)
