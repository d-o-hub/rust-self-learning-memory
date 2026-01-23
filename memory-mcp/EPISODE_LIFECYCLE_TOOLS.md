# Episode Lifecycle Management Tools

**Version**: 0.1.13  
**Added**: 2026-01-21

## Overview

The Episode Lifecycle Management tools provide AI agents with programmatic control over episode creation, tracking, and completion through the MCP interface. These tools enable agents to:

- **Create episodes** to track task execution
- **Log execution steps** to record progress
- **Complete episodes** with outcomes to trigger learning cycles
- **Retrieve episode details** for analysis and debugging
- **Visualize timelines** to understand task progression
- **Delete episodes** when needed (with safeguards)

## Available Tools

### 1. `create_episode`

Create a new episode to track task execution programmatically.

**Purpose**: Start tracking a new task with metadata and context.

**Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task_description` | string | Yes | Clear description of the task to be performed |
| `domain` | string | Yes | Task domain (e.g., "web-api", "cli", "data-processing") |
| `task_type` | enum | Yes | One of: `code_generation`, `debugging`, `refactoring`, `testing`, `analysis`, `documentation` |
| `language` | string | No | Programming language (e.g., "rust", "python") |
| `framework` | string | No | Framework name (e.g., "axum", "tokio") |
| `tags` | array[string] | No | Context tags for organization |
| `complexity` | enum | No | One of: `simple`, `moderate`, `complex` (default: `moderate`) |

**Returns**:
```json
{
  "success": true,
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "task_description": "Implement user authentication",
  "domain": "web-api",
  "task_type": "code_generation",
  "message": "Episode created successfully"
}
```

**Example**:
```json
{
  "task_description": "Implement JWT-based authentication",
  "domain": "web-api",
  "task_type": "code_generation",
  "language": "rust",
  "framework": "axum",
  "tags": ["auth", "security", "api"],
  "complexity": "complex"
}
```

---

### 2. `add_episode_step`

Add an execution step to an ongoing episode to track progress.

**Purpose**: Log individual actions and their results during task execution.

**Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `episode_id` | string (UUID) | Yes | Episode identifier |
| `step_number` | integer | Yes | Sequential step number |
| `tool` | string | Yes | Name of the tool/component performing the action |
| `action` | string | Yes | Description of the action taken |
| `parameters` | object | No | Parameters used in this step |
| `result` | object | No | Result of the step (see below) |
| `latency_ms` | integer | No | Execution time in milliseconds |

**Result Object** (optional):

| Field | Type | Description |
|-------|------|-------------|
| `type` | enum | One of: `success`, `error`, `timeout` |
| `output` | string | Output message (for success) |
| `message` | string | Error message (for error) |

**Returns**:
```json
{
  "success": true,
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "step_number": 1,
  "message": "Step added successfully"
}
```

**Example**:
```json
{
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "step_number": 1,
  "tool": "code_generator",
  "action": "Generating authentication module",
  "parameters": {
    "template": "jwt-auth",
    "framework": "axum"
  },
  "result": {
    "type": "success",
    "output": "Generated 3 files: auth.rs, jwt.rs, middleware.rs"
  },
  "latency_ms": 250
}
```

---

### 3. `complete_episode`

Complete an episode with an outcome and trigger the learning cycle (reward calculation, reflection generation, pattern extraction).

**Purpose**: Finalize the episode and enable the system to learn from the experience.

**Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `episode_id` | string (UUID) | Yes | Episode identifier |
| `outcome_type` | enum | Yes | One of: `success`, `partial_success`, `failure` |

**Additional fields based on outcome type**:

**For `success`**:
- `verdict` (string, required): Description of the successful outcome
- `artifacts` (array[string], optional): Names of created artifacts

**For `partial_success`**:
- `verdict` (string, required): Description of the outcome
- `completed` (array[string], required): Items that were completed
- `failed` (array[string], required): Items that failed

**For `failure`**:
- `reason` (string, required): Reason for failure
- `error_details` (string, optional): Detailed error information

**Returns**:
```json
{
  "success": true,
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "outcome_type": "success",
  "message": "Episode completed successfully. Learning cycle triggered (reward, reflection, patterns)."
}
```

**Examples**:

**Success**:
```json
{
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "outcome_type": "success",
  "verdict": "Authentication system implemented and tested",
  "artifacts": ["auth.rs", "jwt.rs", "auth_tests.rs"]
}
```

**Partial Success**:
```json
{
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "outcome_type": "partial_success",
  "verdict": "Basic authentication works, advanced features pending",
  "completed": ["login", "logout", "token-validation"],
  "failed": ["2fa", "password-reset"]
}
```

**Failure**:
```json
{
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "outcome_type": "failure",
  "reason": "Incompatible dependency versions",
  "error_details": "axum 0.7 requires tokio 1.35+, but project uses tokio 1.28"
}
```

---

### 4. `get_episode`

Retrieve complete details of an episode including all steps, outcome, reflection, and extracted patterns.

**Purpose**: Inspect episode data for debugging, analysis, or reporting.

**Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `episode_id` | string (UUID) | Yes | Episode identifier |

**Returns**:
```json
{
  "success": true,
  "episode": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "task_description": "Implement user authentication",
    "task_type": "CodeGeneration",
    "context": {
      "domain": "web-api",
      "language": "rust",
      "framework": "axum",
      "complexity": "Complex",
      "tags": ["auth", "security"]
    },
    "start_time": "2026-01-21T10:00:00Z",
    "end_time": "2026-01-21T10:15:00Z",
    "steps": [...],
    "outcome": {...},
    "reflection": {...},
    "reward_score": 0.85,
    "patterns": [...]
  }
}
```

---

### 5. `get_episode_timeline`

Get a chronological timeline view of all steps in an episode.

**Purpose**: Visualize task progression and identify bottlenecks.

**Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `episode_id` | string (UUID) | Yes | Episode identifier |

**Returns**:
```json
{
  "success": true,
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "task_description": "Implement user authentication",
  "start_time": "2026-01-21T10:00:00Z",
  "end_time": "2026-01-21T10:15:00Z",
  "duration_seconds": 900.0,
  "step_count": 5,
  "outcome": "success",
  "timeline": [
    {
      "step_number": 1,
      "timestamp": "2026-01-21T10:00:05Z",
      "tool": "planner",
      "action": "Planning implementation",
      "result_type": "success",
      "latency_ms": 50
    },
    ...
  ]
}
```

---

### 6. `delete_episode`

Delete an episode permanently from all storage backends.

**Purpose**: Remove unwanted or test episodes (requires explicit confirmation).

⚠️ **Warning**: This operation cannot be undone!

**Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `episode_id` | string (UUID) | Yes | Episode identifier |
| `confirm` | boolean | Yes | Must be `true` to confirm deletion |

**Returns**:
```json
{
  "success": true,
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "message": "Episode deleted permanently"
}
```

**Example**:
```json
{
  "episode_id": "123e4567-e89b-12d3-a456-426614174000",
  "confirm": true
}
```

---

## Complete Workflow Example

Here's a complete example of using the episode lifecycle tools to track a feature implementation:

```javascript
// 1. Create episode
const createResult = await callTool("create_episode", {
  task_description: "Add rate limiting to API endpoints",
  domain: "web-api",
  task_type: "code_generation",
  language: "rust",
  framework: "axum",
  complexity: "moderate",
  tags: ["api", "security", "rate-limiting"]
});

