# Consolidation Patterns

Common refactoring patterns for consolidating duplicate code and improving codebase structure.

## Pattern 1: Extract Function

**When to Use**: Duplicate code blocks that perform the same operation.

### Example: Duplicate Validation Logic

**Before**:
```rust
// In 3 different handlers
async fn create_episode(req: Json<CreateRequest>) -> ApiResult<Episode> {
    if req.task_description.is_empty() {
        return Err(ApiError::ValidationError("Task description required".into()));
    }
    if req.task_description.len() > 1000 {
        return Err(ApiError::ValidationError("Task description too long".into()));
    }
    // ... rest of handler
}

async fn update_episode(id: Uuid, req: Json<UpdateRequest>) -> ApiResult<Episode> {
    if req.task_description.is_empty() {
        return Err(ApiError::ValidationError("Task description required".into()));
    }
    if req.task_description.len() > 1000 {
        return Err(ApiError::ValidationError("Task description too long".into()));
    }
    // ... rest of handler
}
```

**After**:
```rust
// Extracted validation function
fn validate_task_description(desc: &str) -> Result<(), ApiError> {
    if desc.is_empty() {
        return Err(ApiError::ValidationError("Task description required".into()));
    }
    if desc.len() > 1000 {
        return Err(ApiError::ValidationError("Task description too long".into()));
    }
    Ok(())
}

// Simplified handlers
async fn create_episode(req: Json<CreateRequest>) -> ApiResult<Episode> {
    validate_task_description(&req.task_description)?;
    // ... rest of handler
}

async fn update_episode(id: Uuid, req: Json<UpdateRequest>) -> ApiResult<Episode> {
    validate_task_description(&req.task_description)?;
    // ... rest of handler
}
```

**Benefits**:
- ✅ Single source of truth
- ✅ Easier to update validation rules
- ✅ Can be unit tested independently
- ✅ Reduces LOC by ~40%

## Pattern 2: Extract Module

**When to Use**: Multiple related functions scattered across files.

### Example: Authentication Logic

**Before**:
```
src/
  ├── api/handlers.rs (has auth validation)
  ├── middleware/auth.rs (has token parsing)
  ├── services/user.rs (has password hashing)
  └── utils/jwt.rs (has JWT creation)
```

**After**:
```
src/
  ├── auth/
  │   ├── mod.rs (public API)
  │   ├── validation.rs (token validation)
  │   ├── tokens.rs (JWT creation/parsing)
  │   └── password.rs (hashing/verification)
  └── ... (other modules use auth::*)
```

**Implementation**:
```rust
// auth/mod.rs - unified API
pub use validation::validate_token;
pub use tokens::{create_token, parse_token};
pub use password::{hash_password, verify_password};

pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub async fn authenticate(&self, credentials: Credentials) -> Result<User> {
        // Orchestrates the auth flow using internal modules
    }
}
```

## Pattern 3: Strategy Pattern

**When to Use**: Same logic with different implementations (if/else or match chains).

### Example: Multiple Pattern Extractors

**Before**:
```rust
fn extract_patterns(episode: &Episode) -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // Extract tool sequences
    for window in episode.steps.windows(3) {
        if is_tool_sequence(window) {
            patterns.push(Pattern::ToolSequence(/* ... */));
        }
    }

    // Extract decision points
    for step in &episode.steps {
        if is_decision_point(step) {
            patterns.push(Pattern::DecisionPoint(/* ... */));
        }
    }

    // Extract error recoveries
    for window in episode.steps.windows(2) {
        if is_error_recovery(window) {
            patterns.push(Pattern::ErrorRecovery(/* ... */));
        }
    }

    patterns
}
```

