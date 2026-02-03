# API Reference

**Version**: v0.1.13  
**Last Updated**: 2026-02-01  
**Protocol**: MCP (Model Context Protocol) v2024-11  

---

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [Base URL](#base-url)
- [Request/Response Format](#requestresponse-format)
- [Error Codes](#error-codes)
- [Core Tools](#core-tools)
- [Episode Management Tools](#episode-management-tools)
- [Pattern Tools](#pattern-tools)
- [Batch Operations](#batch-operations)
- [Tag Management Tools](#tag-management-tools)
- [Relationship Tools](#relationship-tools)
- [Embedding Tools](#embedding-tools)
- [Monitoring Tools](#monitoring-tools)
- [Code Execution](#code-execution)
- [Rate Limiting](#rate-limiting)

---

## Overview

The Memory MCP Server exposes a comprehensive set of tools for episodic memory management, pattern analysis, and secure code execution. All tools follow the MCP protocol specification and communicate via JSON-RPC 2.0.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Application                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ JSON-RPC 2.0
┌─────────────────────────────────────────────────────────────┐
│                    Memory MCP Server                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  MCP Tools  │  │  Episode    │  │  Pattern    │         │
│  │  Interface  │  │  Lifecycle  │  │  Analysis   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  Turso Storage  │  │  Redb Cache     │  │  WASM Sandbox   │
│  (Persistent)   │  │  (Fast Access)  │  │  (Secure Exec)  │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## Authentication

The MCP server supports OAuth 2.0 authentication for production deployments.

### Environment Configuration

```bash
# OAuth Configuration (optional, for authenticated deployments)
export OAUTH_ENABLED=true
export OAUTH_CLIENT_ID=your-client-id
export OAUTH_CLIENT_SECRET=your-client-secret
export OAUTH_TOKEN_URL=https://auth.example.com/token
```

### Request Headers

```http
POST /mcp HTTP/1.1
Host: localhost:3000
Content-Type: application/json
Authorization: Bearer <access_token>  # If OAuth enabled
X-Client-ID: my-service              # For rate limiting
```

---

## Base URL

### Local Development

```
http://localhost:3000
```

### Production

```
https://your-memory-server.example.com
```

---

## Request/Response Format

All API calls use JSON-RPC 2.0 format.

### Request Structure

```json
{
  "jsonrpc": "2.0",
  "id": 123,
  "method": "tools/call",
  "params": {
    "name": "query_memory",
    "arguments": {
      "query": "How to implement authentication",
      "domain": "web-api",
      "task_type": "code_generation",
      "limit": 10
    }
  }
}
```

### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 123,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"episodes\": [...], \"total\": 5}"
      }
    ],
    "isError": false
  }
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 123,
  "error": {
    "code": -32000,
    "message": "Invalid parameters",
    "data": {
      "details": "Missing required field: domain"
    }
  }
}
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| `-32700` | Parse Error | Invalid JSON received |
| `-32600` | Invalid Request | JSON-RPC request invalid |
| `-32601` | Method Not Found | Method does not exist |
| `-32602` | Invalid Params | Invalid method parameters |
| `-32603` | Internal Error | Internal server error |
| `-32000` | Server Error | Generic server error |
| `-32001` | Rate Limit Exceeded | Too many requests |
| `-32002` | Authentication Failed | Invalid credentials |
| `-32003` | Authorization Failed | Insufficient permissions |
| `-32004` | Resource Not Found | Episode/pattern not found |
| `-32005` | Validation Error | Input validation failed |
| `-32006` | Storage Error | Database operation failed |
| `-32007` | Sandbox Error | Code execution failed |
| `-32008` | Timeout Error | Operation timed out |

### Error Response Examples

#### Rate Limit Exceeded

```json
{
  "jsonrpc": "2.0",
  "id": 123,
  "error": {
    "code": -32001,
    "message": "Rate limit exceeded",
    "data": {
      "retry_after": 5,
      "limit": 100,
      "remaining": 0,
      "reset": 1706781050
    }
  }
}
```

#### Validation Error

```json
{
  "jsonrpc": "2.0",
  "id": 123,
  "error": {
    "code": -32005,
    "message": "Validation error",
    "data": {
      "field": "domain",
      "issue": "Field is required",
      "value": null
    }
  }
}
```

---

## Core Tools

### query_memory

Query episodic memory for relevant past experiences.

**Method**: `tools/call`  
**Tool Name**: `query_memory`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Search query describing the task |
| `domain` | string | Yes | - | Task domain (e.g., 'web-api') |
| `task_type` | string | No | - | Type: code_generation, debugging, refactoring, testing, analysis, documentation |
| `limit` | integer | No | 10 | Maximum results |
| `sort` | string | No | 'relevance' | Sort: relevance, newest, oldest, duration, success |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_memory",
    "arguments": {
      "query": "How to implement JWT authentication",
      "domain": "web-api",
      "task_type": "code_generation",
      "limit": 5
    }
  }
}
```

#### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"episodes\": [\n    {\n      \"id\": \"550e8400-e29b-41d4-a716-446655440000\",\n      \"context\": \"Implement JWT auth\",\n      \"reward_score\": 0.95,\n      \"duration_ms\": 45000,\n      \"steps\": [...]\n    }\n  ],\n  \"total\": 5\n}"
      }
    ],
    "isError": false
  }
}
```

---

## Episode Management Tools

### create_episode

Create a new episode to track task execution.

**Method**: `tools/call`  
**Tool Name**: `create_episode`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `task_description` | string | Yes | Clear description of the task |
| `domain` | string | Yes | Task domain |
| `task_type` | string | Yes | code_generation, debugging, refactoring, testing, analysis, documentation |
| `language` | string | No | Programming language |
| `framework` | string | No | Framework name |
| `tags` | array | No | Context tags |
| `complexity` | string | No | simple, moderate, complex |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "create_episode",
    "arguments": {
      "task_description": "Implement user authentication",
      "domain": "web-api",
      "task_type": "code_generation",
      "language": "rust",
      "tags": ["auth", "jwt"],
      "complexity": "moderate"
    }
  }
}
```

#### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"episode_id\": \"550e8400-e29b-41d4-a716-446655440001\",\n  \"created_at\": \"2026-02-01T10:30:45Z\",\n  \"status\": \"active\"\n}"
      }
    ],
    "isError": false
  }
}
```

---

### add_episode_step

Add an execution step to an ongoing episode.

**Method**: `tools/call`  
**Tool Name**: `add_episode_step`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |
| `step_number` | integer | Yes | Sequential step number |
| `tool` | string | Yes | Tool/component name |
| `action` | string | Yes | Description of action |
| `parameters` | object | No | Parameters used |
| `result` | object | No | Result with type, output, message |
| `latency_ms` | integer | No | Execution time |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add_episode_step",
    "arguments": {
      "episode_id": "550e8400-e29b-41d4-a716-446655440001",
      "step_number": 1,
      "tool": "analyzer",
      "action": "Analyze authentication requirements",
      "result": {
        "type": "success",
        "output": "Requirements analyzed successfully"
      },
      "latency_ms": 150
    }
  }
}
```

---

### complete_episode

Complete an episode and trigger the learning cycle.

**Method**: `tools/call`  
**Tool Name**: `complete_episode`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |
| `outcome_type` | string | Yes | success, partial_success, failure |
| `verdict` | string | Conditional | Outcome description (for success/partial_success) |
| `artifacts` | array | No | Artifact names (for success) |
| `completed` | array | Conditional | Completed items (for partial_success) |
| `failed` | array | Conditional | Failed items (for partial_success) |
| `reason` | string | Conditional | Failure reason (for failure) |
| `error_details` | string | No | Error details (for failure) |

#### Example Request (Success)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "complete_episode",
    "arguments": {
      "episode_id": "550e8400-e29b-41d4-a716-446655440001",
      "outcome_type": "success",
      "verdict": "Authentication implemented successfully",
      "artifacts": ["auth.rs", "middleware.rs"]
    }
  }
}
```

#### Example Request (Failure)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "complete_episode",
    "arguments": {
      "episode_id": "550e8400-e29b-41d4-a716-446655440001",
      "outcome_type": "failure",
      "reason": "Database connection failed",
      "error_details": "Connection timeout after 30s"
    }
  }
}
```

---

### get_episode

Get complete details of an episode.

**Method**: `tools/call`  
**Tool Name**: `get_episode`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |

#### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"id\": \"550e8400-e29b-41d4-a716-446655440001\",\n  \"context\": \"Implement user authentication\",\n  \"domain\": \"web-api\",\n  \"task_type\": \"code_generation\",\n  \"status\": \"completed\",\n  \"outcome\": {\n    \"type\": \"success\",\n    \"verdict\": \"Authentication implemented successfully\"\n  },\n  \"steps\": [...],\n  \"patterns\": [...],\n  \"reflection\": {...},\n  \"reward_score\": 0.92\n}"
      }
    ],
    "isError": false
  }
}
```

---

### delete_episode

Delete an episode permanently.

**Method**: `tools/call`  
**Tool Name**: `delete_episode`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |
| `confirm` | boolean | Yes | Must be true to confirm |

---

### get_episode_timeline

Get chronological timeline of episode steps.

**Method**: `tools/call`  
**Tool Name**: `get_episode_timeline`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |

---

### bulk_episodes

Retrieve multiple episodes by IDs.

**Method**: `tools/call`  
**Tool Name**: `bulk_episodes`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_ids` | array | Yes | Array of episode UUIDs |

---

## Pattern Tools

### analyze_patterns

Analyze patterns from past episodes.

**Method**: `tools/call`  
**Tool Name**: `analyze_patterns`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `task_type` | string | Yes | - | Type of task to analyze |
| `min_success_rate` | number | No | 0.7 | Minimum success rate (0.0-1.0) |
| `limit` | integer | No | 20 | Maximum patterns to return |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "analyze_patterns",
    "arguments": {
      "task_type": "code_generation",
      "min_success_rate": 0.8,
      "limit": 10
    }
  }
}
```

---

### search_patterns

Search for patterns semantically similar to a query.

**Method**: `tools/call`  
**Tool Name**: `search_patterns`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Natural language query |
| `domain` | string | Yes | - | Domain to search |
| `tags` | array | No | [] | Optional tags |
| `limit` | integer | No | 5 | Maximum results |
| `min_relevance` | number | No | 0.3 | Minimum relevance (0.0-1.0) |
| `filter_by_domain` | boolean | No | false | Filter by domain |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "tools/call",
  "params": {
    "name": "search_patterns",
    "arguments": {
      "query": "How to handle API rate limiting",
      "domain": "web-api",
      "tags": ["rest", "async"],
      "limit": 5,
      "min_relevance": 0.5
    }
  }
}
```

#### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"patterns\": [\n    {\n      \"id\": \"pattern-1\",\n      \"type\": \"ToolSequence\",\n      \"description\": \"Rate limiting with exponential backoff\",\n      \"relevance_score\": 0.89,\n      \"success_rate\": 0.92,\n      \"domain\": \"web-api\"\n    }\n  ],\n  \"total\": 3\n}"
      }
    ],
    "isError": false
  }
}
```

