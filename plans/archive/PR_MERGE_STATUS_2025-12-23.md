# PR Merge Status Report - 2025-12-23

## Summary
9 open PRs reviewed and processed. **1 PR closed** (bincode upgrade rejected).

## Auto-Merge Enabled (7 PRs)
These PRs have passing checks and auto-merge enabled - will merge automatically:

| PR | Title | Status |
|----|-------|--------|
| #171 | chore(deps): bump reqwest from 0.12.25 to 0.12.26 | ✅ Auto-merge enabled |
| #169 | chore(deps): bump deep_causality from 0.11.11 to 0.12.2 | ✅ Auto-merge enabled |
| #168 | chore(deps): bump rquickjs from 0.6.2 to 0.10.0 | ✅ Auto-merge enabled |
| #167 | chore(deps): bump augurs from 0.6.3 to 0.10.1 | ✅ Auto-merge enabled |
| #165 | ci(deps): bump actions/download-artifact from 4 to 7 | ✅ Auto-merge enabled |
| #164 | ci(deps): bump actions/setup-python from 5 to 6 | ✅ Auto-merge enabled |
| #163 | ci(deps): bump actions/upload-artifact from 4 to 6 | ✅ Auto-merge enabled |

## Cannot Merge - Critical Issues (3 PRs)
These PRs have serious problems that require immediate attention:

| PR | Title | Issue |
|----|-------|-------|
| #176 | chore(deps): bump wasmtime-wasi from 24.0.5 to 36.0.3 | Format + Clippy failures |
| #175 | chore(deps): bump bincode from 1.3.3 to 3.0.0 | 🔴 CRITICAL: Bincode 3.0 is discontinued |
| #173 | chore(deps): bump wasmtime from 24.0.5 to 36.0.3 | Format + Clippy failures |

### Critical Findings

#### 🚨 Bincode 3.0.0 is Discontinued
- **PR #175 should be REJECTED** - bincode 3.0.0 contains only an error message
- The project was abandoned due to a doxxing incident
- Migration path chosen: Runtime uses postcard; tests retain bincode (dev-dep) for Options.with_limit checks.
- Note: Wincode is NOT a drop-in replacement for bincode in our use-case (Options API not compatible).
- **Effort**: 2-4 hours for migration
- **Priority**: 🔴 **IMMEDIATE ACTION REQUIRED**

#### ⚠️ Wasmtime 24.0.5 → 36.0.3 (PRs #173, #176)
- Major version jump with significant breaking changes
- Requires Rust 1.89.0+ (was 1.78.0+)
- WASI HTTP header handling changes
- Component model modifications
- **Effort**: 8-16 hours for migration
- **Priority**: 🟡 **Plan migration window**

## Immediate Actions Required
1. **Close PR #175** - bincode 3.0 is discontinued
2. **Create migration plan** for bincode replacement
3. **Plan wasmtime migration** in separate branch
4. Update Cargo.toml: Keep bincode in [dev-dependencies] for tests; runtime paths use postcard