**After**:
```rust
trait PatternExtractor: Send + Sync {
    fn extract(&self, episode: &Episode) -> Vec<Pattern>;
}

struct ToolSequenceExtractor;
impl PatternExtractor for ToolSequenceExtractor {
    fn extract(&self, episode: &Episode) -> Vec<Pattern> {
        episode.steps
            .windows(3)
            .filter(|w| self.is_tool_sequence(w))
            .map(|w| self.build_pattern(w))
            .collect()
    }
}

struct DecisionPointExtractor;
impl PatternExtractor for DecisionPointExtractor { /* ... */ }

struct ErrorRecoveryExtractor;
impl PatternExtractor for ErrorRecoveryExtractor { /* ... */ }

// Composable extraction
struct PatternExtractionService {
    extractors: Vec<Box<dyn PatternExtractor>>,
}

impl PatternExtractionService {
    fn new() -> Self {
        Self {
            extractors: vec![
                Box::new(ToolSequenceExtractor),
                Box::new(DecisionPointExtractor),
                Box::new(ErrorRecoveryExtractor),
            ],
        }
    }

    fn extract_all(&self, episode: &Episode) -> Vec<Pattern> {
        self.extractors
            .iter()
            .flat_map(|e| e.extract(episode))
            .collect()
    }
}
```

**Benefits**:
- ✅ Easy to add new pattern types
- ✅ Each extractor is independently testable
- ✅ Can enable/disable extractors dynamically
- ✅ Follows Open/Closed Principle

## Pattern 4: Builder Pattern

**When to Use**: Complex object construction with many optional parameters.

### Example: Episode Creation

**Before**:
```rust
// 7+ parameters, hard to remember order
fn create_episode(
    task_desc: String,
    task_type: String,
    context: HashMap<String, String>,
    tags: Vec<String>,
    language: Option<String>,
    domain: Option<String>,
    priority: u8,
) -> Result<Episode> {
    // ... construction logic
}

// Call site - what do these parameters mean?
let episode = create_episode(
    "Task description".into(),
    "coding".into(),
    HashMap::new(),
    vec!["rust".into()],
    Some("rust".into()),
    Some("backend".into()),
    5,
)?;
```

**After**:
```rust
pub struct EpisodeBuilder {
    task_description: String,
    task_type: String,
    context: TaskContext,
    tags: Vec<String>,
    language: Option<String>,
    domain: Option<String>,
    priority: u8,
}

impl EpisodeBuilder {
    pub fn new(task_description: impl Into<String>) -> Self {
        Self {
            task_description: task_description.into(),
            task_type: "general".into(),
            context: TaskContext::default(),
            tags: Vec::new(),
            language: None,
            domain: None,
            priority: 5,
        }
    }

    pub fn task_type(mut self, task_type: impl Into<String>) -> Self {
        self.task_type = task_type.into();
        self
    }

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub async fn build(self, service: &MemoryService) -> Result<Episode> {
        service.create_episode_from_builder(self).await
    }
}

// Call site - clear and readable
let episode = EpisodeBuilder::new("Task description")
    .task_type("coding")
    .language("rust")
    .tag("backend")
    .tag("api")
    .build(&service)
    .await?;
```

## Pattern 5: Repository Pattern

**When to Use**: Duplicate CRUD operations for different entities.

### Example: Database Operations

**Before**:
```rust
// Repeated for each entity type
async fn create_episode(db: &Database, episode: Episode) -> Result<Episode> {
    let sql = "INSERT INTO episodes (id, task_description, ...) VALUES (?, ?, ...)";
    db.execute(sql, params![episode.id, episode.task_description, ...]).await?;
    Ok(episode)
}

async fn get_episode(db: &Database, id: Uuid) -> Result<Episode> {
    let sql = "SELECT * FROM episodes WHERE id = ?";
    let row = db.query_row(sql, params![id]).await?;
    Ok(Episode::from_row(&row)?)
}

async fn create_pattern(db: &Database, pattern: Pattern) -> Result<Pattern> {
    let sql = "INSERT INTO patterns (id, pattern_type, ...) VALUES (?, ?, ...)";
    db.execute(sql, params![pattern.id, pattern.pattern_type, ...]).await?;
    Ok(pattern)
}

async fn get_pattern(db: &Database, id: Uuid) -> Result<Pattern> {
    let sql = "SELECT * FROM patterns WHERE id = ?";
    let row = db.query_row(sql, params![id]).await?;
    Ok(Pattern::from_row(&row)?)
}
```

