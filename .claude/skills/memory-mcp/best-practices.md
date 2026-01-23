# Best Practices

## Tool Usage

### DO:
- Use `query_memory` before starting new tasks to learn from past experiences
- Set appropriate `limit` values to avoid over-retrieving
- Specify `task_type` to get more relevant results
- Use `analyze_patterns` to identify proven strategies
- Run `health_check` periodically in production
- Monitor metrics with `get_metrics` for performance insights

### DON'T:
- Query without a clear domain/task context
- Ignore min_success_rate when analyzing patterns
- Execute untrusted code without sandbox validation
- Run advanced analysis on tiny datasets (< 10 points)
- Skip health checks in production deployments

## Configuration

### DO:
- Use environment variables for all configuration
- Set RUST_LOG=off in production for performance
- Enable cache warming for better cold-start performance
- Use absolute paths for database files
- Configure appropriate cache sizes based on workload

### DON'T:
- Hardcode database paths in code
- Enable debug logging in production
- Use file:// URLs with relative paths
- Set cache size too small (< 100 episodes)
- Forget to create data directory before first run

## Testing

### DO:
- Always use MCP Inspector for validation
- Test all tools before deploying
- Verify schema compliance
- Test with realistic data volumes
- Check error handling with invalid inputs

### DON'T:
- Deploy without inspector validation
- Skip schema validation
- Test only happy paths
- Ignore performance testing
- Assume tools work without verification

## Integration Examples

### Query Memory Before Code Generation

```typescript
// Step 1: Query relevant past experiences
const context = await query_memory({
  query: "implement REST API with authentication",
  domain: "web-api",
  task_type: "code_generation",
  limit: 5
});

// Step 2: Analyze patterns from successful implementations
const patterns = await analyze_patterns({
  task_type: "code_generation",
  min_success_rate: 0.8,
  limit: 10
});

// Step 3: Use insights to inform implementation
```

### Performance Analysis Workflow

```typescript
// Step 1: Collect metrics over time
const metrics = await get_metrics({
  metric_type: "performance"
});

// Step 2: Perform advanced analysis
const analysis = await advanced_pattern_analysis({
  analysis_type: "comprehensive",
  time_series_data: {
    latency_ms: metrics.latency_history,
    success_rate: metrics.success_history
  },
  config: {
    forecast_horizon: 10,
    enable_causal_inference: true
  }
});

// Step 3: Use predictions to optimize
```
