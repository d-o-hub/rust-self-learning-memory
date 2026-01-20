# Regex Pattern Search Guide

**Status**: ✅ Implemented (Phase 3)  
**Date**: 2026-01-20  
**Feature**: Advanced pattern matching with regex support

## Overview

Regex search enables powerful pattern matching for complex queries. This feature includes built-in ReDoS (Regular Expression Denial of Service) protection to ensure safe operation.

## Key Features

- **Pattern Matching**: Find episodes using regular expressions
- **ReDoS Protection**: Automatic validation prevents dangerous patterns
- **Case Sensitivity**: Control with `(?i)` flag
- **Multi-Field Support**: Search across all episode fields
- **Filter Integration**: Combine regex with existing filters

## Basic Usage

### Enable Regex Search

```rust
use memory_core::{EpisodeFilter, SelfLearningMemory};
use memory_core::search::SearchMode;

let memory = SelfLearningMemory::new();

// Simple regex pattern
let filter = EpisodeFilter::builder()
    .search_text(r"error.*timeout".to_string())
    .search_mode(SearchMode::Regex)
    .build();

let results = memory.list_episodes_filtered(filter, None, None).await?;
```

## Common Patterns

### 1. Find Version Numbers

```rust
// Match semantic versions: 1.2.3, v2.0.1
let filter = EpisodeFilter::builder()
    .search_text(r"v?\d+\.\d+\.\d+".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 2. Find API Endpoints

```rust
// Match /users/123, /posts/456, etc.
let filter = EpisodeFilter::builder()
    .search_text(r"/\w+/\d+".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 3. Find File Paths

```rust
// Match src/file.rs, lib/module.ts, etc.
let filter = EpisodeFilter::builder()
    .search_text(r"src/\w+\.\w+".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 4. Find Error Messages with Timestamps

```rust
// Match "error" followed by time: error at 10:30:45
let filter = EpisodeFilter::builder()
    .search_text(r"error.*\d{2}:\d{2}:\d{2}".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 5. Case-Insensitive Search

```rust
// Match bug, Bug, BUG, error, Error, etc.
let filter = EpisodeFilter::builder()
    .search_text(r"(?i)(bug|error|fix)".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 6. Email Addresses

```rust
// Match email addresses
let filter = EpisodeFilter::builder()
    .search_text(r"\b[\w.-]+@[\w.-]+\.\w+\b".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 7. IP Addresses

```rust
// Match IPv4 addresses
let filter = EpisodeFilter::builder()
    .search_text(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

### 8. HTTP Status Codes

```rust
// Match 4xx and 5xx errors
let filter = EpisodeFilter::builder()
    .search_text(r"\b[45]\d{2}\b".to_string())
    .search_mode(SearchMode::Regex)
    .build();
```

## Combining with Filters

Regex search works with all existing filters:

```rust
// Regex + domain filter
let filter = EpisodeFilter::builder()
    .search_text(r"API|api".to_string())
    .search_mode(SearchMode::Regex)
    .domains(vec!["backend".to_string()])
    .build();

// Regex + date range + success filter
let filter = EpisodeFilter::builder()
    .search_text(r"deploy.*production".to_string())
    .search_mode(SearchMode::Regex)
    .date_from(Utc::now() - Duration::days(7))
    .success_only(true)
    .build();
```

## ReDoS Protection

The system automatically validates regex patterns to prevent ReDoS attacks:

### Blocked Patterns

```rust
use memory_core::search::validate_regex_pattern;

// Nested quantifiers (catastrophic backtracking)
assert!(validate_regex_pattern("(a+)+").is_err());
assert!(validate_regex_pattern("(a*)*").is_err());
assert!(validate_regex_pattern("(a+)*").is_err());

// Excessive repetitions
assert!(validate_regex_pattern("a{1000}").is_err());

// Too long patterns (>1000 chars)
assert!(validate_regex_pattern(&"a".repeat(1001)).is_err());
```

### Safe Patterns

```rust
// These patterns are safe and allowed
assert!(validate_regex_pattern("error.*timeout").is_ok());
assert!(validate_regex_pattern(r"\d+\.\d+\.\d+").is_ok());
assert!(validate_regex_pattern("(?i)bug|fix").is_ok());
```

## CLI Usage

### Basic Regex Search

```bash
# Find API endpoints
memory-cli episode search "/users/\d+" --regex

# Find version numbers
memory-cli episode search "v?\d+\.\d+\.\d+" --regex

# Case-insensitive OR
memory-cli episode search "(?i)(bug|error|fix)" --regex
```

### Combine with Other Options

```bash
# Regex + limit
memory-cli episode search "error.*timeout" --regex --limit 20

# Regex + domain filter
memory-cli episode search "API|api" --regex --search-fields description,outcome

# Regex with field selection
memory-cli episode search "database.*connection" --regex --search-fields steps,outcome
```

## Performance Considerations

### Pattern Complexity

| Pattern Type | Performance | Recommendation |
|--------------|-------------|----------------|
| Simple string | Fast | Use when possible |
| Anchored (^$) | Fast | Preferred for exact matches |
| Character classes | Fast | Good for validation |
| Alternation (a\|b) | Moderate | Limit alternatives |
| Wildcards (.*) | Moderate | Avoid at start |
| Lookahead/behind | Slow | Use sparingly |

### Optimization Tips

1. **Anchor patterns** when possible: `^error` vs `error`
2. **Avoid leading wildcards**: `error.*` vs `.*error`
3. **Use specific patterns**: `\d{3}` vs `\d+`
4. **Limit alternations**: `bug|fix` vs `(bug|fix|error|warning|...)`

## Regex Syntax Reference

### Basic Patterns

| Pattern | Description | Example |
|---------|-------------|---------|
| `.` | Any character | `a.b` matches "aab", "acb" |
| `\d` | Digit (0-9) | `\d+` matches "123" |
| `\w` | Word char (a-z, A-Z, 0-9, _) | `\w+` matches "test_123" |
| `\s` | Whitespace | `\s+` matches "   " |
| `^` | Start of line | `^error` matches "error: ..." |
| `$` | End of line | `end$` matches "... end" |

### Quantifiers

| Pattern | Description | Example |
|---------|-------------|---------|
| `*` | 0 or more | `a*` matches "", "a", "aa" |
| `+` | 1 or more | `a+` matches "a", "aa" |
| `?` | 0 or 1 | `a?` matches "", "a" |
| `{n}` | Exactly n | `\d{3}` matches "123" |
| `{n,}` | n or more | `\d{3,}` matches "123", "1234" |
| `{n,m}` | Between n and m | `\d{3,5}` matches "123" to "12345" |

### Character Classes

| Pattern | Description | Example |
|---------|-------------|---------|
| `[abc]` | Any of a, b, c | `[aeiou]` matches vowels |
| `[^abc]` | Not a, b, c | `[^0-9]` matches non-digits |
| `[a-z]` | Range a to z | `[A-Z]` matches uppercase |
| `\|` | Alternation (OR) | `cat\|dog` matches "cat" or "dog" |

### Groups

| Pattern | Description | Example |
|---------|-------------|---------|
| `(...)` | Capture group | `(\d+)-(\d+)` captures "123-456" |
| `(?:...)` | Non-capture group | `(?:http\|https)://` |
| `(?i)` | Case-insensitive | `(?i)error` matches "ERROR" |

## Common Use Cases

### Software Development

```rust
// Find commits
search_text: r"commit\s+[a-f0-9]{7,40}"

// Find function calls
search_text: r"\w+\([^)]*\)"

// Find TODO comments
search_text: r"(?i)TODO:.*"

// Find import statements
search_text: r"(?:import|from|require)\s+[\w.]+"
```

### DevOps & Infrastructure

```rust
// Find deployment logs
search_text: r"deploy.*(?:staging|production)"

// Find error codes
search_text: r"\b[45]\d{2}\b"

// Find resource names
search_text: r"(?i)(cpu|memory|disk).*\d+%"

// Find service names
search_text: r"service-\w+-\d+"
```

### Data Analysis

```rust
// Find JSON keys
search_text: r#""[^"]+"\s*:"#

// Find URLs
search_text: r"https?://[\w.-]+/[\w/.-]*"

// Find timestamps
search_text: r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}"

// Find durations
search_text: r"\d+(?:ms|s|m|h)"
```

## API Reference

### Pattern Validation

```rust
pub fn validate_regex_pattern(pattern: &str) -> Result<(), String>
```

Validates a regex pattern before use. Returns `Ok(())` if safe, `Err` with description if not.

### Search Functions

```rust
pub fn regex_search(text: &str, pattern: &str) -> Result<Vec<(usize, String)>, String>
```

Search for regex pattern in text. Returns vector of `(position, matched_text)`.

```rust
pub fn regex_search_case_insensitive(text: &str, pattern: &str) -> Result<Vec<(usize, String)>, String>
```

Case-insensitive regex search.

```rust
pub fn regex_matches(text: &str, pattern: &str) -> Result<bool, String>
```

Check if text matches pattern (returns boolean).

## Examples

See the complete working example:

```bash
cargo run --example regex_search_demo
```

## Testing

Run regex search tests:

```bash
# Unit tests
cargo test search::regex

# Integration tests
cargo test regex_search
```

## Migration from Other Search Modes

### From Exact Search

```rust
// Before (exact)
.search_text("error timeout")
.search_mode(SearchMode::Exact)

// After (regex for flexibility)
.search_text(r"error.*timeout")
.search_mode(SearchMode::Regex)
```

### From Fuzzy Search

```rust
// Before (fuzzy for typos)
.search_text("databse")
.search_mode(SearchMode::Fuzzy { threshold: 0.8 })

// After (regex for patterns)
.search_text(r"data.{0,2}base")  // Match database, databse, databae
.search_mode(SearchMode::Regex)
```

## Troubleshooting

### Pattern Not Matching

1. **Test pattern separately**: Use `validate_regex_pattern()` first
2. **Check case sensitivity**: Add `(?i)` flag if needed
3. **Escape special chars**: Use `\` before `.`, `*`, `+`, etc.
4. **Check field selection**: Ensure searching correct fields

### Performance Issues

1. **Avoid leading wildcards**: `.*error` is slow
2. **Use anchors**: `^error` is faster than `error`
3. **Limit alternations**: Too many `|` operators slow down
4. **Check pattern complexity**: Simplify if possible

### ReDoS Protection Triggered

If your pattern is blocked:
1. **Simplify nested groups**: Avoid `(a+)+` patterns
2. **Reduce repetitions**: Keep `{n}` values reasonable
3. **Use character classes**: `[abc]` instead of `(a|b|c)`

## Security Best Practices

1. **Always validate** user-provided patterns
2. **Set timeouts** for regex execution in production
3. **Log blocked patterns** for security monitoring
4. **Educate users** about safe pattern construction
5. **Review patterns** before deploying to production

## See Also

- [Fuzzy Search Guide](FUZZY_SEARCH_GUIDE.md) - Typo-tolerant search
- [Episode Filtering Guide](EPISODE_FILTERING.md) - Filter API reference
- [Pattern Search Feature](PATTERN_SEARCH_FEATURE.md) - Pattern extraction
