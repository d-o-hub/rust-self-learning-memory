# PHASE 3: EXECUTE ⚙️

> **Goal**: Systematic implementation with progress tracking, decision documentation, and coordinated multi-agent development.

## Overview

This phase transforms plans into working code. We'll implement components incrementally, document decisions, and ensure quality through continuous testing.

## Cognitive Layer: Implementation Tracking

### Progress Monitoring System

```rust
/// Development metrics tracker for measuring implementation progress
pub struct DevelopmentMetrics {
    pub lines_of_code: usize,
    pub test_coverage: f32,
    pub benchmark_results: HashMap<String, Duration>,
    pub integration_tests_passing: bool,
    pub features_complete: Vec<String>,
    pub features_in_progress: Vec<String>,
    pub features_blocked: Vec<(String, String)>, // (feature, blocker)
}

impl DevelopmentMetrics {
    pub fn track_progress(&mut self) {
        // Monitor against plan milestones
        self.calculate_completion_percentage();

        // Document deviations and reasons
        self.log_timeline_deviations();

        // Adjust timeline based on velocity
        self.update_estimated_completion();
    }

    fn calculate_completion_percentage(&self) -> f32 {
        let total_features = self.features_complete.len()
            + self.features_in_progress.len()
            + self.features_blocked.len();

        (self.features_complete.len() as f32 / total_features as f32) * 100.0
    }

    fn log_timeline_deviations(&self) {
        // Track if behind/ahead of schedule
        // Example: Week 3 but only Week 1-2 deliverables complete
    }
}
```

### Decision Documentation

#### Decision Log Template

```markdown
## Decision: [Short Title]
**Date**: YYYY-MM-DD
**Status**: Accepted | Rejected | Superseded
**Context**: What problem are we trying to solve?
**Decision**: What did we decide?
**Alternatives Considered**:
1. Option A - Pros/Cons
2. Option B - Pros/Cons
**Consequences**: What are the implications?
**References**: Links to discussions, PRs, issues
```

#### Key Decisions Made

##### Decision: Use Turso over PostgreSQL

**Date**: 2025-11-06
**Status**: Accepted

**Context**: Need durable storage for episodes and patterns with good Rust support and analytics capabilities.

**Decision**: Use Turso/libSQL for primary storage instead of PostgreSQL.

**Rationale**:
- **Better Rust Integration**: Native libsql crate with async support
- **SQLite Compatibility**: Can use same code for testing (in-memory SQLite)
- **Performance**: 575x faster connection establishment
- **Edge Deployment**: Can deploy closer to users for lower latency
- **Cost**: More economical for startup/small-scale usage

**Alternatives Considered**:
1. **PostgreSQL** - Industry standard but higher latency, more infrastructure overhead
2. **SQLite Direct** - Simple but lacks remote/distributed capabilities
3. **MongoDB** - NoSQL flexibility but weaker analytics, different mental model

**Consequences**:
- ✅ Easier testing (SQLite in-memory)
- ✅ Lower latency for remote storage
- ⚠️ Network dependency (mitigated with circuit breakers)
- ⚠️ Newer ecosystem (fewer examples)

##### Decision: Use redb over sled

**Date**: 2025-11-06
**Status**: Accepted

**Context**: Need embedded key-value store for hot memory cache.

**Decision**: Use redb for cache layer instead of sled.

**Rationale**:
- **Better Performance**: Benchmarks show redb is faster for reads
- **Active Maintenance**: redb actively developed, sled less active
- **Cleaner API**: Simpler transaction model
- **ACID Guarantees**: Full ACID compliance

**Alternatives Considered**:
1. **sled** - Previously popular but maintenance concerns
2. **RocksDB** - High performance but C++ dependency, complex API
3. **LMDB** - Fast but Rust bindings less ergonomic

**Consequences**:
- ✅ Excellent read performance for cache layer
- ✅ Clean, safe API reduces bugs
- ⚠️ Synchronous API (need spawn_blocking for async)
- ⚠️ Smaller community (fewer examples)

##### Decision: TypeScript for MCP Tools

**Date**: 2025-11-06
**Status**: Accepted

**Context**: Need language for agent code execution in MCP integration.

**Decision**: Use TypeScript/JavaScript for executable agent code.

