# GOAP: Runtime Embedding Provider Activation

- **Status**: Proposed implementation plan; no production code changed
- **Date**: 2026-07-26
- **Audit checkout**: `main` at `7c854efb`
- **Decision**: [ADR-077](adr/ADR-077-Runtime-Embedding-Provider-Activation.md)
- **Relevant constraints**: ADR-024, ADR-056, ADR-072, ADR-074, Tokio-only async, postcard, zero clippy warnings
- **Strategy**: Sequential contract and core seam, then parallel provider/test work, followed by converging MCP and end-to-end validation

## Analysis

### Verified implementation gap

| Surface | Current behavior | Evidence |
|---|---|---|
| MCP configuration | Returns success after building config only; does not activate a service | `memory-mcp/src/mcp/tools/embeddings/tool/execute/configure.rs` |
| Memory runtime | Starts with `semantic_service: None`; public API has no installation path | `memory-core/src/memory/core/struct_priv.rs`, `memory-core/src/memory/mod.rs` |
| Tool behavior | Status/generate/search read the missing service and remain unconfigured | `memory-mcp/src/mcp/tools/embeddings/tool/execute/status.rs`, `generate.rs`, `query.rs` |
| Provider factory | Local/OpenAI/mock fallback can differ from requested config; Mistral is not selected | `memory-core/src/embeddings/semantic_service.rs` |
| Provider contract | Cohere is in the MCP enum but rejected; Azure/Custom have config only, not adapters | `memory-mcp/src/mcp/tools/embeddings/tool/definitions.rs`, `memory-core/src/embeddings/config/provider_config.rs` |

### Primary goal

Make MCP embedding configuration truthful and usable: a success response must
activate exactly the requested real provider for subsequent status, generation,
and semantic-query calls in that server process. A failed request must preserve
the prior activation unchanged.

### Non-goals

- Add Cohere, Azure, or Custom provider transports.
- Persist API keys or configuration profiles.
- Automatically re-embed historic data after a model change.
- Change the intentional fail-closed agent-code-execution or deferred batch-tool decisions.
- Claim validation or implementation completion in the canonical status files.

## GOAP state

| Fact | Desired state | Preconditions |
|---|---|---|
| `configure_embeddings` is a simulation | It activates or fails atomically | ADR-077 accepted |
| Provider schema overstates capability | Registry describes actual runtime capability | Feature and adapter inventory complete |
| `SemanticService` is immutable after construction | Readers can obtain an owned live snapshot safely | No lock held across `.await` |
| Provider changes have no activation revision | Revision participates in ADR-074 identity/invalidation | Storage/index policy agreed |

## Action graph

```text
REA-2026-07-26-A1 contract baseline
          |
          v
REA-2026-07-26-A2 core runtime and storage seam
          |
          +------------------------------+
          v                              v
REA-2026-07-26-A3 provider factory   REA-2026-07-26-A4 identity/invalidation
          +--------------+---------------+
                         v
              REA-2026-07-26-A5 MCP activation contract
                         |
                         v
              REA-2026-07-26-A6 integration and gates
```

## Work packages

### REA-2026-07-26-A1: Freeze the observable contract

**Owner surfaces**: `memory-mcp` embedding tool types, definitions, and integration tests.

1. Add regression tests that demonstrate the current false-success sequence:
   configure, then status/generate/query in the same process.
2. Define typed activation outcomes for invalid input, unavailable feature,
   missing credential, failed probe, unhealthy/degraded model, and storage
   setup failure. Keep JSON errors machine-stable.
3. Replace the hard-coded provider enum with a capability-registry projection.
   The initial selectable set is Local, OpenAI, and Mistral only when their
   adapter and build capability are present.

**Exit**: Tests specify successful activation, failure preservation, and no
success response for unsupported or feature-disabled providers.

### REA-2026-07-26-A2: Add the core runtime and storage seam

**Owner surfaces**: `memory-core/src/memory/`, `memory-core/src/embeddings/`.

1. Add an activation API on `SelfLearningMemory` that replaces the semantic
   runtime with an owned immutable service snapshot after construction and probe.
2. Preserve asynchronous safety: callers clone the active `Arc` before any
   provider/storage await; activation never exposes a half-built service.
3. Design a composition-safe `EmbeddingStorageBackend` adapter for the current
   Turso/redb/in-memory construction paths. Do not downcast
   `Arc<dyn StorageBackend>` opportunistically. The adapter must make its
   durable/cache fallback behavior explicit and preserve ADR-056 local-mode
   constraints.
4. Store an activation revision and provider identity with the live runtime;
   return it from the status API without exposing credential material.

**Exit**: A core test can install a deterministic test provider, read a stable
snapshot concurrently, and prove failed replacement retains the former service.

### REA-2026-07-26-A3: Build an exact-provider factory

