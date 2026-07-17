# GOAP Actions Backlog

- **Last Updated**: 2026-07-17 (K3.1 + W2.1)
- **Archived Plans**: `plans/archive/2026-03-consolidation/`
- **Active plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`

## Active Actions (2026-07-17b K3.1 + W2.1)

| ID | Action | Status |
|----|--------|--------|
| ACT-240 | Strict run-evals schema + fixtures (K3.1a) | ✅ Done |
| ACT-241 | Migrate skill evals off noop true / legacy evals | ✅ Done |
| ACT-242 | Gate contract matrix (W2.1a) | ✅ Done |
| ACT-243 | validate-gate-contract.sh | ✅ Done |
| ACT-244 | Open PR for K3.1/W2.1 | 🟡 |
| ACT-245 | CI wire K3.1b / W2.1b | ⏳ |

## Active Actions (2026-07-16b S1.3–S1.6 + W2.2)

| ID | Action | Status |
|----|--------|--------|
| ACT-210 | S1.3 short write locks; no await under episodes write | ✅ Done |
| ACT-211 | S1.4 durable capacity eviction backend deletes | ✅ Done |
| ACT-212 | S1.5 EmbeddingHealth + allow_mock_fallback | ✅ Done |
| ACT-213 | S1.6 retry queue timeout + first attempt free | ✅ Done |
| ACT-214 | W2.2 cargo deny blocking; remove audit soft-pass | ✅ Done |
| ACT-215 | Tests + plans update | ✅ Done |
| ACT-216 | Open PR + CI green + review | 🟡 Next |
| ACT-217 | Cut v0.1.35 via release.yml (closes #841) | ⏳ After merge |

## Completed Actions (2026-07-16 Missing Tasks Swarm — PR #840)

| ID | Action | Status |
|----|--------|--------|
| ACT-200 | #837 restore fuzzy_match public rustdoc | ✅ Done |
| ACT-201 | S1.2 CacheKey identity (language/framework/complexity/tags) | ✅ Done |
| ACT-202 | W2.3 build-rust accept do-memory-* hyphens | ✅ Done |
| ACT-203 | W2.6 split production files >500 LOC | ✅ Done |
| ACT-204 | S1.1a/D3.2 fail-closed execute_agent_code + README fixes | ✅ Done |
| ACT-205 | Update plans/ + CHANGELOG for swarm | ✅ Done |
| ACT-206 | Open PR + CI green + review | ✅ Done (PR #840) |
| ACT-207 | Cut v0.1.35 via release.yml (closes #828/#838) | ⏳ After release ready |

## Completed Actions (v0.1.35 CLI UX Patch — on main)

| ID | Action | Status |
|----|--------|--------|
| ACT-190 | #831 Pattern postcard + get_all_patterns + e2e | ✅ Done |
| ACT-191 | #830 Honor --db-path for redb_path | ✅ Done |
| ACT-192 | #829 config init/show-template + serde default + example | ✅ Done |
| ACT-193 | #832 storage_mode alias + config show | ✅ Done |
| ACT-194 | Prevention tests (postcard, redb list, loader, snapshot) | ✅ Done |
| ACT-195 | Update plans/ + CHANGELOG for 0.1.35 | ✅ Done |
| ACT-196 | Open PR + CI green + merge | ✅ Done (merged) |
| ACT-197 | Cut v0.1.35 via release.yml (closes #828) | ⏳ After release ready |

### Prevention permanently (do not regress)
- Never `#[serde(tag=)]` on postcard types
- StorageBackend new methods → all backends
- CLI path flags → set redb_path
- Cross-process storage features → e2e CLI test

---

## Completed Actions Summary

All actions from v0.1.17 through v0.1.27 sprints are complete. See archived execution plans in `plans/archive/2026-03-consolidation/` for full details.

| Sprint | Actions | Count | Status |
|--------|---------|-------|--------|
| v0.1.27 | Bayesian, GC, Pages, llms.txt, semver fix | 7 | ✅ All Complete |
| v0.1.24 | ACT-089 through ACT-093 | 5 | ✅ All Complete |
| v0.1.23 | ACT-080 through ACT-088 | 9 | ✅ All Complete |
| v0.1.22 | ACT-053 through ACT-075 | 23 | ✅ All Complete |
| v0.1.21 | ACT-038 through ACT-052 | 15 | ✅ All Complete |
| v0.1.20 | ACT-020 through ACT-037 | 18 | ✅ All Complete |
| v0.1.17-19 | ACT-001 through ACT-019 | 19 | ✅ All Complete |

