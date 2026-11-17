# ADR-084: Opt-In OpenTelemetry OTLP Tracing

- **Status**: Accepted (2026-11-17) — implemented in this change
- **Date**: 2026-11-17
- **Deciders**: Project maintainers
- **Plan**: [`../ROADMAPS/ROADMAP_ACTIVE.md`](../ROADMAPS/ROADMAP_ACTIVE.md) (R-F2)
- **Related**: R-F2, ADR-077 (runtime embedding provider activation — redaction
  discipline), ADR-043 (E5 OpenTelemetry opportunity), ADR-028 (#10 metrics)
- **Code evidence**: `memory-core/src/observability/otel.rs`,
  `memory-core/src/observability/mod.rs`, `memory-core/Cargo.toml`, `Cargo.toml`

## Context

R-F2 was gated in `plans/STATUS/spikes/R-F-product-backlog-2026-11-15.json`
because a production exporter needs three properties proven before scheduling:
a pinned OpenTelemetry version, an `otel` feature, and credential-redaction
tests. The workspace already ships `tracing` 0.1.44 and
`tracing-subscriber` 0.3.23, but only to the console/log sink — nothing leaves
the process.

Two deployment shapes matter:

- `do-memory-mcp` speaks JSON-RPC over stdio. Adding a listener to it is
  undesirable, so a pull-based Prometheus endpoint does not fit.
- The CLI and library consumers are ephemeral processes that should not pay for
  an exporter unless they explicitly configure one.

OpenTelemetry Rust 0.33.1 made OTLP metrics and logs stable, while traces are
still **beta**. That is acceptable for an opt-in integration only if the exact
version is pinned and the feature can be removed without touching call sites.

## Decision

1. **Push-based OTLP/HTTP exporter**, feature-gated as `otel` in
   `do-memory-core`. No HTTP listener is added to any binary.
2. **Inert by default**: `init_otel_from_env()` returns `Ok(None)` and builds no
   exporter when `OTEL_EXPORTER_OTLP_ENDPOINT` is unset or blank. No batch
   processor, no worker thread, no network connection.
3. **Environment configuration**:
   - `OTEL_EXPORTER_OTLP_ENDPOINT` — enables the pipeline
   - `OTEL_SERVICE_NAME` — resource service name (default `do-memory-core`)
   - `OTEL_TRACES_SAMPLER_ARG` — trace-id-ratio in `[0, 1]`, default `1.0`,
     clamped; non-finite or unparsable values fall back to the default
   A `from_env_with` accessor keeps the parser testable without mutating the
   process environment.
4. **Redaction**: `OtelConfig::redacted_endpoint()` strips URL userinfo,
   query, and fragment components for logs/diagnostics. Exporter headers stay
   in the exporter's own environment configuration and are never copied into
   span attributes.
5. **Lifecycle**: `OtelGuard` owns the `SdkTracerProvider`. Dropping the guard
   (or calling `shutdown()`) flushes and shuts the exporter down;
   `force_flush()` exposes an explicit flush for tests and shutdown paths.
6. **Test-only feature**: `otel-testing = ["otel", "opentelemetry_sdk/testing"]`
   supplies the in-memory exporter used by the span-export test, so a plain
   `cargo test` never builds the SDK testing surface.
7. **Pinned dependency set**: `opentelemetry`, `opentelemetry_sdk`,
   `opentelemetry-otlp` at `0.33.1`, `tracing-opentelemetry` at `0.34.0`
   (declared in the workspace root). The exporter reuses the workspace's
   `reqwest` 0.13 rather than adding a second HTTP stack.

## Alternatives considered

- **Pull-based Prometheus endpoint** — rejected: requires an HTTP server in the
  MCP process and active scraping; the product needs outbound traces, not a new
  attack surface.
- **Always-on exporter** — rejected: violates the zero-overhead-when-unconfigured
  threshold every binary would pay for.
- **Hand-rolled OTLP/protobuf client** — rejected: reimplements a beta wire
  format for no benefit.
- **`tracing` JSON logs only** — rejected as insufficient for cross-process
  trace correlation; it remains the default sink.

## Consequences

Positive:

- No behavior change, no new thread, and no dependency cost unless the feature
  is enabled and an endpoint is configured.
- Credential redaction is unit-tested; the export path is exercised against the
  SDK's in-memory exporter.
- Call sites use `tracing` spans, so adopting or removing the backend does not
  change instrumentation code.

Negative / risks:

- OTLP **traces** are upstream beta. Upgrading `opentelemetry*` requires a
  re-run of the same evidence and may be breaking; the dependency stays pinned
  to `0.33.x` until traces stabilise.
- The OTLP/HTTP exporter uses the SDK's blocking HTTP client on its own batch
  thread. That is acceptable for export, but callers must not depend on flush
  latency inside request hot paths.
- Enabling `otel` adds a `blocking` feature to the shared `reqwest` build for
  that binary only.

## Acceptance evidence (2026-11-17)

| Check | Command | Result |
|-------|---------|--------|
| Compile with the feature | `cargo check -p do-memory-core --features otel` | ✅ clean |
| Lints | `cargo clippy -p do-memory-core --all-targets --features otel-testing -- -D warnings` | ✅ clean |
| Zero-overhead + redaction + sampler | `cargo nextest run -p do-memory-core --features otel-testing -E 'test(observability)'` | ✅ 4 passed |
| Tokio compatibility | the same nextest run executes the async export test on `#[tokio::test]` | ✅ |
| Formatting | `cargo fmt --all -- --check` | ✅ clean |
