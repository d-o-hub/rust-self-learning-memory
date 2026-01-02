# Quick Start Guide - v0.1.10 Planning

**Created**: 2025-12-30
**Target Version**: v0.1.10 (Embeddings Completion)
**Estimated Effort**: 12-17 hours
**Status**: Ready to Start

---

## Overview

This guide provides a quick reference for v0.1.10 development, which focuses on completing the remaining 15% of embeddings integration. The v0.1.9 release achieved 85% embeddings completion with excellent infrastructure, and v0.1.10 will bring it to 100% by integrating embeddings into CLI, MCP server, and hierarchical retrieval.

### Current State (v0.1.9 ✅ COMPLETE)

**Production Ready**: 100% quality gates passing
- ✅ **Multi-Provider Architecture**: 5 providers (OpenAI, Cohere, Ollama, Local, Custom)
- ✅ **Circuit Breaker**: Resilience pattern for provider failures
- ✅ **Storage Backends**: Turso and redb support for embeddings
- ✅ **Configuration System**: TOML-based provider configuration
- ✅ **Documentation**: 95% complete with examples

**Embeddings Completion Status**:
- Core Infrastructure: ✅ 100%
- CLI Integration: ⚠️ 60% (embeddings disabled by default)
- MCP Integration: ⚠️ 50% (no embedding-specific tools)
- E2E Testing: ⚠️ 70% (missing integration tests)
- **Overall**: ⚠️ 85%

---

## v0.1.10 Goals

### Primary Objective
Complete the remaining 15% of embeddings integration to reach 100% functionality.

### Success Criteria
- [ ] CLI Integration: 60% → 100%
- [ ] MCP Integration: 50% → 100%
- [ ] E2E Testing: 70% → 95%
- [ ] Overall Embeddings: 85% → 100%
- [ ] All new features tested and documented
- [ ] Zero regression in existing functionality

---

## Implementation Tasks

### Week 1: User-Facing Integration (7-9 hours)

#### Task 1: CLI Integration (3-4 hours)
**Priority**: P1 (High User Value)
**Status**: Ready to Start

**Deliverables**:
- [ ] Add `[embeddings]` section to TOML config files
- [ ] Implement `memory-cli embedding` subcommands
- [ ] Add CLI flags for embeddings configuration
- [ ] Update CLI documentation with examples

**Files to Create/Modify**:
```
memory-cli/src/commands/embedding.rs         # NEW - Embedding commands
memory-cli/src/commands/mod.rs               # Add embedding module
memory-cli/src/config/types.rs               # Add EmbeddingConfig
memory-cli/config/local-dev.toml             # Add [embeddings] section
memory-cli/config/cloud-production.toml      # Add [embeddings] section
memory-cli/CLI_USER_GUIDE.md                 # Document embedding commands
```

**Commands to Implement**:
1. `memory-cli embedding test` - Test provider connectivity
2. `memory-cli embedding config` - Show/edit configuration
3. `memory-cli embedding list-providers` - List available providers
4. `memory-cli embedding benchmark` - Benchmark provider performance

**CLI Flags**:
- `--enable-embeddings` - Enable embeddings for operations
- `--embedding-provider <provider>` - Specify provider (openai, local, etc.)
- `--embedding-model <model>` - Specify model name

**Example Usage**:
```bash
# Test OpenAI provider connectivity
memory-cli embedding test --provider openai

# Search episodes semantically
memory-cli episode list --semantic-search "authentication bugs" --enable-embeddings

# Benchmark local provider
memory-cli embedding benchmark --provider local
```

**Validation**:
- [ ] All 4 commands functional
- [ ] Semantic search working in episode list
- [ ] Configuration loads from TOML
- [ ] Documentation includes 3+ examples

#### Task 2: Hierarchical Retrieval Integration (2-3 hours)
**Priority**: P1 (High User Value)
**Status**: Ready to Start