---

### recommend_patterns

Get pattern recommendations for a specific task.

**Method**: `tools/call`  
**Tool Name**: `recommend_patterns`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `task_description` | string | Yes | - | Task description |
| `domain` | string | Yes | - | Task domain |
| `tags` | array | No | [] | Optional tags |
| `limit` | integer | No | 3 | Maximum recommendations |

---

### advanced_pattern_analysis

Statistical analysis and forecasting on time series data.

**Method**: `tools/call`  
**Tool Name**: `advanced_pattern_analysis`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `time_series_data` | object | Yes | - | Variable name -> array of values |
| `analysis_type` | string | Yes | - | statistical, predictive, comprehensive |
| `config` | object | No | - | Analysis configuration |

#### Config Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `forecast_horizon` | integer | 10 | Steps to forecast |
| `anomaly_sensitivity` | number | 0.5 | Anomaly detection sensitivity |
| `enable_causal_inference` | boolean | true | Enable causal analysis |
| `significance_level` | number | 0.05 | Statistical significance |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "tools/call",
  "params": {
    "name": "advanced_pattern_analysis",
    "arguments": {
      "time_series_data": {
        "episode_duration": [100, 120, 110, 130, 125, 140, 135, 150, 145, 160]
      },
      "analysis_type": "comprehensive",
      "config": {
        "forecast_horizon": 5,
        "anomaly_sensitivity": 0.6,
        "enable_causal_inference": true
      }
    }
  }
}
```

---

## Batch Operations

### batch_query_episodes

Query multiple episodes with filtering and aggregation.

**Method**: `tools/call`  
**Tool Name**: `batch_query_episodes`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `episode_ids` | array | No | - | Specific episode IDs |
| `filter` | object | No | - | Filter criteria |
| `include_steps` | boolean | No | true | Include steps |
| `include_patterns` | boolean | No | false | Include patterns |
| `include_reflections` | boolean | No | false | Include reflections |
| `limit` | integer | No | 100 | Maximum episodes |
| `offset` | integer | No | 0 | Pagination offset |
| `aggregate_stats` | boolean | No | true | Compute statistics |

#### Filter Object

| Field | Type | Description |
|-------|------|-------------|
| `domain` | string | Filter by domain |
| `task_type` | string | Filter by task type |
| `tags` | array | Filter by tags |
| `success_only` | boolean | Only successful episodes |

---

### batch_pattern_analysis

Analyze patterns across multiple episodes.

**Method**: `tools/call`  
**Tool Name**: `batch_pattern_analysis`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `domain` | string | Yes | - | Task domain |
| `task_type` | string | No | - | Specific task type |
| `time_range` | object | No | - | Start/end ISO8601 dates |
| `min_episodes` | integer | No | 3 | Minimum episodes for pattern |
| `min_success_rate` | number | No | 0.6 | Minimum success rate |
| `include_anti_patterns` | boolean | No | true | Include low-success patterns |
| `limit` | integer | No | 50 | Maximum patterns |

---

### batch_compare_episodes

Compare multiple episodes.

**Method**: `tools/call`  
**Tool Name**: `batch_compare_episodes`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `episode_ids` | array | Yes | - | 2-10 episode UUIDs |
| `compare_metrics` | array | No | ["duration", "reward_score", "step_count"] | Metrics to compare |
| `compare_approaches` | boolean | No | true | Compare approaches |
| `generate_insights` | boolean | No | true | Generate AI insights |

---

## Tag Management Tools

### add_episode_tags

Add tags to an episode.

**Method**: `tools/call`  
**Tool Name**: `add_episode_tags`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |
| `tags` | array | Yes | Tags to add |

---

### remove_episode_tags

Remove tags from an episode.

**Method**: `tools/call`  
**Tool Name**: `remove_episode_tags`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |
| `tags` | array | Yes | Tags to remove |

---

### set_episode_tags

Set/replace all tags on an episode.

**Method**: `tools/call`  
**Tool Name**: `set_episode_tags`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |
| `tags` | array | Yes | New tags (replaces all) |

---

### get_episode_tags

Get tags for an episode.

**Method**: `tools/call`  
**Tool Name**: `get_episode_tags`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_id` | string | Yes | Episode UUID |

