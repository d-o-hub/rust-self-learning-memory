# GOAP PR304 Rescue Plan (2026-02-17)

## Goal
Fix failing PR #304 by isolating intended docs-only changes on top of current `main`.

## Diagnosis
- PR #304 included unrelated historical commits and failed CI on non-doc code paths.
- The intended change was the docs/plans archive operation.

## Strategy
1. Build a fresh branch from `main`.
2. Apply only the docs archive commit content.
3. Open replacement PR to `main`.
4. Enable rebase auto-merge after checks pass.
5. Close #304 as superseded by replacement PR.

## Outcome
- Replacement PR created from clean base.
- CI expected to validate docs-only delta against current `main`.
