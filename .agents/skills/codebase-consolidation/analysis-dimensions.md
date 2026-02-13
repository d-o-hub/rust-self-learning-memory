# Analysis Dimensions - Detailed Criteria

Comprehensive criteria and checks for each of the 8 analysis dimensions.

## 1. Code Duplication Analysis

### Detection Methods

#### Exact Duplicates
```bash
# Find identical code blocks
# Use tools like:
# - jscpd (JavaScript, TypeScript, Python, etc.)
# - PMD CPD (Java, Python, Ruby, etc.)
# - simian (commercial, multi-language)

# Example with grep for simple pattern matching
rg "fn validate_token" -A 20 | sort | uniq -d
```

#### Near Duplicates
- Code with 80-95% similarity
- Same logic, different variable names
- Copy-paste with minor modifications

#### Structural Duplicates
- Same pattern, different implementation
- Common in CRUD operations, API handlers
- Opportunities for abstraction

### Evaluation Criteria

**Severity Classification**:
- 🔴 **Critical**: Security/auth logic duplicated (immediate fix)
- 🟡 **High**: Business logic duplicated (fix this sprint)
- 🟢 **Medium**: Helper functions duplicated (fix this quarter)
- ⚪ **Low**: Boilerplate code (acceptable if <3 instances)

### Consolidation Decision Matrix

| Similarity | Instances | Complexity | Action |
|-----------|-----------|------------|--------|
| >95% | 2+ | Any | Consolidate immediately |
| 80-95% | 3+ | High | Consolidate with parameterization |
| 60-80% | 5+ | High | Extract common logic |
| <60% | Any | Any | Consider design pattern |

### Common Duplication Patterns

**Pattern 1: Copy-Paste Evolution**
```rust
// auth/local.rs
async fn authenticate_local(credentials: Credentials) -> Result<User> {
    validate_credentials(&credentials)?;
    let user = db.find_user(&credentials.username).await?;
    verify_password(&credentials.password, &user.password_hash)?;
    update_last_login(&user.id).await?;
    Ok(user)
}

// auth/oauth.rs
async fn authenticate_oauth(token: OAuthToken) -> Result<User> {
    validate_token(&token)?;  // Different validation
    let user = db.find_user(&token.email).await?;  // Same DB call
    // Missing: verify_password (not needed for OAuth)
    update_last_login(&user.id).await?;  // Same update
    Ok(user)
}
```

**Refactoring**:
```rust
// auth/service.rs
pub struct AuthService {
    db: Database,
}

impl AuthService {
    async fn authenticate<S: AuthStrategy>(&self, strategy: S) -> Result<User> {
        let user_id = strategy.validate_and_get_user_id(&self.db).await?;
        let user = self.db.find_user_by_id(user_id).await?;
        self.update_last_login(user_id).await?;
        Ok(user)
    }
}

trait AuthStrategy {
    async fn validate_and_get_user_id(&self, db: &Database) -> Result<Uuid>;
}
```

**Pattern 2: Error Handling Boilerplate**
```rust
// Repeated in 15+ handlers
match service.create_episode(req).await {
    Ok(episode) => {
        info!("Episode created: {}", episode.id);
        Json(ApiResponse {
            success: true,
            data: Some(episode),
            error: None,
        })
    }
    Err(e) => {
        error!("Failed to create episode: {}", e);
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })
    }
}
```

**Refactoring**:
```rust
// Use Result type alias and automatic conversion
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;

impl<T: Serialize> IntoResponse for ApiResult<T> {
    fn into_response(self) -> Response {
        match self {
            Ok(data) => (StatusCode::OK, Json(ApiResponse::success(data))).into_response(),
            Err(e) => (e.status_code(), Json(ApiResponse::error(&e))).into_response(),
        }
    }
}

// Now handlers become:
async fn create_episode(req: Json<CreateRequest>) -> ApiResult<Episode> {
    let episode = service.create_episode(req.0).await?;
    info!("Episode created: {}", episode.id);
    Ok(Json(ApiResponse::success(episode)))
}
```

