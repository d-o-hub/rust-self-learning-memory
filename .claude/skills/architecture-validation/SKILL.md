---
skill_name: architecture-validation
description: Validate architecture compliance with planned design decisions, patterns, and system constraints for the Rust self-learning memory project
version: 1.0.0
tags: [architecture, validation, compliance, design, project]
tools: [Read, Glob, Grep, Bash]
---

# Architecture Validation Skill

Systematically validate that the implemented architecture matches the planned architecture decisions, design patterns, and system constraints documented in the project plans.

## Purpose

Ensure the implementation adheres to:
- **Architectural decisions** from plans/02-plan.md
- **Component boundaries** and separation of concerns
- **Data flow patterns** (episode lifecycle, pattern extraction)
- **Storage architecture** (Hybrid Turso + redb)
- **Integration patterns** (MCP protocol, sandbox)
- **Performance targets** and resource constraints
- **Security architecture** and defense-in-depth

## Architecture Dimensions

### 1. System Architecture Overview

**Planned Architecture** (from plans/02-plan.md):
```
┌─────────────────────────────────────────────────────────┐
│              Self-Learning Memory System                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Learning   │  │    MCP       │  │   Storage    │ │
│  │     Core     │  │  Integration │  │    Layer     │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
│         │                 │                 │         │
│         └─────────────────┴─────────────────┘         │
│                           │                           │
│              ┌────────────┴────────────┐              │
│              ▼                         ▼              │
│    ┌──────────────────┐      ┌──────────────────┐    │
│    │ Turso (Durable)  │      │ redb (Cache)     │    │
│    │ - Episodes       │      │ - Hot Episodes   │    │
│    │ - Patterns       │      │ - Patterns       │    │
│    │ - Heuristics     │      │ - Embeddings     │    │
│    └──────────────────┘      └──────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Validation Checks**:
- [ ] Crate boundaries match planned components
- [ ] Dependencies flow in correct direction (no cycles)
- [ ] Core business logic separated from infrastructure
- [ ] Storage abstraction properly implemented

### 2. Crate Architecture & Dependencies

**Planned Crate Structure**:
```
rust-self-learning-memory/
├── memory-core           # Core business logic
├── memory-storage-turso  # Turso storage backend
├── memory-storage-redb   # redb cache layer
├── memory-mcp            # MCP server and sandbox
├── test-utils            # Shared test utilities
└── benches              # Performance benchmarks
```

**Dependency Rules** (from AGENTS.md):
- Core has NO dependency on storage implementations
- Storage implementations depend on core types
- MCP depends on core for memory integration
- No circular dependencies

**Validation**:
```bash
# Check Cargo.toml dependencies
cat memory-core/Cargo.toml | grep -A 20 "\[dependencies\]"

# Verify core doesn't depend on storage
! grep "memory-storage" memory-core/Cargo.toml

# Check dependency graph
cargo tree --package memory-core
```

**Compliance Checks**:
- [ ] memory-core is dependency-free of storage crates
- [ ] Storage crates depend only on memory-core
- [ ] MCP depends on memory-core
- [ ] No circular dependencies exist
- [ ] test-utils is dev-dependency only

### 3. Storage Layer Architecture

**Planned Design** (Hybrid Storage):

**Decision**: Use Turso (durable) + redb (cache)

**Turso Responsibilities**:
- Durable persistence (source of truth)
- Complex analytical queries
- Episode history and analytics
- Pattern aggregation

**redb Responsibilities**:
- Hot-path cache for fast reads
- Recent episode caching (LRU eviction)
- Pattern lookup optimization
- Embedding storage

**Synchronization Strategy**:
- Write to Turso first (durable)
- Async cache update to redb
- Periodic sync for reconciliation
- Conflict resolution: Turso wins

**Validation Checks**:
```bash
# Verify Turso tables
rg "CREATE TABLE" memory-storage-turso/src/schema.rs

# Verify redb tables
rg "TableDefinition" memory-storage-redb/src/lib.rs

