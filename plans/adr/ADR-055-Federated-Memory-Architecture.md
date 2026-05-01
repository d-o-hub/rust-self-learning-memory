# ADR-055: Federated Memory Architecture (Cross-Agent Trust + Gossip)

**Status**: Proposed (prototype-only, no merge to `main` until PoC validated)
**Date**: 2026-05-01
**Deciders**: Architecture WG, Security WG
**Related**: ADR-050 (AgentFS), ADR-051 (External Signal Provider), ADR-053 (Comprehensive Analysis v0.1.31), ADR-054 (Execution Signature Retrieval), WG-128/129/130/131 (CSM Integration)
**Target Version**: v0.2.0 (vision milestone, post-`v0.1.x` line)

---

## Context

Today the memory system is a single-agent, single-machine pipeline:

```diagram
╭───────────────╮     ╭────────────────╮     ╭──────────────╮
│ memory-cli /  │────▶│  memory-core   │────▶│ Turso (durable)│
│ memory-mcp    │     │ (retrieval +   │     │ + redb (cache) │
│  (one agent)  │     │  CSM cascade)  │     ╰──────────────╯
╰───────────────╯     ╰────────────────╯
```

The `csm` feature already re-exports `HVec10240`, `BundleAccumulator`,
`ConceptGraph`, and `TextEncoder` from the upstream `chaotic_semantic_memory =
"0.3.4"` crate (see [memory-core/src/retrieval/mod.rs](file:///home/do/rust-self-learning-memory/memory-core/src/retrieval/mod.rs#L17-L22)).
Conflict resolution today is "Turso wins / most-recent wins" only — see
[memory-core/src/sync/conflict.rs](file:///home/do/rust-self-learning-memory/memory-core/src/sync/conflict.rs#L1-L40).
There is no peer-to-peer transport, no signing key infrastructure, no vector
clocks, and `memory-cli` has no `mesh` command surface
([memory-cli/src/commands/mod.rs](file:///home/do/rust-self-learning-memory/memory-cli/src/commands/mod.rs)).

A federated architecture would let multiple agents (potentially distrusting)
share patterns, episodes, and HDC concept vectors over the network with
verifiable provenance and convergent state.

## Decision

Design and prototype — **but do not yet merge to `main`** — a federated memory
architecture in 4 layers, behind new feature flags, gated by an end-to-end
two-node PoC. Specification and wire protocols ship first; implementation is
minimal and additive.

### Scope

The four user-requested capabilities map onto our current crates as follows:

| # | Capability | Owner Crate | New Module / Symbol |
|---|------------|-------------|---------------------|
| 1 | Signed `HVec10240` for cross-agent trust | `do-memory-core` (`csm` feature) | `retrieval::signed::SignedHVec` |
| 2 | Gossip protocol for pattern propagation | `do-memory-mcp` (new `mesh` feature) | `mesh::gossip` |
| 3 | Vector-clock conflict resolution on `Episode` | `do-memory-core` (`sync` module) | `sync::vector_clock`, extends `ConflictResolution` |
| 4 | `memory mesh` CLI subscribe command | `do-memory-cli` | `commands::mesh` |

### Non-Goals (Explicit)

- No upstream fork of `chaotic_semantic_memory`. Signing wraps `HVec10240`;
  it does not modify the bit vector.
- No replacement of Turso or redb. Mesh is an *additional* propagation channel.
- No Byzantine fault tolerance in v1. Trust = signature verification only.
- No automatic merge of remote episodes into local Turso store in PoC. Remote
  memories land in a quarantined `peer_episodes` namespace until promoted.
- No production deployment guidance. The mesh feature stays `unstable` until a
  follow-up ADR.

## High-Impact Areas (Codebase Audit)

| Area | File / Module | Impact |
|------|---------------|--------|
| HDC re-exports | [memory-core/src/retrieval/mod.rs](file:///home/do/rust-self-learning-memory/memory-core/src/retrieval/mod.rs) | Add `signed` submodule under `csm` feature |
| Episode struct | [memory-core/src/episode/structs.rs#L129](file:///home/do/rust-self-learning-memory/memory-core/src/episode/structs.rs#L129) | Add optional `vector_clock: Option<VectorClock>` field, serde-default for back-compat |
| Conflict resolution | [memory-core/src/sync/conflict.rs](file:///home/do/rust-self-learning-memory/memory-core/src/sync/conflict.rs) | Add `VectorClockMerge` variant to `ConflictResolution` enum |
| Sync mod | [memory-core/src/sync/mod.rs](file:///home/do/rust-self-learning-memory/memory-core/src/sync/mod.rs) | Re-export new `vector_clock` types |
| MCP server | [memory-mcp/src/lib.rs](file:///home/do/rust-self-learning-memory/memory-mcp/src/lib.rs) | New `mesh/` subdir + `mesh` feature flag |
| CLI dispatch | [memory-cli/src/commands/mod.rs](file:///home/do/rust-self-learning-memory/memory-cli/src/commands/mod.rs) | Wire `mesh` subcommand |
| Workspace deps | [Cargo.toml](file:///home/do/rust-self-learning-memory/Cargo.toml) | New optional deps: `ed25519-dalek`, `quinn` or `libp2p`, `prost`/`postcard` (already postcard) |
| Storage schemas | `memory-storage-turso/migrations/` | New `peer_episodes`, `peer_keys`, `vector_clocks` tables (additive migration) |
| Docs/Skills | `agent_docs/`, `.agents/skills/` | New `federated-memory` skill + ops doc |

**Total surface**: ~15 new modules, ~3 schema migrations, ~6 modified files.
This is large enough to justify a dedicated ADR + GOAP and a feature-flagged
prototype branch (`feature/mesh-poc`).

## Wire Protocol & Spec (Design First)

### 1. `SignedHVec` — Cross-Agent Trust on HDC Vectors

```rust
// memory-core/src/retrieval/signed.rs (new, csm feature)
use chaotic_semantic_memory::HVec10240;
use ed25519_dalek::{Signature, VerifyingKey, SigningKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedHVec {
    pub vec: HVec10240,                // 10,240 bits = 1280 bytes
    pub agent_id: AgentId,             // 32-byte ed25519 pubkey hash
    pub origin_episode: Option<EpisodeId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub signature: [u8; 64],           // ed25519 over postcard(vec || agent_id || origin || created_at)
}

impl SignedHVec {
    pub fn sign(vec: HVec10240, key: &SigningKey, ...) -> Self { ... }
    pub fn verify(&self, key: &VerifyingKey) -> Result<(), VerifyError> { ... }
}
```

Signing target is the postcard-serialized canonical tuple
`(vec_bytes, agent_id, origin_episode, created_at)` — postcard already required
by AGENTS.md core invariants. Signatures are detached (not embedded in HVec
bits) so the bit vector remains usable by upstream `chaotic_semantic_memory`
APIs without modification.

### 2. Gossip Protocol — Pattern Propagation

Wire layer: **QUIC via `quinn`** (preferred) or libp2p-noise. Selected per
PoC benchmarking.

```text
MeshFrame := Header || Payload || Signature

Header := {
    version:      u16,         // 1
    msg_type:     u8,          // 0=Hello, 1=Heartbeat, 2=GossipPattern,
                               // 3=GossipEpisode, 4=Subscribe, 5=Ack
    sender:       AgentId,     // ed25519 pubkey hash
    msg_id:       u128,        // ULID
    timestamp:    i64,         // unix-millis (signed for clock skew tolerance)
    vector_clock: VectorClock, // map<AgentId, u64>
}

Payload :=
  | Hello       { pubkey: [u8;32], capabilities: u64, peers: Vec<PeerInfo> }
  | Heartbeat   { health: HealthSnapshot }
  | GossipPattern { pattern: SignedPattern, ttl: u8 }
  | GossipEpisode { episode: SignedEpisode, ttl: u8, hvec: Option<SignedHVec> }
  | Subscribe   { topics: Vec<TopicFilter> }
  | Ack         { msg_id: u128 }

Signature := ed25519(sender_signing_key, postcard(Header || Payload))
```

Serialization: **postcard** (per AGENTS.md core invariants).
Encryption: TLS 1.3 via `quinn` with ed25519-derived self-signed certs.
TTL: hop count, max 4 by default, prevents broadcast storms.
Topology: epidemic anti-entropy (Plumtree-style), 3 random peers per round,
1-second heartbeat, 5-second gossip tick.

### 3. Vector Clocks for `Episode`

```rust
// memory-core/src/sync/vector_clock.rs (new)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock(pub BTreeMap<AgentId, u64>);

#[derive(Debug, PartialEq, Eq)]
pub enum ClockOrdering { Before, After, Equal, Concurrent }

impl VectorClock {
    pub fn increment(&mut self, agent: AgentId) { ... }
    pub fn merge(&mut self, other: &VectorClock) { ... } // pointwise max
    pub fn compare(&self, other: &VectorClock) -> ClockOrdering { ... }
}
```

Episode change (in `episode/structs.rs`):

```rust
pub struct Episode {
    // ... existing fields ...
    #[serde(default)]
    pub vector_clock: Option<VectorClock>,  // None = legacy
}
```

Conflict-resolution change (additive variant):

```rust
pub enum ConflictResolution {
    TursoWins,
    RedbWins,
    MostRecent,
    VectorClockMerge,  // NEW: ClockOrdering decides; Concurrent → MostRecent fallback
}
```

This **does not break** any existing serialized data: `vector_clock` is
`Option<_>` with `serde(default)` and `MostRecent` remains the default
(`#[derive(Default)]` keeps `TursoWins`).

### 4. `memory mesh` CLI

```text
memory mesh <SUBCOMMAND>

SUBCOMMANDS:
  init           Generate ed25519 keypair, write ~/.do-memory/mesh.key
  peer add       <addr> <pubkey>            Trust a peer
  peer list                                 Show known peers + last-seen
  subscribe      <peer> [--topics=<filter>] Stream pattern/episode events
  publish        --pattern=<id>             Manually broadcast a pattern
  status                                    Vector-clock + gossip stats
```

The `subscribe` long-running command consumes peer streams and writes to the
quarantined `peer_episodes` Turso table. A separate `promote` command (future
ADR) would move accepted records into the canonical episode store.

## Two-Node Proof-of-Concept (Acceptance Criteria)

The PoC is the gate. We do not enable the feature flag in default builds
until **all** of the following pass on a single host with two `memory-mcp`
processes (different ports + key files):

1. `memory mesh init` produces deterministic ed25519 keypair files.
2. Node A's `memory mesh subscribe nodeB` receives a pattern published on
   Node B within ≤2 s (p50) over loopback.
3. A signed `HVec10240` round-trips and `verify()` succeeds on the receiver
   with the publisher's pubkey, fails when the bit vector is mutated.
4. Two concurrent edits to the same `Episode` on different nodes resolve
   deterministically: `VectorClockMerge` returns `ClockOrdering::Concurrent`
   and falls back to `MostRecent`; both nodes converge to the same state
   after one gossip round.
5. Tampered frames (`Signature` invalidated) are rejected and logged at
   `WARN`, with a counter `mesh.invalid_signatures_total` incremented.
6. CPU overhead of the gossip thread at idle ≤2% of one core, RSS delta
   ≤25 MB versus baseline `memory-mcp` startup.
7. Integration test `memory-mcp/tests/mesh_two_node.rs` is green under
   `cargo nextest run --features mesh`.
8. Quality gates (`./scripts/quality-gates.sh`) pass on the prototype branch
   with coverage ≥90% on new modules.

## Consequences

### Positive

- Foundation for cross-agent learning ("AgentNet") matching the v0.2.x
  vision in [ROADMAP_V030_VISION.md](file:///home/do/rust-self-learning-memory/plans/ROADMAPS/ROADMAP_V030_VISION.md).
- Vector clocks fix the existing "MostRecent" coarseness in
  [conflict.rs](file:///home/do/rust-self-learning-memory/memory-core/src/sync/conflict.rs)
  even before federation — usable for Turso↔redb dual-writer conflicts.
- Postcard + ed25519 dependencies are small (≈250 KB binary delta), pure
  Rust, no C deps.

### Negative / Risks

- **Crate version surface**: 4 publishable crates touched. We must ship
  mesh modules behind feature flags `mesh = []` (default off) to keep
  `0.1.x` SemVer.
- **Security review burden**: cryptographic code requires audit.
  Mitigation: use only well-vetted `ed25519-dalek` v2 + `rustls`.
  Add `cargo audit` gate enforced in CI for `mesh` feature.
- **Test flakiness**: Network/time-based tests historically caused issues
  (see ADR-025 BOCPD, ADR-030 Test Optimization). Mitigation: all PoC
  tests use `tokio::time::pause()` and in-memory loopback transports.
- **Storage schema drift**: New `peer_episodes`, `peer_keys`,
  `vector_clocks` tables. Mitigation: additive migration only, no rename
  or backfill of existing tables.
- **Performance budget**: 50ms episode-create budget (AGENTS.md) must hold
  with vector-clock increment in hot path. Increment is `O(peers)` ≪ 1µs.

### Decision Reversal Path

All new code lives behind `mesh` (memory-mcp/cli) and `mesh` (memory-core)
feature flags. Removal = deleting feature blocks; no schema rollback needed
(unused additive tables).

## Alternatives Considered

1. **CRDT-based merge (e.g. yjs/automerge)** — heavier deps, mismatched with
   tabular Episode shape. Vector clocks suffice for "last-writer-wins on
   concurrent" semantics.
2. **gRPC + tonic** — pulls in protoc, breaks postcard invariant. Rejected.
3. **Embed signature inside HVec bits** — corrupts the bit vector and breaks
   upstream `chaotic_semantic_memory` similarity math. Rejected.
4. **Reuse Turso replication directly** — couples agents to a shared DB,
   defeats the "distrusting agents" requirement. Rejected.

## Out-of-Scope (Future ADRs)

- ADR-056 (proposed): Trust delegation / web-of-trust for peer keys.
- ADR-057 (proposed): Promotion pipeline `peer_episodes → episodes` with
  reward-weighted gating.
- ADR-058 (proposed): Cross-agent reputation scoring.

---

## References

- [chaotic_semantic_memory v0.3.4 docs](https://docs.rs/chaotic_semantic_memory/0.3.4)
- Upstream tracking issue (additive enhancements, not blockers): [d-o-hub/chaotic_semantic_memory#156](https://github.com/d-o-hub/chaotic_semantic_memory/issues/156) — `Hash` derive on `HVec10240`, `WIRE_VERSION` constants, serde for `BundleAccumulator`/`ConceptGraph`, optional `signing` helper feature. Federation prototype proceeds with downstream wrappers; upstream changes simply reduce wrapper LOC and lock in cross-version canonical form.
- [Plumtree gossip](https://www.dpss.inesc-id.pt/~ler/reports/srds07.pdf)
- [Vector clocks (Lamport, 1978)](https://lamport.azurewebsites.net/pubs/time-clocks.pdf)
- [ed25519-dalek v2](https://docs.rs/ed25519-dalek/2)
- [QUIC via `quinn`](https://docs.rs/quinn/latest/quinn/)
- Companion plan: [GOAP_FEDERATED_MEMORY_2026-05-01.md](../GOAP_FEDERATED_MEMORY_2026-05-01.md)
