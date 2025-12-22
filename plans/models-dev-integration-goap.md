# GOAP Plan: Models.dev API Integration

**Status**: Phase 1 - Research
**Created**: December 22, 2025
**Target Completion**: January 31, 2026
**Priority**: Medium-High

## Goal State

Successfully integrate Models.dev API into the memory management system to:
- Track AI model pricing data for cost monitoring
- Support informed model selection in episodic memory workflows
- Enable semantic search over available model capabilities
- Provide CLI visibility into current model ecosystem state

## Current State

- Memory system exists with episodic memory, semantic embeddings, and multiple backends
- No external model registry integration
- No pricing or cost monitoring capabilities
- CLI supports memory operations but not model visibility

## Phase 1: Research & Design (Weeks 1-2)

### Actions

**R1.1: API Structure Analysis**
- **What**: Fetch and analyze https://models.dev/api.json specification
- **How**: Document endpoint structure, data schemas, authentication method
- **Owner**: Research role
- **Success Criteria**: 
  - Complete endpoint inventory with HTTP methods
  - Full data schema for model objects (fields, types, constraints)
  - Authentication requirements documented
  - Rate limiting and pagination patterns identified

**R1.2: Integration Pattern Evaluation**
- **What**: Assess Models.dev API compatibility with Rust/Tokio architecture
- **How**: Review connection pooling, async patterns, error handling best practices
- **Dependencies**: Complete R1.1
- **Success Criteria**:
  - Documented async patterns for Models.dev requests
  - Error handling strategy (transient vs. permanent failures)
  - Connection pool sizing recommendations

**R1.3: Caching Strategy Design**
- **What**: Plan redb cache integration for model data
- **How**: Define TTL policies, invalidation patterns, fallback mechanisms
- **Dependencies**: Complete R1.2
- **Success Criteria**:
  - Cache key structure documented
  - TTL recommendations by data type (models: 24h, pricing: 6h, capabilities: 7d)
  - Invalidation triggers defined

**R1.4: Cost Monitoring Framework**
- **What**: Design how pricing data integrates with memory operations
- **How**: Map model costs to episodic memory usage patterns
- **Dependencies**: Complete R1.1
- **Success Criteria**:
  - Cost calculation formulas documented
  - Metrics for tracking cost impact identified
  - Integration points with existing memory operations defined

### Prerequisites
- Access to Models.dev API endpoints
- curl/reqwest capability for API exploration
- Read access to existing memory-core crates

### Deliverables
- `MODELS_DEV_API_SPEC.md` - Complete API reference
- `INTEGRATION_ARCHITECTURE.md` - Design patterns and approach
- `CACHING_STRATEGY.md` - Redb integration plan
- `COST_MONITORING_DESIGN.md` - Pricing data usage

---

## Phase 2: Implementation (Weeks 3-5)

### Actions

**I2.1: Create memory-models-dev Crate**
- **What**: New crate for Models.dev client and types
- **How**: Structure as `memory-models-dev/src/{client.rs, models.rs, errors.rs, cache.rs}`
- **Prerequisites**: Completion of Phase 1, R1.1-R1.4
- **Success Criteria**:
  - Compiles without warnings
  - Unit tests achieve >85% coverage
  - Async client supports concurrent requests
  - Error types implement proper Display/Debug

**I2.2: Implement API Client**
- **What**: Async HTTP client with connection pooling
- **How**: Use reqwest with custom middleware for retry logic, timeouts
- **Dependencies**: I2.1 complete
- **Success Criteria**:
  - Handles 5xx errors with exponential backoff
  - Timeout after 30s (configurable)
  - Concurrent request limit enforced
  - Metrics for request latency/success rate

**I2.3: Implement Caching Layer**
- **What**: Redb-backed cache for API responses
- **How**: Create cache wrapper around client with TTL enforcement
- **Dependencies**: I2.2 complete, caching strategy from Phase 1
- **Success Criteria**:
  - Cache hit/miss metrics tracked
  - TTL respects Strategy document
  - Graceful degradation if cache unavailable
  - Memory usage <500MB for full model database

**I2.4: Create Configuration Options**
- **What**: Config schema for Models.dev integration
- **How**: Add to unified-config.toml with serde validation
- **Dependencies**: I2.1 complete
- **Success Criteria**:
  - Supports enable/disable toggle
  - Configurable cache TTLs
  - API endpoint URL override capability
  - Rate limit configuration

### Prerequisites
- Familiarity with reqwest and tokio patterns from existing codebase
- Access to memory-core, memory-storage-redb examples
- Models.dev API credentials (if required)

### Deliverables
- `memory-models-dev/Cargo.toml` - New crate manifest
- `src/client.rs` - HTTP client implementation
- `src/models.rs` - Type definitions (serde-derived)
- `src/cache.rs` - Redb caching wrapper
- Test coverage >85%

---

## Phase 3: Integration (Weeks 6-7)

### Actions