---

### search_episodes_by_tags

Search episodes by tags.

**Method**: `tools/call`  
**Tool Name**: `search_episodes_by_tags`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `tags` | array | Yes | - | Tags to search |
| `require_all` | boolean | No | false | AND logic (true) or OR (false) |
| `limit` | integer | No | 100 | Maximum results |

---

## Relationship Tools

### add_episode_relationship

Add a relationship between two episodes.

**Method**: `tools/call`  
**Tool Name**: `add_episode_relationship`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `from_episode_id` | string | Yes | Source episode UUID |
| `to_episode_id` | string | Yes | Target episode UUID |
| `relationship_type` | string | Yes | parent_child, depends_on, follows, related_to, blocks, duplicates, references |
| `reason` | string | No | Explanation |
| `priority` | integer | No | Priority 1-10 |
| `created_by` | string | No | Creator identifier |

---

### remove_episode_relationship

Remove a relationship.

**Method**: `tools/call`  
**Tool Name**: `remove_episode_relationship`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `relationship_id` | string | Yes | Relationship UUID |

---

### get_episode_relationships

Get relationships for an episode.

**Method**: `tools/call`  
**Tool Name**: `get_episode_relationships`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `episode_id` | string | Yes | - | Episode UUID |
| `direction` | string | No | 'both' | outgoing, incoming, both |
| `relationship_type` | string | No | - | Filter by type |