## 2. Architectural Structure

### Analysis Checklist

- [ ] **Component Identification**: List all major components/modules
- [ ] **Dependency Mapping**: Document dependencies between components
- [ ] **Layer Separation**: Verify proper layering (API → Business → Storage)
- [ ] **Integration Points**: Identify external system connections
- [ ] **Data Flow**: Document how data moves through the system
- [ ] **Control Flow**: Map request/response paths

### Architecture Documentation Template

```markdown
## System Components

### 1. [Component Name]
**Purpose**: [One-line description]
**Location**: [Path/directory]
**Dependencies**: [List of dependencies]
**Exposed APIs**: [Public interfaces]
**Key Responsibilities**:
- [Responsibility 1]
- [Responsibility 2]

### Component Interaction Diagram

[Visual representation of how components interact]

## Design Patterns

1. **[Pattern Name]**: [Where used and why]
```

### Common Architectural Issues

**Issue 1: Circular Dependencies**
```
Module A → Module B → Module C → Module A  ❌
```

**Solution**: Introduce abstraction layer
```
Module A → Interface I ← Module B
           ↑
           Module C
```

**Issue 2: God Module**
```rust
// memory-core/src/memory.rs (1200 LOC)
// Does everything: episodes, patterns, sync, retrieval, validation
```

**Solution**: Split by responsibility
```
memory-core/
  ├── episode/     (Episode management)
  ├── pattern/     (Pattern extraction)
  ├── sync/        (Storage sync)
  ├── retrieval/   (Context retrieval)
  └── validation/  (Input validation)
```

## 3. Code Organization & Modularity

### Module Cohesion Metrics

**High Cohesion** (Good):
- All functions in module work on related data
- Clear single responsibility
- Easy to name the module

**Low Cohesion** (Bad):
- Utility/helper module with unrelated functions
- Mixed concerns (DB + validation + formatting)
- Generic names like "common", "utils", "misc"

### Module Coupling Metrics

**Loose Coupling** (Good):
- Modules communicate through well-defined interfaces
- Changes in one module don't affect others
- Easy to test in isolation

**Tight Coupling** (Bad):
- Modules access each other's internals
- Changes cascade across modules
- Hard to test without full system

### File Size Guidelines

| File Type | Target LOC | Max LOC | Action if Exceeded |
|-----------|-----------|---------|-------------------|
| Module file | <200 | 500 | Split into submodules |
| Service/handler | <150 | 300 | Extract helpers |
| Test file | <300 | 600 | Split by test category |
| Configuration | <100 | 200 | Split by environment |

### Splitting Strategies

**Strategy 1: By Responsibility**
```
user_service.rs (800 LOC)
  ↓
user/
  ├── mod.rs        (100 LOC - public API)
  ├── queries.rs    (200 LOC - database)
  ├── validation.rs (250 LOC - input validation)
  └── permissions.rs (250 LOC - authorization)
```

**Strategy 2: By Feature**
```
api_handlers.rs (900 LOC)
  ↓
handlers/
  ├── mod.rs           (50 LOC - common)
  ├── episodes.rs      (200 LOC)
  ├── patterns.rs      (180 LOC)
  ├── retrieval.rs     (220 LOC)
  └── admin.rs         (250 LOC)
```

**Strategy 3: By Abstraction Level**
```
storage.rs (700 LOC)
  ↓
storage/
  ├── mod.rs           (100 LOC - traits)
  ├── turso.rs         (250 LOC - impl)
  ├── redb.rs          (200 LOC - impl)
  └── sync.rs          (150 LOC - sync logic)
```

## 4. Refactoring Opportunities

### Function Complexity Metrics

**Cyclomatic Complexity** (number of decision points):
- 1-4: Simple (good)
- 5-10: Moderate (acceptable)
- 11-20: Complex (should refactor)
- 21+: Very complex (must refactor)

### Complexity Detection

