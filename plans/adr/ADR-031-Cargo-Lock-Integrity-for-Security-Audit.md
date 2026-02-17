# ADR-031: Cargo.lock Integrity for Security Audit Stability

**Status**: Accepted  
**Date**: 2026-02-17

## Context

`main` branch `Security` workflow failed in `supply-chain-audit` due to a `cargo audit` panic:

- `invalid Cargo.lock dependency tree: Resolution("failed to find dependency: wasm-encoder 0.244.0")`

The failure is caused by lockfile graph inconsistency (dependency references present without corresponding package nodes), not by a RustSec advisory finding.

## Decision

Treat `Cargo.lock` graph integrity as a CI correctness requirement for security scans:

1. Repair lockfile node consistency by regenerating/normalizing `Cargo.lock`.
2. Commit lockfile repair atomically with no unrelated source changes.
3. Validate with local `cargo audit` before PR verification.

## Consequences

- `Security` workflow can execute vulnerability scanning without crashing.
- Security signals return to advisory-based findings instead of tool panic noise.
- Future lockfile anomalies can be triaged as CI correctness regressions.

## Related

- `plans/adr/ADR-022-GOAP-Agent-System.md`
- `plans/adr/ADR-023-CI-CD-GitHub-Actions-Remediation.md`
- `plans/adr/ADR-029-GitHub-Actions-Modernization.md`
