# Phase 21: Architecture Decision Records (ADRs)

**Date**: 2025-11-16
**Status**: LIVING DOCUMENT
**Priority**: P1 (Developer Reference)
**Format**: Lightweight ADR Format
**Coverage**: 10 ADRs documenting key architectural decisions

## Overview ✅ COMPLETE

Architecture Decision Records (ADRs) document significant architectural and design decisions made during the development of the rust-self-learning-memory system. Each ADR captures the context, decision, and consequences of a choice. All 10 core ADRs are documented and implemented.

**ADR Index** (All Implemented ✅):
- [ADR-001: Hybrid Storage Architecture (Turso + redb)](#adr-001) ✅ IMPLEMENTED
- [ADR-002: Async Pattern Extraction with Worker Pool](#adr-002) ✅ IMPLEMENTED
- [ADR-003: Circuit Breaker Pattern for Storage Resilience](#adr-003) ✅ IMPLEMENTED
- [ADR-004: Step Batching for High-Throughput Logging](#adr-004) ✅ IMPLEMENTED
- [ADR-005: Heuristic Learning with Condition-Action Rules](#adr-005) ✅ IMPLEMENTED
- [ADR-006: MCP Sandbox with VM2 Isolation](#adr-006) ✅ IMPLEMENTED
- [ADR-007: Bincode for Serialization with Size Limits](#adr-007) ✅ IMPLEMENTED
- [ADR-008: Connection Pooling with Semaphore](#adr-008) ✅ IMPLEMENTED
- [ADR-009: Episode-Centric Data Model](#adr-009) ✅ IMPLEMENTED
- [ADR-010: Four Pattern Types Strategy](#adr-010) ✅ IMPLEMENTED

---

## ADR-001: Hybrid Storage Architecture (Turso + redb) {#adr-001}

**Status**: ACCEPTED
**Date**: 2025-10-15
**Deciders**: Core Team
**Related**: Storage Layer Implementation

### Context

AI agent memory systems require both durable persistence and high-performance caching:
- **Durable Storage**: Episodes and patterns must survive process restarts
- **Fast Retrieval**: Recent episodes need <1ms access for frequent queries
- **Analytics**: SQL queries for pattern analysis and reporting
- **Scale**: Support 10K+ episodes without performance degradation

**Alternatives Considered**:

1. **Single SQL Database (PostgreSQL/Turso Only)**
   - ✓ Simple architecture
   - ✓ ACID guarantees
   - ✗ Higher latency for frequent reads (5-20ms)
   - ✗ Network overhead for local deployments

2. **Single KV Store (redb/RocksDB Only)**
   - ✓ Fast reads (<1ms)
   - ✓ Embedded (no network)
   - ✗ Poor analytics capabilities (no SQL)
   - ✗ Complex schema migrations

3. **Hybrid: SQL + KV Cache**
   - ✓ Fast reads from cache
   - ✓ SQL analytics on durable storage
   - ✓ Best of both worlds
   - ✗ Added complexity (sync required)

### Decision

Use **hybrid storage**: Turso (libSQL) as source-of-truth + redb as hot cache.

**Architecture**:
```
┌─────────────────────────────────────────┐
│        Self-Learning Memory             │
├─────────────────────────────────────────┤
│  Fast Path (99% of reads)               │
│  ┌─────────────────┐                    │
│  │  redb Cache     │  <1ms reads        │
│  │  (LRU, 1000)    │                    │
│  └─────────────────┘                    │
│           │                              │
│           ↓ (cache miss)                 │
│  Slow Path (1% of reads, all writes)    │
│  ┌─────────────────┐                    │
│  │  Turso/libSQL   │  5-20ms            │
│  │  (Durable SQL)  │                    │
│  └─────────────────┘                    │
└─────────────────────────────────────────┘
```

**Write Strategy**: Write-through (write to both immediately)
**Read Strategy**: Cache-first (redb → Turso on miss)
**Consistency**: Turso is source-of-truth; redb rebuilt on sync

### Consequences

**Positive**:
- Fast reads: Cache hits <1ms, significantly better than SQL-only
- Durable writes: All data persisted to Turso immediately
- SQL analytics: Complex queries on Turso without performance impact
- Scalability: Cache limits memory footprint, Turso handles unlimited episodes

**Negative**:
- Increased complexity: Need sync logic and consistency management
- Cache invalidation: Must handle stale cache scenarios
- Memory overhead: redb cache uses ~100MB for 1000 episodes
- Testing complexity: Need tests for cache hit/miss/sync scenarios

**Mitigations**:
- Implement sync_memories() for cache reconciliation
- Circuit breaker for storage failures
- Comprehensive integration tests for sync scenarios

**Implementation**:
- `memory-storage-turso/`: Durable SQL storage
- `memory-storage-redb/`: Hot KV cache
- `memory-core/src/sync.rs`: Sync logic

---

## ADR-002: Async Pattern Extraction with Worker Pool {#adr-002}

**Status**: ACCEPTED
**Date**: 2025-10-20
**Deciders**: Core Team
**Related**: Pattern Learning

### Context

Pattern extraction is computationally expensive (1-10ms per episode):
- Blocking episode completion is unacceptable
- Multiple patterns extracted per episode (4+ types)
- Need to handle backpressure under high load

**Alternatives Considered**:

1. **Synchronous Extraction (Block on Complete)**
   - ✓ Simple implementation
   - ✗ Episode completion blocked (100-500ms)
   - ✗ Poor user experience

2. **Fire-and-Forget Async (tokio::spawn)**
   - ✓ Non-blocking
   - ✗ No backpressure handling
   - ✗ Unbounded memory growth under load
   - ✗ No extraction status tracking

3. **Async Queue with Worker Pool**
   - ✓ Non-blocking episode completion
   - ✓ Backpressure via bounded queue
   - ✓ Configurable concurrency
   - ✓ Status tracking and metrics
   - ✗ More complex implementation

### Decision

Use **async queue with worker pool** for pattern extraction.

**Architecture**:
```rust
pub struct PatternExtractionQueue {
    tx: mpsc::Sender<ExtractionTask>,
    workers: Vec<JoinHandle<()>>,
    stats: Arc<Mutex<QueueStats>>,
}

// Episode completion sends to queue
async fn complete_episode(&self, ...) -> Result<()> {
    self.storage.mark_complete(...).await?;

    // Non-blocking: send to extraction queue
    self.extraction_queue.enqueue(episode_id).await?;

    Ok(())  // Returns immediately
}

// Workers process queue in background
async fn extraction_worker(rx: mpsc::Receiver<ExtractionTask>) {
    while let Some(task) = rx.recv().await {
        let patterns = extract_patterns(&task.episode).await;
        storage.save_patterns(patterns).await;
    }
}
```

**Configuration**:
- Queue size: 1000 (bounded)
- Worker count: num_cpus() (typically 4-16)
- Backpressure: Block episode completion if queue full

### Consequences

**Positive**:
- Episode completion latency: 2-5ms (vs. 100-500ms synchronous)
- Throughput: 1000+ episodes/sec with 8 workers
- Backpressure protection: Queue bounded, prevents OOM
- Observability: Queue depth, processing latency metrics

**Negative**:
- Pattern extraction delay: 100-500ms after episode completion
- Complexity: Worker lifecycle management
- Testing: Async coordination requires integration tests

**Mitigations**:
- Expose queue stats via metrics endpoint
- Graceful shutdown: drain queue before exit
- Configurable worker count and queue size

**Implementation**:
- `memory-core/src/learning/queue.rs`: Queue and workers (642 LOC)
- `memory-core/src/extraction/`: Pattern extractors

---

## ADR-003: Circuit Breaker Pattern for Storage Resilience {#adr-003}

**Status**: ACCEPTED
**Date**: 2025-10-25
**Deciders**: Core Team
**Related**: Storage Reliability

### Context

Distributed storage (Turso) can experience transient failures:
- Network partitions
- Database overload
- API rate limits
- Deployment issues

**Failure Modes Without Circuit Breaker**:
- Cascading failures (retry storms)
- Slow requests accumulate (thread pool exhaustion)
- User experience degradation (timeouts)

**Alternatives Considered**:

1. **Naive Retry**
   - ✓ Simple
   - ✗ Retry storms worsen outages
   - ✗ No fast-fail mechanism

2. **Exponential Backoff Only**
   - ✓ Reduces load during failures
   - ✗ Still sends requests to failing backend
   - ✗ Slow failure detection

3. **Circuit Breaker**
   - ✓ Fast-fail during outages
   - ✓ Automatic recovery detection
   - ✓ Prevents cascade failures
   - ✗ More complex state machine

### Decision

Implement **circuit breaker pattern** for Turso storage.

**State Machine**:
```
        failures > threshold
CLOSED ────────────────────────→ OPEN
  ↑                                 │
  │                                 │ timeout
  │                                 ↓
  └─────────────────────── HALF_OPEN
       success_count > threshold
```

**States**:
- **CLOSED**: Normal operation, all requests allowed
- **OPEN**: Failure threshold exceeded, fast-fail all requests
- **HALF_OPEN**: Testing recovery, allow limited requests

**Configuration**:
```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,      // 5 failures → OPEN
    pub success_threshold: u32,       // 3 successes → CLOSED
    pub timeout: Duration,            // 30s before HALF_OPEN
    pub half_open_requests: u32,      // 3 test requests
}
```

### Consequences

**Positive**:
- Fast-fail: <1ms response during outages (vs. 30s timeout)
- Automatic recovery: No manual intervention needed
- Reduced load: Stop sending requests to failing backend
- Metrics: Circuit breaker state exposed

**Negative**:
- Complexity: State machine and timing logic (731 LOC)
- False positives: May open on transient issues
- Cache-only mode: Need fallback for reads during OPEN

**Mitigations**:
- Tunable thresholds per deployment
- Alerting on circuit breaker state changes
- Cache-first strategy minimizes impact of OPEN state

**Implementation**:
- `memory-core/src/storage/circuit_breaker.rs`: State machine (731 LOC)
- `memory-storage-turso/src/resilient.rs`: Wrapper with circuit breaker

---

## ADR-004: Step Batching for High-Throughput Logging {#adr-004}

**Status**: ACCEPTED
**Date**: 2025-11-01
**Deciders**: Core Team
**Related**: Episode Step Logging

### Context

Episodes can have hundreds of steps (tool calls, observations):
- Individual DB writes per step: 100+ writes/episode
- High latency: 5-20ms per write
- Total latency: 500ms-2s per episode
- Database load: High write amplification

**Alternatives Considered**:

1. **Individual Writes (No Batching)**
   - ✓ Simple implementation
   - ✗ High latency (500ms-2s per episode)
   - ✗ Database write amplification

2. **Write All Steps on Complete**
   - ✓ Single write operation
   - ✗ Loss of steps if process crashes
   - ✗ No partial progress visibility

3. **Batching with Flush Policy**
   - ✓ Reduced writes (10-50x)
   - ✓ Configurable latency/reliability tradeoff
   - ✓ Graceful degradation
   - ✗ More complex buffer management

### Decision

Implement **step batching with configurable flush policy**.

**Flush Triggers**:
- **Size-based**: Flush when batch reaches max_batch_size (default: 50)
- **Time-based**: Flush every flush_interval (default: 5 seconds)
- **Episode complete**: Flush all pending steps
- **Process shutdown**: Flush all buffers

**Configuration**:
```rust
pub struct StepBufferConfig {
    pub max_batch_size: usize,         // 50 steps
    pub flush_interval: Duration,       // 5 seconds
    pub max_buffer_memory: usize,       // 10 MB
}
```

**Implementation Strategy**:
```rust
pub struct StepBuffer {
    episodes: HashMap<String, Vec<ExecutionStep>>,
    last_flush: Instant,
    config: StepBufferConfig,
}

// Buffer step
async fn log_step(&mut self, episode_id: &str, step: ExecutionStep) {
    self.episodes.entry(episode_id).or_default().push(step);

    if self.should_flush() {
        self.flush().await?;
    }
}

// Flush policy
fn should_flush(&self) -> bool {
    self.total_buffered() >= self.config.max_batch_size ||
    self.last_flush.elapsed() >= self.config.flush_interval
}
```

### Consequences

**Positive**:
- 10-50x reduction in database writes
- Episode logging latency: 1-2ms (vs. 500ms-2s)
- Throughput: 1000+ steps/sec (vs. 20-50 steps/sec)
- Configurable: Tune latency/reliability tradeoff

**Negative**:
- Data loss risk: Up to 5 seconds of steps if crash
- Memory overhead: Buffered steps in RAM (~1-10MB)
- Complexity: Buffer management and flush logic

**Mitigations**:
- Graceful shutdown: Flush buffers on SIGTERM
- Memory limits: Enforce max_buffer_memory
- Tunable flush interval: 1s for low-latency, 30s for high-throughput

**Implementation**:
- `memory-core/src/memory/step_buffer/`: Buffer logic (300+ LOC)
- `memory-core/tests/step_batching.rs`: Comprehensive tests (711 LOC)

---

## ADR-005: Heuristic Learning with Condition-Action Rules {#adr-005}

**Status**: ACCEPTED
**Date**: 2025-11-05
**Deciders**: Core Team
**Related**: Pattern Learning

### Context

Agents need to **learn from experience** and apply learned patterns:
- Extract reusable patterns from successful episodes
- Match patterns to new contexts
- Generate heuristics for future decision-making

**Alternatives Considered**:

1. **No Heuristics (Pattern Storage Only)**
   - ✓ Simple
   - ✗ No proactive learning
   - ✗ Requires manual pattern application

2. **Supervised Learning (Train ML Model)**
   - ✓ Sophisticated pattern matching
   - ✗ Requires labeled training data
   - ✗ Black-box (not interpretable)
   - ✗ High computational cost

3. **Rule-Based Heuristics (If Condition → Then Action)**
   - ✓ Interpretable and explainable
   - ✓ Efficient matching (<1ms)
   - ✓ Incremental learning
   - ✗ Limited expressiveness (vs. ML)

### Decision

Use **rule-based heuristics** with condition-action pairs.

**Heuristic Format**:
```rust
pub struct Heuristic {
    pub id: String,
    pub condition: HeuristicCondition,  // Context match
    pub action: HeuristicAction,        // Recommended action
    pub confidence: f64,                 // 0.0 to 1.0
    pub success_rate: f64,               // Historical accuracy
    pub application_count: u32,
}

pub struct HeuristicCondition {
    pub task_type: Option<String>,
    pub domain: Option<String>,
    pub context_keywords: Vec<String>,
    pub required_tags: Vec<String>,
}

pub struct HeuristicAction {
    pub tool_sequence: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub rationale: String,
}
```

**Extraction Process**:
1. Identify successful episodes (verdict = Success)
2. Extract patterns from execution steps
3. Generalize patterns into heuristic conditions
4. Track confidence based on pattern quality
5. Update success_rate on each application

**Matching Process**:
```rust
async fn match_heuristics(&self, context: &TaskContext) -> Vec<Heuristic> {
    let candidates = self.storage
        .get_heuristics_by_task_type(&context.task_type)
        .await?;

    candidates
        .into_iter()
        .filter(|h| h.matches(context))
        .filter(|h| h.confidence > 0.5)
        .sorted_by_key(|h| (h.confidence * h.success_rate))
        .take(10)
        .collect()
}
```

### Consequences

**Positive**:
- Interpretable: Users understand why heuristic was suggested
- Efficient: Pattern matching <1ms
- Incremental: Learn continuously from new episodes
- Confidence tracking: Know when heuristics are reliable

**Negative**:
- Limited expressiveness: Cannot capture complex patterns
- Manual generalization: Heuristic extraction may be naive
- Cold start: Few heuristics initially (first 100 episodes)

**Mitigations**:
- Hybrid approach: Combine rule-based + embeddings (v0.2.0)
- Bootstrapping: Pre-load common heuristics for common tasks
- Confidence decay: Reduce confidence over time if not used

**Implementation**:
- `memory-core/src/pattern/heuristic.rs`: Heuristic types
- `memory-core/src/patterns/extractors/heuristic/`: Extraction (300+ LOC)
- `memory-core/tests/heuristic_learning.rs`: Tests (755 LOC)

---

## ADR-006: MCP Sandbox with VM2 Isolation {#adr-006}

**Status**: ACCEPTED
**Date**: 2025-11-08
**Deciders**: Core Team
**Related**: Security, MCP Integration

### Context

MCP server executes TypeScript code from patterns/heuristics:
- **Security Risk**: Arbitrary code execution
- **Resource Risk**: Infinite loops, memory leaks
- **File System Risk**: Unauthorized file access

**Alternatives Considered**:

1. **No Sandboxing (Trust Code)**
   - ✓ Simple implementation
   - ✗ Critical security vulnerability
   - ✗ Unacceptable for production

2. **Docker Containers per Execution**
   - ✓ Strong isolation
   - ✗ High latency (100-500ms startup)
   - ✗ Resource overhead

3. **VM2 (V8 Isolates)**
   - ✓ Fast (<10ms startup)
   - ✓ Memory/CPU limits
   - ✓ File system restrictions
   - ✗ Bypass vulnerabilities (historical)

4. **WASM Sandbox**
   - ✓ Strong isolation
   - ✗ Limited ecosystem (no npm packages)
   - ✗ Performance overhead for I/O

### Decision

Use **VM2 (V8 isolates)** with additional defense-in-depth measures.

**Security Layers**:

1. **VM2 Sandbox**: V8 isolate with restricted globals
2. **Resource Limits**: CPU time, memory, execution time
3. **File System Whitelist**: Only allowed paths readable
4. **Network Restrictions**: Domain whitelist, HTTPS-only
5. **Input Validation**: Sanitize all inputs before execution

**Configuration**:
```rust
pub struct SandboxConfig {
    pub timeout_ms: u64,              // 5000ms
    pub max_memory_mb: u64,           // 128MB
    pub allowed_paths: Vec<PathBuf>,  // Whitelist
    pub allowed_domains: Vec<String>, // Network whitelist
    pub enable_network: bool,          // Default: false
}
```

**Implementation**:
```rust
// File system isolation
sandbox.set_fs_mode(FsMode::ReadOnly);
sandbox.set_allowed_paths(config.allowed_paths);

// Network isolation
sandbox.set_network_enabled(config.enable_network);
sandbox.set_allowed_domains(config.allowed_domains);

// Resource limits
sandbox.set_timeout(Duration::from_millis(config.timeout_ms));
sandbox.set_max_memory(config.max_memory_mb * 1024 * 1024);

// Execute with timeout
let result = timeout(
    Duration::from_millis(config.timeout_ms),
    sandbox.execute(code)
).await?;
```

### Consequences

**Positive**:
- Fast execution: <10ms overhead
- Memory safe: Enforced limits
- File system protected: Whitelist-based access
- Network controlled: Domain whitelist
- Comprehensive tests: 18 penetration tests passing

**Negative**:
- VM2 vulnerabilities: Historical sandbox escapes
- Complexity: Multiple security layers
- Debugging: Harder to debug sandboxed code

**Mitigations**:
- Defense-in-depth: Multiple security layers
- Regular updates: Keep VM2 and Node.js updated
- Penetration testing: 18 tests covering escape attempts
- Security monitoring: Log all sandbox violations

**Implementation**:
- `memory-mcp/src/sandbox/`: Sandbox implementation (600+ LOC)
- `memory-mcp/tests/penetration_tests.rs`: Security tests (746 LOC)
- `memory-mcp/SECURITY_AUDIT.md`: Security documentation

---

## ADR-007: Bincode for Serialization with Size Limits {#adr-007}

**Status**: ACCEPTED
**Date**: 2025-11-10
**Deciders**: Core Team
**Related**: Storage Security

### Context

Episodes and patterns need serialization for storage:
- **Performance**: Fast serialization/deserialization
- **Compactness**: Minimize storage size
- **Security**: Prevent malicious payloads (OOM, DoS)

**Alternatives Considered**:

1. **JSON (serde_json)**
   - ✓ Human-readable
   - ✓ Wide support
   - ✗ Larger size (2-3x vs. binary)
   - ✗ Slower (2-5x vs. binary)

2. **Protocol Buffers**
   - ✓ Compact and fast
   - ✓ Schema evolution
   - ✗ Requires .proto definitions
   - ✗ More complexity

3. **Bincode**
   - ✓ Fastest serialization (Rust native)
   - ✓ Most compact (no schema overhead)
   - ✓ Type-safe with serde
   - ✗ Not human-readable
   - ✗ Potential security risks (unbounded deserialization)

### Decision

Use **bincode with enforced size limits**.

**Size Limits**:
```rust
pub const MAX_EPISODE_SIZE: usize = 10 * 1024 * 1024;   // 10 MB
pub const MAX_PATTERN_SIZE: usize = 1 * 1024 * 1024;    // 1 MB
pub const MAX_HEURISTIC_SIZE: usize = 100 * 1024;       // 100 KB
```

**Serialization with Validation**:
```rust
pub fn serialize_episode(episode: &Episode) -> Result<Vec<u8>> {
    let data = bincode::serialize(episode)?;

    if data.len() > MAX_EPISODE_SIZE {
        return Err(Error::EpisodeTooLarge {
            size: data.len(),
            max: MAX_EPISODE_SIZE,
        });
    }

    Ok(data)
}

pub fn deserialize_episode(data: &[u8]) -> Result<Episode> {
    if data.len() > MAX_EPISODE_SIZE {
        return Err(Error::EpisodeTooLarge {
            size: data.len(),
            max: MAX_EPISODE_SIZE,
        });
    }

    bincode::deserialize(data)
        .map_err(|e| Error::DeserializationFailed(e))
}
```

### Consequences

**Positive**:
- Performance: 5-10x faster than JSON
- Compactness: 2-3x smaller than JSON
- Type safety: Compiler-checked with serde
- Security: Size limits prevent OOM attacks

**Negative**:
- Not human-readable (debugging harder)
- No built-in schema evolution (use explicit versioning)
- Size limits may be too small for edge cases

**Mitigations**:
- Provide JSON export for debugging
- Configurable size limits per deployment
- Comprehensive bincode security tests (8 tests)

**Implementation**:
- `memory-storage-turso/src/storage.rs`: Serialization with limits
- `memory-storage-redb/src/cache.rs`: Cache with limits
- `memory-storage-turso/tests/bincode_security_tests.rs`: Security tests

---

## ADR-008: Connection Pooling with Semaphore {#adr-008}

**Status**: ACCEPTED
**Date**: 2025-11-12
**Deciders**: Core Team
**Related**: Storage Performance

### Context

Turso database connections are expensive:
- Connection setup: 50-100ms
- TLS handshake: 20-50ms
- Auth token exchange: 10-20ms

Without pooling: Every request pays connection cost (80-170ms overhead).

**Alternatives Considered**:

1. **No Pooling (New Connection Per Request)**
   - ✓ Simple
   - ✗ High latency (80-170ms overhead per request)
   - ✗ Connection churn (TCP socket exhaustion)

2. **Fixed Pool (bb8, deadpool)**
   - ✓ Connection reuse
   - ✓ Battle-tested libraries
   - ✗ Less control over pool behavior
   - ✗ Harder to integrate with circuit breaker

3. **Semaphore-Based Pool (Custom)**
   - ✓ Simple implementation (176 LOC)
   - ✓ Integrates with circuit breaker
   - ✓ Tunable concurrency control
   - ✗ More code to maintain

### Decision

Implement **semaphore-based connection pool**.

**Architecture**:
```rust
pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<Connection>>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
}

pub struct PoolConfig {
    pub min_connections: usize,   // 10 (keep alive)
    pub max_connections: usize,   // 100 (hard limit)
    pub idle_timeout: Duration,    // 60s (recycle)
    pub health_check_interval: Duration, // 30s
}

// Acquire connection
pub async fn acquire(&self) -> Result<PooledConnection> {
    // Wait for permit (respects max_connections)
    let permit = self.semaphore.acquire().await?;

    // Get connection from pool or create new
    let conn = self.get_or_create_connection().await?;

    Ok(PooledConnection { conn, permit })
}

// Return connection on drop
impl Drop for PooledConnection {
    fn drop(&mut self) {
        self.pool.return_connection(self.conn.take());
    }
}
```

**Tested to 100 Concurrent Requests**:
```rust
#[tokio::test]
async fn test_100_concurrent_requests() {
    let pool = ConnectionPool::new(config).await?;

    let tasks: Vec<_> = (0..100)
        .map(|i| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let conn = pool.acquire().await?;
                conn.execute("SELECT 1").await
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

### Consequences

**Positive**:
- Latency reduction: 80-170ms → <5ms (connection overhead eliminated)
- Throughput: 100+ concurrent requests without degradation
- Resource control: Semaphore limits max connections
- Health checks: Auto-reconnect stale connections

**Negative**:
- Maintenance: Custom pool code to maintain (176 LOC)
- Complexity: Pool lifecycle management

**Mitigations**:
- Comprehensive tests: 6 tests, 176 LOC
- Metrics: Pool size, wait time, utilization
- Graceful shutdown: Close all connections cleanly

**Implementation**:
- `memory-storage-turso/src/pool.rs`: Pool implementation (176 LOC)
- `memory-storage-turso/tests/pool_tests.rs`: Pool tests

---

## ADR-009: Episode-Centric Data Model {#adr-009}

**Status**: ACCEPTED
**Date**: 2025-10-10
**Deciders**: Core Team
**Related**: Data Modeling

### Context

Memory systems can be organized around different entities:
- Tasks/goals
- Conversations/threads
- Episodes (task attempts)
- Steps/actions

**Alternatives Considered**:

1. **Task-Centric**: One task → many attempts
2. **Conversation-Centric**: One thread → many turns
3. **Episode-Centric**: One episode → lifecycle → patterns
4. **Step-Centric**: Fine-grained action logging

### Decision

Use **episode-centric** model.

**Schema**:
```rust
pub struct Episode {
    pub id: String,
    pub task: String,
    pub context: TaskContext,
    pub steps: Vec<ExecutionStep>,
    pub outcome: TaskOutcome,
    pub patterns: Vec<Pattern>,
    pub reflection: Reflection,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}
```

### Consequences

**Positive**:
- Natural lifecycle: start → execute → complete
- Self-contained: Episode includes all context
- Pattern extraction: Episode is atomic unit for learning

**Negative**:
- Duplication: Context repeated per episode
- Large objects: Episodes can be 10-100KB

**Implementation**: `memory-core/src/episode.rs`

---

## ADR-010: Four Pattern Types Strategy {#adr-010}

**Status**: ACCEPTED
**Date**: 2025-10-18
**Deciders**: Core Team
**Related**: Pattern Extraction

### Context

Need to extract diverse, actionable patterns from episodes.

### Decision

Four pattern types:
1. **ToolSequence**: Common tool call chains
2. **DecisionPoint**: Context → action branches
3. **ErrorRecovery**: Error → recovery strategy
4. **ContextPattern**: Successful context characteristics

### Consequences

**Positive**: Covers major pattern categories
**Negative**: May miss specialized patterns

**Implementation**: `memory-core/src/patterns/extractors/`

---

## ADR Template (For Future Decisions)

```markdown
## ADR-011: [Future ADR Template]

**Status**: TEMPLATE
**Date**: N/A
**Deciders**: [Future Contributors]
**Related**: [template, documentation]

### Context
[Describe the problem and constraints]

### Alternatives Considered
1. **Option A**: [Pros/Cons]
2. **Option B**: [Pros/Cons]
3. **Option C**: [Pros/Cons]

### Decision
[Chosen option and rationale]

### Consequences
**Positive**: [Benefits]
**Negative**: [Drawbacks]
**Mitigations**: [Risk mitigations]

**Implementation**: [File paths]
```

---

**Document Version**: 1.0
**Last Updated**: 2025-11-14
**Maintainer**: Core Team
**Status**: LIVING DOCUMENT
