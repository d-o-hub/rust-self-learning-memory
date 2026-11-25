# ADR-085: Comprehensive Analysis — v0.1.51 Post-Release

**Status**: Proposed
**Date**: 2026-11-25
**Deciders**: Agent codebase analysis (workspace audit on `main` @ `eb076391`; no change to
product code is authorized by this ADR — it records verified facts and proposes the next wave)
**Supersedes**: N/A (continues the analysis lineage of ADR-043 / ADR-049 / ADR-052 / ADR-053 / ADR-055)
**Related**: ADR-039 (plans consolidation), ADR-072 (governance), ADR-077 (runtime embedding
provider activation), ADR-084 (OTel), `plans/ROADMAPS/ROADMAP_ACTIVE.md` (R-F backlog),
`plans/STATUS/spikes/R-F-product-backlog-2026-11-15.json` (gate discipline)

---

## Context

The tracker is empty (0 open issues, 0 open pull requests — live `gh` queries, 2026-11-25),
`v0.1.50` is shipped and dogfooded, the workspace sits at the post-release `0.1.51` bump,
and the last product wave (#1283 LearningOrchestrator, #1284 OTel, #1285 discovery example,
#1289 episode summaries + retention cleanup) is merged. This ADR records a fresh,
code-verified analysis of improvement, missing-implementation, and feature candidates so the
next wave starts from measured facts instead of stale prose.

### Verified state (2026-11-25, `main` @ `eb076391`)

| Fact | Verified value | Method |
|------|----------------|--------|
| Workspace version / latest tag | `0.1.51` / `v0.1.50` (`0fe1b89d`) | `Cargo.toml`, roadmap |
| Open issues / PRs | 0 / 0 | `gh` live query |
| `do-harness verify --set feedback` | 5/5 PASS (`fmt`, `loc`, `check`, `commit-ci-tokens`, `agents-md-ids`) | run 2026-11-25 |
| `TODO`/`FIXME`/`todo!()` in prod src | exactly 1 (`memory-cli/src/commands/episode/core/checkpoint.rs:62`) | `grep -rn` across 9 crates |
| `#[ignore]` inventory | 159 entries (ceiling 200) | `plans/ignored_tests_inventory.json` |
| `#[allow(dead_code)]` in prod src | 14 files (`#-per-file`, excl. test files) | `grep -rln` |
| ADR-055's four placeholder gaps | all closed and re-verified in code | spot checks (`monitoring/types.rs`, `cache/wrapper.rs`, `retrieval/cascade/heuristics.rs`, `retrieval/helpers.rs`) |
| Largest source files >500 LOC | test modules only (max 1710: `retrieval/cascade/tests.rs`); prod LOC rule green per `loc` sensor | `wc -l`, sensor |
| MCP protocol versions negotiated | `2025-11-25`, `2024-11-05` | `memory-mcp/src/protocol/types.rs:9` |

### Finding class 1 — documented-but-missing embedding providers (highest-value gap)

Four independent sources describe embeddings support that the code does not provide:

| Source | Claim | Reality |
|--------|-------|---------|
| `AGENTS.md:5` | "embeddings (OpenAI/Cohere/Ollama/local)" | `EmbeddingProvider` enum = `Local`, `OpenAI`, `Mistral`, `AzureOpenAI`, `Custom` (`memory-core/src/embeddings/config/provider_enum.rs:7`) — no Cohere, no Ollama, and Mistral unmentioned |
| `agent_docs/service_architecture.md:48` | "Multi-provider support (OpenAI, Cohere, Ollama, local)" | same enum; Cohere/Ollama adapters do not exist |
| `agent_docs/database_schema.md:294-295` | lists `embed-english-v3.0` (Cohere), `nomic-embed-text-v1` (Ollama) | no runtime adapter exists for either |
| `memory-mcp/.../tool/definitions.rs:108` | tool schema advertises `provider` enum `["openai","local","mistral","azure","cohere"]` | `configure.rs:203` rejects `cohere` ("not implemented"); `memory-cli/.../embedding/provider.rs:75` rejects Azure ("URL query formatting limitations") |

Related dead code: `memory-core/src/constants.rs:136-142` defines `COHERE_ENABLED` gated on a
`cohere-embeddings` feature that **does not exist** in `memory-core/Cargo.toml`, and
`#[allow(unexpected_cfgs)]` suppresses exactly the diagnostic that would reveal it.
`docs/API_REFERENCE.md:200-202` is the only source that tells the truth (selectable:
`local`, `openai`, `mistral`; `azure`/`custom`/`cohere` rejected before any credential work).

This is one gap with two admissible resolutions (fix the prose, or implement the providers); the
ADR does not presume which. Both must respect ADR-077 (activation contract, no silent
substitution, fail-closed credentials).

### Finding class 2 — tracker/docs truth drift

| Item | Drift | Evidence |
|------|-------|----------|
| `docs/TECH_DEBT.md` baseline | "Last Updated 2026-07-20"; `DOC-02` (vision title "v0.1.9+") is already fixed, and `TEST-01` references pre-#1289 numbers | `ROADMAP_V030_VISION.md` now reads "(0.2 → 1.0)" |
| R-F5 backlog row | conflates retention (shipped in #1289: `EpisodeRetentionPolicy`, `cleanup_episodes`, `count_cleanup_candidates` on both backends) with versioning (not implemented) | `CHANGELOG.md` `[Unreleased]` `Added`; `R-F-product-backlog-2026-11-15.json` metric `r_f5_episode_versioning_implemented=false` |
| `AzureOpenAI` enum variant | present in the provider enum but its transport is rejected as unsupported | `memory-cli/src/commands/embedding/provider.rs:75,212` |

### Finding class 3 — hygiene and performance candidates

| ID | Candidate | Location / evidence | Note |
|----|-----------|---------------------|------|
| HY-01 | **Withdrawn 2026-11-25** — `bin/act` was never tracked | `git ls-files bin/` is empty; no `bin/act` object exists in `git rev-list --all --objects`; the file is local-only, ignored by `/bin/` (`.gitignore:124`) | The original "tracked" reading misattributed `git check-ignore`'s pathname output to `git ls-files` (LESSON-088); fetching act stays documented in `docs/YAML_VALIDATION.md` |
| HY-02 | Dead `COHERE_ENABLED` constant behind a non-existent feature, with the discovering lint suppressed | `memory-core/src/constants.rs:136-142` | delete or wire to a real feature |
| HY-03 | Only in-tree TODO: checkpoint listing loads the full episode | `memory-cli/src/commands/episode/core/checkpoint.rs:62` | dedicated storage query (perf); pre-req for reusing #1289 capability pattern |
| HY-04 | Prepared-statement cache default hard-coded at 100 | `memory-storage-turso/src/prepared/cache.rs:456` (TECH_DEBT `STOR-02`) | make capacity configurable |
| HY-05 | `benches` crate is 177 LOC while feature comments state measured-sounding claims (compression "40% reduction", keepalive "45ms→5ms") | `benches/src/`; `memory-storage-turso/Cargo.toml` feature docs | pin the claims with Criterion benches or reword them as targets |
| HY-06 | monitoring storage wrapper reserved-but-unused (TECH_DEBT `ARCH-02`) plus 14 `#[allow(dead_code)]` prod files | `memory-core/src/monitoring/storage/mod.rs:52` | decide use-or-delete on the rebaseline wave |
| HY-07 | MCP `tasks`/`elicitation` capabilities explicitly not implemented | `memory-mcp/src/protocol/mod.rs:173,177` | spec-conformance surface; see Decision Phase 2 |

---

## Decision

### Phase 0 — Tracker truth reconciliation (P0, one docs PR, low risk)

1. Resolve the embedding-provider narrative on all four dishonest sources
   (`AGENTS.md`, `service_architecture.md`, `database_schema.md`, MCP tool schema):
   either align prose/schema to the implemented selectable set
   (`local`, `openai`, `mistral`) and stop advertising `azure`/`cohere` in the tool
   schema enum, or adopt the provider gap as a product decision (Phase 2 F-1).
2. Rebaseline `docs/TECH_DEBT.md` to the 2026-11-25 audit: close `DOC-02`, renumber the
   ignore-ratchet facts (159/200), and re-verify `ARCH-02`, `STOR-01`, `STOR-02` rows against
   current code — same pattern as the ADR-028/042/043 rebaseline and ADR-060 retirement.
3. Split the R-F5 row in `ROADMAP_ACTIVE.md` and the backlog spike register: retention is
   shipped (#1289), versioning remains `NO-GO` (schema migration + `StorageBackend`
   extension + MCP/CLI surface evidence unchanged).

### Phase 1 — Small code hygiene wins (P1, atomic, parallel-safe, test-with-change)

- HY-02: remove the dead constant or wire the feature (`constants.rs`).
- HY-03: dedicated checkpoint query (also removes the last TODO in prod src).
- HY-04: configurable statement-cache capacity with default 100.
- HY-01: **withdrawn 2026-11-25** — `bin/act` was never tracked (local-only, ignored by
  `/bin/`); no untracking or fetch script is needed. Fetching act remains documented in
  `docs/YAML_VALIDATION.md`.
- HY-05: add the missing benches first (measurement before claims), then reword the
  feature comments to cite the benchmark numbers.

Each item ships as its own conventional commit with tests where behavior changes.

### Phase 2 — New-feature candidates (P2, gated by the standing spike discipline)

Every promotion needs a measured spike (`commands`, thresholds, owner, ADR, release slot)
exactly like `R-F2-otel-2026-11-17.json`; a GO artifact is approval to implement, not
evidence that code exists.

| ID | Candidate | Why now | Gate evidence to produce |
|----|-----------|---------|--------------------------|
| F-1 | Cohere + Ollama embedding adapters (ADR-077 pattern) | four docs claims already promise them; Ollama complements the `local-embeddings` ort path | spike: endpoint/token contract, dimension-stability plan, cost of dimension mismatch (reindex) |
| F-2 | Azure OpenAI transport | enum variant already exists; failure is isolated to URL query formatting | spike: request-mapping fix + replay tests of the rejection cases |
| F-3 | Batch MCP tools (`MCP-02`) | deferred wave stale since 2026-07; tool contract parity tests exist | spike: handler contract + parity fixtures update |
| F-4 | Prometheus-style metrics export | adjacent to shipped R-F2 OTel tracing; `get_metrics`/`health_check` today are ad-hoc JSON | spike: which existing metrics map cleanly vs. need instrumentation (vision Phase 2) |
| F-5 | MCP `elicitation`/`tasks` capabilities | protocol negotiation already advertises `2025-11-25`; capability methods exist as explicit not-implemented branches | spike: client-compat matrix of target MCP clients (HY-07) |
| F-6 | A/B testing framework (vision Phase 1) | largest unstarted vision item; product-critical after feedback ranking (ADR-082) landed | spike: minimum viable experiment loop + success metrics from existing reward/recommendation telemetry |

R-F1/R-F3/R-F4/R-F6/R-F7 remain `NO-GO`; nothing in this analysis changes their gates.

---

## Consequences

**Positive**
- Docs and tool contracts stop promising embeddings providers the code cannot deliver, or the
  providers arrive deliberately (either way the mismatch resolves).
- Rebaselined `TECH_DEBT.md` and split R-F5 restore tracker trust at the same level the code
  already enjoys (ADR-055 showed doc drift is the repo's recurring failure mode, not code decay).
- The next feature wave has a gated, owner-ready candidate list instead of an empty tracker.

**Negative / risks**
- Resolving F-1 by "fix the prose" loses a capability the architecture docs wanted since the
  project's start; resolving by implementation invites dimension-mismatch reindex churn
  (ADR-077 makes silent substitution impossible, which is the right constraint to pay for).
- HY-01 was withdrawn on isolated re-verification (2026-11-25): `bin/act` was never
  tracked, so no workflow changes; the durable takeaway is the output-attribution trap
  itself (LESSON-088).

**Follow-ups**
- Re-run this audit class after the next release wave; the doc-truth findings (Phase 0) are
  the items that historically regress first.

## Verification Commands (run 2026-11-25 on `main` @ `eb076391`)

```bash
gh issue list --state open        # 0
gh pr list --state open           # 0
do-harness verify --set feedback  # 5/5 PASS
grep -rn "TODO\|FIXME\|unimplemented!\|todo!()" --include="*.rs" \
  memory-core/src memory-mcp/src memory-cli/src \
  memory-storage-turso/src memory-storage-redb/src examples/src tests/src   # 1 hit
grep -rn "cohere\|ollama" --include="*.rs" memory-core/src        # constants.rs + provider docs only
grep -n "Cohere\|Ollama" AGENTS.md agent_docs/service_architecture.md  # 1 + 1 stale claims
git ls-files bin/                 # empty — nothing under bin/ is tracked
git check-ignore -v bin/act       # .gitignore:124:'/bin/' — local file, ignored
git rev-list --all --objects | grep -c 'bin/act'   # 0 — never in history (erratum 2026-11-25)
```
