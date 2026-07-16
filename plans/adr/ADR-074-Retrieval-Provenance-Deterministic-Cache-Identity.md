# ADR-074: Retrieval Provenance and Deterministic Cache Identity

- **Status**: Accepted (partial implementation 2026-07-16 — S1.2 TaskContext identity in CacheKey; mode/provider/index generation and provenance envelope deferred)
- **Date**: 2026-07-14
- **Partial implement**: `feat/goap-missing-tasks-swarm-2026-07-16`
- **Deciders**: Project maintainers
- **Related**: ADR-024, ADR-072; `memory-core/src/memory/retrieval/context.rs`; `memory-core/src/retrieval/cache/`
- **Plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md` actions S1.2 and F4.1

## Context

`SelfLearningMemory::retrieve_relevant_context` currently builds a query-cache key from task description, domain, and result limit. Retrieval and ranking also depend on language, framework, tags, complexity, retrieval mode, ranking configuration, semantic provider/model, and index contents.

Two requests can therefore share a cache entry even though they should produce different rankings. Provider/model or index changes can also reuse an entry created under an older retrieval state. The cached response has no structured provenance explaining which tier, model, index generation, score components, or cache state produced it.

This is both a correctness defect and an observability gap. Correct identity and explainability should be designed together so the cache and diagnostics cannot drift into separate notions of “same request.”

## Decision

### 1. Introduce a versioned typed request identity

Create `RetrievalRequestIdentityV1` containing every input that can alter membership, ordering, or scoring:

- normalized query text;
- domain, language, framework, complexity, task type, and normalized tags;
- time/scope filters and result limit;
- retrieval mode and enabled tier/feature set;
- ranking/scoring configuration version and relevant weights/thresholds;
- embedding provider, model, dimensions, and provider configuration revision;
- index/schema generation or snapshot identity; and
- tenant/namespace identity if multi-tenancy is introduced.

The cache uses the full typed value as an equality key, not a digest alone. New behavior that affects results must add a field or increment the identity version.

### 2. Canonical normalization

- Tags are trimmed, normalized according to the existing tag contract, sorted, and deduplicated.
- Optional empty values normalize consistently to `None` where semantically equivalent.
- Query normalization is intentionally conservative: normalize line endings and agreed surrounding whitespace, but do not case-fold or rewrite semantic content without retrieval-quality evidence.
- Floating weights/thresholds are represented by stable configuration revision or canonical fixed/bit representation, not locale-formatted strings.

Canonical identity bytes use a versioned deterministic representation consistent with project serialization policy (postcard). A standard cryptographic digest of those bytes may be emitted as an opaque diagnostic fingerprint, but cache correctness relies on full typed equality.

### 3. Track index and provider generation

Relevant data mutations advance an index/cache generation or explicitly invalidate affected entries. Provider/model/config changes produce a new provider identity. Cache entries cannot survive an identity generation they do not declare.

Broad invalidation is acceptable initially; domain/namespace-scoped generations may be added after measurement.

### 4. Attach structured retrieval provenance

Internal retrieval results carry a `RetrievalProvenance` envelope with:

- request identity version and opaque fingerprint;
- cache hit/miss and entry generation;
- selected retrieval tier/backend and fallback sequence;
- provider/model and index generation;
- candidate counts before/after filtering;
- score component names/versions and final score;
- truncation/limit decisions; and
- bounded stage/total latency.

Public CLI/MCP diagnostics expose a redacted subset. Raw query/context and sensitive identifiers are not logged by default. Provenance is additive and can initially be optional to preserve API compatibility.

### 5. Test and benchmark contract

Tests cover:

- same text/domain/limit with Rust/Axum versus Python/Django contexts, in both request orders;
- tag order and duplicate-tag normalization;
- changes in complexity, framework, retrieval mode, ranking config, provider/model, and index generation;
- invalidation after episode/embedding mutation;
- stable canonical serialization/fingerprint fixtures; and
- redaction/no raw-query logging.

Benchmarks measure identity construction, cache lookup, provenance collection, memory overhead, and P95 retrieval latency. The initial promotion budget is less than 2% P95 overhead for provenance-enabled retrieval, unless maintainers accept a measured tradeoff.

## Consequences

### Positive

- Cache hits cannot cross result-affecting contexts.
- Provider/index changes do not silently serve stale rankings.
- Explainability and debugging use the same identity contract as caching.
- Versioning makes future ranking changes explicit.

### Negative

- Keys and cache entries become larger.
- Mutation paths need generation/invalidation wiring.
- Provider and ranking configurations need stable identities.
- Provenance can increase latency and telemetry volume if not bounded.

### Neutral

- Exact hashing/digest crate selection is an implementation detail, provided the diagnostic digest is standard, deterministic, and tested.
- Broad invalidation may reduce hit rate until scoped generations are justified.

## Alternatives considered

1. **Add only language/framework/tags to the existing key**: rejected because mode, configuration, provider, and index changes would remain unsafe.
2. **Disable query caching**: correct but discards a valuable performance feature and does not solve provenance.
3. **Use only a hash digest as the cache key**: rejected because collision handling and debugging are weaker than full typed equality.
4. **Shorten TTL**: rejected; shorter incorrectness is still incorrectness and provider/index drift can occur immediately.
5. **Log ad hoc score details without identity**: rejected because observability could not prove which cache/request state produced a result.

## Migration

1. Add the typed identity and parity tests without changing public output.
2. Replace current key construction and add conservative invalidation/generation.
3. Add internal provenance and debug/metrics hooks.
4. Expose an opt-in redacted CLI/MCP explanation.
5. Benchmark overhead and cache hit-rate changes; optimize/scoped-invalidate only from evidence.
6. Version the identity on any later result-affecting contract change.

## Acceptance criteria

- Every result-affecting current input is represented by identity or generation.
- Context-distinct queries cannot share a cache entry.
- Provider/model/index/ranking changes cannot reuse stale entries.
- Canonical tag normalization and serialization fixtures are deterministic.
- Provenance explains tier, model/index generation, candidate filtering, scoring, cache status, and latency without leaking raw sensitive context.
- Targeted tests and benchmarks pass within the accepted overhead budget.
