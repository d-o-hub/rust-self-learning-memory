# Codebase Analysis Latest — 2026-07-26

**Branch**: `main` @ `648e11ad`  
**Workspace**: `0.1.37` · **Released tag**: `v0.1.36`  
**Companion**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`

## Architecture (as implemented)

| Crate | Role |
|-------|------|
| `do-memory-core` | Episodes, patterns, rewards, retrieval (CSM cascade), embeddings, F4 provenance/journal |
| `do-memory-storage-turso` | Durable libSQL / Turso |
| `do-memory-storage-redb` | Embedded cache |
| `do-memory-mcp` | MCP server, lazy tools, audit, fail-closed code exec |
| `do-memory-cli` | Operator CLI |
| `do-memory-test-utils` / benches / examples / e2e | Support |

**Stack**: Rust 2024, Tokio, Turso/libSQL, redb, postcard, optional embeddings, `csm` cascade.

## Health summary

| Check | Result |
|-------|--------|
| Production `todo!` / unimplemented | None |
| Production LOC >500 (non-test `src`) | **0** |
| Released tag | **v0.1.36** |
| Workspace advanced post-tag | **0.1.37** |
| Skills with evals / routes | 40/40 |
| Fail-closed code execution | Preserved (ADR-073) |
| Open issues | **0** |
| Open PRs | **0** (none) |
| ADR-071 | Accepted / Implemented (auto-checkpoint on Abstained) |
| ADR-072 | Accepted / Implemented (authority + governance) |
| ADR-073 | Accepted / Implemented (S1.1c NO-GO, fail-closed) |

## Strengths

1. Correctness campaign (locks, eviction, cache identity, embedding health).  
2. Gate honesty (deny, benchmarks, cancelled guards, docs integrity ship gate).  
3. Skill eval schema + high- and medium-risk behavioral fixtures (40 skills, all routed).  
4. Singular release path (`release-manager` + `release.yml`).  
5. Rich episodic/pattern/playbook MCP+CLI surface.
6. R-F8 CLI relationship box-drawing panel + R-F9 HNSW persistence/eviction (#893).

## Weaknesses / residual

1. Historical ADR filename collisions (025/054 aliased — docs only, no code action).  
2. Transitive Dependabot advisories (upstream chains, monitor only).  
3. Remaining product/research epics spike-gated (R-F1…R-F7, R-F10).  

## Recommended focus order

1. #894 merged; ADR-077 A6 merged #897 — no open PRs.  
2. Cut v0.1.37 once sufficient unreleased commits accumulate.  
3. Research spikes only after individual GO artifacts under `plans/STATUS/spikes/`.

Full prioritized backlog: recommendations plan §3–4.