**After**:
```rust
#[async_trait]
trait Repository<T>: Send + Sync {
    async fn create(&self, entity: T) -> Result<T>;
    async fn get(&self, id: Uuid) -> Result<T>;
    async fn update(&self, id: Uuid, entity: T) -> Result<T>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<T>>;
}

struct EpisodeRepository {
    db: Database,
}

#[async_trait]
impl Repository<Episode> for EpisodeRepository {
    async fn create(&self, episode: Episode) -> Result<Episode> {
        let sql = "INSERT INTO episodes (id, task_description, ...) VALUES (?, ?, ...)";
        self.db.execute(sql, episode.to_params()).await?;
        Ok(episode)
    }

    // ... other methods
}

struct PatternRepository {
    db: Database,
}

#[async_trait]
impl Repository<Pattern> for PatternRepository {
    // Similar implementation for patterns
}

// Usage
let episode_repo = EpisodeRepository::new(db);
let episode = episode_repo.create(new_episode).await?;
```

## Pattern 6: Facade Pattern

**When to Use**: Simplifying complex subsystem interactions.

### Example: Memory System Facade

**Before**:
```rust
// Client code has to know about all subsystems
let turso = TursoStorage::new(config).await?;
let redb = RedbStorage::new(path)?;
let embeddings = EmbeddingService::new(api_key)?;
let extractor = PatternExtractor::new();

// Manual orchestration
let episode = turso.create_episode(request).await?;
redb.cache_episode(&episode).await?;
let embedding = embeddings.embed(&episode.task_description).await?;
redb.store_embedding(episode.id, embedding).await?;
```

**After**:
```rust
// Facade provides simplified API
pub struct SelfLearningMemory {
    turso: TursoStorage,
    redb: RedbStorage,
    embeddings: Option<EmbeddingService>,
    extractor: PatternExtractor,
}

impl SelfLearningMemory {
    pub async fn start_episode(&self, request: StartEpisodeRequest) -> Result<Episode> {
        // Orchestrates all subsystems
        let episode = self.turso.create_episode(&request).await?;
        self.redb.cache_episode(&episode).await?;

        if let Some(embeddings) = &self.embeddings {
            let embedding = embeddings.embed(&request.task_description).await?;
            self.redb.store_embedding(episode.id, embedding).await?;
        }

        Ok(episode)
    }
}

// Client code is simple
let memory = SelfLearningMemory::new(config).await?;
let episode = memory.start_episode(request).await?;
```

## Pattern 7: Template Method

**When to Use**: Common algorithm structure with varying steps.

### Example: Storage Sync Operations

**Before**:
```rust
async fn sync_episodes() -> Result<()> {
    // 1. Fetch from Turso
    let episodes = turso.fetch_recent_episodes().await?;

    // 2. Filter modified
    let modified = episodes.into_iter()
        .filter(|e| needs_sync(&e))
        .collect::<Vec<_>>();

    // 3. Update redb
    for episode in modified {
        redb.update_episode(&episode).await?;
    }

    // 4. Log sync
    info!("Synced {} episodes", modified.len());

    Ok(())
}

async fn sync_patterns() -> Result<()> {
    // Same structure but for patterns
    let patterns = turso.fetch_recent_patterns().await?;
    let modified = patterns.into_iter()
        .filter(|p| needs_sync(&p))
        .collect::<Vec<_>>();
    for pattern in modified {
        redb.update_pattern(&pattern).await?;
    }
    info!("Synced {} patterns", modified.len());
    Ok(())
}
```

