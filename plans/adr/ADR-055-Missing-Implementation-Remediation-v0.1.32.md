# ADR-055: Missing Implementation Remediation Sprint v0.1.32

**Status**: Proposed
**Date**: 2026-05-21
**Deciders**: Maintainers
**Companion Plan**: [`plans/GOAP_MISSING_IMPLEMENTATION_2026-05-21.md`](../GOAP_MISSING_IMPLEMENTATION_2026-05-21.md)
**Supersedes**: Partial residuals from ADR-044, ADR-051, ADR-053

---

## Context

`v0.1.31` is released. A fresh `rg` audit of `memory-core`, `memory-mcp`, `memory-cli`,
`memory-storage-redb`, and `memory-storage-turso` finds the codebase is clean of
`TODO`/`FIXME`/`unimplemented!()`/`todo!()` markers but still contains a small set of
**advertised-but-unimplemented surfaces**. These are user-visible (CLI commands that
`anyhow::bail!` on `not yet implemented`, MCP tools that return stub values, embedding
providers that silently fall back) and erode trust in the API contract.

The previous gap analysis (`plans/STATUS/GAP_ANALYSIS_LATEST.md`, dated 2026-04-22)
predates these findings and only flags coverage, dead-code count, and one flaky test.
This ADR enumerates the **remaining functional gaps** and the decision for each:
implement, document, or remove.

---

## Inventory of Missing Implementations (2026-05-21 audit)

### P1 — User-facing functional gaps (CLI / MCP contract)

| # | Surface | Location | Current behavior | Decision |
|---|---------|----------|------------------|----------|
| G1 | `do-memory-cli relationship show <id>` | [`memory-cli/src/commands/relationships/core.rs#L285`](../../memory-cli/src/commands/relationships/core.rs) | `anyhow::bail!("Direct relationship lookup by ID is not yet implemented")` | **Implement** — add storage-layer `get_relationship_by_id` |
| G2 | `do-memory-cli relationship validate` (global) | [`memory-cli/src/commands/relationships/core.rs#L355`](../../memory-cli/src/commands/relationships/core.rs) | `bail!` when no `--episode` given | **Implement** — iterate all episodes + DFS, or **remove** flag |
| G3 | `do-memory-cli eval --custom-thresholds` | [`memory-cli/src/commands/eval.rs#L417`](../../memory-cli/src/commands/eval.rs) | Prints "not yet implemented in Phase 2" warning | **Implement** — wire overrides into `AdaptiveThresholds` |
| G4 | `Cohere` embedding provider | [`memory-mcp/.../configure.rs#L31`](../../memory-mcp/src/mcp/tools/embeddings/tool/execute/configure.rs) | Silent fallback to `Local` with warning | **Implement** via `cohere` feature, or **reject** the request explicitly |
| G5 | `Mistral` binary dequantization | [`memory-core/src/embeddings/mistral/client.rs#L161`](../../memory-core/src/embeddings/mistral/client.rs) | `anyhow::bail!("Binary dequantization not yet implemented")` | **Implement** per Mistral cookbook, or **remove** `binary` encoding from public config |
| G6 | `test_agentfs_connection` MCP tool | [`memory-mcp/.../external_signals/test_connection.rs`](../../memory-mcp/src/server/tools/external_signals/test_connection.rs) | Stub: always reports "SDK unavailable" | **Implement** real SDK probe (ADR-051 follow-through) or **gate** behind `agentfs` feature |

### P2 — Telemetry / observability accuracy gaps

| # | Surface | Location | Current behavior | Decision |
|---|---------|----------|------------------|----------|
| G7 | `pattern_match_score` time-series variable | [`time_series.rs#L55`](../../memory-mcp/src/mcp/tools/advanced_pattern_analysis/time_series.rs) | Hard-coded `0.8` | **Implement** real pattern-match scoring |
| G8 | `memory_usage_mb` time-series variable | [`time_series.rs#L59`](../../memory-mcp/src/mcp/tools/advanced_pattern_analysis/time_series.rs) | Hard-coded `50.0` | **Implement** via `sysinfo` or **remove** from variable enum |
| G9 | `episode_success_rate` health metric | [`monitoring/types.rs#L363`](../../memory-mcp/src/monitoring/types.rs) | Hard-coded `99.0` | **Compute** from real error counts |
| G10 | `uptime_seconds` (CLI health) | [`memory-cli/src/commands/health.rs#L307`](../../memory-cli/src/commands/health.rs) | Returns `std::process::id()` (!) | **Implement** via process start-time tracking |
| G11 | Turso cache `query_hits` / `query_misses` / `evictions` / `expirations` | [`memory-storage-turso/src/cache/wrapper.rs#L142`](../../memory-storage-turso/src/cache/wrapper.rs) | Hard-coded `0` | **Implement** atomic counters |

