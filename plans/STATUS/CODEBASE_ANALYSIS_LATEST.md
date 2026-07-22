# Codebase Analysis Latest — 2026-07-22

**Branch**: `main` (+ R-E2 PR)  
**Workspace**: `0.1.36` · **Released tag**: `v0.1.35`  
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
| Production LOC >500 (non-test `src`) | **0** |
| Open GitHub issues | **0** |
| Skills with evals | 34/34 (medium-risk behavioral second wave) |
| Skill routes | 34/34 |
| Mainline CI (recent runs) | Success (Quick Check, Skill Evals, Storage Matrix, Release Drift) |
| Fail-closed code execution | Preserved |
| Plans active-set | Canonical after 2026-07 consolidation |
| Release readiness | `verify-release-state` ready for v0.1.36 |

## Strengths

1. Strong correctness campaign (locks, eviction, cache identity, embedding health).  
2. Gate honesty (deny blocking, benchmark hard-fail, workflow cancelled guards).  
3. Skill eval schema + high-risk and medium-risk behavioral fixtures.  
4. Release path singularity (`release-manager` + `release.yml` + cadence manager).  
5. Rich episodic/pattern/playbook/checkpoint MCP+CLI surface.

## Weaknesses

1. Release lag (0.1.36 unreleased).  
2. Historical ADR filename collisions (aliased, not renumbered).  
3. Transitive Dependabot advisories on upstream chains.  
4. Product/research epics remain spike-gated (R-F*).

## Recommended focus order

1. Ship v0.1.36 + post-bump 0.1.37  
2. Optional research spikes only after GO artifacts  
3. Upstream security chain hygiene when direct upgrades are available  

Full prioritized backlog: recommendations plan §3–4.
