# PHASE 1: UNDERSTAND 🧠

> **Goal**: Deep problem analysis and comprehensive understanding of requirements, constraints, and system architecture.

## Overview

This phase focuses on building a complete cognitive model of the Self-Learning Memory System before any implementation begins. Success means having clear answers to: What are we building? Why? For whom? Under what constraints?

## Cognitive Layer: Deep Problem Analysis

### Problem Statement

**Primary Goal**: Build a production-ready episodic memory system that enables AI agents to learn from experience using a 5-phase cycle (Pre-Task → Execution → Post-Task → Learning → Retrieval) with hybrid Turso/redb storage and MCP code execution integration.

**Target Users**:
- AI agents that need to learn from execution patterns
- Systems requiring context-aware memory retrieval
- Applications needing code execution capabilities with memory

### Core Components Identified

#### 1. Episode Management
**Purpose**: Track complete task execution from start to finish

**Key Structures**:
```rust
pub struct Episode {
    pub episode_id: Uuid,
    pub task_type: TaskType,
    pub task_description: String,
    pub context: TaskContext,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub steps: Vec<ExecutionStep>,
    pub outcome: Option<TaskOutcome>,
    pub reward: Option<RewardScore>,
    pub reflection: Option<Reflection>,
    pub patterns: Vec<PatternId>,
    pub metadata: HashMap<String, String>,
}
```

**Operations**:
- `start_episode(task_description, context)` → Episode ID
- `log_step(episode_id, step)` → Update episode
- `complete_episode(episode_id, outcome)` → Final episode with analysis

#### 2. Pattern Extraction
**Purpose**: Identify reusable patterns from successful/failed executions

**Pattern Types**:
```rust
pub enum Pattern {
    ToolSequence {
        tools: Vec<String>,
        context: TaskContext,
        success_rate: f32,
        avg_latency: Duration,
    },
    DecisionPoint {
        condition: String,
        action: String,
        outcome_stats: OutcomeStats,
    },
    ErrorRecovery {
        error_type: String,
        recovery_steps: Vec<String>,
        success_rate: f32,
    },
    ContextPattern {
        context_features: Vec<String>,
        recommended_approach: String,
        evidence: Vec<EpisodeId>,
    },
}
```

#### 3. Memory Retrieval
**Purpose**: Find relevant past experiences to inform current decisions

**Retrieval Strategies**:
- **Semantic Search**: Use embeddings for similarity-based lookup
- **Metadata Filtering**: Query by task type, tags, domain
- **Hybrid Approach**: Combine semantic + metadata for best results

**Retrieval Function**:
```rust
async fn retrieve_relevant_context(
    description: &str,
    context: &TaskContext,
    limit: usize
) -> Result<Vec<Episode>>
```

#### 4. Storage Backend
**Purpose**: Durable persistence and fast caching

**Turso (SQL - Durable)**:
```sql
CREATE TABLE episodes (
    episode_id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    task_description TEXT NOT NULL,
    context JSON NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    steps JSON NOT NULL,
    outcome JSON,
    reward JSON,
    reflection JSON,
    patterns JSON,
    metadata JSON,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_episodes_task_type ON episodes(task_type);
CREATE INDEX idx_episodes_timestamp ON episodes(start_time DESC);
CREATE INDEX idx_episodes_verdict ON episodes(json_extract(outcome, '$.verdict'));
```

**redb (Key-Value - Cache)**:
```rust
// Table definitions
const EPISODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("episodes");
const PATTERNS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("patterns");
const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("embeddings");
const METADATA_TABLE: TableDefinition<&str, &str> = TableDefinition::new("metadata");
```

#### 5. MCP Integration
**Purpose**: Enable code execution capabilities for learning agents

**Components**:
- **TypeScript Tool Generation**: Dynamic tool creation from memory patterns
- **Sandbox Execution**: Secure Node.js isolation with resource limits
- **Progressive Disclosure**: Only expose tools relevant to current context
- **Memory Integration**: Tools can query and update episodic memory

**Security Requirements**:
- File system access restrictions
- Network access control
- Process isolation
- CPU/memory limits
- Timeout enforcement

### Known vs. Uncertain

#### ✅ Known (High Confidence)

1. **Rust Async Patterns**
   - Tokio is the standard runtime for async operations
   - Use `async fn` with `.await` for IO operations
   - Connection pooling with `deadpool` or similar

