# ADR-060: `handle_shutdown` Deprecation Cleanup (PR #708 fallout)

- **Status**: 🟡 Proposed
- **Date**: 2026-07-02
- **Deciders**: Project maintainers
- **Related**: PR #708 (Strict JSON-RPC Content-Length validation),
  Issue #697, ADR-024 (MCP Lazy Tool Loading),
  ADR-038 (Local CI Parity — Clippy/Tests),
  ADR-058 (CI Health — Gitleaks/Release Drift),
  `main` commit `d85d9d10` (the same `#[allow(deprecated)]` suppressions)

## Context

PR #708 (Strict JSON-RPC Content-Length validation) was merged into CI on a
feature branch and failed `"Quick PR Check (Format + Clippy)"` with three
identical errors:

```
error: use of deprecated function `do_memory_mcp::protocol::handle_shutdown`
  --> memory-mcp/src/bin/server_impl/core.rs:17:5
  --> memory-mcp/src/bin/server_impl/jsonrpc.rs:5:5
  --> memory-mcp/src/bin/server_impl/jsonrpc.rs:375:13
```

The root cause is structural, not cosmetic:

1. `memory-mcp/src/protocol/handlers.rs:159` declares
   `#[deprecated] pub async fn handle_shutdown(...)` — **no replacement message**
   and **no new non-deprecated variant**.
2. In contrast, the sibling deprecated function
   `handle_list_tools` (line ~100) carries a concrete migration target:
   `#[deprecated = "Use handle_list_tools_with_lazy instead"]` and a real
   replacement (`handle_list_tools_with_lazy`, `handle_describe_tool`,
   `handle_describe_tools`) is in place per ADR-024.
3. PR #708 innocently removed a few `#[allow(deprecated)]` annotations because
   they were co-located with edits to the same files. Once removed, the
   `-D warnings` clippy step in CI fails on the deprecation lint in **all
   three** import/call sites, blocking merge even though the PR's substantive
   Content-Length work is correct.

Two precedents exist for remediation:

- An earlier commit on `main` (`d85d9d10`, "fix(ci): …, fix deprecated
  handle_shutdown") had **already added** the same three site-level
  `#[allow(deprecated)]` annotations in the same files PR #708 later edits.
  PR #708 removed those suppressions while editing the surrounding code,
  re-breaking `Quick PR Check (Format + Clippy)` with `-D warnings`. The
  suppression PR is shipping the same fix forward; it does **not** address
  the underlying deprecation drift.
- The `benches/*.rs` and `memory-cli/...` crates already use `#![allow(deprecated)]`
  at crate level, treating the deprecation as a tolerated background lint.

### Why this matters

- **Clippy honesty.** `#[allow(deprecated)]` is a lint suppression, not a
  fix; it hides the fact that the public API surface advertises a deprecation
  with no migration path. Downstream consumers (the binary crate, future
  embedders, IDE integrations) cannot tell why `handle_shutdown` is deprecated
  or what they should do instead.
- **CI noise.** The same suppression will keep being re-applied or deleted
  accidentally by future PRs touching `server_impl`, re-breaking PR CI
  (cf. ADR-057's PR #616 clippy-in-tests pattern and ADR-058's hygiene debt).
- **API contract.** `shutdown` is a JSON-RPC method every MCP client sends;
  the public handler must remain usable. Marking it `#[deprecated]` without
  a replacement risks future removals that break clients.

## Decision

Pick **one** of the following four remediation paths (Option 3 is omitted — see Option 4), in order of preference:

### Option 0 (cheapest structural fix): crate-level `#![allow(deprecated)]`

Collapse the three site-level `#[allow(deprecated)]` annotations added in
this PR / `d85d9d10` into a single crate-level attribute on both
`memory-mcp/src/bin/server_impl/core.rs` and
`memory-mcp/src/bin/server_impl/jsonrpc.rs` (e.g.
`#![allow(deprecated)]` at the top of each module, matching the benches/*
pattern). This does **not** address the underlying API contract drift, but
it pushes suppression responsibility to the file boundary so future PRs
editing one call site cannot accidentally re-introduce the original PR-708
clippy failure (cf. ADR-057's PR-616 clippy-in-tests pattern).

### Option 1 (preferred structural fix): remove the `#[deprecated]` attribute

The `shutdown` JSON-RPC method is part of the MCP core spec required by
clients; the handler is small, side-effect-free, and has no plan for
replacement. Remove `#[deprecated]` from `handle_shutdown` in
`memory-mcp/src/protocol/handlers.rs` and update the module-level doc in
`memory-mcp/src/protocol/mod.rs` to drop the "from library (deprecated)"
annotation in `memory-mcp/src/bin/server_impl/core.rs`. Also remove the three
`#[allow(deprecated)]` annotations added by this PR and `d85d9d10`.

```rust
// memory-mcp/src/protocol/handlers.rs
- /// Handle shutdown request
- #[deprecated]
- pub async fn handle_shutdown(request: JsonRpcRequest) -> Option<JsonRpcResponse> { ... }
+ /// Handle shutdown request
+ pub async fn handle_shutdown(request: JsonRpcRequest) -> Option<JsonRpcResponse> { ... }
```

### Option 2 (acceptable): ship a non-deprecated variant

If the team wants to keep the deprecation marker as a soft signal, introduce
`handle_shutdown_typed` (or rename + retain `handle_shutdown` as the deprecated
alias) so that downstream callers have an obvious alternative. Migrate the
three call sites to the new name and remove the `#[allow(deprecated)]`
annotations. This is more code but preserves the "this is being phased out"
signal for external consumers.

### Option 3 (omitted)

There is no Option 3 — it is intentionally skipped in numbering to avoid
silent renumbering of downstream references.

### Option 4 (status quo, rejected)

Keep site-level `#[allow(deprecated)]` suppressions long-term and accept CI
noise when future PRs touch the surrounding files. **Rejected** because it
treats the symptom, leaves an unexplained API contract, and re-imposes the
PR-708 / PR-616 class of failures on every future bin/server_impl edit.

### Selection guidance

- **Choose Option 0** if the deprecation is accepted permanently and you only
  want to stop re-breaking CI churn. Strictly less code than Options 1/2.
- **Choose Option 1** if `handle_shutdown` is intended to stay on the public
  surface (the most likely case — the MCP `shutdown` method is required by
  clients).
- **Choose Option 2** if a future migration is planned and the team wants a
  soft deprecation signal until then.

## Consequences

**Positive — CI stability** (Options 0, 1, 2):

- Removes the latent CI failure mode for any PR touching `server_impl`.
- Eliminates a class of "innocent suppression removal" failures (cf. PR #708,
  PR #616 in ADR-057).

**Positive — API honesty** (Options 1 and 2 only):

- Makes the public API honest about the lifecycle of `handle_shutdown`.
- Tests added in this PR (the end-to-end `shutdown` framing suite) bind the
  handler contract going forward, so removing the `#[deprecated]` marker is
  safe.

**Negative / risks**

- Option 0 leaves the deprecation marker unexplained; downstream consumers
  cannot tell why the handler is deprecated or what to do instead. Treat it
  as a stop-gap, not a final state.
- Option 1 removes a deprecation signal — if the team later wants to migrate
  to a new handler, they will need to re-introduce a non-`#[deprecated]`
  variant anyway (effectively Option 2). Mitigation: prefer Option 1 only if
  the handler is truly stable.
- Option 2 adds a small public-API surface increase (~15 lines); package it in
  one commit with the migration.
- Any external consumer that was filtering on the deprecation message will
  need updating. No such consumer identified in this repo (the binary crate is
  the only caller in-tree).

**Follow-up actions** (atomic commits)

1. `fix(mcp): suppress deprecated handle_shutdown lint in server_impl (#708)`
   — keeps the PR-708 CI gate green. **Shipped as part of this PR.**
2. `test(mcp): add end-to-end shutdown json-rpc integration test`
   — locks in the Content-Length + shutdown dispatch contract. **Shipped as
   part of this PR.**
3. `docs(plans): add ADR-060 for handle_shutdown deprecation cleanup`
   — tracks the structural fix. **This PRD.**
4. (Future) `refactor(mcp): remove #[deprecated] from handle_shutdown` (Option
   1) **or** `feat(mcp): add handle_shutdown_typed and migrate callers`
   (Option 2) to actually address the drift.

## ADR style / hygiene notes

- Naming follows the existing ADR-NNN sequence; next number after ADR-059
  (LOC Boundary Splits) is 060.
- File path mirrors siblings: `plans/adr/ADR-060-handle-shutdown-deprecation-cleanup.md`.
- Status is **Proposed** until the chosen option is implemented and merged
  on `main`.