**Owner surfaces**: `memory-core/src/embeddings/semantic_service.rs`, provider modules, feature manifests.

1. Replace the explicit-MCP fallback path with a factory that matches every
   supported `ProviderConfig` exactly.
2. Wire existing OpenAI and Mistral clients only behind their declared features
   and standard credential environments; retain local activation only when
   `EmbeddingHealth::Real` is established.
3. Validate configured versus probed dimensions before installation. Reject
   unknown models or dimensions rather than silently substituting defaults.
4. Remove Azure, Custom, and Cohere from selectable MCP input until their full
   adapters are independently designed and tested.

**Parallelism**: This package can proceed with A4 after A2's activation API is
fixed. Provider HTTP contract tests must mock transport and never call a live API.

**Exit**: Factory tests cover correct provider selection, disabled features,
missing environment variables, probe failure, vector-dimension mismatch, and
degraded-local rejection.

### REA-2026-07-26-A4: Maintain provider and vector identity

**Owner surfaces**: embedding storage/index wiring, retrieval cache identity, HNSW persistence.

1. Treat `provider:model:dimension:activation_revision` as an embedding
   namespace/revision compatible with ADR-074's retrieval identity.
2. On activation change, invalidate or partition query/index state before a
   retrieval can mix old and new dimensions. Define whether an old namespace is
   retained, cleared, or marked reindex-required for each backend.
3. Add an explicit operator-visible `reindex_required` status. Re-embedding is
   a separate command and must not be triggered implicitly by configuration.
4. Test provider/model/dimension changes against query-cache and persisted HNSW
   load paths; no cross-identity vector comparison is permitted.

**Exit**: A model change cannot return a cache hit or ANN result generated under
the prior identity, and status gives an actionable reindex state.

### REA-2026-07-26-A5: Wire the MCP operation end to end

**Owner surfaces**: `memory-mcp/src/mcp/tools/embeddings/`, server handlers, tool registry.

1. Parse and validate input, resolve only the named environment variable, build
   and probe the exact provider, then call the core activation API.
2. Report the installed provider, model, actual dimension, health, revision, and
   reindex status. Redact secrets from all errors and audit fields.
3. Make status, test, generation, search-by-vector, and semantic-query retrieve
   the current runtime snapshot. Do not emulate semantic scores when an activated
   provider or its storage query fails; return a typed error or clearly labelled
   non-semantic fallback according to the established tool contract.
4. Update lazy tool definitions and protocol/snapshot tests together so the
   exposed enum, error shape, and runtime behavior cannot drift.

**Exit**: The complete same-process MCP sequence succeeds with a test provider;
unsupported inputs fail before state changes; replacing a provider changes the
observed revision.

### REA-2026-07-26-A6: Validate, document, and release-gate

**Owner surfaces**: focused crate tests, API reference, CLI/MCP docs, release notes.

1. Add unit tests for registry/factory/runtime swaps and integration tests for
   Local, OpenAI, and Mistral feature permutations using mocked transports.
2. Add concurrency tests for reads during a replacement and storage tests for
   redb-only, Turso-local, dual-storage, and in-memory modes.
3. Add regression coverage for credential redaction, identity invalidation,
   feature-disabled providers, and failed activation preservation.
4. Update user documentation only after the tool contract is exercised; describe
   feature availability, required environment-variable names, no persistence,
   and reindex-required behavior.

**Exit**: All defined validation passes and docs describe only supported,
exercised capabilities.

## Quality gates and evidence

Run after implementation, recording commit, feature set, UTC timestamp, and
test artifact in the implementation PR as required by ADR-072:

```bash
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace
./scripts/build-rust.sh check
cargo nextest run -p do-memory-core
cargo nextest run -p do-memory-mcp
cargo nextest run --all
cargo test --doc
./scripts/quality-gates.sh
./scripts/validate-plans.sh --active-set --adrs --identifiers --links
```

Feature-specific checks must include `openai`, `mistral`, and `embeddings-full`
where those paths are compiled. HTTP tests use local/mock servers and do not
require external credentials or network access.

## Promotion gates

1. **Accept ADR-077** before implementation starts.
2. **A2 design review** must select the embedding-storage composition and
   snapshot mechanism without a lock-across-await path.
3. **A4 review** must prove identity invalidation/partitioning before enabling
   provider replacement in MCP.
4. **Release review** must verify that no selectable provider lacks an adapter,
   health test, dimension contract, and documented credential handling.

## Definition of done

- All A1-A6 exits are met.
- MCP configuration success is equivalent to immediate usable activation.
- Runtime behavior, tool schema, provider status, and documentation agree.
- Provider changes cannot cross embedding/cache/index identities.
- New ADR status is updated to `Accepted / Implemented` only with code and
  validation evidence; otherwise it remains Proposed or Accepted with explicit
  drift state.