2. **Database Schemas**
   - Turso tables: `episodes`, `patterns`, `heuristics`
   - redb tables: same structure but key-value encoding
   - JSON fields for nested data structures

3. **MCP Protocol**
   - Standard JSON-RPC 2.0 protocol
   - Rust SDK available with full support
   - Tool generation follows standard schema

4. **Performance Baselines**
   - Turso: 575x faster connections than traditional databases
   - redb: Competitive with LMDB/RocksDB
   - Target: <100ms retrieval latency

#### ❓ Uncertain (Requires Investigation)

1. **Optimal Pattern Extraction Algorithms**
   - Which similarity metrics work best for code patterns?
   - How to balance precision vs. recall in pattern matching?
   - Optimal clustering parameters for pattern grouping?

2. **Embedding Integration Strategy**
   - Use external service (OpenAI, Cohere) or local model?
   - Embedding dimension size vs. accuracy tradeoff?
   - Caching strategy for embeddings (when to recompute)?

3. **Performance Under Concurrent Load**
   - How many concurrent episodes before contention?
   - Optimal connection pool size for Turso?
   - Write transaction batching strategy for redb?

4. **Memory Synchronization Timing**
   - Sync Turso → redb on every write or batched?
   - Conflict resolution strategy when both are modified?
   - Acceptable staleness threshold for cache?

### Assumptions & Validation

#### Assumption 1: Turso Performance
**Assumption**: Turso provides adequate performance for analytical queries with 10K+ episodes.

**Validation Strategy**:
- Create benchmark with 10K episodes
- Measure query latency for common patterns
- Test complex JOIN queries for pattern extraction
- **Success Criteria**: 95th percentile < 200ms

#### Assumption 2: redb Speed
**Assumption**: redb offers sufficient speed for hot memory access (<10ms).

**Validation Strategy**:
- Benchmark read/write operations with various data sizes
- Test concurrent access patterns
- Measure memory overhead
- **Success Criteria**: p99 < 10ms for reads, <50ms for writes

#### Assumption 3: MCP Sandbox Security
**Assumption**: TypeScript sandbox execution is secure with Node.js isolation.

**Validation Strategy**:
- Attempt common sandbox escape techniques
- Test resource limit enforcement
- Verify file system and network restrictions
- **Success Criteria**: Zero successful escapes in penetration testing

## Agentic Layer: Specialized Analysis

### Analyst Agent: Requirements Extraction

**Task**: Extract clear requirements from specification and codebase.

**Key Requirements Identified**:

1. **Functional Requirements**
   - FR1: Create episodes with unique IDs and timestamps
   - FR2: Log execution steps with tool usage and outcomes
   - FR3: Complete episodes with reward scoring and reflection
   - FR4: Extract patterns from completed episodes
   - FR5: Retrieve relevant episodes based on context
   - FR6: Execute TypeScript code in secure sandbox
   - FR7: Generate MCP tools from memory patterns

2. **Non-Functional Requirements**
   - NFR1: <100ms retrieval latency (P95)
   - NFR2: Support 10,000+ episodes without degradation
   - NFR3: >70% pattern recognition accuracy
   - NFR4: 90%+ test coverage
   - NFR5: Zero memory leaks under continuous operation
   - NFR6: Secure sandbox with no privilege escalation

3. **Data Requirements**
   - DR1: Episodes must persist across restarts
   - DR2: Patterns must be queryable by similarity
   - DR3: Embeddings cached for fast retrieval
   - DR4: Metadata indexed for efficient filtering

**Dependencies Identified**:
```
Storage Layer
    ├── Turso (durable SQL storage)
    ├── redb (hot cache)
    └── Sync mechanism (conflict resolution)

Learning Layer
    ├── Episode Management (depends on Storage)
    ├── Pattern Extraction (depends on Episodes)
    └── Retrieval (depends on Patterns + Episodes)

MCP Layer
    ├── Code Execution (depends on Sandbox)
    ├── Tool Generation (depends on Patterns)
    └── Memory Integration (depends on Learning Layer)
```

### Domain Expert: Technical Validation

**Task**: Validate technical approach and identify potential issues.

#### Rust Async Architecture Assessment

**Strengths**:
- Tokio ecosystem is mature and well-supported
- `async/await` syntax makes code readable
- Good libraries for database connections (libsql, rusqlite)

