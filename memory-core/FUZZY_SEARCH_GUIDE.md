# Fuzzy Search Guide

**Status**: ✅ Implemented (v0.1.13)  
**Date**: 2026-01-20  
**Feature**: Typo-tolerant search for episode retrieval

## Overview

Fuzzy search enables finding episodes even when search queries contain typos or spelling variations. This is implemented using the Levenshtein distance algorithm to calculate similarity scores between the query and episode content.

## Key Features

- **Typo Tolerance**: Find "database" even when searching for "databse"
- **Configurable Threshold**: Control how strict matching should be (0.0 to 1.0)
- **Multi-Field Search**: Search across descriptions, steps, outcomes, tags, and domains
- **Combines with Filters**: Use fuzzy search with existing filters (domains, dates, task types, etc.)
- **Fast Performance**: Optimized for searching thousands of episodes

## Basic Usage

### Enable Fuzzy Search

```rust
use memory_core::{EpisodeFilter, SelfLearningMemory};
use memory_core::search::SearchMode;

let memory = SelfLearningMemory::new();

// Simple fuzzy search with default threshold (0.8)
let filter = EpisodeFilter::builder()
    .search_text("databse".to_string())  // Typo: missing 'a'
    .fuzzy_search()  // Enables fuzzy matching
    .build();

let results = memory.list_episodes_filtered(filter, None, None).await?;
// Will find episodes containing "database"
```

### Custom Threshold

```rust
// Strict matching (0.9) - only very close matches
let filter = EpisodeFilter::builder()
    .search_text("async".to_string())
    .fuzzy_search_with_threshold(0.9)
    .build();

// Lenient matching (0.6) - allows more variation
let filter = EpisodeFilter::builder()
    .search_text("asynch".to_string())
    .fuzzy_search_with_threshold(0.6)
    .build();
```

### Using SearchMode Enum

```rust
use memory_core::search::SearchMode;

// Explicit fuzzy search
let filter = EpisodeFilter::builder()
    .search_text("conection".to_string())
    .search_mode(SearchMode::Fuzzy { threshold: 0.8 })
    .build();

// Exact search (default behavior)
let filter = EpisodeFilter::builder()
    .search_text("connection".to_string())
    .search_mode(SearchMode::Exact)
    .build();
```

## Multi-Field Search

Search across different episode fields:

```rust
use memory_core::search::SearchField;

// Search only in episode descriptions (default)
let filter = EpisodeFilter::builder()
    .search_text("databse".to_string())
    .fuzzy_search()
    .search_fields(vec![SearchField::Description])
    .build();

// Search in step actions and results
let filter = EpisodeFilter::builder()
    .search_text("conection".to_string())
    .fuzzy_search()
    .search_fields(vec![SearchField::Steps])
    .build();

// Search in outcomes
let filter = EpisodeFilter::builder()
    .search_text("sucessful".to_string())  // Typo
    .fuzzy_search()
    .search_fields(vec![SearchField::Outcome])
    .build();

// Search in tags
let filter = EpisodeFilter::builder()
    .search_text("netwrking".to_string())  // Typo
    .fuzzy_search()
    .search_fields(vec![SearchField::Tags])
    .build();

// Search all fields
let filter = EpisodeFilter::builder()
    .search_text("databse".to_string())
    .fuzzy_search()
    .search_fields(vec![SearchField::All])
    .build();
```

## Combining with Other Filters

Fuzzy search works seamlessly with existing filters:

```rust
// Fuzzy search + domain filter
let filter = EpisodeFilter::builder()
    .search_text("databse".to_string())
    .fuzzy_search()
    .domains(vec!["backend".to_string()])
    .build();

// Fuzzy search + success filter
let filter = EpisodeFilter::builder()
    .search_text("authentiction".to_string())  // Typo
    .fuzzy_search()
    .success_only(true)
    .build();

// Fuzzy search + date range + task type
let filter = EpisodeFilter::builder()
    .search_text("postgres".to_string())
    .fuzzy_search()
    .date_from(Utc::now() - Duration::days(7))
    .task_types(vec![TaskType::CodeGeneration])
    .build();
```

## Threshold Guidelines

The threshold parameter controls how similar text must be to match:

| Threshold | Description | Use Case |
|-----------|-------------|----------|
| **1.0** | Exact match only | When precision is critical |
| **0.9** | Very strict | Minor typos only (1 character) |
| **0.8** | Recommended default | Common typos (1-2 characters) |
| **0.7** | Lenient | Multiple typos or variations |
| **0.6** | Very lenient | Significant spelling differences |
| **< 0.6** | Too permissive | May produce false matches |

### Examples by Threshold

```rust
// threshold = 0.9 (strict)
"async" matches "async" ✓
"async" matches "asnyc" ✗ (too different)

// threshold = 0.8 (recommended)
"database" matches "database" ✓
"database" matches "databse" ✓ (1 char missing)
"database" matches "datbase" ✓ (1 char missing)

// threshold = 0.7 (lenient)
"connection" matches "conection" ✓ (1 char missing)
"connection" matches "conecton" ✓ (2 chars different)

// threshold = 0.6 (very lenient)
"authentication" matches "authentiction" ✓
"authentication" matches "autentication" ✓
```

