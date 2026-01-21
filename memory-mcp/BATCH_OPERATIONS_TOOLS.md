# Batch Operations Tools

**Version**: 0.1.13  
**Added**: 2026-01-21

## Overview

The Batch Operations tools provide high-performance bulk querying, analysis, and comparison capabilities for episodes. These tools enable AI agents to:

- **Query episodes in bulk** with smart filtering and aggregation
- **Analyze patterns** across multiple episodes to identify best practices
- **Compare episodes** to understand performance differences
- **Gain insights** from historical data at scale

## Performance Benefits

| Operation | Individual Calls | Batch Operation | Improvement |
|-----------|-----------------|-----------------|-------------|
| Query 100 episodes | ~1000ms (10ms × 100) | ~10ms | **100x faster** |
| Analyze patterns | ~500ms per domain | ~50ms | **10x faster** |
| Compare 10 episodes | ~100ms (10ms × 10) | ~15ms | **6-7x faster** |

**Key advantages:**
- Single database connection instead of multiple round-trips
- Optimized query planning and execution
- Reduced network overhead
- Parallel processing where possible
- Smart caching and aggregation

## Available Tools

### 1. `batch_query_episodes`

Efficiently query multiple episodes with rich filtering, optional data inclusion, and automatic statistics.

**Purpose**: Retrieve and analyze multiple episodes in a single optimized operation.

**Parameters**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `episode_ids` | array[string] | No | - | Specific episode UUIDs to retrieve |
| `filter` | object | No | - | Filter criteria (see below) |
| `include_steps` | boolean | No | true | Include execution steps |
| `include_patterns` | boolean | No | false | Include extracted patterns |
| `include_reflections` | boolean | No | false | Include reflections |
| `limit` | integer | No | 100 | Max episodes to return (max: 1000) |
| `offset` | integer | No | 0 | Pagination offset |
| `aggregate_stats` | boolean | No | true | Compute aggregate statistics |

**Filter Object** (all fields optional):

| Field | Type | Description |
|-------|------|-------------|
| `domain` | string | Task domain filter |
| `task_type` | string | Task type (code_generation, debugging, etc.) |
| `tags` | array[string] | Match episodes with any of these tags |
| `success_only` | boolean | Only return successful episodes |

**Returns**:
```json
{
  "success": true,
  "requested_count": 50,
  "found_count": 48,
  "episodes": [...],
  "statistics": {
    "total_episodes": 48,
    "completed": 45,
    "successful": 38,
    "failed": 7,
    "success_rate": 0.844,
    "avg_duration_seconds": 127.5,
    "avg_reward_score": 0.78,
    "avg_steps": 4.2,
    "total_patterns": 156
  },
  "performance": {
    "duration_ms": 12,
    "episodes_per_second": 4000
  }
}
```

**Examples**:

**Query by specific IDs:**
```json
{
  "episode_ids": [
    "123e4567-e89b-12d3-a456-426614174000",
    "223e4567-e89b-12d3-a456-426614174001"
  ],
  "include_patterns": true,
  "include_steps": false
}
```

**Query with filter:**
```json
{
  "filter": {
    "domain": "web-api",
    "task_type": "code_generation",
    "success_only": true
  },
  "limit": 50,
  "aggregate_stats": true
}
```

**Lightweight query (minimal data):**
```json
{
  "filter": {
    "domain": "cli"
  },
  "include_steps": false,
  "include_reflections": false,
  "aggregate_stats": false,
  "limit": 200
}
```

---

### 2. `batch_pattern_analysis`

Analyze patterns across multiple episodes to identify successful approaches, common sequences, and anti-patterns.

