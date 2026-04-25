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