const episodeId = createResult.episode_id;

// 2. Log planning step
await callTool("add_episode_step", {
  episode_id: episodeId,
  step_number: 1,
  tool: "architect",
  action: "Designing rate limiting strategy",
  result: {
    type: "success",
    output: "Chose token bucket algorithm with Redis backend"
  },
  latency_ms: 100
});

// 3. Log implementation step
await callTool("add_episode_step", {
  episode_id: episodeId,
  step_number: 2,
  tool: "code_generator",
  action: "Implementing rate limiter middleware",
  parameters: {
    algorithm: "token-bucket",
    backend: "redis"
  },
  result: {
    type: "success",
    output: "Created middleware and tests"
  },
  latency_ms: 450
});

// 4. Log testing step
await callTool("add_episode_step", {
  episode_id: episodeId,
  step_number: 3,
  tool: "test_runner",
  action: "Running integration tests",
  result: {
    type: "success",
    output: "All 12 tests passed"
  },
  latency_ms: 800
});

// 5. Check progress
const timeline = await callTool("get_episode_timeline", {
  episode_id: episodeId
});
console.log(`Completed ${timeline.step_count} steps in ${timeline.duration_seconds}s`);

// 6. Complete episode
await callTool("complete_episode", {
  episode_id: episodeId,
  outcome_type: "success",
  verdict: "Rate limiting successfully implemented and tested",
  artifacts: ["rate_limiter.rs", "rate_limiter_tests.rs", "redis_backend.rs"]
});