**After**:
```rust
#[async_trait]
trait Syncable: Send + Sync {
    type Item;

    async fn fetch_from_source(&self) -> Result<Vec<Self::Item>>;
    async fn update_cache(&self, item: &Self::Item) -> Result<()>;
    fn needs_sync(&self, item: &Self::Item) -> bool;
    fn entity_name(&self) -> &str;
}

struct SyncService<T: Syncable> {
    syncable: T,
}

impl<T: Syncable> SyncService<T> {
    // Template method - same algorithm for all types
    async fn sync(&self) -> Result<usize> {
        // 1. Fetch
        let items = self.syncable.fetch_from_source().await?;

        // 2. Filter
        let to_sync: Vec<_> = items.into_iter()
            .filter(|item| self.syncable.needs_sync(item))
            .collect();

        // 3. Update
        for item in &to_sync {
            self.syncable.update_cache(item).await?;
        }

        // 4. Log
        info!("Synced {} {}", to_sync.len(), self.syncable.entity_name());

        Ok(to_sync.len())
    }
}

// Implementations
struct EpisodeSyncable { /* ... */ }
impl Syncable for EpisodeSyncable { /* ... */ }

struct PatternSyncable { /* ... */ }
impl Syncable for PatternSyncable { /* ... */ }

// Usage
let episode_sync = SyncService::new(EpisodeSyncable::new(turso, redb));
episode_sync.sync().await?;
```

## Pattern 8: Type State Pattern

**When to Use**: Enforcing state transitions at compile time.

### Example: Episode Lifecycle

**Before**:
```rust
struct Episode {
    id: Uuid,
    status: EpisodeStatus, // Can be invalid states
    steps: Vec<ExecutionStep>,
    outcome: Option<TaskOutcome>,
    reward: Option<RewardScore>,
}

// Runtime checks everywhere
fn complete_episode(episode: &mut Episode, outcome: TaskOutcome) -> Result<()> {
    if episode.status != EpisodeStatus::InProgress {
        return Err(Error::InvalidState);
    }
    if episode.steps.is_empty() {
        return Err(Error::NoSteps);
    }
    episode.status = EpisodeStatus::Completed;
    episode.outcome = Some(outcome);
    Ok(())
}
```

**After**:
```rust
// Type states
struct Created;
struct InProgress;
struct Completed;

struct Episode<State = Created> {
    id: Uuid,
    task_description: String,
    _state: PhantomData<State>,
}

// State-specific data
struct EpisodeInProgress {
    steps: Vec<ExecutionStep>,
}

struct EpisodeCompleted {
    steps: Vec<ExecutionStep>,
    outcome: TaskOutcome,
    reward: RewardScore,
}

// Type-safe transitions
impl Episode<Created> {
    pub fn start(self) -> (Episode<InProgress>, EpisodeInProgress) {
        (
            Episode {
                id: self.id,
                task_description: self.task_description,
                _state: PhantomData,
            },
            EpisodeInProgress {
                steps: Vec::new(),
            },
        )
    }
}

impl Episode<InProgress> {
    pub fn log_step(&self, data: &mut EpisodeInProgress, step: ExecutionStep) {
        data.steps.push(step);
    }

    pub fn complete(
        self,
        data: EpisodeInProgress,
        outcome: TaskOutcome,
    ) -> (Episode<Completed>, EpisodeCompleted) {
        let reward = calculate_reward(&data.steps, &outcome);
        (
            Episode {
                id: self.id,
                task_description: self.task_description,
                _state: PhantomData,
            },
            EpisodeCompleted {
                steps: data.steps,
                outcome,
                reward,
            },
        )
    }
}

// Usage - invalid states are impossible
let episode = Episode::create(request);
let (episode, mut data) = episode.start();
episode.log_step(&mut data, step1);
episode.log_step(&mut data, step2);
let (episode, data) = episode.complete(data, outcome);
// episode.log_step() <- Compile error! Episode is completed
```

