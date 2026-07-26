# ADR-077: Runtime Embedding Provider Activation and Capability-Truthful MCP Configuration

- **Status**: Accepted / Implemented
- **Date**: 2026-07-26
- **Deciders**: Project maintainers
- **Plan**: [`../GOAP_RUNTIME_EMBEDDING_ACTIVATION_2026-07-26.md`](../GOAP_RUNTIME_EMBEDDING_ACTIVATION_2026-07-26.md)
- **Related**: ADR-024 (lazy MCP tools), ADR-056 (local storage), ADR-072 (authority and evidence), ADR-074 (provider cache identity)
- **Code evidence**: `memory-mcp/src/mcp/tools/embeddings/tool/execute/configure.rs`, `memory-core/src/embeddings/semantic_service.rs`, `memory-core/src/memory/core/struct_priv.rs`, `memory-core/src/embeddings/config/provider_config.rs`

## Context

`configure_embeddings` accepts a provider and returns `success: true`, but it only
builds an `EmbeddingConfig`. The implementation explicitly describes its result as
a simulated configuration response and does not install a `SemanticService` in
`SelfLearningMemory`. The MCP handler returns that response unchanged.

The gap is observable through the rest of the tool surface:

- `SelfLearningMemory` starts with `semantic_service: None` and exposes only a
  read accessor.
- `embedding_provider_status`, `generate_embedding`, and `search_by_embedding`
  read that service and consequently remain unconfigured after a successful
  `configure_embeddings` result.
- `SemanticService::with_fallback` selects Local, then OpenAI, then mock rather
  than honoring the requested `ProviderConfig`; it can therefore report the
  requested model and dimensions while using a different provider.
- Mistral has an `EmbeddingProvider` implementation behind its feature flag,
  but the semantic-service factory does not construct it. Azure and Custom have
  configuration types but no runtime provider adapter. Cohere appears in the
  MCP schema but is immediately rejected and has no `ProviderConfig` variant.

This is a product-contract defect, not a request to add another provider in
isolation. A configuration command must either atomically activate exactly the
requested capability or fail without changing the active service.

## Decision

### 1. `configure_embeddings` is an activation operation

The command returns success only after all of the following complete:

1. parse and validate provider-specific input;
2. resolve the named credential from the supplied environment-variable name;
3. construct the requested provider supported by the current build;
4. run a bounded health probe and validate the returned vector dimension; and
5. atomically install a usable `SemanticService` runtime snapshot.

Failure leaves the previous active service unchanged. The response identifies the
active provider, model, dimension, health, and configuration revision; it never
includes credentials or raw provider response bodies.

### 2. Requested provider identity is authoritative

There is no cross-provider or mock fallback for an explicit MCP activation.
Fallback is acceptable only for an explicit library startup policy that names the
fallback chain and exposes its degraded health. An MCP caller that requests
OpenAI, Mistral, or Local receives that provider or a typed unavailable/error
result. A degraded mock provider is never reported as configured and available.

### 3. Capability registry drives the MCP contract

The public schema and validation use one provider-capability registry. A provider
is selectable only when it has all of: a `ProviderConfig` representation, a
runtime `EmbeddingProvider` adapter, feature-gated construction, credential
requirements, known model/dimension validation, and test coverage.

The initial implementation target is Local, OpenAI, and Mistral where their
build features and real-model requirements are satisfied. Azure, Custom, and
Cohere are removed from the selectable MCP schema until a separate ADR supplies
their adapters and credential/endpoint contracts. Feature-disabled providers may
remain named only if the operation returns a typed `provider_unavailable` error,
never a successful configuration.

### 4. Runtime ownership must be concurrent-safe

`SelfLearningMemory` owns an interior-mutability runtime slot that can atomically
replace and return an owned `Arc<SemanticService>` snapshot. Readers obtain a
snapshot before awaiting provider or storage work; no lock is held across
`.await`. The concrete synchronization primitive is an implementation detail,
provided it preserves those semantics and does not block Tokio worker threads.

### 5. Provider changes are retrieval-state changes

Activation assigns a monotonically advancing configuration revision. It updates
the provider identity used by ADR-074 and invalidates incompatible embedding
query state before it can be reused. Embeddings from different provider/model/
dimension identities cannot be compared or served from one undifferentiated
index. Automatic full re-embedding is out of scope; the operation returns an
explicit reindex-needed state when retained embeddings are incompatible.

### 6. Credentials are runtime-only

Only an environment-variable *name* may be accepted by MCP. API keys are read at
activation time, are not stored in `EmbeddingConfig`, output, audit logs, or plan
artifacts, and are not persisted for restart recovery. Durable profile management
is a separate feature with its own security decision.

## Consequences

### Positive

- Every successful MCP configuration immediately enables the embedding tools.
- Status, generation, and semantic retrieval describe the provider actually in
  use, including health and dimensions.
- Provider changes cannot silently mix incompatible vectors or cache entries.
- Unsupported providers are honest API errors rather than schema promises.

### Negative and trade-offs

- The core needs a new runtime ownership seam and an embedding-storage adapter
  that can compose with the existing `StorageBackend` trait objects.
- Reconfiguration can invalidate existing semantic results and require an
  operator-driven reindex.
- Builds without cloud-provider features expose fewer usable capabilities.

## Alternatives considered

1. **Keep configuration declarative only**: rejected because the current tool
   claims activation and users cannot use the configured provider afterward.
2. **Keep the fallback factory for MCP requests**: rejected because reported and
   actual model dimensions can diverge.
3. **Add Cohere first**: rejected because it extends an already false contract
   without fixing runtime activation, storage, or provider identity.
4. **Mutate the service through a synchronous lock**: rejected because async
   provider calls must not retain a blocking or async lock across `.await`.
5. **Treat all embeddings as interchangeable after configuration**: rejected by
   ADR-074; model and dimension are result-affecting retrieval identity.

## Acceptance criteria

- A successful configuration changes `embedding_provider_status` from
  unconfigured to the requested real provider in the same process.
- `generate_embedding` and `query_semantic_memory` use the activated provider,
  model, dimensions, and configuration revision.
- Failed activation preserves the prior usable provider and its revision.
- Requested, configured, and observed provider/model/dimension agree in unit,
  integration, and MCP protocol tests.
- No mock/degraded provider yields `available: true` for a production request.
- Provider/model/dimension changes cannot reuse embeddings, indexes, or cached
  results that lack a matching identity.
- Azure, Custom, and Cohere are absent from the selectable schema until their
  complete runtime adapters and tests exist.

## References

- `plans/GOAP_RUNTIME_EMBEDDING_ACTIVATION_2026-07-26.md`
- `memory-mcp/tests/embeddings_integration.rs`
- `memory-core/tests/embedding_integration_test.rs`