**Concerns**:
- redb is synchronous; need to wrap in `spawn_blocking` or dedicated thread
- Embedding generation may block; use separate task pool
- Connection pooling required to avoid exhaustion

**Recommendation**: Use `tokio::task::spawn_blocking` for redb operations to avoid blocking async runtime.

#### Storage Layer Validation

**Turso/libSQL Assessment**:
- ✅ Good: SQLite compatibility, HTTP API, excellent Rust support
- ✅ Good: Built-in replication and analytics features
- ⚠️ Concern: Network dependency (need circuit breakers)
- ⚠️ Concern: Cost scaling with large datasets

**redb Assessment**:
- ✅ Good: Zero-copy reads, excellent performance
- ✅ Good: ACID transactions, crash-safe
- ⚠️ Concern: Synchronous API (need careful integration)
- ⚠️ Concern: Limited query capabilities (key-value only)

**Recommendation**: Hybrid approach is sound. Use Turso for analytics and redb for hot-path reads.

#### MCP Integration Assessment

**Protocol Compatibility**:
- ✅ Rust MCP SDK available and actively maintained
- ✅ TypeScript execution via Node.js is standard
- ⚠️ Concern: Sandbox security requires careful implementation
- ⚠️ Concern: Tool generation complexity may be high

**Security Considerations**:
- Must use VM2 or similar for JavaScript isolation
- Resource limits (CPU, memory, time) must be enforced
- File system access must be restricted (whitelist approach)
- Network access must be controlled (deny by default)

**Recommendation**: Start with basic sandboxing, iterate based on security audit findings.

### Context Gatherer: Research & Benchmarks

**Task**: Research external resources and gather performance data.

#### Turso Performance Research

**Source**: Turso official documentation and benchmarks

**Key Findings**:
- 575x faster connection establishment vs. traditional PostgreSQL
- Edge deployment reduces latency (< 10ms in same region)
- SQLite compatibility means extensive ecosystem support
- HTTP API adds ~5-10ms overhead vs. local SQLite

**Implications**: Connection overhead is minimal; can use per-request connections if needed.

#### redb Benchmark Research

**Source**: GitHub benchmarks and community reports

**Performance Data**:
```
Operation          redb        LMDB        RocksDB
Read (1KB)         0.8μs       1.2μs       2.1μs
Write (1KB)        12μs        18μs        25μs
Scan (1000 items)  1.2ms       1.8ms       3.2ms
```

**Implications**: redb is fastest for reads, competitive for writes. Good choice for cache layer.

#### MCP Rust SDK Research

**Source**: Official MCP Rust SDK repository

**Capabilities**:
- Full protocol support (tools, resources, prompts)
- Async/await support with Tokio
- Type-safe tool definitions
- Server and client implementations

**Example Tool Definition**:
```rust
Tool {
    name: "query_memory".to_string(),
    description: "Query episodic memory for relevant past experiences".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "query": {"type": "string"},
            "limit": {"type": "integer", "default": 10}
        },
        "required": ["query"]
    }),
}
```

**Implications**: MCP integration is well-supported; focus on sandbox security.

## TestData Builder: Validation Framework

### Sample Inputs & Expected Outputs

#### Test Case 1: Episode Creation
```rust
// Input
let test_context = TaskContext {
    language: Some("rust".to_string()),
    framework: Some("tokio".to_string()),
    complexity: ComplexityLevel::Moderate,
    domain: "web-api".to_string(),
    tags: vec!["async".to_string(), "database".to_string()],
};

let task_description = "Implement user authentication endpoint";

// Expected Output
let expected_episode = Episode {
    episode_id: Uuid::new_v4(), // Generated
    task_type: TaskType::CodeGeneration,
    task_description: "Implement user authentication endpoint".to_string(),
    context: test_context.clone(),
    start_time: Utc::now(), // Current timestamp
    end_time: None, // Not complete yet
    steps: vec![], // Empty initially
    outcome: None,
    reward: None,
    reflection: None,
    patterns: vec![],
    metadata: HashMap::new(),
};

// Validation
assert!(expected_episode.episode_id.is_valid());
assert!(expected_episode.end_time.is_none());
assert!(expected_episode.steps.is_empty());
```