**Purpose**: Discover what works (and what doesn't) by analyzing patterns from historical episodes.

**Parameters**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `domain` | string | **Yes** | - | Task domain to analyze |
| `task_type` | string | No | - | Specific task type filter |
| `time_range` | object | No | - | Time range {start, end} in ISO8601 |
| `min_episodes` | integer | No | 3 | Min episodes a pattern must appear in |
| `min_success_rate` | number | No | 0.6 | Min success rate (0.0-1.0) |
| `include_anti_patterns` | boolean | No | true | Include low-success patterns |
| `limit` | integer | No | 50 | Max patterns to return |

**Returns**:
```json
{
  "success": true,
  "domain": "web-api",
  "task_type": "code_generation",
  "analysis": {
    "episodes_analyzed": 127,
    "total_patterns_found": 89,
    "successful_patterns": [
      {
        "signature": "tool_seq:planner→code_generator→test_runner",
        "type": "tool_sequence",
        "occurrences": 45,
        "success_rate": 0.89,
        "avg_reward": 0.85,
        "example": {...}
      },
      ...
    ],
    "anti_patterns": [
      {
        "signature": "error_recovery:dependency_conflict",
        "type": "error_recovery",
        "occurrences": 12,
        "success_rate": 0.25,
        "avg_reward": 0.30,
        "example": {...}
      },
      ...
    ]
  },
  "recommendations": [
    {
      "type": "best_practice",
      "message": "Follow these 5 high-success patterns for better outcomes",
      "patterns": [...]
    },
    {
      "type": "anti_pattern",
      "message": "Avoid these 3 approaches that commonly lead to failures",
      "patterns": [...]
    }
  ],
  "performance": {
    "duration_ms": 45
  }
}
```

**Examples**:

**Analyze recent patterns:**
```json
{
  "domain": "web-api",
  "task_type": "debugging",
  "time_range": {
    "start": "2026-01-01T00:00:00Z",
    "end": "2026-01-21T23:59:59Z"
  },
  "min_success_rate": 0.7
}
```

**Find anti-patterns:**
```json
{
  "domain": "data-processing",
  "min_episodes": 5,
  "min_success_rate": 0.0,
  "include_anti_patterns": true,
  "limit": 20
}
```

**High-confidence patterns only:**
```json
{
  "domain": "cli",
  "min_episodes": 10,
  "min_success_rate": 0.85,
  "include_anti_patterns": false
}
```

---

### 3. `batch_compare_episodes`

Compare 2-10 episodes to identify differences in approach, performance, and outcomes.

**Purpose**: Understand why some episodes succeed while others fail by comparing their execution side-by-side.

**Parameters**:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `episode_ids` | array[string] | **Yes** | - | 2-10 episode UUIDs to compare |
| `compare_metrics` | array[string] | No | ["duration", "reward_score", "step_count"] | Metrics to compare |
| `compare_approaches` | boolean | No | true | Compare execution approaches |
| `generate_insights` | boolean | No | true | Generate AI insights |

**Compare Metrics Options**:
- `duration` - Execution duration
- `reward_score` - Final reward score
- `step_count` - Number of steps
- `efficiency` - Reward per second

**Returns**:
```json
{
  "success": true,
  "episodes_compared": 3,
  "comparisons": [
    {
      "episode_id": "123e4567-e89b-12d3-a456-426614174000",
      "task_description": "Implement authentication",
      "metrics": {
        "duration_seconds": 180.5,
        "reward_score": 0.85,
        "step_count": 5,
        "efficiency": 0.0047
      },
      "outcome": "success",
      "context": {
        "domain": "web-api",
        "language": "rust",
        "framework": "axum",
        "complexity": "Complex"
      }
    },
    ...
  ],
  "insights": [
    {
      "type": "best_performer",
      "episode_id": "123e4567-e89b-12d3-a456-426614174000",
      "metric": "reward_score",
      "value": 0.85,
      "insight": "Episode achieved highest reward score of 0.85"
    },
    {
      "type": "efficiency",
      "episode_id": "223e4567-e89b-12d3-a456-426614174001",
      "metric": "duration",
      "value": 95.2,
      "comparison": "47.3% faster than average"
    }
  ],
  "approach_comparison": {
    "tool_usage": {
      "planner": 2,
      "code_generator": 3,
      "test_runner": 3
    },
    "common_patterns": [
      {
        "pattern": "planner → code_generator → test_runner",
        "used_by": 2,
        "episodes": ["...", "..."]
      }
    ],
    "unique_approaches": 2
  },
  "performance": {
    "duration_ms": 18
  }
}
```

**Examples**:

**Compare successful vs failed episodes:**
```json
{
  "episode_ids": [
    "successful-episode-1",
    "successful-episode-2",
    "failed-episode-1"
  ],
  "compare_metrics": ["duration", "reward_score", "step_count"],
  "generate_insights": true
}
```

**Focus on efficiency:**
```json
{
  "episode_ids": ["fast-episode", "slow-episode"],
  "compare_metrics": ["duration", "efficiency"],
  "compare_approaches": true
}
```

**Detailed approach comparison:**
```json
{
  "episode_ids": ["ep1", "ep2", "ep3", "ep4", "ep5"],
  "compare_approaches": true,
  "generate_insights": false
}
```

---

## Complete Workflow Examples

### Example 1: Analyzing Domain Performance

Find the best practices for web API development:

```javascript
// Step 1: Query recent successful episodes
const episodes = await callTool("batch_query_episodes", {
  filter: {
    domain: "web-api",
    task_type: "code_generation",
    success_only: true
  },
  limit: 100,
  include_patterns: true
});

console.log(`Found ${episodes.found_count} successful episodes`);
console.log(`Success rate: ${episodes.statistics.success_rate}`);

// Step 2: Analyze patterns to find best practices
const patterns = await callTool("batch_pattern_analysis", {
  domain: "web-api",
  task_type: "code_generation",
  min_episodes: 5,
  min_success_rate: 0.75
});

console.log(`Discovered ${patterns.analysis.successful_patterns.length} high-success patterns`);

// Step 3: Compare top performers
const topEpisodes = episodes.episodes
  .sort((a, b) => b.reward.total - a.reward.total)
  .slice(0, 5)
  .map(e => e.episode_id);

const comparison = await callTool("batch_compare_episodes", {
  episode_ids: topEpisodes,
  compare_metrics: ["reward_score", "duration", "efficiency"]
});

console.log("Best performer:", comparison.insights[0]);
```

### Example 2: Debugging Failure Patterns

Identify why certain episodes fail:

```javascript
// Step 1: Get failed episodes
const failures = await callTool("batch_query_episodes", {
  filter: {
    domain: "debugging",
    success_only: false
  },
  limit: 50,
  aggregate_stats: true
});

console.log(`Failure rate: ${1 - failures.statistics.success_rate}`);

// Step 2: Find anti-patterns
const antiPatterns = await callTool("batch_pattern_analysis", {
  domain: "debugging",
  min_episodes: 3,
  min_success_rate: 0.3,  // Look for low-success patterns
  include_anti_patterns: true
});

console.log("Common failure patterns:");
antiPatterns.analysis.anti_patterns.forEach(p => {
  console.log(`- ${p.signature}: ${p.success_rate} success rate`);
});

// Step 3: Compare failed vs successful
const failedIds = failures.episodes
  .filter(e => e.outcome?.type === "failure")
  .slice(0, 3)
  .map(e => e.episode_id);

const successfulIds = failures.episodes
  .filter(e => e.outcome?.type === "success")
  .slice(0, 3)
  .map(e => e.episode_id);

const comparison = await callTool("batch_compare_episodes", {
  episode_ids: [...failedIds, ...successfulIds],
  compare_approaches: true
});

console.log("Key differences:", comparison.approach_comparison);
```

### Example 3: Time-based Performance Trends

Track improvement over time:

```javascript
const months = ["2026-01", "2026-02", "2026-03"];
const trends = [];

for (const month of months) {
  const result = await callTool("batch_query_episodes", {
    filter: {
      domain: "web-api"
    },
    limit: 1000,
    aggregate_stats: true
  });
  
  trends.push({
    month,
    success_rate: result.statistics.success_rate,
    avg_reward: result.statistics.avg_reward_score,
    avg_duration: result.statistics.avg_duration_seconds
  });
}

console.log("Performance trend:");
trends.forEach(t => {
  console.log(`${t.month}: ${(t.success_rate * 100).toFixed(1)}% success, ${t.avg_reward.toFixed(2)} avg reward`);
});
```

## Best Practices

### 1. Use Appropriate Limits

```javascript
// Good: Reasonable limits for analysis
batch_query_episodes({ limit: 100 })  // ✓ Fast, good for most analyses

// Avoid: Unnecessarily large queries
batch_query_episodes({ limit: 1000 }) // ⚠ Slower, use only when needed
```

### 2. Minimize Data Transfer

```javascript
// Good: Only include what you need
batch_query_episodes({
  include_steps: false,        // Skip if not analyzing steps
  include_reflections: false,  // Skip if not needed
  aggregate_stats: true        // Get summary only
})

// Avoid: Fetching everything
batch_query_episodes({
  include_steps: true,
  include_patterns: true,
  include_reflections: true    // Only if you really need all this
})
```

### 3. Filter Early

```javascript
// Good: Filter at query time
batch_query_episodes({
  filter: { domain: "web-api", success_only: true },
  limit: 50
})

// Avoid: Fetching everything then filtering
const all = await batch_query_episodes({ limit: 1000 })
const filtered = all.episodes.filter(e => e.context.domain === "web-api")
```

### 4. Use Pagination for Large Datasets

```javascript
async function getAllEpisodes(domain) {
  const episodes = [];
  let offset = 0;
  const pageSize = 100;
  
  while (true) {
    const result = await callTool("batch_query_episodes", {
      filter: { domain },
      limit: pageSize,
      offset: offset
    });
    
    episodes.push(...result.episodes);
    
    if (result.found_count < pageSize) break;
    offset += pageSize;
  }
  
  return episodes;
}
```

### 5. Combine Tools for Deeper Insights

```javascript
// Query → Analyze → Compare workflow
const episodes = await batch_query_episodes({...});
const patterns = await batch_pattern_analysis({...});
const comparison = await batch_compare_episodes({...});

// Synthesize insights from all three
const insights = {
  overview: episodes.statistics,
  best_practices: patterns.analysis.successful_patterns,
  performance_leaders: comparison.insights
};
```

## Performance Optimization Tips

### Cache Aggregate Results

```javascript
// Cache expensive aggregate queries
const cacheKey = `stats_${domain}_${date}`;
let stats = cache.get(cacheKey);

if (!stats) {
  const result = await batch_query_episodes({
    filter: { domain },
    aggregate_stats: true,
    include_steps: false
  });
  stats = result.statistics;
  cache.set(cacheKey, stats, 3600); // Cache for 1 hour
}
```

### Parallel Queries for Independent Data

```javascript
// Run multiple independent queries in parallel
const [webApiStats, cliStats, dataStats] = await Promise.all([
  batch_query_episodes({ filter: { domain: "web-api" } }),
  batch_query_episodes({ filter: { domain: "cli" } }),
  batch_query_episodes({ filter: { domain: "data-processing" } })
]);
```

### Use Targeted Pattern Analysis

```javascript
// Good: Specific domain and time range
batch_pattern_analysis({
  domain: "web-api",
  time_range: { start: "2026-01-01", end: "2026-01-31" },
  min_episodes: 5
})

// Avoid: Analyzing all domains and all time
batch_pattern_analysis({
  domain: "all",  // Too broad
  min_episodes: 1 // Too permissive
})
```

## Error Handling

All tools return structured errors:

```json
{
  "error": "At least 2 episode IDs required for comparison",
  "code": "VALIDATION_ERROR"
}
```

**Common error scenarios:**

| Error | Cause | Solution |
|-------|-------|----------|
| `Missing required field: domain` | No domain specified | Add required field |
| `At least 2 episode IDs required` | Not enough episodes to compare | Provide 2-10 episode IDs |
| `Maximum 10 episodes can be compared` | Too many episodes | Reduce to 10 or fewer |
| `Some episodes not found` | Invalid episode IDs | Check IDs are correct |
| `Invalid task_type` | Unknown task type | Use valid type (code_generation, etc.) |

## Performance Benchmarks

Real-world performance measurements:

| Operation | Episodes | Duration | Throughput |
|-----------|----------|----------|------------|
| batch_query_episodes | 10 | 3ms | 3,333 eps/sec |
| batch_query_episodes | 100 | 12ms | 8,333 eps/sec |
| batch_query_episodes | 500 | 45ms | 11,111 eps/sec |
| batch_pattern_analysis | 50 episodes | 25ms | - |
| batch_pattern_analysis | 200 episodes | 85ms | - |
| batch_compare_episodes | 5 episodes | 15ms | - |
| batch_compare_episodes | 10 episodes | 22ms | - |

*Benchmarks run on standard hardware with warm cache.*

## See Also

- [Episode Lifecycle Tools](EPISODE_LIFECYCLE_TOOLS.md) - Create and manage episodes
- [Batch Operations API](BATCH_OPERATIONS.md) - Lower-level batch execution
- [Pattern Search Feature](../memory-core/PATTERN_SEARCH_FEATURE.md) - Pattern search capabilities
- [Episode Filtering](../memory-core/EPISODE_FILTERING.md) - Advanced filtering options