```bash
# Find long functions (Rust)
rg "^\s*pub\s+(async\s+)?fn\s+\w+" -A 100 | \
  awk '/^pub fn/,/^}$/' | \
  wc -l

# Find deep nesting
rg "^\s{16,}" | wc -l  # 16+ spaces = 4+ levels

# Find complex conditionals
rg "if.*&&.*\|\|" | wc -l
```

### Refactoring Patterns

**Pattern 1: Extract Method**
```rust
// Before: 80-line function
fn process_episode(episode: Episode) -> Result<ProcessedEpisode> {
    // 20 lines of validation
    // 30 lines of processing
    // 30 lines of storage
}

// After: Split into focused functions
fn process_episode(episode: Episode) -> Result<ProcessedEpisode> {
    validate_episode(&episode)?;
    let processed = transform_episode(episode)?;
    store_episode(&processed)?;
    Ok(processed)
}
```

**Pattern 2: Replace Conditional with Polymorphism**
```rust
// Before
fn extract_pattern(step: &ExecutionStep, pattern_type: &str) -> Option<Pattern> {
    match pattern_type {
        "tool_sequence" => extract_tool_sequence(step),
        "decision_point" => extract_decision_point(step),
        "error_recovery" => extract_error_recovery(step),
        _ => None,
    }
}

// After
trait PatternExtractor {
    fn extract(&self, step: &ExecutionStep) -> Option<Pattern>;
}

struct ToolSequenceExtractor;
impl PatternExtractor for ToolSequenceExtractor { /* ... */ }

// Use polymorphism instead of conditionals
fn extract_pattern(step: &ExecutionStep, extractor: &dyn PatternExtractor) -> Option<Pattern> {
    extractor.extract(step)
}
```

**Pattern 3: Introduce Parameter Object**
```rust
// Before: Too many parameters
fn create_episode(
    task_desc: String,
    task_type: String,
    context: HashMap<String, String>,
    tags: Vec<String>,
    language: Option<String>,
    domain: Option<String>,
    priority: u8,
) -> Result<Episode> { /* ... */ }

// After: Group related parameters
struct EpisodeRequest {
    task_description: String,
    task_type: String,
    context: TaskContext,
    metadata: EpisodeMetadata,
}

fn create_episode(request: EpisodeRequest) -> Result<Episode> { /* ... */ }
```

## 5. Technical Debt

### Debt Categories

#### Type 1: Code Debt
- TODOs and FIXMEs
- Commented-out code
- Dead code (unused functions)
- Temporary workarounds

#### Type 2: Test Debt
- Missing tests
- Skipped tests
- Flaky tests
- Low coverage areas

#### Type 3: Documentation Debt
- Missing API docs
- Outdated documentation
- No architecture decisions recorded
- Missing examples

#### Type 4: Dependency Debt
- Outdated dependencies
- Security vulnerabilities
- Deprecated APIs
- License incompatibilities

### Debt Quantification

**Effort Estimation**:
```markdown
| Debt Type | Instances | Effort/Instance | Total Effort |
|-----------|-----------|-----------------|--------------|
| TODO cleanup | 45 | 30 min | 22.5 hours |
| Missing tests | 23 | 2 hours | 46 hours |
| Outdated deps | 8 | 4 hours | 32 hours |
| **Total** | **76** | - | **100.5 hours (12.5 days)** |
```

**Interest Rate** (cost of not fixing):
- **High interest**: Security vulnerabilities (compounds daily)
- **Medium interest**: Missing tests (compounds with each change)
- **Low interest**: Documentation gaps (constant low cost)

### Technical Debt Detection

```bash
# Find TODOs and FIXMEs
rg "TODO|FIXME|HACK|XXX" --count-matches

# Find commented code (simple heuristic)
rg "^\s*//.*[{};]" | wc -l

# Find dead code (Rust)
cargo +nightly rustc -- -W dead_code 2>&1 | grep "warning:"

# Find outdated dependencies
cargo outdated

# Security vulnerabilities
cargo audit
```

## 6. Quality Metrics

### LOC (Lines of Code)