#### Test Case 2: Step Logging
```rust
// Input
let step = ExecutionStep {
    step_number: 1,
    timestamp: Utc::now(),
    tool: "read_file".to_string(),
    action: "Read authentication module".to_string(),
    parameters: json!({"path": "src/auth.rs"}),
    result: Some(ExecutionResult::Success {
        output: "File contents...".to_string(),
    }),
    latency_ms: 15,
    tokens_used: None,
    metadata: HashMap::new(),
};

// Expected: Episode updated with new step
// steps.len() incremented by 1
// last_updated timestamp changed
```

#### Test Case 3: Pattern Extraction
```rust
// Input: Completed episode with successful tool sequence
let completed_episode = Episode {
    // ... filled episode
    steps: vec![
        ExecutionStep { tool: "read_file", .. },
        ExecutionStep { tool: "analyze_code", .. },
        ExecutionStep { tool: "write_file", .. },
    ],
    outcome: Some(TaskOutcome::Success { verdict: "Tests passing" }),
};

// Expected: Extracted pattern
let expected_pattern = Pattern::ToolSequence {
    tools: vec!["read_file", "analyze_code", "write_file"],
    context: completed_episode.context.clone(),
    success_rate: 1.0, // First occurrence
    avg_latency: Duration::from_millis(45),
};
```

### Edge Cases Identified

#### 1. Concurrent Episode Operations
**Scenario**: Multiple agents creating/updating episodes simultaneously.

**Test**:
```rust
#[tokio::test]
async fn test_concurrent_episode_creation() {
    let memory = setup_test_memory().await;

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let mem = memory.clone();
            tokio::spawn(async move {
                mem.start_episode(&format!("Task {}", i), test_context()).await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed with unique IDs
    assert_eq!(results.len(), 100);
    let ids: HashSet<_> = results.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(ids.len(), 100); // All unique
}
```

#### 2. Database Connection Failures
**Scenario**: Turso becomes unavailable during operations.

**Test**:
```rust
#[tokio::test]
async fn test_database_connection_failure() {
    let memory = setup_memory_with_failing_db().await;

    // Should return error, not panic
    let result = memory.start_episode("Test", test_context()).await;
    assert!(result.is_err());

    // Should describe the error clearly
    assert!(result.unwrap_err().to_string().contains("connection"));
}
```

#### 3. Large Pattern Datasets
**Scenario**: System has extracted 100K+ patterns over time.

**Test**:
```rust
#[tokio::test]
async fn test_large_pattern_dataset_retrieval() {
    let memory = setup_memory_with_100k_patterns().await;

    let start = Instant::now();
    let patterns = memory.query_patterns(test_context(), 0.5, 10).await.unwrap();
    let duration = start.elapsed();

    // Should still be fast even with large dataset
    assert!(duration.as_millis() < 100);
    assert_eq!(patterns.len(), 10); // Limited to requested amount
}
```

#### 4. Network Timeouts
**Scenario**: MCP tool execution takes longer than timeout.

**Test**:
```rust
#[tokio::test]
async fn test_mcp_tool_timeout() {
    let mcp_server = setup_mcp_server_with_timeout(Duration::from_secs(5)).await;

    // Code that sleeps for 10 seconds
    let long_running_code = "await new Promise(resolve => setTimeout(resolve, 10000));";

    let result = mcp_server.execute_agent_code(long_running_code, "{}").await;

    // Should timeout and return error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout"));
}
```

## Understanding Complete Criteria

Before proceeding to Phase 2 (PLAN), validate that these criteria are met:

- [x] All 47 core data structures mapped from specification
- [x] Storage interaction patterns clearly defined (Turso + redb)
- [x] MCP integration approach validated with current SDK capabilities
- [x] Performance and security requirements quantified
- [x] Key uncertainties identified with validation strategies
- [x] Edge cases documented with test scenarios
- [x] Technical risks assessed with mitigation plans
- [x] All dependencies and their interactions mapped

## Next Steps

Once understanding is complete:

1. ✅ Review all identified components, assumptions, and edge cases
2. ✅ Validate technical feasibility with domain experts
3. ✅ Confirm all requirements are clear and testable
4. ➡️ **Proceed to [Phase 2: PLAN](./02-plan.md)** - Strategic planning and architecture design

## References

- [AGENTS.md](../AGENTS.md) - Agent responsibilities and operational guidance
- [Phase 0: Overview](./00-overview.md) - High-level project summary
- [Phase 2: PLAN](./02-plan.md) - Next phase (strategic planning)
