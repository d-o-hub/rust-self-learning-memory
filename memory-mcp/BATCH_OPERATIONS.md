# Batch Operations for MCP Server

## Overview

Batch operations allow you to execute multiple MCP tool calls in a single request, dramatically improving performance for complex workflows. This feature provides:

- **3-5x faster execution** for multi-tool workflows
- **Dependency management** with automatic DAG validation
- **Parallel execution** of independent operations
- **Partial failure handling** - continue on errors
- **Flexible execution modes** - parallel, sequential, or fail-fast

## Quick Start

### Basic Parallel Execution

Execute multiple independent operations concurrently:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "batch/execute",
  "params": {
    "operations": [
      {
        "id": "query1",
        "tool": "query_memory",
        "arguments": {
          "query": "authentication patterns",
          "domain": "web-api",
          "limit": 5
        }
      },
      {
        "id": "metrics1",
        "tool": "get_metrics",
        "arguments": {
          "metric_type": "performance"
        }
      },
      {
        "id": "health1",
        "tool": "health_check",
        "arguments": {}
      }
    ],
    "mode": "parallel",
    "max_parallel": 10
  }
}
```

**Result**: All 3 operations execute concurrently, completing in ~100ms instead of ~300ms.

### Operations with Dependencies

Create complex workflows with dependency chains:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "batch/execute",
  "params": {
    "operations": [
      {
        "id": "fetch",
        "tool": "query_memory",
        "arguments": {
          "query": "error handling patterns",
          "domain": "api"
        }
      },
      {
        "id": "analyze",
        "tool": "analyze_patterns",
        "arguments": {
          "task_type": "error_recovery",
          "min_success_rate": 0.7
        },
        "depends_on": ["fetch"]
      },
      {
        "id": "recommend",
        "tool": "recommend_patterns",
        "arguments": {
          "task_description": "implement retry logic",
          "domain": "api"
        },
        "depends_on": ["analyze"]
      }
    ],
    "mode": "parallel"
  }
}
```

**Result**: Operations execute in order (fetch → analyze → recommend), but the system automatically handles dependencies.

## Execution Modes

### Parallel Mode (Default)

Executes independent operations concurrently while respecting dependencies.

```json
{
  "mode": "parallel",
  "max_parallel": 10
}
```

- **Best for**: Maximum throughput
- **Behavior**: Independent operations run concurrently
- **Errors**: Continues executing remaining operations
- **Use case**: Most workflows benefit from this mode

### Sequential Mode

Executes all operations one after another in insertion order.

```json
{
  "mode": "sequential"
}
```

- **Best for**: Operations with side effects or strict ordering requirements
- **Behavior**: Executes operations in the exact order provided
- **Errors**: Continues executing all operations
- **Use case**: When order matters and parallelism isn't safe

### Fail-Fast Mode

Stops execution on the first error encountered.

```json
{
  "mode": "failfast"
}
```

- **Best for**: Validation workflows or critical operation chains
- **Behavior**: Stops immediately when any operation fails
- **Errors**: Returns partial results up to the failure point
- **Use case**: Pre-flight checks, validation pipelines

## Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "results": [
      {
        "id": "query1",
        "success": true,
        "result": { "content": [{ "type": "text", "text": "..." }] },
        "duration_ms": 45
      },
      {
        "id": "metrics1",
        "success": true,
        "result": { "content": [{ "type": "text", "text": "..." }] },
        "duration_ms": 23
      },
      {
        "id": "health1",
        "success": false,
        "error": {
          "code": -32000,
          "message": "Service unavailable"
        },
        "duration_ms": 12
      }
    ],
    "total_duration_ms": 48,
    "success_count": 2,
    "failure_count": 1,
    "stats": {
      "total_operations": 3,
      "parallel_executed": 3,
      "sequential_executed": 0,
      "avg_duration_ms": 26.7
    }
  }
}
```

## Advanced Features

### Complex Dependency Graphs

Create workflows with multiple parallel branches:

```json
{
  "operations": [
    {
      "id": "init",
      "tool": "configure_embeddings",
      "arguments": { "provider": "openai" }
    },
    {
      "id": "query_code",
      "tool": "query_memory",
      "arguments": { "domain": "code_generation" },
      "depends_on": ["init"]
    },
    {
      "id": "query_debug",
      "tool": "query_memory",
      "arguments": { "domain": "debugging" },
      "depends_on": ["init"]
    },
    {
      "id": "merge_results",
      "tool": "analyze_patterns",
      "arguments": { "task_type": "combined" },
      "depends_on": ["query_code", "query_debug"]
    }
  ],
  "mode": "parallel"
}
```

**Execution flow**:
```
       init
      /    \
 query_code  query_debug
      \    /
   merge_results
```

### Partial Failure Handling

By default, batch operations continue on failure:

```json
{
  "operations": [
    {"id": "op1", "tool": "query_memory", "arguments": {...}},
    {"id": "op2", "tool": "invalid_tool", "arguments": {...}},
    {"id": "op3", "tool": "get_metrics", "arguments": {...}}
  ],
  "mode": "parallel"
}
```

**Result**: Operations 1 and 3 succeed, operation 2 fails, but all results are returned.

### Rate Limiting with max_parallel

Control concurrency to avoid overwhelming the system:

```json
{
  "operations": [...],  // 20 operations
  "mode": "parallel",
  "max_parallel": 5     // Only 5 execute at once
}
```

## Performance Characteristics

| Workflow Type | Sequential Time | Batch Time | Speedup |
|--------------|----------------|------------|---------|
| 3 independent queries | ~300ms | ~100ms | **3x** |
| 5 analysis operations | ~500ms | ~100ms | **5x** |
| Complex DAG (8 ops) | ~800ms | ~200ms | **4x** |

**Real-world example**:
- Traditional: 5 separate requests = 5 round-trips + 5× latency
- Batch: 1 request = 1 round-trip + parallel execution

## Error Handling

### Validation Errors

Returned before any execution:

```json
{
  "error": {
    "code": -32602,
    "message": "Invalid batch request params",
    "data": {
      "details": "Circular dependency detected: op1 -> op2 -> op1"
    }
  }
}
```

### Execution Errors

Returned with partial results:

```json
{
  "results": [
    {"id": "op1", "success": true, ...},
    {"id": "op2", "success": false, "error": {...}}
  ],
  "success_count": 1,
  "failure_count": 1
}
```

## Validation Rules

1. **Unique IDs**: Each operation must have a unique ID
2. **Valid Dependencies**: All `depends_on` IDs must exist
3. **Acyclic Graph**: No circular dependencies allowed
4. **Valid Tools**: All tool names must be recognized

## Best Practices

### 1. Use Meaningful Operation IDs

```json
// Good
{"id": "fetch_user_data", ...}
{"id": "validate_permissions", ...}

// Bad
{"id": "op1", ...}
{"id": "op2", ...}
```

### 2. Group Related Operations

Batch operations that are part of the same logical workflow:

```json
// Good: Related workflow
["fetch_patterns", "analyze_effectiveness", "recommend_best"]

// Bad: Unrelated operations
["random_query1", "unrelated_metric", "different_domain"]
```

### 3. Set Appropriate max_parallel

```json
// Light operations: higher parallelism
{"max_parallel": 20}

// Heavy operations (embeddings, analysis): lower parallelism
{"max_parallel": 3}
```

### 4. Use Dependencies Wisely

Only specify dependencies when truly needed:

```json
// Good: Real dependency
{
  "id": "analyze",
  "depends_on": ["fetch_data"]  // Needs data first
}

// Bad: Unnecessary dependency
{
  "id": "independent_query",
  "depends_on": ["unrelated_op"]  // Slows down execution
}
```

### 5. Handle Failures Gracefully

Check individual operation success:

```javascript
const response = await executeBatch(...);

