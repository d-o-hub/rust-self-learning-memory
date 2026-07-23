# Codebase Analysis Latest — 2026-07-23

**Branch**: `main` @ `66286948`  
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
| Production `todo!` / unimplemented | None found (prior audits) |
| Production LOC >500 (non-test `src`) | **0** |
| Released tag | **v0.1.36** |
| Workspace advanced post-tag | **0.1.37** |
| Skills with evals / routes | 34/34 |
| Fail-closed code execution | Preserved |
| Open issues | **0** |
| Open PRs | 3 (#887–#889) |

## Strengths

1. Correctness campaign (locks, eviction, cache identity, embedding health).  
2. Gate honesty (deny, benchmarks, cancelled guards, docs integrity ship gate).  
3. Skill eval schema + high- and medium-risk behavioral fixtures.  
4. Singular release path (`release-manager` + `release.yml`).  
5. Rich episodic/pattern/playbook MCP+CLI surface.

## Weaknesses / residual

1. Historical ADR filename collisions (aliased only).  
2. Transitive Dependabot advisories.  
3. Product/research epics still spike-gated (R-F*).  
4. Open hygiene/perf PRs not yet landed.

## Recommended focus order

1. Land or close open PRs #887–#889 with evidence.  
2. Optional #888 only with bench comparison.  
3. Research spikes only after GO artifacts.  

Full prioritized backlog: recommendations plan §3–4.