# Check sync implementation
rg "sync|synchronize" memory-core/src/ -A 5
```

**Compliance**:
- [ ] Turso has tables: episodes, patterns, heuristics
- [ ] redb has tables: episodes, patterns, embeddings, metadata
- [ ] Sync mechanism exists (memory-core/src/sync.rs)
- [ ] Turso is primary source of truth
- [ ] redb operates as cache with eviction

### 4. Learning Cycle Architecture

**Planned 5-Phase Cycle** (from plans/00-overview.md):

1. **Pre-Task**: Context gathering and memory retrieval
2. **Execution**: Step-by-step action logging
3. **Post-Task**: Outcome analysis and scoring
4. **Learning**: Pattern extraction and heuristic updates
5. **Retrieval**: Context-aware episode lookup

**Implementation Validation**:
```rust
// Expected API in memory-core
impl SelfLearningMemory {
    // Phase 1: Pre-Task
    async fn start_episode(...) -> Uuid;

    // Phase 2: Execution
    async fn log_step(episode_id, step) -> Result<()>;

    // Phase 3: Post-Task
    async fn complete_episode(episode_id, outcome) -> Result<Episode>;

    // Phase 4: Learning (within complete_episode)
    // - RewardCalculator::calculate_reward
    // - ReflectionGenerator::generate_reflection
    // - PatternExtractor::extract_patterns

    // Phase 5: Retrieval
    async fn retrieve_relevant_context(...) -> Result<Vec<Episode>>;
}
```

**Validation**:
```bash
# Check memory.rs API
rg "pub async fn (start_episode|log_step|complete_episode|retrieve)" memory-core/src/memory.rs

# Verify reward calculation
test -f memory-core/src/reward.rs && echo "✓ Reward module exists"

# Verify reflection generation
test -f memory-core/src/reflection.rs && echo "✓ Reflection module exists"

# Verify pattern extraction
test -f memory-core/src/extraction.rs && echo "✓ Extraction module exists"
```

**Compliance**:
- [ ] start_episode creates episode with unique ID
- [ ] log_step appends to episode.steps
- [ ] complete_episode triggers reward, reflection, patterns
- [ ] retrieve_relevant_context queries by similarity
- [ ] All 5 phases implemented

### 5. Pattern Extraction Architecture

**Planned Pattern Types** (from plans/01-understand.md):
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

**Extraction Strategy** (Hybrid from plans/02-plan.md):
- Phase 1: Rule-based extraction (tool sequences, decision points)
- Phase 2: Embedding-based similarity (semantic patterns)

**Validation**:
```bash
# Check Pattern definition
rg "pub enum Pattern" memory-core/src/pattern.rs -A 30

# Check extractor implementations
rg "impl.*PatternExtractor" memory-core/src/extraction.rs -A 10

# Verify pattern types
rg "ToolSequence|DecisionPoint|ErrorRecovery|ContextPattern" memory-core/src/
```

**Compliance**:
- [ ] Pattern enum has all 4 planned variants
- [ ] PatternExtractor trait exists
- [ ] Rule-based extractors implemented
- [ ] Pattern storage in both Turso and redb
- [ ] Pattern similarity scoring exists

### 6. MCP Integration Architecture

**Planned Architecture** (from plans/03-execute.md):

**Components**:
- MCP Server (JSON-RPC 2.0)
- Tool Definitions (query_memory, execute_agent_code, analyze_patterns)
- Secure Sandbox (TypeScript/JavaScript execution)
- Progressive Tool Disclosure

**Security Layers**:
1. Input Validation
2. Process Isolation
3. Resource Limits (CPU, memory, timeout)
4. Filesystem Restrictions
5. Network Access Control

**Validation**:
```bash
# Check MCP server
test -f memory-mcp/src/server.rs && echo "✓ MCP server exists"

# Check sandbox
test -f memory-mcp/src/sandbox.rs && echo "✓ Sandbox exists"

