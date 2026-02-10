# Log Processing Examples

Real-world examples of using flu for log analysis and processing.

## Basic Log Filtering

### Find Errors

```bash
# Find all ERROR lines
tail -f app.log | flu '_.filter(|x| x.contains("ERROR"))'

# Find ERROR or WARN
tail -f app.log | flu '_.filter(|x| x.contains("ERROR") || x.contains("WARN"))'

# Case-insensitive search
tail -f app.log | flu '_.filter(|x| x.to_lowercase().contains("error"))'
```

### Count by Log Level

```bash
# Count errors
cat app.log | flu '_.filter(|x| x.contains("ERROR")).count()'

# Count all levels
for level in DEBUG INFO WARN ERROR; do
  count=$(cat app.log | flu "_.filter(|x| x.contains(\"$level\")).count()")
  echo "$level: $count"
done
```

## Time-Based Filtering

### Recent Logs

```bash
# Last 100 lines
tail -100 app.log | flu '_.to_list()'

# First 100 lines
head -100 app.log | flu '_.to_list()'
```

### Time Window

```bash
# Logs containing specific timestamp pattern
cat app.log | flu '_.filter(|x| x.contains("2024-01-15"))'
```

## Pattern Extraction

### Extract IP Addresses

```bash
# Lines containing IP patterns (simplified)
cat access.log | flu '_.filter(|x| x.contains(".")):. && x.split_whitespace().any(|w| w.matches(".").count() == 3))'
```

### Extract HTTP Status Codes

```bash
# Assuming format: "... HTTP/1.1 200 ..."
cat access.log | flu '_.map(|x| {
  x.split_whitespace()
    .skip_while(|w| !w.contains("HTTP"))
    .nth(1)
    .unwrap_or("000")
    .to_string()
})'
```

## Log Analysis

### Top Errors

```bash
# Group identical error messages and count
cat app.log | flu '_.filter(|x| x.contains("ERROR"))
  .group_by(|x| x.clone())
  .map(|(msg, occurrences)| (msg, occurrences.len()))
  .to_list()' | sort -t, -k2 -nr | head -10
```

### Error Rate

```bash
# Calculate error percentage
total=$(cat app.log | flu '_.count()')
errors=$(cat app.log | flu '_.filter(|x| x.contains("ERROR")).count()')
echo "scale=2; $errors * 100 / $total" | bc
```

## Real-Time Monitoring

### Live Error Stream

```bash
# Follow log file and show only errors
tail -f app.log | flu '_.filter(|x| x.contains("ERROR"))'
```

### Alert on Pattern

```bash
# Alert when specific error appears
tail -f app.log | flu '_.filter(|x| x.contains("OutOfMemoryError"))' | while read line; do
  echo "ALERT: $line"
  # Send notification, email, etc.
done
```

### Rate Limiting

```bash
# Show max 10 errors per second
tail -f app.log | flu '_.filter(|x| x.contains("ERROR")).take(10)'
```

## Log Transformation

### Anonymize Sensitive Data

```bash
# Replace email addresses (simplified)
cat app.log | flu '_.map(|x| {
  if x.contains("@") {
    x.split("@").next().unwrap_or("") + "@REDACTED"
  } else {
    x
  }
})'
```

### Extract JSON Fields

```bash
# For JSON logs, extract specific field (requires jq)
cat app.log | flu '_.filter(|x| x.contains("{"))' | jq -r '.level, .message'
```

### Convert Format

```bash
# Convert to CSV (timestamp, level, message)
cat app.log | flu '_.map(|x| {
  let parts: Vec<_> = x.split_whitespace().collect();
  format!("{},{},{}", parts[0], parts[1], parts[2..].join(" "))
})'
```

## Debugging

### Context Around Errors

```bash
# Show 2 lines before and after each error (using grep)
grep -B 2 -A 2 "ERROR" app.log

# Or use flu for just errors and save, then investigate
cat app.log | flu '_.filter(|x| x.contains("ERROR"))' > errors.txt
```

### Deduplication

```bash
# Show unique error messages only
cat app.log | flu '_.filter(|x| x.contains("ERROR")).unique()'
```

### Sample Logs

```bash
# Take every 10th line (sampling)
cat app.log | flu '_.enumerate().filter(|(i, _)| i % 10 == 0).map(|(_, x)| x)'
```

## Performance Tips

1. **Use `grep` for simple patterns** - It's faster
2. **Combine flu with standard tools** - Chain efficiently
3. **Filter early** - Reduce data as soon as possible
4. **Sample large files** - Use `take` or `skip` for testing

## Complete Example: Log Analysis Pipeline

```bash
#!/bin/bash
# Analyze app.log for errors

echo "=== Log Analysis Report ==="
echo ""

# Total lines
total=$(cat app.log | flu '_.count()')
echo "Total log entries: $total"

# Error count
errors=$(cat app.log | flu '_.filter(|x| x.contains("ERROR")).count()')
echo "Errors: $errors"

# Error rate
if [ $total -gt 0 ]; then
  rate=$(echo "scale=2; $errors * 100 / $total" | bc)
  echo "Error rate: ${rate}%"
fi

# Top 5 unique errors
echo ""
echo "Top 5 unique errors:"
cat app.log | flu '_.filter(|x| x.contains("ERROR")).unique().take(5)' | nl

# Recent errors (last 10)
echo ""
echo "Most recent errors:"
cat app.log | flu '_.filter(|x| x.contains("ERROR"))' | tail -10
```
