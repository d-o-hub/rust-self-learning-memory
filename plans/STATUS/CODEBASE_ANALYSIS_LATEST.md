# Codebase Analysis Latest — 2026-07-20

**Commit**: `2e0a2b89` · **Branch**: `main`  
**Workspace**: `0.1.36` · **Released**: `v0.1.35`  
**Companion**: `plans/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`

## Architecture (as implemented)

| Crate | Role |
|-------|------|
| `do-memory-core` | Episodes, patterns, rewards, retrieval (incl. CSM cascade), embeddings, F4 provenance/journal |
| `do-memory-storage-turso` | Durable libSQL / Turso |
| `do-memory-storage-redb` | Embedded cache |
| `do-memory-mcp` | MCP server, lazy tools, audit, fail-closed code exec |
| `do-memory-cli` | Operator CLI (episode, pattern, storage, config, …) |
| `do-memory-test-utils` / benches / examples / e2e | Support |

**Stack**: Rust 2024, Tokio, Turso/libSQL, redb, postcard, optional OpenAI/Mistral/local embeddings, `csm` cascade.

## Health summary

| Check | Result |
|-------|--------|
| Production `todo!` / unimplemented strings | None found |
| Production LOC >500 | **1** (`embeddings/config/provider_config.rs`) |
| Open GitHub issues/PRs | **0** |
| Skills with evals | 33/34 (`ci-poll` missing) |
| Skill routes | 16/34 |
| Mainline CI (recent runs) | Success (CI, Security, Skill Evals, Storage Matrix) |
| Fail-closed code execution | Preserved |
| Plans active-set | Canonical after 2026-07 consolidation |

## Strengths

1. Strong correctness campaign (locks, eviction, cache identity, embedding health).  
2. Gate honesty (deny blocking, benchmark hard-fail, workflow cancelled guards).  
3. Skill eval schema + high-risk behavioral fixtures.  
4. Release path singularity (`release-manager` + `release.yml` + cadence manager).  
5. Rich episodic/pattern/playbook/checkpoint MCP+CLI surface.

## Weaknesses

1. Release lag (0.1.36 unreleased).  
2. Skills discoverability incomplete (routes, SKILLS.md, one missing eval).  
3. F4 pilots under-exposed to operators.  
4. ADR numbering collisions.  
5. Some user docs / TECH_DEBT lag code truth.

## Recommended focus order

1. Release v0.1.36  
2. LOC + skill contract completion  
3. F4 productization  
4. Optional research spikes  

Full prioritized backlog: recommendations plan §3–4.