**Rationale**:
- **Ecosystem**: Massive npm package ecosystem
- **Familiarity**: Most developers know JavaScript/TypeScript
- **Sandboxing**: Good isolation options (VM2, Deno)
- **MCP Alignment**: MCP examples primarily use TypeScript

**Alternatives Considered**:
1. **Python** - Popular for AI/ML but harder to sandbox securely
2. **Rust** - Safe but compilation overhead, smaller ecosystem
3. **WASM** - Great sandboxing but limited ecosystem

**Consequences**:
- ✅ Rich ecosystem for tool development
- ✅ Fast iteration (no compilation step)
- ⚠️ Dynamic typing (mitigated with TypeScript)
- ⚠️ Sandbox security requires careful implementation

## Agentic Layer: Coordinated Development

### Executor Agent Assignments

#### 1. Storage Agent: Database Layer Implementation

**Responsibilities**: Implement storage backends (Turso + redb) with error handling and synchronization.

**Implementation Plan**:

```rust
// Week 1-2 Deliverables

// === Turso Storage Implementation ===

pub struct TursoStorage {
    client: Connection,
    config: StorageConfig,
}

impl TursoStorage {
    pub async fn new(url: &str, token: &str) -> Result<Self> {
        // Initialize Turso connection
        let client = libsql::Builder::new_remote(url.to_string(), token.to_string())
            .build()
            .await?;

        // Create schema if not exists
        Self::initialize_schema(&client).await?;

        Ok(Self {
            client: client.connect()?,
            config: StorageConfig::default(),
        })
    }

    async fn initialize_schema(client: &Database) -> Result<()> {
        let conn = client.connect()?;

        // Episodes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS episodes (
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
            )",
            [],
        ).await?;

        // Indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_episodes_task_type
             ON episodes(task_type)",
            [],
        ).await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_episodes_timestamp
             ON episodes(start_time DESC)",
            [],
        ).await?;

        // Patterns table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patterns (
                pattern_id TEXT PRIMARY KEY,
                pattern_type TEXT NOT NULL,
                pattern_data JSON NOT NULL,
                context JSON NOT NULL,
                success_rate REAL NOT NULL,
                occurrence_count INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ).await?;

        // Heuristics table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS heuristics (
                heuristic_id TEXT PRIMARY KEY,
                condition TEXT NOT NULL,
                action TEXT NOT NULL,
                confidence REAL NOT NULL,
                evidence JSON NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ).await?;

        Ok(())
    }

    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        let episode_json = serde_json::to_string(&episode)?;

        self.client.execute(
            "INSERT OR REPLACE INTO episodes (
                episode_id, task_type, task_description, context,
                start_time, end_time, steps, outcome, reward,
                reflection, patterns, metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                episode.episode_id.to_string(),
                episode.task_type.to_string(),
                episode.task_description,
                serde_json::to_string(&episode.context)?,
                episode.start_time.to_rfc3339(),
                episode.end_time.map(|t| t.to_rfc3339()),
                serde_json::to_string(&episode.steps)?,
                episode.outcome.as_ref().map(|o| serde_json::to_string(o).ok()).flatten(),
                episode.reward.as_ref().map(|r| serde_json::to_string(r).ok()).flatten(),
                episode.reflection.as_ref().map(|r| serde_json::to_string(r).ok()).flatten(),
                serde_json::to_string(&episode.patterns)?,
                serde_json::to_string(&episode.metadata)?,
                Utc::now().to_rfc3339(),
                Utc::now().to_rfc3339(),
            ],
        ).await?;

        Ok(())
    }

    pub async fn get_episode(&self, episode_id: Uuid) -> Result<Option<Episode>> {
        let mut rows = self.client.query(
            "SELECT * FROM episodes WHERE episode_id = ?",
            params![episode_id.to_string()],
        ).await?;

        if let Some(row) = rows.next().await? {
            let episode = Self::row_to_episode(&row)?;
            Ok(Some(episode))
        } else {
            Ok(None)
        }
    }

    pub async fn query_episodes_by_context(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<Episode>> {
        let mut rows = self.client.query(
            "SELECT * FROM episodes
             WHERE json_extract(context, '$.domain') = ?
             ORDER BY start_time DESC
             LIMIT ?",
            params![context.domain, limit],
        ).await?;

        let mut episodes = Vec::new();
        while let Some(row) = rows.next().await? {
            episodes.push(Self::row_to_episode(&row)?);
        }

        Ok(episodes)
    }
}

// === redb Storage Implementation ===

pub struct RedbStorage {
    db: Arc<redb::Database>,
    config: StorageConfig,
}

impl RedbStorage {
    pub fn new(path: &Path) -> Result<Self> {
        let db = redb::Database::create(path)?;

        Ok(Self {
            db: Arc::new(db),
            config: StorageConfig::default(),
        })
    }

    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        let db = self.db.clone();
        let episode_data = bincode::serialize(episode)?;
        let episode_id = episode.episode_id.to_string();

        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            {
                let mut table = write_txn.open_table(EPISODES_TABLE)?;
                table.insert(episode_id.as_str(), episode_data.as_slice())?;
            }
            write_txn.commit()?;
            Ok::<_, Error>(())
        })
        .await??;

        Ok(())
    }

    pub async fn get_episode(&self, episode_id: Uuid) -> Result<Option<Episode>> {
        let db = self.db.clone();
        let episode_id = episode_id.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read()?;
            let table = read_txn.open_table(EPISODES_TABLE)?;

            if let Some(value) = table.get(episode_id.as_str())? {
                let episode: Episode = bincode::deserialize(value.value())?;
                Ok(Some(episode))
            } else {
                Ok(None)
            }
        })
        .await??;

        Ok(result)
    }

    pub async fn store_embedding(
        &self,
        episode_id: Uuid,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let db = self.db.clone();
        let id = episode_id.to_string();
        let embedding_bytes = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect::<Vec<u8>>();

        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE)?;
                table.insert(id.as_str(), embedding_bytes.as_slice())?;
            }
            write_txn.commit()?;
            Ok::<_, Error>(())
        })
        .await??;

        Ok(())
    }
}

// === Storage Synchronization ===

pub struct StorageSynchronizer {
    turso: Arc<TursoStorage>,
    redb: Arc<RedbStorage>,
}

impl StorageSynchronizer {
    pub async fn sync_episode_to_cache(&self, episode_id: Uuid) -> Result<()> {
        // Fetch from Turso (source of truth)
        let episode = self.turso.get_episode(episode_id).await?
            .ok_or_else(|| Error::NotFound(episode_id))?;

        // Store in redb cache
        self.redb.store_episode(&episode).await?;

        Ok(())
    }

    pub async fn sync_all_recent_episodes(&self, since: DateTime<Utc>) -> Result<()> {
        // Fetch recent episodes from Turso
        let episodes = self.turso.query_episodes_since(since).await?;

        // Batch update redb
        for episode in episodes {
            self.redb.store_episode(&episode).await?;
        }

        Ok(())
    }
}
```

