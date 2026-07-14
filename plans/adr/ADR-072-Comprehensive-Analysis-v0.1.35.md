# ADR-072: Comprehensive Analysis v0.1.35 — Supply Chain Fix & Patch Release

- **Status**: Accepted
- **Date**: 2026-07-14
- **Context**: Sprint planning analysis after v0.1.34 release; version correction and patch release
- **Skill**: `goap-agent` → `codebase-analyzer` → `pr-readiness`

---

## Context

Following the v0.1.34 release (2026-07-09), PR #822 incorrectly bumped the workspace to v0.2.0 citing a "semver breaking change" in the MCP crate. However, the project is pre-1.0 and follows `0.1.x` patch releases exclusively. The version must be corrected to `0.1.35`.

Additionally, the `spin` crate v0.9.8 was yanked from crates.io, causing `cargo-deny` failures in both the `Supply Chain Security` and `Security` CI workflows on main and all open PRs.

### Current State (2026-07-14)

| Aspect | Status |
|--------|--------|
| Workspace version (Cargo.toml) | 0.2.0 (INCORRECT — must be 0.1.35) |
| Latest release | v0.1.34 |
| Local quality (build/clippy/fmt) | ✅ All pass |
| Main CI (tests, semver, MCP) | ✅ Pass |
| Main CI (supply chain, security) | ❌ Fail (yanked spin 0.9.8) |
| Open issues | 1 (#823 — release drift) |
| Open PRs | 3 (#824 ready, #820/#821 blocked) |

---

## Decision

### D1: Revert workspace version to 0.1.35

The project is pre-1.0. Under [semver pre-1.0 rules](https://semver.org/#spec-item-4), breaking changes are permitted in any 0.x.y release. There is no need to jump to 0.2.0. The consistent release line is `0.1.x` and the next patch is `0.1.35`.

**Action**: Change `version = "0.2.0"` → `version = "0.1.35"` in workspace `Cargo.toml` and all member crates.

### D2: Merge PR #824 immediately (spin fix)

PR #824 updates `spin` from yanked 0.9.8 to 0.9.9. It is:
- `mergeable: MERGEABLE`
- `mergeStateStatus: CLEAN`
- All CI checks: `SUCCESS`

This is the critical-path fix that unblocks all other work.

### D3: Sequential dependency PR merge after #824

After #824 merges to main, PRs #820 and #821 need rebasing to pick up the spin fix. Merge order:
1. #824 (spin fix)
2. #820 (rust-patch-minor: 3 dep updates)
3. #821 (sysinfo 0.38.4 → 0.39.6 — major bump, but pre-1.0 allows this)

### D4: Cut v0.1.35 patch release

After all PRs merged and version corrected, tag `v0.1.35` to resolve issue #823 (release drift).

### D5: No new features in this sprint

This sprint is purely maintenance: fix supply chain, correct version, merge deps, release.

### D6: Enforce version policy across all agent docs and skills

Added `Version Policy (MANDATORY)` section to:
- `AGENTS.md` — Core Invariants + dedicated section
- `.agents/skills/release-guard/SKILL.md` — Core Rules
- `.agents/skills/goap-agent/SKILL.md` — Hard Constraint section
- `.agents/skills/feature-implement/SKILL.md` — Project Standards
- `scripts/verify-release-state.sh` — Automated enforcement check

This prevents future accidental version over-bumps by tooling (e.g., cargo-semver-checks suggesting 0.2.0).

---

## Consequences

### Positive
- Version line stays consistent (0.1.x)
- Main branch CI fully green (all 10+ workflows)
- Zero open PRs after sprint
- Zero open issues (drift auto-closes on release)
- Clean patch release state for v0.1.35

### Negative / Risks
- The "semver breaking change" from PR #822 (MCP crate) is now released as a patch. Since the project is pre-1.0, this is acceptable per semver spec, but consumers should be aware of API changes in the MCP crate.
- sysinfo 0.38 → 0.39 is a major bump in that dependency; tests already pass.

### Neutral
- Release drift monitoring continues via `release-drift.yml`
- Backlog items (#743, #746, #749, #753) remain unchanged

---

## Work Groups

| WG | Task | Effort | Priority | Dependencies |
|----|------|--------|----------|--------------|
| WG-186 | Merge PR #824 (spin fix) | 5 min | Critical | None |
| WG-187 | Revert version 0.2.0 → 0.1.35 | 10 min | Critical | None |
| WG-188 | Rebase + merge #820, #821 | 30 min | High | WG-186 |
| WG-189 | Update CHANGELOG.md for v0.1.35 | 15 min | High | WG-187, WG-188 |
| WG-190 | Update STATUS/CURRENT.md | 10 min | High | WG-188 |
| WG-191 | Tag v0.1.35 release | 10 min | High | WG-189, WG-190 |
| WG-192 | Verify release workflow | 20 min | High | WG-191 |
| WG-193 | Confirm #823 auto-closed | 2 min | Low | WG-192 |

---

## Quality Gates

| Gate | Trigger | Verification |
|------|---------|--------------|
| Gate 1 | After WG-186 | `gh run list --branch main --limit 5` shows Security + Supply Chain = success |
| Gate 2 | After WG-187 | `cargo metadata --format-version 1 \| jq '.packages[].version'` all show 0.1.35 |
| Gate 3 | After WG-188 | `gh pr list --state open` returns empty |
| Gate 4 | After WG-192 | `gh release view v0.1.35` succeeds; `gh issue list --state open` returns empty |

---

## Skill Invocation Order

1. `pr-readiness` — verify #824 merge state, execute merge
2. `ci-poll` — wait for main CI to go green
3. `feature-implement` — version correction in Cargo.toml
4. `ci-fix` — rebase #820/#821 if conflicts arise
5. `pr-readiness` — merge #820, #821
6. `agents-update` — update CHANGELOG, STATUS, ROADMAP
7. `release-guard` — tag and push v0.1.35
8. `ci-poll` — verify release workflow completes

---

## Recommendations (Post-Sprint)

### R1: Trusted Publishing Migration (Medium Priority)

Migrate crates.io publishing from `CARGO_REGISTRY_TOKEN` to OIDC-based Trusted Publishing. This eliminates secret rotation burden and reduces supply chain attack surface. See <https://crates.io/docs/trusted-publishing>.

**Effort**: ~2 hours
**Tracking**: Future ADR (referenced in ADR-045)

### R2: Code Coverage Push to 70% (Medium Priority)

Current coverage is 61.22%. ADR-042 Phase 1 target is 70%. Focus areas:
- `memory-core/src/retrieval/` — cascade tiers
- `memory-mcp/src/bin/` — server integration paths
- `memory-cli/src/commands/` — CLI handlers

**Effort**: ~1 sprint
**Tracking**: ADR-042

### R3: Stale Branch Cleanup (Low Priority)

42 remote branches exist. Many are from merged PRs. Run `git fetch --prune` and delete merged branches.

### R4: Dependabot Auto-Merge for Patch Updates (Low Priority)

Enable auto-merge for dependabot PRs that only bump patch versions and pass all CI.

### R5: Prevent Future Version Over-Bumps

Add a CI check or pre-commit hook that validates the workspace version stays on the `0.1.x` line until a deliberate 1.0 decision is documented via ADR.

---

## Related Documents

| Document | Purpose |
|----------|---------|
| `plans/GOAP_STATE.md` | Updated GOAP state (this sprint) |
| `plans/ROADMAPS/ROADMAP_ACTIVE.md` | Active roadmap (update after release) |
| `plans/adr/ADR-071-Auto-Checkpoint-on-Abstained.md` | Previous ADR |
| `.agents/skills/pr-readiness/SKILL.md` | PR merge verification skill |
| `.agents/skills/release-guard/SKILL.md` | Release process skill |
| `.agents/skills/ci-fix/SKILL.md` | CI failure diagnosis skill |