```bash
# Total LOC by language
tokei

# LOC by module
tokei --sort code

# Track LOC over time
echo "$(date),$(tokei --output json | jq '.Total.code')" >> loc_history.csv
```

### Cyclomatic Complexity

Target complexity by function type:
- **Simple functions**: 1-4 (data access, getters)
- **Business logic**: 5-10 (acceptable complexity)
- **Orchestration**: 11-15 (should be simplified)
- **Too complex**: 16+ (must refactor)

### Test Coverage

```bash
# Rust
cargo tarpaulin --out Html --output-dir coverage

# View summary
cargo tarpaulin | grep "% coverage"
```

**Coverage Targets**:
- **Core business logic**: 95%+
- **API handlers**: 90%+
- **Utilities**: 85%+
- **Overall**: 90%+

### Code Churn

Files with high churn (changed frequently) need:
- Better abstraction
- More tests
- Clearer responsibilities

```bash
# Find files with most commits (high churn)
git log --all --name-only --format="" | sort | uniq -c | sort -rn | head -20
```

## 7. Design Patterns & Idioms

### Common Patterns to Identify

**Creational Patterns**:
- Builder: Complex object construction
- Factory: Object creation abstraction
- Singleton: Global shared state (use sparingly)

**Structural Patterns**:
- Adapter: Interface compatibility
- Facade: Simplified interface
- Repository: Data access abstraction

**Behavioral Patterns**:
- Strategy: Interchangeable algorithms
- Observer: Event notification
- Chain of Responsibility: Request handling pipeline

### Anti-Pattern Detection

**Anti-Pattern 1: God Object**
```bash
# Find large structs with many methods
rg "^impl\s+\w+" -A 1000 | grep "fn " | wc -l
```

**Anti-Pattern 2: Spaghetti Code**
- Deep nesting (4+ levels)
- Long functions (50+ LOC)
- Complex conditionals

**Anti-Pattern 3: Premature Optimization**
- Complex caching without benchmarks
- Hand-rolled solutions for solved problems
- Sacrificing readability for minor gains

## 8. Cross-Cutting Concerns

### Error Handling Consistency

**Check**:
- [ ] Custom error type defined
- [ ] All errors implement `std::error::Error`
- [ ] Error messages include context
- [ ] No `.unwrap()` in production code
- [ ] No `.expect()` without clear justification

### Logging & Observability

**Check**:
- [ ] Consistent logging framework (tracing, log, slog)
- [ ] Structured logging with context
- [ ] Appropriate log levels used
- [ ] Sensitive data not logged
- [ ] Key operations instrumented

### Configuration Management

**Check**:
- [ ] Environment-specific configs
- [ ] No hardcoded values
- [ ] Secrets from environment/vault
- [ ] Validation on startup
- [ ] Defaults for optional configs

### Security Practices

**Check**:
- [ ] Input validation on all external data
- [ ] Parameterized SQL queries
- [ ] Authentication/authorization enforced
- [ ] Sensitive data encrypted
- [ ] Security headers in HTTP responses
- [ ] Rate limiting implemented
- [ ] CORS properly configured

### Performance Considerations

**Check**:
- [ ] Database queries optimized (indexes, N+1)
- [ ] Caching strategy defined
- [ ] Resource limits enforced
- [ ] Async/await used correctly
- [ ] No blocking in async context
- [ ] Benchmarks for critical paths

## Scoring System

Each dimension scored 0-10:

**10**: Exemplary - Best practices throughout
**8-9**: Good - Minor improvements possible
**6-7**: Acceptable - Some issues, worth addressing
**4-5**: Poor - Significant issues, needs attention
**0-3**: Critical - Major problems, immediate action required

**Overall Score**: Average of 8 dimensions

**Grade Assignment**:
- 90-100: A+ (Excellent)
- 80-89: A (Good)
- 70-79: B (Acceptable)
- 60-69: C (Needs Improvement)
- 50-59: D (Poor)
- 0-49: F (Critical Issues)