**Testing Strategy**:

```rust
#[cfg(test)]
mod storage_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_turso_episode_roundtrip() {
        let turso = TursoStorage::new(":memory:", "").await.unwrap();
        let episode = create_test_episode();

        turso.store_episode(&episode).await.unwrap();
        let retrieved = turso.get_episode(episode.episode_id).await.unwrap();

        assert_eq!(retrieved, Some(episode));
    }

    #[tokio::test]
    async fn test_redb_concurrent_access() {
        let temp_dir = TempDir::new().unwrap();
        let redb = Arc::new(RedbStorage::new(temp_dir.path()).unwrap());

        let handles: Vec<_> = (0..100)
            .map(|i| {
                let storage = redb.clone();
                tokio::spawn(async move {
                    let episode = create_test_episode_with_id(i);
                    storage.store_episode(&episode).await
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn test_storage_sync() {
        let turso = Arc::new(TursoStorage::new(":memory:", "").await.unwrap());
        let temp_dir = TempDir::new().unwrap();
        let redb = Arc::new(RedbStorage::new(temp_dir.path()).unwrap());

        let sync = StorageSynchronizer {
            turso: turso.clone(),
            redb: redb.clone(),
        };

        let episode = create_test_episode();
        turso.store_episode(&episode).await.unwrap();

        sync.sync_episode_to_cache(episode.episode_id).await.unwrap();

        let cached = redb.get_episode(episode.episode_id).await.unwrap();
        assert_eq!(cached, Some(episode));
    }
}
```

#### 2. Learning Agent: Episodic Learning Cycle

**Responsibilities**: Implement episode lifecycle, pattern extraction, and memory retrieval.