## Pattern 9: Newtype Pattern

**When to Use**: Preventing type confusion with similar primitives.

### Example: ID Types

**Before**:
```rust
async fn get_episode(episode_id: Uuid) -> Result<Episode> { /* ... */ }
async fn get_pattern(pattern_id: Uuid) -> Result<Pattern> { /* ... */ }

// Easy to mix up
let episode_id = get_episode_id();
let pattern = get_pattern(episode_id).await?; // Wrong ID type! 🐛
```

**After**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpisodeId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternId(Uuid);

impl EpisodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

async fn get_episode(episode_id: EpisodeId) -> Result<Episode> { /* ... */ }
async fn get_pattern(pattern_id: PatternId) -> Result<Pattern> { /* ... */ }

// Type safety!
let episode_id = EpisodeId::new();
let pattern_id = PatternId::new();
let pattern = get_pattern(episode_id).await?; // Compile error! ✅
```

## Pattern 10: Parallel Refactoring

**When to Use**: Safely refactoring while keeping old code working.

### Example: Migrating to New API

**Step 1**: Create new API alongside old
```rust
// Old API (deprecated but still works)
#[deprecated(note = "Use MemoryService::start_episode instead")]
pub async fn create_episode(req: CreateRequest) -> Result<Episode> {
    // Forward to new API
    let service = get_memory_service();
    service.start_episode(req).await
}

// New API
pub struct MemoryService { /* ... */ }
impl MemoryService {
    pub async fn start_episode(&self, req: StartEpisodeRequest) -> Result<Episode> {
        // New implementation
    }
}
```

**Step 2**: Migrate callers incrementally
```rust
// Phase 1: Some callers still use old API
let episode = create_episode(req).await?; // Warning: deprecated

// Phase 2: New callers use new API
let service = MemoryService::new(config);
let episode = service.start_episode(req).await?;
```

**Step 3**: Remove old API after migration complete

## Consolidation Checklist

When consolidating code, verify:

- [ ] **Semantics preserved**: Refactored code has same behavior
- [ ] **Tests pass**: All existing tests still pass
- [ ] **New tests added**: Extracted code has unit tests
- [ ] **Performance maintained**: No significant slowdown
- [ ] **Documentation updated**: Docs reflect new structure
- [ ] **No breaking changes**: Or documented migration path provided
- [ ] **Linter warnings resolved**: No new warnings introduced
- [ ] **Code review completed**: Changes reviewed by team

## Anti-Patterns to Avoid

### Anti-Pattern 1: Over-Abstraction
```rust
// Too abstract - harder to understand than original
trait AbstractFactoryStrategy<T, U, V>: Send + Sync {
    type Output;
    fn create(&self, input: T) -> impl Future<Output = Result<Self::Output>>;
    fn transform(&self, value: U) -> V;
}
```

**Better**: Use concrete types until abstraction is clearly needed (Rule of Three).

### Anti-Pattern 2: Premature Generalization
Don't extract after 1 duplicate. Wait for 2-3 instances to see the true pattern.

### Anti-Pattern 3: Wrong Abstraction
Abstracting too early can create the wrong abstraction, making future changes harder.

**Guideline**: Prefer duplication over wrong abstraction initially.

## Summary

**Pattern Selection Guide**:

| Situation | Pattern |
|-----------|---------|
| Duplicate code blocks | Extract Function |
| Related functions scattered | Extract Module |
| Multiple implementations of same concept | Strategy Pattern |
| Complex object creation | Builder Pattern |
| Repeated CRUD operations | Repository Pattern |
| Complex subsystem | Facade Pattern |
| Algorithm with varying steps | Template Method |
| State machine | Type State Pattern |
| Primitive obsession | Newtype Pattern |
| Safe migration | Parallel Refactoring |