---

### find_related_episodes

Find episodes related to a given episode.

**Method**: `tools/call`  
**Tool Name**: `find_related_episodes`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `episode_id` | string | Yes | - | Episode UUID |
| `relationship_type` | string | No | - | Filter by type |
| `limit` | integer | No | 10 | Maximum results |
| `include_metadata` | boolean | No | false | Include metadata |

---

### check_relationship_exists

Check if a relationship exists.

**Method**: `tools/call`  
**Tool Name**: `check_relationship_exists`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `from_episode_id` | string | Yes | Source episode UUID |
| `to_episode_id` | string | Yes | Target episode UUID |
| `relationship_type` | string | Yes | Relationship type |

---

### get_dependency_graph

Get relationship graph for visualization.

**Method**: `tools/call`  
**Tool Name**: `get_dependency_graph`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `episode_id` | string | Yes | - | Root episode UUID |
| `depth` | integer | No | 2 | Maximum depth (1-5) |
| `format` | string | No | 'json' | json, dot |

---

### validate_no_cycles

Check if adding a relationship would create a cycle.

**Method**: `tools/call`  
**Tool Name**: `validate_no_cycles`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `from_episode_id` | string | Yes | Source episode UUID |
| `to_episode_id` | string | Yes | Target episode UUID |
| `relationship_type` | string | Yes | Relationship type |