**Implementation Plan**:

```rust
// Week 3-4 Deliverables

pub struct SelfLearningMemory {
    turso_storage: Arc<TursoStorage>,
    redb_storage: Arc<RedbStorage>,
    pattern_extractor: Arc<PatternExtractor>,
    reward_calculator: Arc<RewardCalculator>,
    reflection_generator: Arc<ReflectionGenerator>,
    config: MemoryConfig,
}

impl SelfLearningMemory {
    pub async fn new(
        turso_url: &str,
        turso_token: &str,
        redb_path: &str,
    ) -> Result<Self> {
        let turso = Arc::new(TursoStorage::new(turso_url, turso_token).await?);
        let redb = Arc::new(RedbStorage::new(Path::new(redb_path))?);

        Ok(Self {
            turso_storage: turso,
            redb_storage: redb,
            pattern_extractor: Arc::new(PatternExtractor::new()),
            reward_calculator: Arc::new(RewardCalculator::new()),
            reflection_generator: Arc::new(ReflectionGenerator::new()),
            config: MemoryConfig::default(),
        })
    }

    /// Pre-Task: Start new episode with context
    #[instrument(skip(self))]
    pub async fn start_episode(
        &self,
        task_description: &str,
        context: TaskContext,
    ) -> Result<Uuid> {
        let episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: Self::infer_task_type(task_description, &context),
            task_description: task_description.to_string(),
            context,
            start_time: Utc::now(),
            end_time: None,
            steps: Vec::new(),
            outcome: None,
            reward: None,
            reflection: None,
            patterns: Vec::new(),
            metadata: HashMap::new(),
        };

        // Store in both Turso and redb
        self.turso_storage.store_episode(&episode).await?;
        self.redb_storage.store_episode(&episode).await?;

        info!(episode_id = %episode.episode_id, "Episode started");
        Ok(episode.episode_id)
    }

    /// Execution: Log step during task execution
    #[instrument(skip(self))]
    pub async fn log_step(
        &self,
        episode_id: Uuid,
        step: ExecutionStep,
    ) -> Result<()> {
        // Fetch episode from cache (redb) first
        let mut episode = self.redb_storage.get_episode(episode_id).await?
            .ok_or_else(|| Error::NotFound(episode_id))?;

        // Add step
        episode.steps.push(step);

        // Update both storages
        self.turso_storage.store_episode(&episode).await?;
        self.redb_storage.store_episode(&episode).await?;

        debug!(episode_id = %episode_id, step_count = episode.steps.len(), "Step logged");
        Ok(())
    }

    /// Post-Task: Complete episode with outcome and analysis
    #[instrument(skip(self))]
    pub async fn complete_episode(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<Episode> {
        let mut episode = self.turso_storage.get_episode(episode_id).await?
            .ok_or_else(|| Error::NotFound(episode_id))?;

        episode.end_time = Some(Utc::now());
        episode.outcome = Some(outcome.clone());

        // Calculate reward
        episode.reward = Some(
            self.reward_calculator.calculate_reward(&episode, &outcome).await?
        );

        // Generate reflection
        episode.reflection = Some(
            self.reflection_generator.generate_reflection(&episode, &outcome).await?
        );

        // Learning: Extract patterns
        let patterns = self.pattern_extractor.extract_patterns(&episode).await?;

        // Store patterns
        for pattern in &patterns {
            self.store_pattern(pattern).await?;
        }

        episode.patterns = patterns.iter().map(|p| p.id()).collect();

        // Update storages
        self.turso_storage.store_episode(&episode).await?;
        self.redb_storage.store_episode(&episode).await?;

        info!(episode_id = %episode_id, patterns_extracted = patterns.len(), "Episode completed");
        Ok(episode)
    }

    /// Retrieval: Find relevant episodes based on context
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_context(
        &self,
        description: &str,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<Episode>> {
        // Try cache first (redb)
        let cache_results = self.query_cache_by_context(context, limit).await?;

        if !cache_results.is_empty() {
            debug!("Retrieved {} episodes from cache", cache_results.len());
            return Ok(cache_results);
        }

        // Fallback to Turso
        let results = self.turso_storage
            .query_episodes_by_context(context, limit)
            .await?;

        debug!("Retrieved {} episodes from Turso", results.len());
        Ok(results)
    }

    async fn query_cache_by_context(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<Episode>> {
        // Implementation: scan redb for matching episodes
        // This is a simplified version; real implementation would use indexing
        Ok(Vec::new())
    }
}

// === Reward Calculation ===

pub struct RewardCalculator;

impl RewardCalculator {
    pub async fn calculate_reward(
        &self,
        episode: &Episode,
        outcome: &TaskOutcome,
    ) -> Result<RewardScore> {
        let duration = episode.end_time.unwrap() - episode.start_time;

        let base_reward = match outcome {
            TaskOutcome::Success { .. } => 1.0,
            TaskOutcome::PartialSuccess { .. } => 0.5,
            TaskOutcome::Failure { .. } => 0.0,
        };

        // Adjust for efficiency
        let efficiency_multiplier = Self::calculate_efficiency(duration, episode.steps.len());

        // Adjust for complexity
        let complexity_multiplier = Self::calculate_complexity_bonus(&episode.context);

        Ok(RewardScore {
            total: base_reward * efficiency_multiplier * complexity_multiplier,
            base: base_reward,
            efficiency: efficiency_multiplier,
            complexity_bonus: complexity_multiplier,
        })
    }

    fn calculate_efficiency(duration: Duration, step_count: usize) -> f32 {
        // Reward faster execution with fewer steps
        let duration_secs = duration.num_seconds() as f32;
        let efficiency = 1.0 / (1.0 + (duration_secs / 60.0).ln() + (step_count as f32 / 10.0));
        efficiency.clamp(0.5, 1.5)
    }

    fn calculate_complexity_bonus(context: &TaskContext) -> f32 {
        match context.complexity {
            ComplexityLevel::Simple => 1.0,
            ComplexityLevel::Moderate => 1.2,
            ComplexityLevel::Complex => 1.5,
        }
    }
}

// === Reflection Generation ===

pub struct ReflectionGenerator;

impl ReflectionGenerator {
    pub async fn generate_reflection(
        &self,
        episode: &Episode,
        outcome: &TaskOutcome,
    ) -> Result<Reflection> {
        // Analyze what worked well
        let successes = Self::identify_successes(&episode.steps, outcome);

        // Analyze what could be improved
        let improvements = Self::identify_improvements(&episode.steps, outcome);

        // Generate key insights
        let insights = Self::generate_insights(&episode.steps, outcome);

        Ok(Reflection {
            successes,
            improvements,
            insights,
            generated_at: Utc::now(),
        })
    }

    fn identify_successes(steps: &[ExecutionStep], outcome: &TaskOutcome) -> Vec<String> {
        let mut successes = Vec::new();

        // Identify successful tool sequences
        let successful_tools: Vec<_> = steps
            .iter()
            .filter(|s| s.result.as_ref().map(|r| r.is_success()).unwrap_or(false))
            .map(|s| s.tool.clone())
            .collect();

        if !successful_tools.is_empty() {
            successes.push(format!(
                "Successfully used tools: {}",
                successful_tools.join(", ")
            ));
        }

        successes
    }

    fn identify_improvements(steps: &[ExecutionStep], outcome: &TaskOutcome) -> Vec<String> {
        let mut improvements = Vec::new();

        // Identify failed steps
        let failed_steps: Vec<_> = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.result.as_ref().map(|r| r.is_success()).unwrap_or(true))
            .collect();

        for (idx, step) in failed_steps {
            improvements.push(format!(
                "Step {}: {} failed - consider alternative approach",
                idx + 1,
                step.tool
            ));
        }

        improvements
    }

    fn generate_insights(steps: &[ExecutionStep], outcome: &TaskOutcome) -> Vec<String> {
        vec![
            format!("Task completed in {} steps", steps.len()),
            format!("Outcome: {:?}", outcome),
        ]
    }
}
```