# Verify security config
rg "SandboxConfig|SecurityConfig" memory-mcp/src/ -A 10

# Check resource limits
rg "max_.*|timeout|limit" memory-mcp/src/types.rs
```

**Compliance**:
- [ ] MemoryMCPServer implemented
- [ ] Tool definitions include query_memory, execute_agent_code
- [ ] CodeSandbox with security config
- [ ] Resource limits enforced (timeout, memory)
- [ ] Malicious code detection implemented

### 7. Data Model Architecture

**Core Types** (from plans/01-understand.md):

```rust
// Episode structure
pub struct Episode {
    episode_id: Uuid,
    task_type: TaskType,
    task_description: String,
    context: TaskContext,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    steps: Vec<ExecutionStep>,
    outcome: Option<TaskOutcome>,
    reward: Option<RewardScore>,
    reflection: Option<Reflection>,
    patterns: Vec<PatternId>,
    metadata: HashMap<String, String>,
}

// ExecutionStep structure
pub struct ExecutionStep {
    step_number: usize,
    timestamp: DateTime<Utc>,
    tool: String,
    action: String,
    parameters: serde_json::Value,
    result: Option<ExecutionResult>,
    latency_ms: u64,
    tokens_used: Option<u64>,
    metadata: HashMap<String, String>,
}
```

**Validation**:
```bash
# Check Episode definition
rg "pub struct Episode" memory-core/src/episode.rs -A 15

# Check ExecutionStep
rg "pub struct ExecutionStep" memory-core/src/episode.rs -A 10