for (const result of response.results) {
  if (result.success) {
    processResult(result.result);
  } else {
    console.error(`Operation ${result.id} failed:`, result.error.message);
  }
}
```

## Use Cases

### 1. Dashboard Data Loading

Load all dashboard metrics in one request:

```json
{
  "operations": [
    {"id": "episode_count", "tool": "get_metrics", "arguments": {"metric_type": "episodes"}},
    {"id": "performance", "tool": "get_metrics", "arguments": {"metric_type": "performance"}},
    {"id": "health", "tool": "health_check", "arguments": {}},
    {"id": "recent_patterns", "tool": "search_patterns", "arguments": {"query": "recent", "limit": 5}}
  ],
  "mode": "parallel",
  "max_parallel": 10
}
```

### 2. Multi-Source Query

Query multiple domains simultaneously:

```json
{
  "operations": [
    {"id": "web_patterns", "tool": "query_memory", "arguments": {"domain": "web-api"}},
    {"id": "cli_patterns", "tool": "query_memory", "arguments": {"domain": "cli"}},
    {"id": "db_patterns", "tool": "query_memory", "arguments": {"domain": "database"}}
  ],
  "mode": "parallel"
}
```

### 3. Pipeline Workflow

Execute a multi-stage pipeline:

```json
{
  "operations": [
    {"id": "configure", "tool": "configure_embeddings", "arguments": {...}},
    {"id": "query", "tool": "query_semantic_memory", "arguments": {...}, "depends_on": ["configure"]},
    {"id": "analyze", "tool": "advanced_pattern_analysis", "arguments": {...}, "depends_on": ["query"]},
    {"id": "recommend", "tool": "recommend_patterns", "arguments": {...}, "depends_on": ["analyze"]}
  ],
  "mode": "parallel"
}
```

### 4. Pre-Flight Validation

Validate multiple conditions before proceeding:

```json
{
  "operations": [
    {"id": "check_health", "tool": "health_check", "arguments": {}},
    {"id": "test_embeddings", "tool": "test_embeddings", "arguments": {}},
    {"id": "verify_storage", "tool": "get_metrics", "arguments": {"metric_type": "system"}}
  ],
  "mode": "failfast"  // Stop on first failure
}
```

## Limitations

1. **Maximum Operations**: Recommended limit of 50 operations per batch
2. **Timeout**: Individual operations subject to normal timeout rules
3. **No Streaming**: Results returned after all operations complete
4. **Memory Usage**: All results held in memory until completion

## Comparison with JSON-RPC Batch

MCP batch operations are **different** from standard JSON-RPC 2.0 batch requests:

| Feature | JSON-RPC Batch | MCP Batch Operations |
|---------|----------------|---------------------|
| Dependency Management | ❌ No | ✅ Yes |
| Parallel Execution | ❌ Sequential | ✅ Parallel |
| Partial Failure | ✅ Yes | ✅ Yes |
| Execution Modes | ❌ One mode | ✅ Three modes |
| Performance Gains | 🟡 Moderate | ✅ 3-5x |

## Examples

See `memory-mcp/examples/batch_operations_demo.rs` for runnable examples.

## Testing

Run batch operation tests:

```bash
cargo test --package memory-mcp --test batch_operations_test
```

Run all 11 comprehensive tests covering:
- Parallel execution
- Dependency management
- Error handling
- Execution modes
- Performance characteristics

## Future Enhancements

Planned features:

- **Streaming results**: Return results as they complete
- **Conditional execution**: Skip operations based on prior results
- **Result interpolation**: Pass results between operations
- **Transaction support**: All-or-nothing semantics
- **Progress callbacks**: Real-time execution updates

## Support

For issues or questions:
- GitHub Issues: [memory system repository]
- Documentation: `memory-mcp/README.md`
- Examples: `memory-mcp/examples/`
