# ADR-039: Plans Directory Consolidation and Codebase Gap Analysis

- **Status**: Accepted
- **Date**: 2026-03-11
- **Deciders**: Project maintainers
- **Related**: ADR-028 (Feature Enhancement Roadmap), ADR-033 (Modern Testing Strategy), ADR-037 (Selective Workflow Automation Adoption)

## Context

The `plans/` directory had accumulated 95 active (non-archived, non-ADR) markdown files across 9 subdirectories. Many described completed work, superseded plans, or duplicated status information across multiple documents. Key problems:

1. **Version drift**: `README.md` referenced v0.1.14, `INDEX.md` referenced v0.1.16, while actual version was v0.1.17.
2. **Duplicate status tracking**: Four overlapping status documents (`PROJECT_STATUS_UNIFIED.md`, `IMPLEMENTATION_STATUS.md`, `GOAP_STATE.md`, `VALIDATION_LATEST.md`) with conflicting timestamps.
3. **Completed work in active tree**: Episode relationships, tagging, skills consolidation, configuration optimization — all completed features with 30+ files still in active `plans/`.
4. **Stale GOAP snapshots**: Five dated GOAP analysis files plus five GOAP agent meta-process documents occupying active space.
5. **Undiscovered gaps**: Roadmap claimed items as "not started" that were actually fully implemented but not wired, while other truly missing features went untracked.

## Decision

### 1. Consolidate to 13 active files

Archive 82 files into `plans/archive/2026-03-consolidation/` organized by category:

| Category | Files Archived | Destination |
|----------|---------------|-------------|
| Superseded GOAP analyses | 10 | `superseded-goap/` |
| Completed features (episodes, relationships, tagging) | 7 | `completed-features/` |
| Completed initiatives (skills consolidation, config, research, CI fixes) | 10 + 10 config + 24 research | `completed-initiatives/` |
| Historical status reports | 9 | `historical-status/` |
| Operations reference (circuit breaker, production guides) | 4 | `operations-reference/` |
| Validation & benchmark evidence | 6 | `validation-benchmarks/` |

### 2. Establish canonical documents

| Document | Purpose | Rule |
|----------|---------|------|
| `README.md` | Navigation index | Single entry point, no status tracking |
| `STATUS/CURRENT.md` | Live project status | One source of truth for version, metrics, gaps |
| `ROADMAPS/ROADMAP_ACTIVE.md` | Forward-looking roadmap | No historical content; only next sprint + backlog |
| `GOALS.md` + `ACTIONS.md` + `GOAP_STATE.md` | GOAP workflow state | Kept from ADR-037 |
| `STATUS/VALIDATION_LATEST.md` | Automated validation | Updated by analysis runs |
| `STATUS/CODEBASE_ANALYSIS_LATEST.md` | Latest GOAP analysis | Renamed from dated file; only one active analysis |

### 3. Conduct code-verified gap analysis

Verified all roadmap claims against actual code to categorize gaps accurately:

#### Built but Not Wired (integration work, not greenfield)

| Subsystem | Location | Gap |
|-----------|----------|-----|
| Adaptive TTL | `do-memory-storage-turso/src/cache/adaptive_ttl.rs` (435 LOC) | Not integrated into `TursoStorage` query paths |
| Transport Compression | `do-memory-storage-turso/src/transport/wrapper.rs` | `CompressedTransport` only used in tests; `TursoConfig.enable_transport_compression` flag unused in constructors |
| Adaptive Cache | `do-memory-storage-redb/src/cache/adaptive/` (327 LOC) | `RedbStorage` creates `LRUCache`; AdaptiveCache has different interface (stores values vs metadata) |

#### Not Built (Updated 2026-03-13)

| Feature | Evidence | Status |
|---------|----------|--------|
| CLI `--domain`/`--type` episode filters | Ignored e2e test at `cli_workflows.rs:678` | Not built |
| ~~CLI pattern discovery commands~~ | ~~Ignored e2e test at `cli_workflows.rs:554`~~ | ✅ **IMPLEMENTED** - See `commands/pattern/` |
| ~~MCP Completion protocol~~ | ~~TODO at `types.rs:81`, returns empty placeholders~~ | ✅ **IMPLEMENTED** - See `mcp/completion.rs` (203 LOC) |
| ~~MCP Elicitation protocol~~ | ~~TODO at `types.rs:138`, stub only~~ | ✅ **IMPLEMENTED** - See `mcp/elicitation.rs` (250 LOC) |
| ~~MCP Rate Limiting~~ | ~~TODO at `types.rs:332~, `#[allow(dead_code)]` | ✅ **IMPLEMENTED** - See `server/mod.rs:83` with RateLimiter |
| ~~MCP Embedding Config~~ | Was not tracked | ✅ **IMPLEMENTED** - See `jsonrpc.rs:28-128` |
| ~~MCP Tasks protocol~~ | Was not tracked | ✅ **IMPLEMENTED** - See `mcp/tasks.rs` (350 LOC) |
| Changelog automation (git-cliff) | ADR-034 Phase 4, not started | Not built |

#### Disabled

| Feature | Issue |
|---------|-------|
| `execute_agent_code` WASM tool | Registered conditionally, always returns error ("WASM sandbox compilation issues") |

### 4. Adopt documentation hygiene rules

- **Only active/forward-looking docs stay unarchived**
- **One canonical file per topic** — no parallel status documents
- **Dated files are snapshots** — archive immediately after newer version exists
- **Completed feature plans archive on release**

## Consequences

### Positive

- Plans directory reduced from 95 → 13 active files (86% reduction)
- Single source of truth for status (`STATUS/CURRENT.md`), roadmap, and navigation
- Gap analysis reveals 3 subsystems that are built but unwired — integration work, not new development
- 6 genuinely missing features now tracked with code evidence
- Version drift eliminated

### Negative

- Archived docs may have broken internal cross-references (accepted; not worth fixing in archived files)
- Moving files changes git blame history for those paths
- Operations reference docs (circuit breaker, production guides) may need to resurface in `docs/` if users need them

### Neutral

- ADR directory untouched (17 ADRs remain as immutable reference)
- `archive/` directory grows but is explicitly excluded from active navigation

## Compliance

- No code changes required
- No CI/CD impact
- `./scripts/check-docs-integrity.sh` will report fewer broken links in active docs (archived files excluded from active count)
