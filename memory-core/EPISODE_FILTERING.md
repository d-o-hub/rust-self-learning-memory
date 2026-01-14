# Advanced Episode Filtering

This guide demonstrates how to use the new advanced filtering capabilities to query episodes based on multiple criteria.

## Overview

The `EpisodeFilter` API provides rich filtering capabilities for querying episodes:

- **Tags** - Filter by ANY or ALL tags
- **Task Types** - Filter by specific task types
- **Domains** - Filter by domain names
- **Date Ranges** - Filter by creation date
- **Completion Status** - Filter completed/incomplete episodes
- **Outcome Types** - Filter by success/failure/partial success
- **Reward Scores** - Filter by minimum/maximum reward
- **Archive Status** - Include or exclude archived episodes
- **Text Search** - Search in task descriptions

## Quick Start

### Basic Filtering

```rust
use memory_core::{SelfLearningMemory, EpisodeFilter, TaskType};

let memory = SelfLearningMemory::new();

// Get all successful episodes
let filter = EpisodeFilter::builder()
    .success_only(true)
    .build();

let episodes = memory.list_episodes_filtered(filter, None, None).await?;
```

### Filter by Tags

```rust
// Episodes with ANY of these tags
let filter = EpisodeFilter::builder()
    .with_any_tags(vec!["async".to_string(), "networking".to_string()])
    .build();

// Episodes with ALL of these tags
let filter = EpisodeFilter::builder()
    .with_all_tags(vec!["rust".to_string(), "async".to_string()])
    .build();
```

### Filter by Task Type and Domain

```rust
let filter = EpisodeFilter::builder()
    .task_types(vec![TaskType::CodeGeneration, TaskType::Debugging])
    .domains(vec!["web-api".to_string()])
    .build();

let episodes = memory.list_episodes_filtered(filter, Some(10), None).await?;
```

### Filter by Date Range

```rust
use chrono::Utc;

// Episodes from the last 7 days
let filter = EpisodeFilter::builder()
    .date_from(Utc::now() - chrono::Duration::days(7))
    .build();

// Episodes in a specific date range
let filter = EpisodeFilter::builder()
    .date_from(Utc::now() - chrono::Duration::days(30))
    .date_to(Utc::now() - chrono::Duration::days(7))
    .build();
```

### Filter by Reward Score

```rust
// High-quality episodes (reward >= 1.5)
let filter = EpisodeFilter::builder()
    .min_reward(1.5)
    .build();

// Episodes in a reward range
let filter = EpisodeFilter::builder()
    .min_reward(1.0)
    .max_reward(2.0)
    .build();
```

### Archive Management

```rust
// Exclude archived episodes (default view)
let filter = EpisodeFilter::builder()
    .exclude_archived(true)
    .build();

// Only archived episodes
let filter = EpisodeFilter::builder()
    .archived_only(true)
    .build();
```

## Advanced Usage

### Complex Queries

Combine multiple filters for precise queries:

```rust
// Find successful async code generation episodes from the last month
let filter = EpisodeFilter::builder()
    .task_types(vec![TaskType::CodeGeneration])
    .with_any_tags(vec!["async".to_string()])
    .success_only(true)
    .exclude_archived(true)
    .date_from(Utc::now() - chrono::Duration::days(30))
    .min_reward(1.0)
    .build();

let episodes = memory.list_episodes_filtered(filter, Some(20), None).await?;
```

### Pagination

Use limit and offset for paginated results:

```rust
let filter = EpisodeFilter::builder()
    .completed_only(true)
    .build();

// Get first page (20 episodes)
let page1 = memory.list_episodes_filtered(filter.clone(), Some(20), None).await?;

// Get second page (next 20 episodes)
let page2 = memory.list_episodes_filtered(filter.clone(), Some(20), Some(20)).await?;
```

### Search by Description

```rust
// Case-insensitive search in task descriptions
let filter = EpisodeFilter::builder()
    .search_text("authentication".to_string())
    .build();

let matching = memory.list_episodes_filtered(filter, None, None).await?;
```

### Outcome-Based Filtering

```rust
use memory_core::OutcomeType;

// Only failures
let filter = EpisodeFilter::builder()
    .outcome_type(OutcomeType::Failure)
    .build();

// Only partial successes
let filter = EpisodeFilter::builder()
    .outcome_type(OutcomeType::PartialSuccess)
    .build();
```

## Practical Examples

### Find Learning Opportunities

```rust
// Find failed episodes to learn from
let filter = EpisodeFilter::builder()
    .outcome_type(OutcomeType::Failure)
    .date_from(Utc::now() - chrono::Duration::days(7))
    .exclude_archived(true)
    .build();

let failures = memory.list_episodes_filtered(filter, Some(10), None).await?;
for episode in failures {
    println!("Failed task: {}", episode.task_description);
    // Analyze what went wrong
}
```