## Learning Delta (2026-03)

### redb 3.x Breaking Changes
- `begin_read()` moved to `ReadableDatabase` trait (must import it)
- `begin_write()` remains on `Database` struct (no change)

### rand 0.10 Breaking Changes
- `thread_rng()` → `rand::rng()` (function rename)
- `Rng::gen()` → `RngExt::random()` (method rename)
- `Rng::gen_range()` → `RngExt::random_range()` (method rename)
- Import `RngExt` for user-level RNG methods
- Keep `rand` and `rand_chacha` versions aligned

## Active Actions (v0.1.33 Sprint — Release + CI + Quality)

### GOAP Skills in Use

- **Coordinator**: `goap-agent`
- **Validation**: `code-quality`, `test-runner`, `architecture-validation`
- **Release**: `github-release-best-practices`, `release-guard`
- **CI**: `ci-fix`

### Phase 1: Release (Closes #674)

- **ACT-175**: Tag and release v0.1.33
   - Goal: WG-175
   - Skills: `goap-agent`, `github-release-best-practices`, `release-guard`
   - Action: Generate CHANGELOG for 94 commits since v0.1.32 → `gh release create v0.1.33`
   - Dependencies: None (push CI is green)
   - Status: 🟡 Queued

### Phase 2: CI Health