**Testing Strategy**:

```rust
#[cfg(test)]
mod learning_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_learning_cycle() {
        let memory = setup_test_memory().await;

        // Start episode
        let episode_id = memory
            .start_episode("Test task", test_context())
            .await
            .unwrap();

        // Log steps
        for i in 0..5 {
            let step = create_test_step(i);
            memory.log_step(episode_id, step).await.unwrap();
        }

        // Complete episode
        let outcome = TaskOutcome::Success {
            verdict: "All tests passing".to_string(),
            artifacts: vec![],
        };
        let completed = memory.complete_episode(episode_id, outcome).await.unwrap();

        // Verify analysis
        assert!(completed.reward.is_some());
        assert!(completed.reflection.is_some());
        assert!(completed.end_time.is_some());
    }

    #[tokio::test]
    async fn test_memory_retrieval() {
        let memory = setup_test_memory().await;

        // Create and store multiple episodes
        for i in 0..10 {
            let episode = create_completed_episode(i);
            memory.turso_storage.store_episode(&episode).await.unwrap();
        }

        // Retrieve relevant episodes
        let results = memory
            .retrieve_relevant_context("test query", &test_context(), 5)
            .await
            .unwrap();

        assert!(!results.is_empty());
        assert!(results.len() <= 5);
    }
}
```