**IN3.1: Integrate with Memory Core**
- **What**: Connect model pricing to memory operation costs
- **How**: Add cost_estimate field to episodic memory records
- **Prerequisites**: Phase 2 complete, I2.1-I2.4
- **Success Criteria**:
  - MemoryRecord includes optional model_cost field
  - Cost calculated for each operation
  - Aggregation queries support cost filtering
  - No breaking changes to existing API

**IN3.2: Extend MCP Server**
- **What**: Add Models.dev tools to memory-mcp
- **How**: Create MCP tools: list_models, get_model_pricing, check_model_availability
- **Dependencies**: Phase 3 in progress, MCP server understanding
- **Success Criteria**:
  - Tools conform to MCP specification
  - Handle rate limits gracefully
  - Proper error propagation
  - JSON schema validation

**IN3.3: CLI Enhancement**
- **What**: Add models subcommand to memory-cli
- **How**: Commands: `memory models list`, `memory models pricing`, `memory models sync`
- **Dependencies**: IN3.2 complete
- **Success Criteria**:
  - Formatted table output for model listings
  - Pricing displayed with cost analysis
  - Sync command updates local cache
  - Help text comprehensive

**IN3.4: Update Database Schema**
- **What**: Add tables for cached model metadata
- **How**: Create models table in Turso, sync routine
- **Dependencies**: IN3.1 complete
- **Success Criteria**:
  - Schema supports model name, pricing, capabilities
  - Indexes on frequently queried fields
  - Migration script for existing databases
  - Triggers for auto-invalidation if needed

### Prerequisites
- Completion of Phase 2 implementation
- Understanding of memory-mcp tool specification
- CLI command patterns from existing code

### Deliverables
- Modified `memory-core/src/memory.rs` - MemoryRecord cost fields
- Modified `memory-mcp/src/tools.rs` - New MCP tools
- Modified `memory-cli/src/commands/` - New models subcommand
- Database migration script
- Integration tests (>80% coverage)

---

## Phase 4: Testing & Validation (Weeks 8)

### Actions

**T4.1: Unit Test Coverage**
- **What**: Achieve >90% code coverage across memory-models-dev
- **How**: Mock API responses, test cache behavior, error scenarios
- **Success Criteria**:
  - All error paths tested
  - Cache TTL enforcement verified
  - Concurrent request handling validated

**T4.2: Integration Tests**
- **What**: End-to-end scenarios with memory operations
- **How**: Spin up test database, populate with models, run cost calculations
- **Dependencies**: T4.1 complete
- **Success Criteria**:
  - Cost calculations accurate within 1%
  - Cache invalidation works correctly
  - API failures don't crash memory operations
  - >80% coverage of integration paths

**T4.3: Performance Benchmarks**
- **What**: Validate caching strategy effectiveness
- **How**: Benchmark 1000 model queries with/without cache
- **Dependencies**: T4.1, T4.2 complete
- **Success Criteria**:
  - Cache improves latency by >90%
  - Memory overhead <100MB
  - API client maintains <100ms p99 latency
  - Throughput >1000 req/sec

**T4.4: Documentation & Examples**
- **What**: User guide for Models.dev integration
- **How**: Create examples in memory-cli, document cost monitoring setup
- **Success Criteria**:
  - README with quickstart examples
  - Cost monitoring dashboard walkthrough
  - Troubleshooting guide for API failures
  - Example configurations

### Prerequisites
- Phase 3 complete
- Benchmark infrastructure from existing benches/
- Test data generators ready

### Deliverables
- Integration test suite with >80% coverage
- Benchmark results in benches/models_dev.rs
- User documentation in memory-cli/MODELS_DEV_GUIDE.md
- Performance validation report

---

## Success Metrics

- **Functional**: All 4 phases complete, integration tests passing
- **Performance**: <100ms API latency (p99), >90% cache hit rate
- **Code Quality**: >85% test coverage, cargo clippy passes
- **Documentation**: All APIs documented with examples
- **Stability**: <0.1% error rate in production usage

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Models.dev API rate limits | Degraded functionality | Implement exponential backoff, cache aggressively |
| Breaking API changes | Integration failure | Version pinning, deprecation monitoring, vendor communication |
| Network latency | Slow memory operations | Aggressive caching, async operations, circuit breaker pattern |
| Schema incompatibility | Type mismatches | Generate types from schema, validation tests |

## Decision Points

1. **Week 2 End**: Approve caching strategy before implementation
2. **Week 5 End**: Review integration approach before MCP additions
3. **Week 7 End**: Performance benchmarks meet requirements before release

## Dependencies

- Completion of each phase before next begins
- No blocking dependencies on other roadmap items
- Internal dependencies: memory-core, memory-mcp, memory-cli

## Rollback Plan

If Models.dev integration becomes problematic:
1. Disable via configuration toggle (zero code changes)
2. Remove memory-models-dev from dependencies
3. Revert MemoryRecord schema changes (add deprecation period)
4. CLI commands remain but return "feature unavailable"