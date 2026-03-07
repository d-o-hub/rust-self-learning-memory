# Agent Coding Guidelines

## Quick Reference
- **Build**: `./scripts/build-rust.sh dev|release|check|clean`
- **Quality**: `./scripts/code-quality.sh fmt|clippy|audit|check`
- **Tests**: `cargo nextest run --all` (doctests: `cargo test --doc`)
- **Quality Gates**: `./scripts/quality-gates.sh`
- **Docs Integrity**: `./scripts/check-docs-integrity.sh`
- **Release Ops**: `./scripts/release-manager.sh validate|prepare|publish|rollback|full`

## Project Overview
Memory management system with episodic memory, semantic embeddings, Turso/libSQL + redb cache, MCP server.

**Stack**: Rust/Tokio + Turso + redb + embeddings (OpenAI/Cohere/Ollama/local)

**Crates**: memory-core, memory-storage-turso, memory-storage-redb, memory-mcp, memory-cli, test-utils, benches, examples

## Repo Orientation

| Crate | Purpose | Entry Point |
|-------|---------|-------------|
| `memory-core` | Domain types, episode lifecycle, embeddings, patterns | `src/lib.rs` |
| `memory-storage-turso` | Turso/libSQL persistent storage | `src/lib.rs` |
| `memory-storage-redb` | redb local cache layer | `src/lib.rs` |
| `memory-mcp` | MCP server, tool registry, Wasmtime sandbox | `src/bin/server.rs` |
| `memory-cli` | CLI interface, config management | `src/main.rs` |
| `test-utils` | Shared test helpers and fixtures | `src/lib.rs` |
| `benches` | Criterion benchmarks | `src/lib.rs` |

**Version**: Always check `Cargo.toml` workspace version (never hardcode in docs).

## Skill + CLI Pattern (CRITICAL)

**ALWAYS use Skill + CLI first** for high-frequency operations:

| Operation | Skill | Script/CLI | Token Cost |
|-----------|-------|-------------|-------------|
| Build | `build-rust` | `./scripts/build-rust.sh` | Low |
| Format/Lint | `code-quality` | `./scripts/code-quality.sh` | Low |
| Quality Gates | `code-quality` | `./scripts/quality-gates.sh` | Medium |
| CI Issues | `github-workflows` | `gh workflow list` | Low |
| Tests | `test-runner` | `cargo nextest run --all` | Medium |
| Debug | `debug-troubleshoot` | `RUST_LOG=debug cargo nextest run` | Medium |

**Before using task tool:**
1. Is there a skill in `.agents/skills/`? → Use it
2. Is there a script in `scripts/`? → Use it
3. Is this high-frequency? → Use Skill + CLI
4. Is this complex multi-agent? → Use task tool

**Example:**
```bash
# Load skill then run
skill: build-rust, code-quality
./scripts/build-rust.sh check
./scripts/code-quality.sh fmt
```

## Change Workflow (Golden Path)
1. Identify owner crate + relevant module
2. Read existing code patterns before modifying
3. Add/update tests (unit first, integration if cross-crate)
4. `./scripts/code-quality.sh fmt` → fix formatting
5. `cargo clippy --all -- -D warnings` → fix warnings
6. `cargo nextest run -p <crate>` → targeted tests
7. `cargo nextest run --all` → full suite (doctests: `cargo test --doc`)
8. `./scripts/quality-gates.sh` → final validation
9. After pushing PR fixes, verify checks attach on head SHA (`gh pr view --json statusCheckRollup,mergeStateStatus`)

## CI Monitoring Learnings (2026-03)
- Treat empty required-check rollup as a blocker, not as implicit success
- Avoid plans-only follow-up commits before remediation CI checks are attached
- If rollup remains empty, investigate workflow trigger/path conditions and use GH CLI monitoring evidence in `plans/STATUS/VALIDATION_LATEST.md`

## Token Efficiency (2026-03)

**Tool Selection Priority (lowest token cost first):**
1. **Glob** - File discovery (cheapest, structured output)
2. **Grep** - Code search (cheap, file-by-file breakdown)
3. **Read** - File inspection (medium)
4. **Bash** - Shell commands (expensive - prefer scripts)

**Verified Patterns (tested):**
- Grep tool: 1 call → structured file-by-file breakdown
- Glob tool: 1 call → all matching files with paths
- Scripts: 1 call → multiple operations combined

**Before Each Bash Command:**
- Can Grep do this? → Use Grep tool
- Can Glob do this? → Use Glob tool
- Is there a script? → Use `./scripts/*.sh`
- Is there a skill? → Load skill first

**Target Ratios:**
- Read:Edit = 2:1 (understand before modifying)
- Grep:Bash = 1:2 (search before shell)
- Script:Raw = 3:1 (prefer scripts over raw commands)