**Current Issue**:
```rust
// memory-core/src/memory/retrieval.rs:369
query_embedding: None, // TODO: Add embedding support in future
```

**Deliverables**:
- [ ] Generate query embeddings in retrieval logic
- [ ] Update HierarchicalRetriever to use embeddings
- [ ] Add fallback for missing embeddings
- [ ] Add comprehensive tests

**Files to Modify**:
```
memory-core/src/memory/retrieval.rs              # Generate query embeddings
memory-core/src/spatiotemporal/retriever.rs      # Use query embeddings
memory-core/tests/semantic_retrieval_test.rs     # NEW - Tests
```

**Implementation Steps**:
1. Update `retrieve_relevant_episodes()` to generate query embeddings
2. Pass embeddings to `HierarchicalRetriever`
3. Update similarity scoring to include embedding similarity
4. Add graceful fallback when embeddings unavailable
5. Write tests for both paths (with/without embeddings)

**Validation**:
- [ ] Query embeddings generated when provider available
- [ ] Embedding similarity incorporated in scoring
- [ ] Fallback to keyword search works
- [ ] Tests pass with/without embeddings enabled

### Week 2: Advanced Integration (5-8 hours)

#### Task 3: MCP Server Integration (4-6 hours)
**Priority**: P1 (High User Value)
**Status**: Ready to Start

**Current State**: MCP server has 6 tools but no embedding-specific tools

**Deliverables**:
- [ ] Add `configure_embeddings` MCP tool
- [ ] Add `query_semantic_memory` MCP tool
- [ ] Add `test_embeddings` MCP tool
- [ ] Update server initialization
- [ ] Document new tools

**Files to Create/Modify**:
```
memory-mcp/src/mcp/tools/embeddings.rs          # NEW - Embedding tools
memory-mcp/src/mcp/tools/mod.rs                 # Add pub mod embeddings
memory-mcp/src/server.rs                        # Add tools to create_default_tools
memory-mcp/README.md                            # Document new tools
memory-mcp/tests/embeddings_integration.rs      # NEW - Tests
```

**Tool Specifications**:

1. **configure_embeddings**
   - **Purpose**: Configure embedding provider and model
   - **Inputs**: `provider` (string), `model` (string), `api_key_env` (optional)
   - **Outputs**: Configuration status, connectivity test result
   - **Example**: `{"provider": "openai", "model": "text-embedding-3-small"}`

2. **query_semantic_memory**
   - **Purpose**: Search episodes using semantic similarity
   - **Inputs**: `query` (string), `limit` (number), `filters` (optional)
   - **Outputs**: Ranked list of relevant episodes with similarity scores
   - **Example**: `{"query": "how to handle database errors", "limit": 5}`

3. **test_embeddings**
   - **Purpose**: Validate embedding provider connectivity and performance
   - **Inputs**: `provider` (string), `test_text` (optional)
   - **Outputs**: Connection status, latency, embedding dimensions
   - **Example**: `{"provider": "local"}`

**Validation**:
- [ ] All 3 tools registered in MCP server
- [ ] `configure_embeddings` updates server state
- [ ] `query_semantic_memory` returns relevant results
- [ ] `test_embeddings` validates connectivity
- [ ] Integration tests pass

#### Task 4: E2E Testing (3-4 hours)
**Priority**: P1 (Quality Assurance)
**Status**: Ready to Start

**Current Gap**: No comprehensive end-to-end tests across integration points

**Deliverables**:
- [ ] OpenAI provider E2E test
- [ ] Local provider E2E test
- [ ] CLI integration E2E test
- [ ] MCP integration E2E test

**Files to Create**:
```
memory-core/tests/openai_e2e_test.rs            # NEW - OpenAI E2E
memory-core/tests/local_e2e_test.rs             # NEW - Local E2E
memory-cli/tests/integration/embeddings.rs      # NEW - CLI E2E
memory-mcp/tests/embeddings_e2e_test.rs         # NEW - MCP E2E
```