---

### get_topological_order

Get topological ordering of episodes.

**Method**: `tools/call`  
**Tool Name**: `get_topological_order`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `episode_ids` | array | Yes | Array of episode UUIDs |

---

## Embedding Tools

### configure_embeddings

Configure semantic embedding provider.

**Method**: `tools/call`  
**Tool Name**: `configure_embeddings`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider` | string | Yes | openai, local, mistral, azure, cohere |
| `model` | string | No | Model name |
| `api_key_env` | string | No | API key environment variable |
| `base_url` | string | No | Custom base URL |
| `batch_size` | integer | No | Batch size (1-2048) |
| `similarity_threshold` | number | No | Min similarity (0.0-1.0) |

---

### query_semantic_memory

Search episodic memory using semantic similarity.

**Method**: `tools/call`  
**Tool Name**: `query_semantic_memory`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Search query |
| `domain` | string | No | - | Filter by domain |
| `task_type` | string | No | - | Filter by task type |
| `limit` | integer | No | 10 | Maximum results |
| `similarity_threshold` | number | No | 0.7 | Min similarity |

---

### test_embeddings

Test embedding provider connectivity.

**Method**: `tools/call`  
**Tool Name**: `test_embeddings`

#### Parameters

None.

---

## Monitoring Tools

### health_check

Check server health status.

**Method**: `tools/call`  
**Tool Name**: `health_check`

#### Parameters

None.

#### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"status\": \"healthy\",\n  \"components\": {\n    \"storage\": \"healthy\",\n    \"cache\": \"healthy\",\n    \"embeddings\": \"healthy\"\n  },\n  \"timestamp\": \"2026-02-01T10:30:45Z\"\n}"
      }
    ],
    "isError": false
  }
}
```

---

### get_metrics

Get comprehensive monitoring metrics.

**Method**: `tools/call`  
**Tool Name**: `get_metrics`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `metric_type` | string | No | 'all' | all, performance, episodes, system |

---

### quality_metrics

Get memory quality metrics and noise reduction statistics.

**Method**: `tools/call`  
**Tool Name**: `quality_metrics`

#### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `time_range` | string | No | '7d' | 24h, 7d, 30d, 90d, all |
| `quality_threshold` | number | No | 0.7 | Quality threshold |
| `include_trends` | boolean | No | true | Include trend analysis |

---

## Code Execution

### execute_agent_code

Execute TypeScript/JavaScript in secure sandbox.

**Method**: `tools/call`  
**Tool Name**: `execute_agent_code`

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `code` | string | Yes | TypeScript/JavaScript code |
| `context` | object | Yes | Execution context |

#### Context Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `task` | string | Yes | Task description |
| `input` | object | Yes | Input data as JSON |

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "method": "tools/call",
  "params": {
    "name": "execute_agent_code",
    "arguments": {
      "code": "function process(data) { return data.map(x => x * 2); } process(input);",
      "context": {
        "task": "Double array values",
        "input": [1, 2, 3, 4, 5]
      }
    }
  }
}
```

#### Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "[2,4,6,8,10]"
      }
    ],
    "isError": false
  }
}
```

#### Sandbox Limits

| Resource | Default Limit |
|----------|---------------|
| Execution time | 5000ms |
| Memory | 128MB |
| CPU | 50% |
| Concurrent executions | 20 |

---

## Rate Limiting

The API implements token bucket rate limiting with separate limits for read and write operations.

### Default Limits

| Operation Type | RPS | Burst |
|----------------|-----|-------|
| Read | 100 | 150 |
| Write | 20 | 30 |

