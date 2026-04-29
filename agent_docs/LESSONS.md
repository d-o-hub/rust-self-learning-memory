# Lessons Log

Compact log for non-obvious workflow learnings. Pair each entry here with a short distilled note in the nearest `AGENTS.md`.

## LESSON-001: Verify release/package truth before roadmap edits

- Issue: Planning docs treated `v0.1.30` as unreleased even though GitHub already had a published release and publishable crates were still at `0.1.30`.
- Root Cause: `plans/` had drifted from repo reality and version work was being planned from stale documents instead of live release/package data.
- Solution: Check `gh release view <tag>` and `cargo metadata --no-deps --format-version 1` before changing roadmap, status, or version-bump plans.

## LESSON-002: Sprint reprioritization must update all live plan files together

- Issue: CPU/token-efficiency work was added in one place while adjacent GOAP and status docs still described the old sprint goal.
- Root Cause: The live planning surface is split across roadmap, goals, actions, GOAP state, and status files.
- Solution: Update `plans/ROADMAPS/ROADMAP_ACTIVE.md`, `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`, and `plans/STATUS/CURRENT.md` in one pass.

## LESSON-003: `learn` workflow needs both the distilled note and the log file

- Issue: `.agents/skills/learn/SKILL.md` required dual-write to `agent_docs/LESSONS.md`, but that file did not exist.
- Root Cause: The workflow contract was documented before the backing log file was added.
- Solution: Keep `agent_docs/LESSONS.md` as the verbose log and reserve root `AGENTS.md` for compact workflow notes only.

## LESSON-004: Highest-leverage efficiency work is retrieval-path focused

- Issue: The repo's best near-term CPU/token wins were easy to miss because they are split across plans and code placeholders.
- Root Cause: Performance opportunities span both implementation gaps and prompt-assembly workflow docs.
- Solution: Prioritize cached retrieval wiring, QueryCache contention, bounded context assembly, hierarchical reranking, and compact high-frequency skill/docs before larger research features.

## LESSON-005: Secure JWT validation requires signature verification

- Issue: The MCP server used simplified JWT parsing that skipped signature verification, allowing token forgery.
- Root Cause: Development-only validation logic was left in place without enforcing secure production defaults.
- Solution: Use robust libraries like `jsonwebtoken` for verification. Enforce mandatory secret configuration for production modes and include security tests for signature and claim (iss, aud, exp, sub) validation.

## LESSON-006: CI Optimization - paths-based benchmark triggering

- Issue: Benchmark workflow runs ~54 min on every PR, even docs-only changes.
- Root Cause: No path filtering on benchmark workflow triggers.
- Solution: Add `paths` filter for perf-critical code only (storage, core, benches). GitHub Actions doesn't support `paths` + `paths-ignore` at same trigger level - use `paths` alone.
- Key insight: Use `.claude/skills/github-workflows` and `.claude/skills/ci-fix` skills for CI issues.

## LESSON-007: cosine_similarity normalization range [-1,1]→[0,1]

- Issue: Coverage test assertions expected raw cosine values (-1 to 1), but implementation normalizes to 0-1 range.
- Root Cause: `cosine_similarity` in `similarity.rs` normalizes: `(similarity + 1.0) / 2.0` to keep all values positive.
- Solution: Test assertions must account for normalization:
  - Identical vectors: 1.0 → 1.0 (unchanged)
  - Opposite vectors: -1.0 → 0.0
  - Orthogonal vectors: 0.0 → 0.5
  - Low similarity (orthogonal): 0.0 → 0.5
- Location: `memory-storage-turso/src/retrieval/similarity.rs` lines 52-54