## Performance Characteristics

### Benchmarks

| Operation | Episodes | Time (P95) | Notes |
|-----------|----------|------------|-------|
| Fuzzy search (description) | 1,000 | < 50ms | Single-field |
| Fuzzy search (all fields) | 1,000 | < 150ms | Multi-field |
| Exact search | 1,000 | < 20ms | Baseline |

### Optimization Tips

1. **Use Appropriate Threshold**: Higher thresholds (0.8-0.9) are faster
2. **Limit Search Fields**: Search specific fields instead of `SearchField::All`
3. **Combine with Filters**: Reduce the search space with domain/date filters first
4. **Cache Results**: Reuse filter results when possible

## Algorithm Details

### Levenshtein Distance

Fuzzy search uses the normalized Levenshtein distance algorithm via the `strsim` crate:

- **Character-level**: Compares character-by-character similarity
- **Word-level**: Searches individual words and word combinations
- **Case-insensitive**: All comparisons are lowercased
- **Fast path**: Exact substring matches return 1.0 immediately

### Scoring

```rust
similarity_score = normalized_levenshtein(query, text)
// Returns: 0.0 (completely different) to 1.0 (identical)

match_found = similarity_score >= threshold
```

## Common Patterns

### 1. Typo-Tolerant Search

```rust
// User mistyped "database" as "databse"
let filter = EpisodeFilter::builder()
    .search_text("databse".to_string())
    .fuzzy_search()
    .build();
```

### 2. Search Variations

```rust
// Find "asynchronous" when searching "async"
let filter = EpisodeFilter::builder()
    .search_text("async".to_string())
    .fuzzy_search_with_threshold(0.6)
    .build();
```

### 3. Multi-Language Typos

```rust
// Handle common misspellings
let filter = EpisodeFilter::builder()
    .search_text("recieve".to_string())  // Common misspelling of "receive"
    .fuzzy_search()
    .build();
```

### 4. Partial Words

```rust
// Find "connection" with partial word
let filter = EpisodeFilter::builder()
    .search_text("connec".to_string())
    .fuzzy_search_with_threshold(0.7)
    .build();
```

## API Reference

### `SearchMode` Enum

```rust
pub enum SearchMode {
    /// Exact substring match (case-insensitive)
    Exact,
    
    /// Fuzzy match with similarity threshold (0.0 to 1.0)
    Fuzzy { threshold: f64 },
    
    /// Regex pattern matching (future)
    Regex,
}
```

### `SearchField` Enum

```rust
pub enum SearchField {
    Description,  // Task description
    Steps,        // Step actions and results
    Outcome,      // Episode outcome message
    Tags,         // Context tags
    Domain,       // Context domain
    All,          // All fields
}
```

### Builder Methods

```rust
impl EpisodeFilterBuilder {
    /// Enable fuzzy search with default threshold (0.8)
    pub fn fuzzy_search(self) -> Self;
    
    /// Enable fuzzy search with custom threshold
    pub fn fuzzy_search_with_threshold(self, threshold: f64) -> Self;
    
    /// Set search mode explicitly
    pub fn search_mode(self, mode: SearchMode) -> Self;
    
    /// Set which fields to search in
    pub fn search_fields(self, fields: Vec<SearchField>) -> Self;
}
```

## Examples

See the complete working example:

```bash
cargo run --example fuzzy_search_demo
```

## Testing

Run fuzzy search tests:

```bash
# Unit tests
cargo test search::fuzzy

# Integration tests
cargo test fuzzy_search
```

## Migration from Exact Search

Existing code using exact search continues to work without changes:

```rust
// Old code (still works)
let filter = EpisodeFilter::builder()
    .search_text("database".to_string())
    .build();  // Defaults to SearchMode::Exact

// New code with fuzzy search
let filter = EpisodeFilter::builder()
    .search_text("database".to_string())
    .fuzzy_search()  // Opt-in to fuzzy matching
    .build();
```

## Future Enhancements

Planned improvements for fuzzy search:

- [ ] Phonetic matching (Soundex, Metaphone)
- [ ] Synonym expansion
- [ ] Query suggestion ("Did you mean...?")
- [ ] Search result highlighting
- [ ] Relevance ranking with multiple signals
- [ ] Language-specific fuzzy rules

## Related Features

- **Semantic Search**: Use embeddings for meaning-based search (requires Turso + embeddings)
- **Pattern Search**: Find patterns similar to a query
- **Hybrid Search**: Combine vector similarity with full-text search

## See Also

- [Regex Search Guide](REGEX_SEARCH_GUIDE.md) - Advanced pattern matching
- [Episode Filtering Guide](EPISODE_FILTERING.md)
- [Pattern Search Feature](PATTERN_SEARCH_FEATURE.md)
- [Quick Start Guide](QUICK_START_EMBEDDINGS.md)
