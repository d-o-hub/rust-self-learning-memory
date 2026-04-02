# GOAP State Snapshot

- **Last Updated**: 2026-04-02 (v0.1.28 sprint executing)
- **Version**: `0.1.26` (released 2026-04-01, published to crates.io)
- **Branch**: `fix/v0.1.28-security-codeql-plans`
- **Validation**: `plans/STATUS/VALIDATION_LATEST.md`
- **Gap Analysis**: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- **Primary ADRs**: ADR-048 (v0.1.24 stability), ADR-049 (v0.1.25 analysis)

---

## Current Focus: v0.1.28 Sprint (Executing)

| Task | WG | Status | Details |
|------|----|--------|---------|
| Merge PR #406 (ai-slop) | WG-091 | ⏳ Auto-merge armed (rebase) | All CI ✅, closes #401 |
| Fix CodeQL alert #60 | WG-093 | ✅ Complete | Removed cleartext IDs from dry-run logging |
| Resolve Dependabot alerts | WG-092 | ✅ Tracked | All 3 transitive, in audit.toml/deny.toml |
| DyMoE routing-drift protection | WG-089 | 🔵 Planned | Pattern affinity classifier + assignment guard |
| Dual reward scoring | WG-090 | 🔵 Planned | Stability + novelty signals |
| Close issue #401 | — | ⏳ Pending | Auto-closed when PR #406 merges |

## v0.1.27 Sprint (Complete ✅)

| Task | WG | Status |
|------|----|--------|
| Bayesian ranking | WG-073 | ✅ Wilson score from attribution data |
| Diversity retrieval | WG-077 | ✅ MMR reranking |
| Episode GC/TTL | WG-075 | ✅ Retention policy |
| MCP Server Card | WG-078 | ✅ `.well-known/mcp.json` |
| spawn_blocking audit | WG-079 | ✅ CPU-heavy async paths |
| GitHub Pages | WG-084 | ✅ mdBook + cargo doc |
| llms.txt | WG-085 | ✅ LLM context file |

## v0.1.26 Release (Complete ✅)

| Task | Status |
|------|--------|
| Crate renaming (`memory-*` → `do-memory-*`) | ✅ |
| Version bump `0.1.25` → `0.1.26` | ✅ |
| crates.io publish (all 4 crates) | ✅ |
| Binary names (`do-memory-mcp-server`, `do-memory-cli`) | ✅ |
| GitHub Release (tag v0.1.26, multi-platform) | ✅ |

---

## Key Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Workspace version | 0.1.26 | — |
| Total tests | ~2,849 | — |
| Ignored tests | 341 annotations / 82 files | ceiling ≤125 |
| `allow(dead_code)` (prod) | 37 | ≤40 |
| Clippy | Clean | 0 warnings |
| Doctests | 0 failures | 0 |
| Cargo audit | 3 unmaintained warnings | transitive |
| Dependabot alerts | 3 open | all transitive, tracked |
| CodeQL alerts | 1 open | fix in PR |

---

## Active Issues / Blockers

| Issue | Status | Notes |
|-------|--------|-------|
| CLI_TURSO_SEGFAULT | Known | libsql upstream memory corruption; 71 Turso tests `#[ignore]` (ADR-027) |
| Dependabot alerts (3) | Tracked | All transitive deps, documented in `audit.toml` / `deny.toml` |
| CodeQL alert #60 | Fix in PR | Cleartext session/episode IDs in dry-run logging; fix on branch |
| Issue #401 | Pending | Auto-closes when PR #406 merges |

---

## Self-Learnings (Consolidated)

Patterns extracted from v0.1.17–v0.1.27 sprint history (34 sessions, 234 msgs, 97 commits):

### API / Dependency Changes
- **redb 3.x**: `begin_read()` is on the `ReadableDatabase` trait — import accordingly
- **rand 0.10**: `thread_rng()` → `rand::rng()`, `gen()` → `random()`, `gen_range()` → `random_range()`

### Development Workflow
- **Turso local dev**: Use `turso dev` — no auth token needed
- **Doctest quality**: New features must have tested doctests before merge
- **File size invariant**: Templates extraction prevents LOC growth (≤500 LOC per file)
- **Integration tests**: Feature implementation without integration tests leaves gaps
- **Documentation sync**: ROADMAP / CURRENT must stay in sync with releases
- **Plan document drift**: Always verify metrics by running actual commands, not trusting stale docs

### CI / GitHub
- **PR supersession**: Track supersession chains (e.g., #388 → #389 → #391) to avoid referencing stale PRs
- **codecov/patch**: Not a required CI check — configure thresholds in `codecov.yml`, don't block merges
- **Jules PRs**: Reconcile external agent PRs before planning more work — they may implement "pending" items

### Quality Gates
- **Ignored test ceiling**: Monitor the 125 ceiling metric; actual count can creep close
- **Tool selection**: Target Bash:Grep ratio of 2:1 (historical 17:1 indicates over-reliance on shell)
- **Atomic commits**: One change per commit — 5 excessive_changes instances in early sprints
- **Wrong approach**: Read 3+ source files before implementing — 8 instances of proceeding without patterns

---

## Cross-References

| Document | Location |
|----------|----------|
| Archived Execution Plans | `plans/archive/2026-03-consolidation/` |
| Active Roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| Latest Validation | `plans/STATUS/VALIDATION_LATEST.md` |
| Latest Gap Analysis | `plans/STATUS/GAP_ANALYSIS_LATEST.md` |
| ADR Index | `plans/adr/` |
