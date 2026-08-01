# ADR-078: OIDC Trusted Publishing for crates.io

- **Status**: Accepted
- **Date**: 2026-07-28
- **Deciders**: Project maintainers
- **Spike**: [`../STATUS/spikes/R-F10.json`](../STATUS/spikes/R-F10.json)
- **Related**: ADR-072 (release authority and evidence governance — not changed by this ADR)
- **Workflow affected**: `.github/workflows/publish-crates.yml` (4 `cargo publish` steps at lines ~81, 143, 205, 267)

## Context

`publish-crates.yml` authenticates to crates.io using a static long-lived
`CARGO_REGISTRY_TOKEN` repository secret. The same secret is injected at four
separate `cargo publish` steps (one per crate: `do-memory-core`,
`do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-mcp`).

Static long-lived tokens carry supply-chain risk:

- The token has an indefinite lifetime; a leak anywhere in the Actions runner
  environment exposes crate publish rights until the token is manually rotated.
- Rotation is a manual operational task that is easy to defer and hard to audit.
- The token grants publish rights to multiple crates under the same credential;
  there is no per-crate or per-workflow scope boundary.
- Nothing in the audit log distinguishes a legitimate publish from one made with
  a leaked copy of the same secret.

crates.io OIDC trusted publishing reached general availability in 2026
(see <https://crates.io/docs/trusted-publishing>). The mechanism mirrors the
GitHub Actions OIDC exchange already used by PyPI and npm. A workflow requests a
short-lived, repository-scoped OIDC token from the GitHub token endpoint, then
exchanges it for a crates.io publish token that expires after a single use. No
static secret is stored.

The R-F10 spike (commit `53e31629`, 2026-07-28) confirmed:

- `rust_code_changes_required: false` — this is a CI/CD-only change.
- The four `cargo publish` steps and the `crates.io` environment are the only
  affected locations.
- A fallback path exists: the existing `CARGO_REGISTRY_TOKEN` secret can be
  retained as a conditional fallback, making rollback a one-line revert.
- Estimated scope: one PR, approximately 40 lines of YAML.

ADR-072 governs release authority — who may trigger a release and what evidence
is required before the tag is pushed. This ADR amends only the token mechanism
used by the publish workflow; it does not alter the release authority chain,
the `release-guard` skill, or the `release-manager.sh` tooling.

## Decision

Replace the static `CARGO_REGISTRY_TOKEN` secret with a GitHub Actions OIDC
exchange in `publish-crates.yml`.

### 1. OIDC token exchange replaces the static secret

Each `cargo publish` job acquires a short-lived crates.io publish token by:

1. Adding `id-token: write` to the workflow-level `permissions` block.
2. Running a token-exchange step before `cargo publish` that calls the GitHub
   OIDC endpoint and writes the resulting token to `CARGO_REGISTRY_TOKEN` in
   the step environment.

The exchanged token is scoped to the specific crate, expires after a single use,
and is cryptographically bound to the repository, workflow, and ref that
requested it via the OIDC subject claim.

### 2. Static secret retained as explicit fallback

`CARGO_REGISTRY_TOKEN` is kept as a repository secret. It is injected as an
environment variable **only** when the OIDC exchange step is skipped or
unavailable, guarded by:

```yaml
env:
  CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

on a step conditioned on `if: env.CARGO_REGISTRY_TOKEN != ''`. This means:

- Under normal operation the secret variable is empty (not passed to the step)
  and the OIDC-derived token is used.
- To roll back, a single change re-enables the fallback `env:` block without
  removing the OIDC steps; or the OIDC steps can be commented out and the
  fallback promoted, which is a one-line revert per job.

### 3. No other publish mechanics are changed

The crate publish order (`core` → `redb` → `turso` → `mcp`), the `needs` chain,
the version-existence check, the `--locked` flag, the semver-checks step, and the
`crates.io` environment protection all remain unchanged. The `release-guard`
pre-flight is token-agnostic and requires no modification.

### 4. crates.io trusted publisher registration is a prerequisite

Before the workflow change is merged, each crate (`do-memory-core`,
`do-memory-storage-redb`, `do-memory-storage-turso`, `do-memory-mcp`) must be
registered as a trusted publisher on crates.io under the
`d-o-hub/rust-self-learning-memory` repository with the `publish-crates.yml`
workflow and the `crates.io` environment name. This is a one-time manual step
performed by a crate owner in the crates.io dashboard.

## Consequences

### Positive

- Static token rotations are eliminated; the exchanged token is single-use and
  expires automatically.
- Every publish is bound to a specific repository, workflow file, ref, and
  environment via the OIDC subject claim (`repo:d-o-hub/rust-self-learning-memory:environment:crates.io`),
  creating an auditable, unforgeable publish trail.
- A leaked copy of the environment secret can no longer be used to publish
  outside the registered workflow and environment context.
- No Rust code changes are required; the change is isolated to CI/CD YAML.

### Negative and trade-offs

- The crates.io trusted publisher registration must be completed by a crate
  owner before the workflow lands; the YAML change alone is not sufficient to
  activate OIDC publishing.
- If crates.io OIDC is unavailable at publish time the fallback path requires
  the `CARGO_REGISTRY_TOKEN` secret to be valid and non-empty. The secret must
  be kept current even though it is no longer the primary path.
- The `id-token: write` permission on the workflow grants broader OIDC token
  minting capability to all jobs in the workflow; jobs that do not publish
  crates run with this elevated permission unless the permission is scoped to
  individual jobs.

### Mitigations

- Scope `id-token: write` to the individual publish jobs rather than the
  workflow level to limit the blast radius of the elevated permission.
- Document the trusted publisher registration step in the release runbook so it
  is not forgotten when new crates are added to the workspace.

## Alternatives considered

1. **Keep the static token indefinitely**: rejected because the supply-chain risk
   and rotation burden are ongoing with no corresponding benefit once OIDC is
   available.
2. **Rotate the static token on a schedule via Dependabot or a cron job**:
   rejected because automation still requires the secret to exist as a
   long-lived credential; it reduces exposure window but does not eliminate the
   static-secret risk category.
3. **Remove the fallback entirely**: deferred until OIDC trusted publishing has
   been validated through at least one full release cycle; the fallback path
   preserves manual publish capability during the transition period.
4. **Use a fine-grained GitHub personal access token instead of OIDC**: rejected
   because fine-grained PATs are still long-lived secrets that require manual
   rotation and provide no improvement over the current state.

## Acceptance criteria

- `publish-crates.yml` no longer passes `secrets.CARGO_REGISTRY_TOKEN` as the
  primary `env:` value to any `cargo publish` step.
- Each of the four publish jobs acquires a crates.io token via the GitHub OIDC
  exchange before calling `cargo publish`.
- The fallback `env:` block is present and conditioned so it activates only when
  `CARGO_REGISTRY_TOKEN` is non-empty in the environment.
- All four crates are registered as trusted publishers on crates.io under the
  `d-o-hub/rust-self-learning-memory` repository, `publish-crates.yml`
  workflow, and `crates.io` environment before the PR is merged.
- A dry-run (`workflow_dispatch` with `dry-run: true`) completes without error
  after the workflow change is deployed, confirming the OIDC exchange succeeds.
- The existing `needs` chain, publish order, version-existence guard, and
  `--locked` flag are preserved unchanged.
- ADR-072 release authority and the `release-guard` skill are unaffected.

## References

- `plans/STATUS/spikes/R-F10.json`
- `.github/workflows/publish-crates.yml`
- <https://crates.io/docs/trusted-publishing>
- ADR-072: Authority, Evidence, and Enforcement Governance