#### 3. MCP Agent: Code Execution Integration

**Responsibilities**: Implement MCP server, tool generation, and secure sandboxing.

**Implementation Plan**:

```rust
// Week 7-8 Deliverables

pub struct MemoryMCPServer {
    memory: Arc<SelfLearningMemory>,
    sandbox: Arc<CodeSandbox>,
    tools: Arc<RwLock<Vec<Tool>>>,
}

impl MemoryMCPServer {
    pub async fn new(memory: Arc<SelfLearningMemory>) -> Result<Self> {
        Ok(Self {
            memory,
            sandbox: Arc::new(CodeSandbox::new()?),
            tools: Arc::new(RwLock::new(Self::create_default_tools())),
        })
    }

    fn create_default_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "query_memory".to_string(),
                description: "Query episodic memory for relevant past experiences".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"},
                        "context": {"type": "object", "description": "Task context"},
                        "limit": {"type": "integer", "default": 10}
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "execute_agent_code".to_string(),
                description: "Execute TypeScript code in secure sandbox".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string", "description": "TypeScript code to execute"},
                        "context": {"type": "string", "description": "Execution context as JSON"}
                    },
                    "required": ["code", "context"]
                }),
            },
        ]
    }

    pub async fn execute_agent_code(
        &self,
        code: &str,
        context: String,
    ) -> Result<ExecutionResult> {
        self.sandbox.execute(code, context).await
    }
}

// === Code Sandbox ===

pub struct CodeSandbox {
    config: SandboxConfig,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_execution_time: Duration,
    pub max_memory_mb: usize,
    pub allowed_modules: Vec<String>,
}

impl CodeSandbox {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: SandboxConfig {
                max_execution_time: Duration::from_secs(5),
                max_memory_mb: 128,
                allowed_modules: vec!["fs".to_string(), "path".to_string()],
            },
        })
    }

    pub async fn execute(
        &self,
        code: &str,
        context: String,
    ) -> Result<ExecutionResult> {
        // Create isolated Node.js process
        let wrapper_code = self.create_wrapper(code, &context)?;

        // Execute with timeout
        let output = tokio::time::timeout(
            self.config.max_execution_time,
            self.run_in_isolated_process(&wrapper_code),
        )
        .await
        .map_err(|_| Error::ExecutionTimeout)??;

        Ok(output)
    }

    fn create_wrapper(&self, user_code: &str, context: &str) -> Result<String> {
        // Wrap user code with safety checks
        let wrapped = format!(
            r#"
            const {{ VM }} = require('vm2');
            const vm = new VM({{
                timeout: {},
                sandbox: {{
                    context: {},
                    console: console,
                }},
            }});

            try {{
                const result = vm.run(`
                    (async () => {{
                        {}
                    }})()
                `);
                console.log(JSON.stringify({{ success: true, result }}));
            }} catch (error) {{
                console.error(JSON.stringify({{ success: false, error: error.message }}));
            }}
            "#,
            self.config.max_execution_time.as_millis(),
            context,
            user_code
        );

        Ok(wrapped)
    }

    async fn run_in_isolated_process(&self, code: &str) -> Result<ExecutionResult> {
        let output = tokio::process::Command::new("node")
            .arg("-e")
            .arg(code)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ExecutionResult::Success { output: stdout })
        } else {
            Ok(ExecutionResult::Error { message: stderr })
        }
    }
}
```

**Testing Strategy**:

```rust
#[cfg(test)]
mod mcp_tests {
    use super::*;

    #[tokio::test]
    async fn test_code_execution() {
        let sandbox = CodeSandbox::new().unwrap();

        let code = r#"
            const result = 1 + 1;
            console.log(result);
        "#;

        let result = sandbox.execute(code, "{}".to_string()).await.unwrap();

        match result {
            ExecutionResult::Success { output } => {
                assert!(output.contains("2"));
            }
            _ => panic!("Expected successful execution"),
        }
    }

    #[tokio::test]
    async fn test_sandbox_timeout() {
        let sandbox = CodeSandbox::new().unwrap();

        let code = r#"
            await new Promise(resolve => setTimeout(resolve, 10000));
        "#;

        let result = sandbox.execute(code, "{}".to_string()).await;
        assert!(result.is_err());
    }
}
```

### Coordinator: Integration & Error Handling

**Responsibilities**: Ensure consistent patterns across components and coordinate integration.

**Error Handling Strategy**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Learning error: {0}")]
    Learning(String),

    #[error("MCP error: {0}")]
    MCP(String),

    #[error("Episode not found: {0}")]
    NotFound(Uuid),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] libsql::Error),

    #[error("Execution timeout")]
    ExecutionTimeout,

    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
}

impl Error {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::Storage(_) => true,        // Retry with backoff
            Error::Database(_) => true,       // Retry with backoff
            Error::Learning(_) => false,      // Log and continue
            Error::MCP(_) => true,           // Fallback to basic tools
            Error::NotFound(_) => false,      // Cannot recover
            Error::Serialization(_) => false, // Data corruption
            Error::ExecutionTimeout => true,  // Retry with different settings
            Error::CircuitBreakerOpen => true, // Wait and retry
        }
    }
}
```

## Execution Complete Criteria

Before proceeding to Phase 4 (REVIEW), ensure:

- [x] Storage layer complete with Turso + redb implementations ✅ **DONE** (2025-11-08)
  - Note: Created unified `StorageBackend` trait abstraction in memory-core/src/storage.rs
  - Note: redb v2.1 API updated with proper `clear_all()` implementation
  - Note: Both Turso and redb backends fully integrated into memory-core
- [x] Episode lifecycle working (start → log → complete) ✅ **DONE** (2025-11-08)
  - Note: Full lifecycle tested in memory-core tests
  - Note: Proper integration with both storage backends
- [x] Pattern extraction generating valid patterns ✅ **DONE** (2025-11-08)
  - Note: Pattern extraction logic implemented in memory-core
  - Note: Tested with compliance tests FR4
- [x] Memory retrieval returning relevant episodes ✅ **DONE** (2025-11-08)
  - Note: Retrieval tested in compliance tests FR5
  - Note: Context-aware retrieval implemented
- [x] MCP server functional with basic tools ✅ **DONE** (2025-11-08)
  - Note: MCP server wired to actual memory system
  - Note: Integration tests confirm proper operation
- [x] Code execution sandbox secure and tested ✅ **DONE** (2025-11-08)
  - Note: 30+ penetration tests created and passing
  - Note: Sandbox security validated with comprehensive attack simulations
  - Note: Process isolation, timeout enforcement, resource limits all tested
- [x] All unit tests passing (>90% coverage) ✅ **DONE** (2025-11-08)
  - Note: cargo-llvm-cov added to CI with >90% coverage gate
  - Note: Comprehensive test suite covering all major components
- [x] Integration tests passing (full cycle) ✅ **DONE** (2025-11-08)
  - Note: Compliance tests (FR1-FR7) created and passing
  - Note: Performance tests (NFR1-NFR5) created
  - Note: Regression tests created
- [x] Documentation for all public APIs ✅ **DONE** (2025-11-08)
  - Note: Comprehensive rustdoc added to all public APIs in memory-core
  - Note: 10+ working examples with detailed explanations
  - Note: 38 doc tests, all passing

## Next Steps

Once execution is complete:

1. ✅ Verify all components implemented
2. ✅ Run full test suite
3. ✅ Document any deviations from plan
4. ➡️ **Proceed to [Phase 4: REVIEW](./04-review.md)** - Quality assessment

## References

- [Phase 2: PLAN](./02-plan.md) - Implementation roadmap
- [Phase 4: REVIEW](./04-review.md) - Next phase (quality review)
- [AGENTS.md](../AGENTS.md) - Coding standards and practices