### Response Headers

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1706781045
```

### Rate Limit Exceeded

```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1706781050
Retry-After: 5
```

### Operation Classification

**Read Operations** (higher limits):
- `health_check`
- `get_metrics`
- `quality_metrics`
- `query_memory`
- `search_patterns`
- `recommend_patterns`
- `get_episode`
- `get_episode_timeline`
- `bulk_episodes`
- `batch_query_episodes`
- `get_episode_tags`
- `search_episodes_by_tags`
- `get_episode_relationships`
- `find_related_episodes`
- `check_relationship_exists`
- `get_dependency_graph`
- `validate_no_cycles`
- `get_topological_order`

**Write Operations** (lower limits):
- `create_episode`
- `add_episode_step`
- `complete_episode`
- `delete_episode`
- `analyze_patterns`
- `advanced_pattern_analysis`
- `batch_pattern_analysis`
- `batch_compare_episodes`
- `add_episode_tags`
- `remove_episode_tags`
- `set_episode_tags`
- `add_episode_relationship`
- `remove_episode_relationship`
- `configure_embeddings`
- `execute_agent_code`

---

## SDK Examples

### Python

```python
import requests
import json

class MemoryMCPClient:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
        self.request_id = 0
    
    def call(self, tool_name, arguments):
        self.request_id += 1
        response = requests.post(
            f"{self.base_url}/mcp",
            json={
                "jsonrpc": "2.0",
                "id": self.request_id,
                "method": "tools/call",
                "params": {
                    "name": tool_name,
                    "arguments": arguments
                }
            }
        )
        return response.json()
    
    def create_episode(self, task, domain, task_type):
        return self.call("create_episode", {
            "task_description": task,
            "domain": domain,
            "task_type": task_type
        })
    
    def query_memory(self, query, domain, **kwargs):
        return self.call("query_memory", {
            "query": query,
            "domain": domain,
            **kwargs
        })

# Usage
client = MemoryMCPClient()
episode = client.create_episode(
    "Implement authentication",
    "web-api",
    "code_generation"
)
print(f"Created episode: {episode['result']['content'][0]['text']}")
```

### JavaScript/TypeScript

```typescript
interface MCPRequest {
  jsonrpc: "2.0";
  id: number;
  method: string;
  params: {
    name: string;
    arguments: Record<string, any>;
  };
}

class MemoryMCPClient {
  private baseUrl: string;
  private requestId: number = 0;

  constructor(baseUrl: string = "http://localhost:3000") {
    this.baseUrl = baseUrl;
  }

  async call(toolName: string, arguments_: Record<string, any>): Promise<any> {
    this.requestId++;
    const response = await fetch(`${this.baseUrl}/mcp`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: this.requestId,
        method: "tools/call",
        params: { name: toolName, arguments: arguments_ }
      })
    });
    return response.json();
  }

  async createEpisode(
    task: string,
    domain: string,
    taskType: string
  ): Promise<any> {
    return this.call("create_episode", {
      task_description: task,
      domain,
      task_type: taskType
    });
  }

  async queryMemory(
    query: string,
    domain: string,
    options: Record<string, any> = {}
  ): Promise<any> {
    return this.call("query_memory", { query, domain, ...options });
  }
}

// Usage
const client = new MemoryMCPClient();
const episode = await client.createEpisode(
  "Implement authentication",
  "web-api",
  "code_generation"
);
console.log("Created episode:", episode);
```

### Rust

```rust
use serde_json::json;

pub struct MemoryMCPClient {
    base_url: String,
    client: reqwest::Client,
    request_id: u64,
}

impl MemoryMCPClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            request_id: 0,
        }
    }

    pub async fn call(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        self.request_id += 1;
        let response = self
            .client
            .post(format!("{}/mcp", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": self.request_id,
                "method": "tools/call",
                "params": {
                    "name": tool_name,
                    "arguments": arguments
                }
            }))
            .send()
            .await?;
        Ok(response.json().await?)
    }

    pub async fn create_episode(
        &mut self,
        task: &str,
        domain: &str,
        task_type: &str,
    ) -> anyhow::Result<serde_json::Value> {
        self.call("create_episode", json!({
            "task_description": task,
            "domain": domain,
            "task_type": task_type
        })).await
    }
}

// Usage
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MemoryMCPClient::new("http://localhost:3000");
    let episode = client
        .create_episode("Implement auth", "web-api", "code_generation")
        .await?;
    println!("Created episode: {:?}", episode);
    Ok(())
}
```

---

## See Also

- [Security Operations Guide](./SECURITY_OPERATIONS.md)
- [Deployment Security Guide](./DEPLOYMENT_SECURITY.md)
- [Performance Tuning Guide](./PERFORMANCE_TUNING.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [MCP Protocol Specification](https://modelcontextprotocol.io)

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-01  
**Maintained By**: Development Team