- **ACT-176**: Add 3 missing gitleaks fingerprints
   - Goal: WG-176
   - Skills: `ci-fix`
   - Action: Run gitleaks locally to get fingerprints → append to `.gitleaksignore`
   - Dependencies: None
   - Status: ✅ Complete (PR #675, merged 2026-06-30)

- **ACT-177**: Add disk cleanup step to nightly-tests.yml
   - Goal: WG-177
   - Skills: `ci-fix`
   - Action: Add `df -h && docker system prune -af` step before build in nightly workflow
   - Dependencies: None
   - Status: ✅ Complete (PR #675, merged 2026-06-30)

- **ACT-178**: Scope mutation testing to memory-core + 2h ceiling
   - Goal: WG-178
   - Skills: `ci-fix`
   - Action: Update `mutants.yml` to target only `memory-core/src` with `timeout: 7200`
   - Dependencies: None
   - Status: ✅ Complete (PR #675, merged 2026-06-30)

- **ACT-179**: Bump upload-artifact to Node 24-compatible version
   - Goal: WG-179
   - Skills: `ci-fix`
   - Action: Update `actions/upload-artifact` SHA across all workflows to latest v4+ that targets Node 24
   - Dependencies: None
   - Status: ✅ Complete (PR #675 + PR #681, merged 2026-06-30)

### Phase 3: Code Quality

- **ACT-180**: Fix clippy `--all-features` lints in mistral/client.rs
   - Goal: WG-180
   - Skills: `code-quality`
   - Action: Extract nested match arms into helpers; remove unnecessary Result wrapper from `dequantize_binary_embeddings`
   - Dependencies: None
   - Status: ✅ Complete (PR #675, merged 2026-06-30)

- **ACT-181**: Split cache/wrapper.rs below 500 LOC
   - Goal: WG-181
   - Skills: `code-quality`
   - Action: Extract bulk/batch operations into `cache/wrapper_ops.rs`; keep core wrapper at <500 LOC
   - Dependencies: None
   - Status: ✅ Complete (PR #675, merged 2026-06-30 — extracted to wrapper_backend.rs)

### Phase 4: Architecture

- **ACT-182**: Add tracing::warn! to non-CSM cascade fallback
   - Goal: WG-182
   - Skills: `feature-implement`
   - Action: Add `tracing::warn!("CSM feature not enabled; cascade retrieval returns empty results")` to the `#[cfg(not(feature = "csm"))]` branch
   - Dependencies: None
   - Status: ✅ Complete (PR #675, merged 2026-06-30)

### Phase 5: DevX Backlog

- **ACT-183**: Implement llms.txt generator (closes #652)
   - Goal: WG-183
   - Skills: `feature-implement`
   - Action: Create `scripts/generate-llms-txt.sh` that concatenates README, AGENTS.md, and key docs into `llms.txt`/`llms-full.txt`
   - Dependencies: None
   - Status: 🟡 Queued

- **ACT-184**: ADR for VERSION file decision (closes #653)
   - Goal: WG-184
   - Skills: `goap-agent`
   - Action: Write ADR-059 evaluating VERSION file vs Cargo.toml workspace version; recommend keeping Cargo.toml as single source
   - Dependencies: None
   - Status: 🟡 Queued

### Phase 4: Research Backlog (Deferred until CPU/token wins are landed)

- **ACT-114**: Add temporal graph edges to episode store
   - Goal: WG-123
   - Action: Add Turso schema for episode→episode and episode→pattern edges with temporal weights; implement graph traversal queries
   - Paper: REMem (ICLR 2026, arXiv:2602.13530)
   - Status: ✅ Complete (PR #570: weighted traversal, pattern edges, significance weights, storage schema)

- **ACT-115**: Add procedural memory type
   - Goal: WG-124
   - Action: New `ProceduralMemory` type in memory-core; storage traits in turso/redb; extends existing episodic+semantic with learned skill patterns
   - Paper: ParamAgent (2026) three-tier memory
   - Status: ✅ Complete (PR #569 merged via admin)

- **ACT-116**: Evaluate Routing-Free MoE for DyMoE
   - Goal: WG-125
   - Action: Read arXiv:2604.00801 + reference implementation; write evaluation ADR comparing to current DyMoE routing-drift protection
   - Paper: arXiv:2604.00801
   - Status: 🔵 Backlog

- **ACT-125**: Evaluate LottaLoRA-inspired local classifier
   - Goal: WG-132
   - Action: Read arXiv:2604.08749; prototype frozen-random-backbone + LoRA for episode-type classification (CPU-only, no API)
   - Paper: LottaLoRA (arXiv:2604.08749, Apr 2026)
   - Status: ✅ Complete — evaluation document at `plans/WG-132_LottaLoRA_Evaluation.md`

- **ACT-126**: Align memory architecture with agentic memory taxonomy
   - Goal: WG-133
   - Action: Map current episodic/semantic/pattern types to arXiv:2602.19320's 4-structure taxonomy; update architecture docs
   - Paper: Anatomy of Agentic Memory (arXiv:2602.19320)
   - Status: ✅ Complete — evaluation document at `plans/WG-133_AgenticMemoryTaxonomy_Evaluation.md`

- **ACT-127**: Evaluate DAG-based state management
   - Goal: WG-134
   - Action: Adapt arXiv:2602.22398 DAG-based conversation state approach for episode context assembly; target 20-86% token reduction
   - Paper: arXiv:2602.22398
   - Status: ✅ Complete — ~1,320 LOC in `memory-core/src/context/dag/`, 24 tests, ADR-054

- **ACT-129**: Implement CloudEvents EventEmitter
   - Goal: WG-149
   - Skills: `goap-agent`, `feature-implement`, `architecture-validation`, `test-runner`
   - Action: Add CloudEvent struct (1.0 spec), EventEmitter trait, MemoryEventMapping, LogEmitter, NoOpEmitter, HttpEmitter (http-emitter feature), EventEmitterMode enum, wired into SelfLearningMemory, Environment variable support (MEMORY_EVENT_EMITTER, MEMORY_EVENT_EMITTER_URL)
   - Dependencies: None
   - Status: ✅ Complete — 13 emitter tests pass, all 1050 core tests pass

- **ACT-128**: Evaluate federated HDC for multi-agent memory
   - Goal: WG-135
   - Action: Evaluate HDC prototype exchange (arXiv:2603.20037) as bandwidth-efficient alternative for WG-126 MemCollab
   - Paper: arXiv:2603.20037
   - Status: 🔵 Evaluated — evaluation document at `plans/WG-135_FederatedHDC_Evaluation.md`

## Completed Actions (v0.1.30 Sprint)

- **ACT-097**: Implement `MemoryEvent` broadcast channel — ✅ Complete (WG-103)
- **ACT-098**: Replace sorted retrieval with `select_nth_unstable_by` — ✅ Complete (WG-104)
- **ACT-099**: Idempotent cargo publish guard — ✅ Already exists (WG-105)
- **ACT-100**: Create `memory-context` skill — ✅ Complete (WG-106)
- **ACT-101**: Create `learn` skill — ✅ Complete (WG-107)

## Completed Actions (v0.1.28–v0.1.29)

- **ACT-094**: Merge PR #406 (ai-slop detector) — ✅ Merged (WG-091)
- **ACT-095**: Fix CodeQL cleartext logging — ✅ Complete, commit fc9c302c (WG-093)
- **ACT-096**: Archive completed plans/ noise — ✅ 87% noise reduction
