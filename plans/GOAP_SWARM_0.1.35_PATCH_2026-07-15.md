# GOAP Swarm Orchestration — v0.1.35 CLI Patch

**Date**: 2026-07-15  
**Coordinator**: goap-agent + agent-coordination  
**Strategy**: Hybrid (Swarm validation + Sequential quality gates)  
**Branch**: `fix/0.1.35-patch-issues`  
**PR**: #834  
**Issues**: #828, #829, #830, #831, #832  

---

## Goal Hierarchy

```
G0: Ship verified fix for open CLI UX issues (#829–#832) + version align (#828)
├── G1: Implementation complete & correct vs issue repros
├── G2: Regression guards prevent recurrence
├── G3: Quality gates (fmt, clippy, tests)
├── G4: External CLI verification (issue parity)
└── G5: PR readiness (CI CLEAN, plans updated)
```

## World State (pre-swarm)

| Fact | Value |
|------|-------|
| Commit | `caab0e5d fix: resolve 0.1.35 patch issues #828-#832` |
| PR | #834 OPEN |
| Issues | #828–#832 still OPEN (close on merge) |
| Risk | #830 may need always-override redb_path if partial fix landed |

## Swarm Assignment

| Agent | Role | Tasks | Depends |
|-------|------|-------|---------|
| **W1 explore** | Gap analysis | Diff issue requirements vs tree; flag missing pieces | — |
| **W2 debugger** | #830 path override | Ensure `--db-path` always sets redb_path; unit/CLI proof | W1 findings |
| **W3 feature-implementer** | Prevention + CLI skill | Harden missing guards; update do-memory-cli-ops docs | W1 |
| **W4 test-runner** | Quality | fmt, clippy, targeted nextest | W2, W3 |
| **W5 general-purpose** | External CLI | Reproduce each GitHub issue outside repo | W2, W4 |
| **W6 code-reviewer** | Review | Diff review against AGENTS.md | W2, W3 |
| **W7 general-purpose** | Plans + PR | Update plans/, PR body Fixes, CI poll | W4, W5, W6 |

## Execution Strategy

```
Phase A (SWARM parallel):  W1
Phase B (SWARM parallel):  W2, W3  (after W1)
Phase C (SEQUENTIAL):      W4 → W5 → W6
Phase D (CONVERGE):        W7 synthesize + PR readiness
```

## Success Criteria

1. Exact issue #830: `--db-path /tmp/.../memory.db` opens that path (not XDG)
2. Exact issue #831: create → complete → `pattern list` ≥ 1 in fresh process
3. Exact issue #829: `config init` + partial TOML load
4. Exact issue #832: `[storage].storage_mode = "local"` works
5. clippy clean; regression tests pass
6. PR #834 body references Fixes; plans/ reflect completion

## Quality Gates

| Gate | Check |
|------|-------|
| QG1 | W1 gap report: no critical missing code |
| QG2 | W2+W3: edits compile |
| QG3 | W4: nextest targeted pass |
| QG4 | W5: all 4 issue CLI repros pass |
| QG5 | W6: no high-severity review findings |
| QG6 | W7: mergeStateStatus path clear or documented |

---

## Swarm Log

### Phase A — W1 explore (gap analysis)
- **Result**: No critical missing impl for #829–#832. Medium: #830 unit test + CHANGELOG drift.
- **QG1**: PASS

### Phase B — W2 fix + W3 docs
- **W2**: Extracted `apply_db_path_override` in `config/cli_overrides.rs` — always override XDG; redb-only uses exact path; turso feature uses siblings. 3 unit tests.
- **W3**: Updated do-memory-cli-ops skill + LESSON-017.
- **QG2**: PASS (compiles)

### Phase C — W4 tests + W5 CLI + W6 review
- **W4**: fmt/clippy/143 cli lib tests/postcard/redb list — all PASS
- **W5**: External #829–#832 all PASS on 0.1.35 release binary
- **W6**: Requested sibling mapping for turso + unit test — addressed in W2 follow-up
- **QG3–QG5**: PASS

### Phase D — W7 converge
- Plans updated; commit follow-up to PR #834; CI re-run expected

### Final world state
| Goal | Status |
|------|--------|
| G1 Implementation | ✅ |
| G2 Prevention | ✅ (cli_overrides tests + skill/LESSON) |
| G3 Quality | ✅ |
| G4 External CLI | ✅ |
| G5 PR push | 🟡 this commit |