### P3 — Internal correctness / debt

| # | Surface | Location | Current behavior | Decision |
|---|---------|----------|------------------|----------|
| G12 | `CascadeRetriever::analyze_query` placeholder branch | [`retrieval/cascade/mod.rs#L446`](../../memory-core/src/retrieval/cascade/mod.rs) | Static analysis stub | **Implement** real heuristic or **remove** method |
| G13 | `generate_simple_embedding` placeholder helper | [`memory/retrieval/helpers.rs#L59`](../../memory-core/src/memory/retrieval/helpers.rs) | 10-dim hashed pseudo-embedding | **Remove** (callers should use real provider) or **document** as test-only |
| G14 | WG-149 `emit_event` not wired to lifecycle | [`memory/types.rs#L185`](../../memory-core/src/memory/types.rs), [`memory/core/struct_priv.rs#L90`](../../memory-core/src/memory/core/struct_priv.rs) | `#[allow(dead_code)]` on `MemoryEvent` emission helpers | **Wire** to `create_episode`/`complete_episode` |
| G15 | Stale "extraction is not implemented" test comment | [`extraction/tests.rs#L49`](../../memory-core/src/extraction/tests.rs) | Comment contradicts working `PatternExtractor::extract` | **Delete** misleading comment + tighten assertion |

---

## Decision

Adopt a **single remediation sprint v0.1.32** that lands G1–G15 as ≤15 atomic PRs grouped by
crate. Each gap is resolved either by **implementing** the advertised behavior or by
**removing/feature-gating** the surface so the public contract no longer lies. Stub
behavior is never preserved silently.

### Guiding rules
1. **No silent fallbacks**: if a provider/tool is requested and unavailable, return a
   typed error — do not substitute another provider.
2. **Feature-gate honest stubs**: any temporarily-stubbed integration (`agentfs`, `cohere`)
   must compile out unless its feature flag is set.
3. **Telemetry must be real or absent**: hard-coded `99.0` / `50.0` / `0.8` values are
   removed from the variable enum if not computed from real data.
4. **CLI commands either work or are hidden** (`#[command(hide = true)]`) — they do not
   `bail!` from happy paths.

---

## Consequences

### Positive
- Eliminates the gap between advertised CLI/MCP surface and actual behavior.
- Removes 5 hard-coded telemetry constants that distort dashboards.
- Wires WG-149 CloudEvents to real lifecycle (closes the `dead_code` annotation).
- Restores trust in the embedding provider matrix (no silent provider swaps).

### Negative
- Some surface area shrinks (e.g., `Cohere` may be removed instead of implemented).
- New `sysinfo` dependency (G8/G10) increases build time slightly.
- ~10–15 atomic PRs to land; sprint adds ~1 week of focused work.

### Neutral
- Coverage will likely tick up as real implementations replace stub branches.
- Some `#[allow(dead_code)]` annotations will drop (G14), reducing debt count further.

---

## Validation gates (per PR)

Every PR in this sprint MUST pass:

- [ ] `./scripts/code-quality.sh fmt`
- [ ] `./scripts/code-quality.sh clippy --workspace`
- [ ] `cargo nextest run --all`
- [ ] `cargo test --doc`
- [ ] `./scripts/quality-gates.sh` (≥90% coverage)
- [ ] `cargo doc --no-deps --document-private-items`
- [ ] `git status` clean

Sprint exit criteria: zero matches for `rg -i "not yet implemented|placeholder|stub\b"`
across `memory-*/src/` (excluding `*/tests*` and SQL `?`-placeholders).

---

## Cross-references

- [GOAP_MISSING_IMPLEMENTATION_2026-05-21.md](../GOAP_MISSING_IMPLEMENTATION_2026-05-21.md) — execution plan
- [STATUS/GAP_ANALYSIS_LATEST.md](../STATUS/GAP_ANALYSIS_LATEST.md) — preceding gap analysis
- ADR-044 — Original v0.1.20 feature roadmap
- ADR-051 — External Signal Provider (AgentFS) baseline
- ADR-053 — Comprehensive Analysis v0.1.29
- ADR-054 — CloudEvents EventEmitter (WG-149)