**Test Scenarios**:

1. **OpenAI Provider E2E**:
   - Configure OpenAI provider
   - Create episode with auto-embedding
   - Query semantically
   - Verify similarity ranking

2. **Local Provider E2E**:
   - Configure local provider
   - Create multiple episodes
   - Query semantically
   - Verify results with CPU embeddings

3. **CLI Integration E2E**:
   - `embedding test` command
   - `episode list --semantic-search`
   - Configuration loading
   - Error handling

4. **MCP Integration E2E**:
   - `configure_embeddings` tool
   - `query_semantic_memory` tool
   - `test_embeddings` tool
   - Full workflow: configure → create → query

**Validation**:
- [ ] All E2E tests pass in CI
- [ ] Happy path covered for all providers
- [ ] Error scenarios handled gracefully
- [ ] Performance benchmarks established

---

## Known Limitations (v0.1.9)

These are documented limitations that users should be aware of:

### 1. Embeddings Disabled by Default
- **Location**: `memory-cli/src/config/storage.rs`
- **Impact**: Users must manually enable embeddings
- **Workaround**: Set `enable_embeddings: true` in config
- **Fix**: v0.1.10 will make this easier via CLI flags and commands

### 2. No Semantic Search in CLI
- **Impact**: CLI users cannot leverage semantic embeddings
- **Workaround**: Use direct API or wait for v0.1.10
- **Fix**: v0.1.10 adds `--semantic-search` flag

### 3. No MCP Embedding Tools
- **Impact**: MCP clients cannot configure or test embeddings
- **Workaround**: Configure via TOML files
- **Fix**: v0.1.10 adds 3 embedding-specific MCP tools

### 4. Limited E2E Testing
- **Impact**: Integration paths not fully validated
- **Risk**: Potential issues in production workflows
- **Fix**: v0.1.10 adds comprehensive E2E test suite

---

## Development Setup

### Prerequisites
- Rust 1.70+ (stable)
- libSQL/Turso database
- OpenAI API key (optional, for OpenAI provider testing)

### Environment Variables
```bash
# Required for local development
export DATABASE_URL="file:./data/memory.db"
export CACHE_PATH="./data/cache.redb"

# Optional: For OpenAI embeddings
export OPENAI_API_KEY="sk-..."

# Optional: For Cohere embeddings
export COHERE_API_KEY="..."
```

### Build & Test
```bash
# Build all packages
cargo build --all

# Run all tests
cargo test --all

# Run specific embedding tests
cargo test --package memory-core --test embedding_integration_test

# Run benchmarks
cargo bench --bench query_cache_benchmark
```

### Quick Test Workflow
```bash
# 1. Build
cargo build --release

# 2. Test CLI
./target/release/memory-cli episode create \
  --title "Test Episode" \
  --task-type "testing"

# 3. Test embeddings (after v0.1.10)
./target/release/memory-cli embedding test --provider local
./target/release/memory-cli episode list --semantic-search "test"
```

---

## Testing Strategy

### Unit Tests
- **Scope**: Individual functions and methods
- **Location**: `src/` files with `#[cfg(test)] mod tests`
- **Run**: `cargo test --package <package-name>`
- **Target**: >90% coverage per module

### Integration Tests
- **Scope**: Multi-module interactions
- **Location**: `tests/` directories
- **Run**: `cargo test --test <test-name>`
- **Target**: Cover all major workflows

### E2E Tests (New in v0.1.10)
- **Scope**: Full system workflows
- **Location**: Provider-specific test files
- **Run**: `cargo test --test *_e2e_test`
- **Target**: Cover all integration points

### Benchmarks
- **Scope**: Performance validation
- **Location**: `benches/` directory
- **Run**: `cargo bench`
- **Target**: No regressions, validate optimizations

---

## Release Checklist

Before releasing v0.1.10, ensure:

### Code Quality
- [ ] All tests passing (>99% pass rate)
- [ ] Zero clippy warnings
- [ ] Code formatted with rustfmt
- [ ] Test coverage >90%

### Documentation
- [ ] All new features documented
- [ ] CLI_USER_GUIDE.md updated
- [ ] README.md updated with new commands
- [ ] CHANGELOG.md updated

### Integration
- [ ] CLI commands functional
- [ ] MCP tools registered and tested
- [ ] Configuration loading works
- [ ] E2E tests pass

### Performance
- [ ] No regressions in benchmarks
- [ ] Embedding operations <500ms
- [ ] CLI startup <200ms

### Security
- [ ] No new vulnerabilities
- [ ] API keys properly secured
- [ ] Input validation in place

---

## Rollback Plan

If critical issues are found after release:

### Severity 1: Critical Bug (Immediate Rollback)
1. Revert to v0.1.9: `git checkout v0.1.9`
2. Publish hotfix release: v0.1.9.1
3. Document issue in GitHub
4. Fix in separate branch, test thoroughly

### Severity 2: Major Bug (Patch Release)
1. Create fix branch from v0.1.10
2. Apply minimal fix
3. Test extensively
4. Release as v0.1.10.1

### Severity 3: Minor Bug (Include in v0.1.11)
1. Document in GitHub Issues
2. Add to v0.1.11 roadmap
3. Fix in normal development cycle

---

## Timeline

### Week 1 (Days 1-5): User-Facing Integration
- **Days 1-2**: CLI Integration (Task 1)
- **Day 3**: Hierarchical Retrieval (Task 2)
- **Deliverable**: CLI semantic search functional

### Week 2 (Days 6-10): Advanced Integration
- **Days 6-7**: MCP Server Integration (Task 3)
- **Days 8-10**: E2E Testing (Task 4)
- **Deliverable**: Full embeddings integration, all tests passing

### Week 3 (Days 11-12): Polish & Release
- **Day 11**: Documentation, final testing
- **Day 12**: Release v0.1.10

**Total Effort**: 12-17 hours over 2-3 weeks

---

## Resources

### Documentation
- **Gap Analysis**: `plans/GAP_ANALYSIS_REPORT_2025-12-29.md`
- **Implementation Plan**: `plans/IMPLEMENTATION_PRIORITY_PLAN_2025-12-29.md`
- **Embeddings Roadmap**: `plans/EMBEDDINGS_COMPLETION_ROADMAP.md`
- **Project Status**: `plans/STATUS/PROJECT_STATUS_UNIFIED.md`

### Code References
- **Embedding Providers**: `memory-core/src/embeddings/`
- **CLI Commands**: `memory-cli/src/commands/`
- **MCP Tools**: `memory-mcp/src/mcp/tools/`
- **Configuration**: `memory-cli/src/config/`

### Examples
- **Multi-Provider Demo**: `memory-core/examples/multi_provider_embeddings.rs`
- **Semantic Search**: `memory-core/examples/semantic_embeddings_demo.rs`
- **E2E Demo**: `memory-core/examples/embeddings_end_to_end.rs`

---

## Support

### Getting Help
- **Documentation**: Check `README.md` and `CLI_USER_GUIDE.md`
- **Issues**: File GitHub issues for bugs
- **Discussions**: Use GitHub Discussions for questions

### Common Issues

**Issue**: Embeddings not working after upgrade
- **Solution**: Check configuration, ensure provider credentials are set

**Issue**: CLI commands not found
- **Solution**: Rebuild: `cargo build --release`

**Issue**: Tests failing
- **Solution**: Check environment variables, database setup

---

**Document Status**: Ready for v0.1.10 Development
**Next Action**: Begin Task 1 (CLI Integration)
**Estimated Completion**: 2-3 weeks from start

---

*This quick start guide provides all necessary information to begin v0.1.10 development. For detailed technical specifications, refer to the implementation plan and gap analysis documents.*