### Quality Audit

```rust
// Find low-quality episodes for review
let filter = EpisodeFilter::builder()
    .max_reward(0.5)
    .completed_only(true)
    .build();

let low_quality = memory.list_episodes_filtered(filter, None, None).await?;
println!("Found {} low-quality episodes for review", low_quality.len());
```

### Domain-Specific Analysis

```rust
// Analyze success rate for a specific domain
let web_api_filter = EpisodeFilter::builder()
    .domains(vec!["web-api".to_string()])
    .completed_only(true)
    .build();

let all_web = memory.list_episodes_filtered(web_api_filter.clone(), None, None).await?;

let successful_filter = EpisodeFilter::builder()
    .domains(vec!["web-api".to_string()])
    .success_only(true)
    .build();

let successful_web = memory.list_episodes_filtered(successful_filter, None, None).await?;

let success_rate = (successful_web.len() as f64 / all_web.len() as f64) * 100.0;
println!("Web API success rate: {:.1}%", success_rate);
```

### Tag Analysis

```rust
// Find most common patterns with specific tags
let async_filter = EpisodeFilter::builder()
    .with_any_tags(vec!["async".to_string()])
    .success_only(true)
    .min_reward(1.0)
    .build();

let async_episodes = memory.list_episodes_filtered(async_filter, None, None).await?;

// Analyze common patterns in async episodes
for episode in async_episodes {
    println!("Task: {}", episode.task_description);
    println!("Patterns: {:?}", episode.patterns);
}
```

## Filter Builder API

The builder pattern provides a fluent API for constructing filters:

```rust
let filter = EpisodeFilter::builder()
    .with_any_tags(vec![...])      // Match ANY of these tags
    .with_all_tags(vec![...])      // Match ALL of these tags
    .task_types(vec![...])         // Filter by task types
    .domains(vec![...])            // Filter by domains
    .date_from(date)               // Episodes after date
    .date_to(date)                 // Episodes before date
    .completed_only(true)          // Only completed episodes
    .archived_only(true)           // Only archived episodes
    .exclude_archived(true)        // Exclude archived episodes
    .success_only(true)            // Only successful outcomes
    .min_reward(score)             // Minimum reward score
    .max_reward(score)             // Maximum reward score
    .search_text(text)             // Search in description
    .outcome_type(OutcomeType::Success)  // Filter by outcome
    .build();
```

## Programmatic Filtering

You can also filter episodes programmatically without using the memory system:

```rust
let filter = EpisodeFilter::builder()
    .success_only(true)
    .build();

// Check if an episode matches
if filter.matches(&episode) {
    println!("Episode matches filter");
}

// Filter a collection
let all_episodes = vec![...];
let filtered = filter.apply(all_episodes);

// Count matches
let match_count = filter.count_matches(&all_episodes);
```

## Performance Considerations

1. **Use specific filters** - More specific filters reduce the result set faster
2. **Apply pagination** - Use limit/offset for large result sets
3. **Archive old episodes** - Exclude archived episodes by default to improve performance
4. **Cache filter results** - If running the same filter multiple times, cache the results

## Best Practices

1. **Start broad, refine narrow** - Begin with broad filters and progressively narrow down
2. **Use domain filtering** - Domain is typically the most selective filter
3. **Combine with tags** - Tags provide excellent categorization
4. **Archive old data** - Keep active episode count manageable
5. **Monitor performance** - Use pagination for queries that may return many results

## API Reference

### EpisodeFilter Methods

- `new()` - Create an empty filter (matches all episodes)
- `builder()` - Create a filter builder
- `matches(&episode)` - Check if episode matches filter
- `apply(episodes)` - Filter a collection of episodes
- `count_matches(&episodes)` - Count matching episodes

### Backward Compatibility

The old `list_episodes` method is still available:

```rust
// Old API (still works)
let episodes = memory.list_episodes(Some(10), None, Some(true)).await?;

// New API (recommended)
let filter = EpisodeFilter::builder()
    .completed_only(true)
    .build();
let episodes = memory.list_episodes_filtered(filter, Some(10), None).await?;
```

## CLI Integration

The filtering API is designed to work seamlessly with the memory-cli. See the CLI documentation for command-line usage examples.

## Future Enhancements

Planned features for future releases:

- **Saved filters** - Save and reuse common filter configurations
- **Filter templates** - Pre-defined filters for common use cases
- **Advanced operators** - NOT, OR logic for complex queries
- **Custom predicates** - User-defined filter functions

## Support

For issues or questions:
- GitHub Issues: [memory system repository]
- Documentation: `memory-core/README.md`
- Examples: `memory-core/tests/episode_filtering_test.rs`