// 7. Retrieve full episode for reporting
const episode = await callTool("get_episode", {
  episode_id: episodeId
});
console.log("Reward score:", episode.episode.reward_score);
console.log("Reflection:", episode.episode.reflection);
```

## Integration with Learning Cycle

When you call `complete_episode`, the system automatically:

1. **Calculates Reward Score** (0.0 - 1.0)
   - Based on outcome, efficiency, and quality metrics
   - Success outcomes receive higher rewards
   - Failure outcomes help identify areas for improvement

2. **Generates Reflection**
   - AI-powered analysis of what worked and what didn't
   - Identifies successes, challenges, and learnings
   - Suggests improvements for future tasks

3. **Extracts Patterns**
   - Identifies recurring sequences and behaviors
   - Builds knowledge base for future predictions
   - Improves recommendation quality over time

## Best Practices

### 1. Granular Step Logging
Log meaningful steps that represent discrete actions:
```javascript
// Good: Clear, actionable steps
add_episode_step({ tool: "compiler", action: "Compiling main.rs" })
add_episode_step({ tool: "test_runner", action: "Running unit tests" })

// Avoid: Too vague
add_episode_step({ tool: "system", action: "Working on it" })
```

### 2. Include Latency Metrics
Track execution time for performance analysis:
```javascript
const start = Date.now();
// ... perform action ...
const latency_ms = Date.now() - start;

add_episode_step({
  ...,
  latency_ms: latency_ms
});
```

### 3. Use Appropriate Outcome Types
- **Success**: All goals achieved, no issues
- **Partial Success**: Some goals achieved, some failed (be specific in `completed`/`failed` arrays)
- **Failure**: Could not complete the task (include detailed error information)

### 4. Add Meaningful Context
Use tags and complexity levels to improve pattern matching:
```javascript
create_episode({
  ...,
  tags: ["authentication", "jwt", "security", "api"],
  complexity: "complex"  // Helps prioritize and allocate resources
});
```

### 5. Clean Up Test Episodes
Delete test or experimental episodes to keep data clean:
```javascript
delete_episode({
  episode_id: testEpisodeId,
  confirm: true
});
```

## Error Handling

All tools return structured errors:

```json
{
  "error": "Missing required field: task_description",
  "code": "VALIDATION_ERROR"
}
```

Common error scenarios:
- **Invalid UUID**: Episode ID format incorrect
- **Missing fields**: Required parameters not provided
- **Invalid enum**: Unknown task_type or outcome_type
- **Episode not found**: Episode doesn't exist or was deleted
- **Confirmation required**: Delete without `confirm: true`

## Performance

| Operation | Target Latency | Typical Latency |
|-----------|----------------|-----------------|
| create_episode | < 50ms | ~2.5µs |
| add_episode_step | < 20ms | ~1.1µs |
| complete_episode | < 500ms | ~3.8µs |
| get_episode | < 100ms | ~10µs |
| get_episode_timeline | < 100ms | ~15µs |
| delete_episode | < 50ms | ~5µs |

*Note: complete_episode triggers additional async learning processes that don't block the response.*

## Security Considerations

- **UUID Validation**: All episode IDs are validated as proper UUIDs
- **Delete Confirmation**: Requires explicit `confirm: true` to prevent accidental deletion
- **Input Sanitization**: All string inputs are sanitized before storage
- **No Code Execution**: These tools only manage data, no code execution capabilities

## See Also

- [Batch Operations](BATCH_OPERATIONS.md) - Bulk episode operations
- [MCP Protocol](README.md) - General MCP server documentation
- [Pattern Search](../memory-core/PATTERN_SEARCH_FEATURE.md) - Pattern analysis features
- [Episode Management](../memory-core/EPISODE_MANAGEMENT.md) - Core episode APIs
