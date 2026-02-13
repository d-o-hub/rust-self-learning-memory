# MCP Tools Reference

## 1. query_memory

Query episodic memory for relevant past experiences and learned patterns.

**Parameters**:
- `query` (required): Search query describing the task or context
- `domain` (required): Task domain (e.g., 'web-api', 'data-processing')
- `task_type` (optional): Type of task - `code_generation`, `debugging`, `refactoring`, `testing`, `analysis`, `documentation`
- `limit` (default: 10): Maximum number of episodes to retrieve

**Example**:
```json
{
  "query": "implement async storage with error handling",
  "domain": "rust-backend",
  "task_type": "code_generation",
  "limit": 5
}
```

**Use when**: You need relevant past experiences to inform current work.

## 2. analyze_patterns

Analyze patterns from past episodes to identify successful strategies.

**Parameters**:
- `task_type` (required): Type of task to analyze patterns for
- `min_success_rate` (default: 0.7): Minimum success rate (0.0-1.0)
- `limit` (default: 20): Maximum number of patterns to return

**Example**:
```json
{
  "task_type": "debugging",
  "min_success_rate": 0.8,
  "limit": 10
}
```

**Use when**: You want to identify proven successful approaches for a task type.

## 3. advanced_pattern_analysis

Perform advanced statistical analysis, predictive modeling, and causal inference.

**Parameters**:
- `analysis_type` (required): `statistical`, `predictive`, or `comprehensive`
- `time_series_data` (required): Object mapping variable names to numeric arrays
- `config` (optional): Analysis configuration
  - `significance_level` (default: 0.05): Statistical significance level
  - `forecast_horizon` (default: 10): Steps to forecast ahead
  - `anomaly_sensitivity` (default: 0.5): Anomaly detection sensitivity
  - `enable_causal_inference` (default: true): Perform causal analysis
  - `max_data_points` (default: 10000): Maximum data points
  - `parallel_processing` (default: true): Enable parallel processing

**Example**:
```json
{
  "analysis_type": "comprehensive",
  "time_series_data": {
    "latency_ms": [120, 115, 130, 125, 140],
    "success_rate": [0.95, 0.98, 0.96, 0.97, 0.99]
  },
  "config": {
    "forecast_horizon": 5,
    "anomaly_sensitivity": 0.6
  }
}
```

**Use when**: You need deep statistical insights and predictions from historical data.

## 4. execute_agent_code

Execute TypeScript/JavaScript code in a secure sandbox environment.

**Parameters**:
- `code` (required): TypeScript/JavaScript code to execute
- `context` (required): Execution context
  - `task`: Task description
  - `input`: Input data as JSON object

**Example**:
```json
{
  "code": "function process(data) { return data.map(x => x * 2); } process(context.input.numbers);",
  "context": {
    "task": "Double all numbers in array",
    "input": { "numbers": [1, 2, 3, 4, 5] }
  }
}
```

**Note**: Only available if WASM sandbox is enabled.

**Use when**: You need to safely execute user-provided or generated code.

## 5. health_check

Check the health status of the MCP server and its components.

**Parameters**: None

**Use when**: Diagnosing server issues or verifying operational status.

## 6. get_metrics

Get comprehensive monitoring metrics and statistics.

**Parameters**:
- `metric_type` (default: "all"): `all`, `performance`, `episodes`, or `system`

**Use when**: Monitoring server performance or gathering operational insights.