# Verify all fields exist
rg "episode_id|task_type|task_description|context|start_time|end_time|steps|outcome|reward|reflection|patterns|metadata" memory-core/src/episode.rs
```

**Compliance**:
- [ ] Episode has all planned fields
- [ ] ExecutionStep has all planned fields
- [ ] TaskContext includes language, domain, tags
- [ ] RewardScore structure implemented
- [ ] Reflection structure implemented
- [ ] Pattern references stored (Vec<PatternId>)

### 8. Performance Architecture

**Planned Targets** (from plans/00-overview.md):

| Metric | Target | Validation Method |
|--------|--------|-------------------|
| Retrieval Latency (P95) | <100ms | Benchmark test |
| Episode Creation | <50ms | Benchmark test |
| Step Logging | <20ms | Benchmark test |
| Pattern Extraction | <1000ms | Benchmark test |
| Storage Capacity | 10,000+ episodes | Load test |
| Concurrent Ops | 1000+ ops/s | Stress test |
| Memory Usage | <500MB for 10K | Memory profiling |

**Validation**:
```bash
# Check benchmarks exist
ls benches/*.rs

# Run benchmarks
cargo bench --no-run

# Check for performance tests
rg "bench|criterion" benches/ -l
```

**Compliance**:
- [ ] Retrieval benchmark exists
- [ ] Episode lifecycle benchmark exists
- [ ] Pattern extraction benchmark exists
- [ ] Performance targets documented in code
- [ ] Profiling infrastructure exists

### 9. Security Architecture

**Planned Defense-in-Depth** (from plans/05-secure.md):

**Attack Surface Coverage**:
1. MCP Code Execution
2. Database Injection
3. Memory Exhaustion
4. Deserialization Attacks
5. Network Interception

**Mitigations Required**:
- Parameterized SQL queries (no string interpolation)
- Input validation and size limits
- Sandbox with process isolation
- Resource limits (CPU, memory, timeout)
- TLS enforcement for Turso

**Validation**:
```bash
# Check for SQL parameterization
rg "execute\(.*params!" memory-storage-turso/src/

# Check for input validation
rg "validate|check.*size|limit" memory-core/src/ memory-mcp/src/

# Verify sandbox security
rg "SecurityConfig|validate.*code|malicious" memory-mcp/src/sandbox.rs

# Check resource limits
rg "ResourceLimit|max_memory|timeout" memory-mcp/src/
```

**Compliance**:
- [ ] All SQL uses parameterized queries
- [ ] Input size validation exists
- [ ] Sandbox has security configuration
- [ ] Resource limits enforced
- [ ] Malicious code detection implemented
- [ ] No hardcoded secrets in code

### 10. Error Handling Architecture

**Planned Strategy** (Rust best practices):
- Custom Error enum with thiserror
- Result<T> for all fallible operations
- Error propagation with ? operator
- Specific error variants for each failure mode
- Error context with helpful messages

**Validation**:
```bash
# Check Error enum
rg "pub enum Error" memory-core/src/error.rs -A 20

# Verify thiserror usage
rg "#\[derive.*Error\]" memory-core/src/error.rs

# Check Result type alias
rg "pub type Result" memory-core/src/error.rs

# Verify error variants
rg "Storage|EpisodeNotFound|InvalidInput|Timeout" memory-core/src/error.rs
```

**Compliance**:
- [ ] Custom Error enum with thiserror
- [ ] Result<T> type alias defined
- [ ] Error variants cover all failure modes
- [ ] Storage errors wrapped properly
- [ ] Error messages are descriptive

## Validation Workflow

### Phase 1: Architecture Document Review
1. Read all architecture decisions from plans/02-plan.md
2. Extract key architectural patterns and constraints
3. Document expected component structure

### Phase 2: Code Structure Analysis
```bash
# Analyze crate structure
tree -L 2 -I target

# Check dependencies
cargo tree --depth 1

# Verify module organization
find . -name "lib.rs" -o -name "mod.rs" | xargs cat
```

### Phase 3: Component Boundary Validation
```bash
# Check core doesn't depend on impl
! grep "memory-storage-turso\|memory-storage-redb" memory-core/Cargo.toml

# Verify storage depends on core
grep "memory-core" memory-storage-turso/Cargo.toml
grep "memory-core" memory-storage-redb/Cargo.toml

# Check MCP integration
grep "memory-core" memory-mcp/Cargo.toml
```

### Phase 4: Data Flow Validation
1. Trace episode lifecycle through code
2. Verify storage sync mechanism
3. Check pattern extraction pipeline
4. Validate retrieval query flow

### Phase 5: API Compliance Check
```bash
# Verify public API matches plan
rg "pub (async )?fn" memory-core/src/memory.rs

# Check type definitions
rg "pub struct|pub enum" memory-core/src/types.rs memory-core/src/episode.rs memory-core/src/pattern.rs
```

### Phase 6: Performance Target Validation
```bash
# Check benchmarks
ls benches/
cat benches/*.rs | rg "target|criterion"

# Verify performance constants
rg "const.*TIMEOUT|const.*LIMIT|const.*MAX" memory-core/src/ memory-mcp/src/
```

### Phase 7: Security Architecture Validation
```bash
# Check security implementations
rg "SecurityConfig|validate|sanitize" memory-mcp/src/

# Verify SQL safety
rg "execute\(|query\(" memory-storage-turso/src/ -A 2
```

## Compliance Matrix

| Architecture Dimension | Planned | Implemented | Status |
|------------------------|---------|-------------|--------|
| Crate Structure | 6 crates | ? | ? |
| Dependency Flow | Core → Storage | ? | ? |
| Storage Layer | Turso + redb | ? | ? |
| Learning Cycle | 5 phases | ? | ? |
| Pattern Types | 4 variants | ? | ? |
| MCP Integration | Server + Sandbox | ? | ? |
| Data Model | Episode + Step | ? | ? |
| Performance Targets | 7 metrics | ? | ? |
| Security Mitigations | 5 attack surfaces | ? | ? |
| Error Handling | Custom Error enum | ? | ? |

## Output Format

```markdown
# Architecture Validation Report
**Generated**: [Date]
**Project**: rust-self-learning-memory
**Validation Against**: plans/02-plan.md

## Executive Summary
- **Overall Compliance**: X% (Y/Z dimensions)
- **Critical Violations**: N
- **Architecture Drift**: M areas
- **Recommendations**: P action items

## Compliance Score by Dimension

### 1. System Architecture: ✅ COMPLIANT
- Crate boundaries match planned structure
- Component separation properly implemented
- No architectural violations detected

### 2. Storage Layer: ⚠️ PARTIAL
- ✅ Turso storage implemented
- ✅ redb cache implemented
- ❌ Sync mechanism incomplete
  - **Missing**: Two-phase commit (plans/06-feedback-loop.md)
  - **Impact**: Data consistency risk
  - **Priority**: High

### 3. Learning Cycle: ✅ COMPLIANT
- All 5 phases implemented
- Episode lifecycle complete
- Pattern extraction functional

### 4. MCP Integration: ⚠️ PARTIAL
- ✅ MCP server implemented
- ✅ Sandbox basic security
- ⚠️ Progressive disclosure not fully implemented
- ❌ Resource limits not enforced
  - **Missing**: Memory limit enforcement
  - **Plan**: plans/05-secure.md:35-50

## Detailed Findings

### Critical Violations
None detected.

### Architecture Drift

#### 1. Sync Mechanism Incomplete
**Planned** (plans/02-plan.md):
- Two-phase commit for consistency
- Conflict resolution (Turso wins)
- Periodic reconciliation

**Actual**:
- Basic sync exists in memory-core/src/sync.rs
- Two-phase commit not implemented
- No conflict resolution strategy

**Impact**: Data inconsistency between Turso and redb possible

**Recommendation**:
Implement two-phase commit as documented in plans/06-feedback-loop.md:125-158

#### 2. Resource Limits Not Enforced
**Planned** (plans/05-secure.md):
```rust
pub struct SandboxSecurityConfig {
    pub max_memory_mb: usize,  // 128MB default
    pub max_cpu_percent: f32,   // 50% default
}
```

**Actual**:
- Timeout implemented
- Memory limits defined but not enforced
- CPU limits not implemented

**Impact**: DoS risk via resource exhaustion

**Recommendation**:
Implement resource enforcement in memory-mcp/src/sandbox.rs

### Partial Implementations

#### 1. Pattern Extraction
**Planned**: Hybrid (Rule-based + Embeddings)
**Actual**: Rule-based only

**Status**: Acceptable (Phase 1 complete, Phase 2 optional)

#### 2. Progressive Tool Disclosure
**Planned**: Context-aware tool filtering
**Actual**: All tools always available

**Status**: Enhancement opportunity

## Recommendations

### High Priority
1. **Implement two-phase commit** for storage sync
   - File: memory-core/src/sync.rs
   - Effort: 2-3 days
   - Reference: plans/06-feedback-loop.md:125-158

2. **Enforce resource limits** in sandbox
   - File: memory-mcp/src/sandbox.rs
   - Effort: 1-2 days
   - Reference: plans/05-secure.md:35-50

### Medium Priority
1. **Add progressive tool disclosure** to MCP server
   - File: memory-mcp/src/server.rs
   - Effort: 1-2 days

2. **Implement pattern extraction queue** for async processing
   - File: memory-core/src/extraction.rs
   - Effort: 2-3 days
   - Reference: plans/06-feedback-loop.md:60-117

### Low Priority
1. Add embedding-based pattern extraction (Phase 2 feature)
2. Implement advanced reflection generation
3. Add distributed tracing

## Architecture Decision Compliance

| Decision | Status | Notes |
|----------|--------|-------|
| Hybrid Storage (Turso + redb) | ✅ | Fully implemented |
| Node.js + VM2 Sandbox | ✅ | Implemented |
| Rule-based Pattern Extraction | ✅ | Phase 1 complete |
| Circuit Breakers | ✅ | Implemented in Turso |
| Feature Flags | ❌ | Not implemented |
| Telemetry with tracing | ✅ | Basic implementation |

## Next Steps

1. Review and prioritize architecture drift items
2. Create tickets for missing implementations
3. Update architecture documentation if intentional changes made
4. Schedule architecture review session