**Anti-Patterns (waste tokens):**
- `grep -r pattern` in Bash → Use Grep tool
- `find . -name` in Bash → Use Glob tool
- `cat file` in Bash → Use Read tool
- Multiple cargo commands → Use `./scripts/quality-gates.sh`

## Required Checks Before Commit
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --tests -- -D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used` (CI parity)
- [ ] `cargo build --all`
- [ ] `cargo nextest run --all`
- [ ] `./scripts/quality-gates.sh`

## CI Parity (2026-03)

**CRITICAL**: Local checks must match CI exactly to prevent "works locally, fails in CI".

| Check | Local Command | CI Workflow |
|-------|---------------|-------------|
| Format | `cargo fmt --all -- --check` | `quick-check.yml` |
| Clippy (lib) | `cargo clippy --lib -- -D warnings ...` | `quick-check.yml` |
| Clippy (tests) | `cargo clippy --tests -- -D warnings ...` | `quick-check.yml` |

**Use the script for CI parity:**
```bash
./scripts/code-quality.sh clippy   # Runs --tests with CI flags
./scripts/code-quality.sh check     # Full CI parity check
```

**Pre-commit hook (optional):**
```bash
pip install pre-commit
pre-commit install  # Uses .pre-commit-config.yaml
```

## Pre-Flight Validation (2026-03)

**Before Modifying GitHub Actions:**
1. Check action versions: `gh api repos/<owner>/<action>/releases/latest --jq .tag_name`
2. Validate workflow syntax: `actionlint .github/workflows/*.yml` (if installed)
3. Use `yaml-validator` skill for structured validation

**GitHub Actions Job Dependency Patterns (CRITICAL):**
When a job has `needs: [upstream-job]` and the upstream job is conditionally skipped:
- **Problem**: Downstream jobs skip by default when dependency is skipped
- **Solution**: Use `always()` in the conditional to evaluate even when dependency was skipped

```yaml
# WRONG: Job skips when check-quick-check is skipped (push events)
needs: [check-quick-check]
if: ${{ github.event_name != 'pull_request' || needs.check-quick-check.result == 'success' }}

# CORRECT: Job runs on push events even when dependency is skipped
needs: [check-quick-check]
if: ${{ 
  always() &&
  github.event_name != 'pull_request' ||
  needs.check-quick-check.result == 'success'
}}
```

**Pattern Recognition:**
- If job A only runs on `pull_request` events → it's skipped on `push`
- If job B `needs: [A]` → B skips when A is skipped (default behavior)
- Use `always()` + explicit result checks to allow B to run when A is skipped

**Before Adding Dependencies:**
1. Check existing: `cargo tree -d | grep -i <module>`
2. Verify feature flags: `cargo build --all-features`
3. Run `./scripts/code-quality.sh check`

**Before Debugging Failures:**
1. Use `debug-troubleshoot` skill first
2. Check `agent_docs/code_conventions.md` for patterns
3. Run targeted tests: `cargo nextest run -p <crate>`

**Skill Loading for Common Tasks:**
| Task | Skills to Load |
|------|----------------|
| CI fixes | `ci-fix`, `github-workflows`, `yaml-validator` |
| Debugging | `debug-troubleshoot`, `test-fix` |
| Code search | `codebase-locator`, `codebase-analyzer` |
| Testing | `test-runner`, `rust-async-testing` |

## Code Conventions
- **Max 500 LOC per file** (source code)
- Zero warnings policy (clippy)
- Single responsibility per module

## Core Invariants (Never Break)
- **Async**: Tokio runtime everywhere. No blocking in async paths (use `spawn_blocking`)
- **Storage**: Parameterized SQL only. Short transactions. No locks across `.await`
- **Serialization**: Postcard required (not bincode). See `agent_docs/code_conventions.md`
- **Clippy**: Zero warnings enforced (`-D warnings`). Fix, don't suppress
- **Files**: ≤500 LOC per source file. Split into submodules when exceeded
- **Tests**: ≥90% coverage. `#[tokio::test]` for async. AAA pattern (Arrange-Act-Assert)

## Feature Flags
- `openai`: OpenAI embeddings
- `local-embeddings`: CPU embeddings (ort, tokenizers)
- `turso`: Turso cloud storage
- `redb`: Cache layer
- `embeddings-full`: All providers
- `full`: All features

## Security
- Use environment variables (never hardcode)
- Parameterized SQL queries
- Validate inputs at API boundaries
- Use postcard for serialization

## Environment Variables
| Variable | Purpose | Required |
|----------|---------|----------|
| `TURSO_DATABASE_URL` | Turso database URL | For Turso backend |
| `TURSO_AUTH_TOKEN` | Turso authentication | For Turso backend |
| `OPENAI_API_KEY` | OpenAI embeddings | For openai feature |
| `RUST_LOG` | Logging level (debug/info/warn) | No (default: info) |

See `.env.example` for full list. Never commit secrets.

## Commit Format
`[module] description` or `fix(module): description`

## Performance Targets
- Episode Creation: < 50ms
- Step Logging: < 20ms
- Episode Completion: < 500ms
- Memory Retrieval: < 100ms

## Disk Space Management
- **Dev profile**: `debug = "line-tables-only"`, deps `debug = false` (ADR-032)
- **Linker**: Use `mold` on Linux for faster links (ADR-032)
- **Cleanup**: `cargo clean` or `scripts/clean-artifacts.sh` periodically
- **Monitor**: `cargo tree -d | grep -cE "^[a-z]"` for duplicate dep count

## Testing Best Practices (2026)
- **Runner**: `cargo nextest run` everywhere (except doctests → `cargo test --doc`)
- **Profiles**: `.config/nextest.toml` with `default`, `ci`, `nightly` profiles
- **Mutation**: `cargo mutants` on memory-core (nightly CI)
- **Property**: `proptest` for invariant testing (serialization, state machines)
- **Snapshot**: `insta` for output regression (MCP responses, CLI output)
- See ADR-033 for full strategy

## Dependency Major Version Upgrades (2026-03)

**CRITICAL**: Always check docs.rs for breaking changes before upgrading major versions.

### redb 3.x Breaking Changes
- `begin_read()` moved to `ReadableDatabase` trait - must import it:
  ```rust
  use redb::ReadableDatabase;  // Required for begin_read()
  ```
- `begin_write()` remains on `Database` struct (no trait needed)

### rand 0.10 Breaking Changes
- `rand::thread_rng()` → `rand::rng()` (new function name)
- `Rng::gen()` → `RngExt::random()` (renamed method)
- `Rng::gen_range()` → `RngExt::random_range()` (renamed method)
- `Rng::gen_bool()` → `RngExt::random_bool()` (renamed method)
- Import `RngExt` trait for user-level RNG methods:
  ```rust
  use rand::RngExt;  // Required for random(), random_range(), random_bool()
  ```
- `SeedableRng` remains in `rand` (re-exported from `rand_core`)
- Keep `rand` and `rand_chacha` versions aligned (both 0.10)

### General Upgrade Process
1. Check docs.rs for the crate's changelog/breaking changes
2. Run `cargo build` to identify compilation errors
3. **Search entire codebase** for old API usage:
   ```bash
   # Example: Find all old rand API usage
   grep -r "\.gen_range\|\.r#gen\|thread_rng\|gen_bool" --include="*.rs"
   ```
4. Fix imports and API changes in ALL files (including benches, tests, examples)
5. Run `cargo clippy --all -- -D warnings`
6. Run `cargo nextest run --all`

## Release Workflow
- **Command**: `/release [patch|minor|major]` - Comprehensive release with gates
- **Ops Wrapper**: `./scripts/release-manager.sh` (dry-run by default; pass `--execute` to run)
- **Versioning**: `cargo release patch|minor|major` (ADR-034)
- **Semver**: `cargo semver-checks check-release` in CI (ADR-034)
- **Distribution**: cargo-dist for binaries + installers (ADR-034)
- **Commits**: Conventional format (`feat(module):`, `fix(module):`)
- **Quality Gates**: Run `./scripts/quality-gates.sh` before release
- **See**: `.opencode/command/release.md` for full workflow

## Cross-References
| Topic | Document |
|-------|----------|
| Build commands & features | `agent_docs/building_the_project.md` |
| Testing strategies | `agent_docs/running_tests.md` |
| Code style & patterns | `agent_docs/code_conventions.md` |
| System architecture | `agent_docs/service_architecture.md` |
| Database schema | `agent_docs/database_schema.md` |
| Communication patterns | `agent_docs/service_communication_patterns.md` |
| Active roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| Architecture decisions | `plans/adr/` |
| Disk space optimization | `plans/adr/ADR-032-Disk-Space-Optimization.md` |
| Modern testing strategy | `plans/adr/ADR-033-Modern-Testing-Strategy.md` |
| Release engineering | `plans/adr/ADR-034-Release-Engineering-Modernization.md` |
| Rust 2024 edition | `plans/adr/ADR-035-Rust-2024-Edition-Migration.md` |
| Dependency optimization | `plans/adr/ADR-036-Dependency-Deduplication.md` |
| Master execution plan | `plans/GOAP_DISK_TESTING_RELEASE_2026.md` |
