# GOAP Plan: Federated Memory Architecture (Spec + Two-Node PoC)

**Date**: 2026-05-01
**ADR**: [ADR-055](adr/ADR-055-Federated-Memory-Architecture.md)
**Branch (proposed)**: `feature/mesh-poc` (NOT to be merged to `main` until Phase F passes)
**Status**: 🟡 Plan accepted, not yet executing
**Owner**: Architecture WG
**Estimated effort**: 5–7 working days, ~3,500 LOC across 5 crates

## Goal

Ship a feature-flagged prototype of cross-agent federated memory with:

1. Cryptographically signed `HVec10240` vectors (`SignedHVec`).
2. A QUIC-based gossip wire protocol between `memory-mcp` peers.
3. Vector-clock conflict resolution layered onto existing `ConflictResolution`.
4. A `memory mesh subscribe` CLI command driving a live two-node PoC.

Specification and wire protocols **must land before any networking code**.

## Codebase Impact (verified by audit, 2026-05-01)

| Crate | Surface | Risk |
|-------|---------|------|
| `do-memory-core` | `retrieval::signed`, `sync::vector_clock`, Episode field | Medium — touches hot-path Episode + sync |
| `do-memory-mcp` | new `mesh/` module tree (~1.5k LOC) | High — first networking subsystem |
| `do-memory-cli` | new `commands/mesh/` (~400 LOC) | Low — additive subcommand |
| `do-memory-storage-turso` | additive migration: `peer_episodes`, `peer_keys`, `vector_clocks` | Medium — schema, but additive |
| `do-memory-storage-redb` | new `peer_*` tables in redb namespace | Low |

**Files known to require edits** (paths verified):
- [memory-core/src/retrieval/mod.rs](file:///home/do/rust-self-learning-memory/memory-core/src/retrieval/mod.rs)
- [memory-core/src/episode/structs.rs](file:///home/do/rust-self-learning-memory/memory-core/src/episode/structs.rs#L129)
- [memory-core/src/sync/mod.rs](file:///home/do/rust-self-learning-memory/memory-core/src/sync/mod.rs)
- [memory-core/src/sync/conflict.rs](file:///home/do/rust-self-learning-memory/memory-core/src/sync/conflict.rs)
- [memory-cli/src/commands/mod.rs](file:///home/do/rust-self-learning-memory/memory-cli/src/commands/mod.rs)
- [memory-mcp/src/lib.rs](file:///home/do/rust-self-learning-memory/memory-mcp/src/lib.rs)
- [Cargo.toml](file:///home/do/rust-self-learning-memory/Cargo.toml) (workspace deps)

## World State (Initial)

```yaml
worldstate:
  spec_written: false
  feature_flag_added: false
  signed_hvec_implemented: false
  signed_hvec_tested: false
  vector_clock_implemented: false
  episode_field_added: false
  conflict_variant_added: false
  gossip_codec_implemented: false
  quic_transport_wired: false
  mesh_state_machine_done: false
  cli_skeleton_done: false
  cli_subscribe_works: false
  schema_migration_added: false
  two_node_poc_running: false
  poc_p50_under_2s: false
  cpu_overhead_under_2pct: false
  signature_tampering_detected: false
  vc_concurrent_resolves: false
  quality_gates_green: false
  coverage_ge_90: false
  docs_written: false
  skill_added: false
```

## Goal State

```yaml
goal:
  spec_written: true
  feature_flag_added: true
  signed_hvec_implemented: true
  signed_hvec_tested: true
  vector_clock_implemented: true
  episode_field_added: true
  conflict_variant_added: true
  gossip_codec_implemented: true
  quic_transport_wired: true
  mesh_state_machine_done: true
  cli_skeleton_done: true
  cli_subscribe_works: true
  schema_migration_added: true
  two_node_poc_running: true
  poc_p50_under_2s: true
  cpu_overhead_under_2pct: true
  signature_tampering_detected: true
  vc_concurrent_resolves: true
  quality_gates_green: true
  coverage_ge_90: true
  docs_written: true
  skill_added: true
```

## Action Graph

```diagram
╭───────────────────────╮
│ A1: Write spec doc    │ Phase A
│ + wire-format tables  │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ A2: Add feature flags │
│ mesh / mesh-server    │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮     ╭───────────────────────╮
│ B1: SignedHVec module │     │ C1: VectorClock type  │ Phase B+C (parallel)
│  + ed25519 sign/verify│     │  + ClockOrdering      │
╰──────────┬────────────╯     ╰──────────┬────────────╯
           ▼                             ▼
╭───────────────────────╮     ╭───────────────────────╮
│ B2: SignedHVec proptest│    │ C2: Episode field +   │
│  + tamper-detection   │     │ ConflictResolution::VC│
╰──────────┬────────────╯     ╰──────────┬────────────╯
           ▼                             ▼
        (merge)            ◀────────────────
           ▼
╭───────────────────────╮
│ D1: MeshFrame codec   │ Phase D
│ (postcard, fuzz tests)│
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ D2: QUIC transport    │
│  via quinn + rustls   │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ D3: Mesh state machine│
│ (anti-entropy, Plum-  │
│  tree, TTL, dedup)    │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮     ╭───────────────────────╮
│ E1: Schema migrations │     │ E2: CLI mesh init/    │ Phase E (parallel)
│ peer_* + vector_clocks│     │ peer/subscribe/status │
╰──────────┬────────────╯     ╰──────────┬────────────╯
           ▼                             ▼
        (merge)            ◀────────────────
           ▼
╭───────────────────────╮
│ F1: Two-node loopback │ Phase F (gating)
│ integration test      │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ F2: PoC perf gate     │
│ p50≤2s, CPU≤2%        │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ F3: Adversarial gate  │
│ tamper + concurrency  │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ G1: docs + skill      │ Phase G
│ federated-memory      │
╰──────────┬────────────╯
           ▼
╭───────────────────────╮
│ G2: Quality gates +   │
│ coverage ≥90%         │
╰───────────────────────╯
```

## Phases & Tasks

### Phase A — Spec (Day 1, no code)

| Task | Description | Owner | Acceptance |
|------|-------------|-------|------------|
| A1 | Finalize wire-format tables (ADR-055 §"Wire Protocol") | Arch WG | ADR-055 merged to `plans/adr/` |
| A2 | Add `mesh` feature flag rows to all 5 publishable Cargo.tomls (no code yet) | Build WG | `cargo build --features mesh` no-ops cleanly |

### Phase B — Cryptographic Layer (Day 2, parallel with C)

| Task | Description | Files | Verification |
|------|-------------|-------|--------------|
| B1 | `SignedHVec` struct + `sign`/`verify` | `memory-core/src/retrieval/signed.rs` (new) | Unit + roundtrip test |
| B2 | Property test: 100 signed → verify, mutate any byte → verify fails | `memory-core/tests/signed_hvec_proptest.rs` | proptest 256 cases |

### Phase C — Vector Clocks (Day 2, parallel with B)

| Task | Description | Files | Verification |
|------|-------------|-------|--------------|
| C1 | `VectorClock` + `ClockOrdering` | `memory-core/src/sync/vector_clock.rs` (new) | Lamport happens-before tests |
| C2 | Episode `vector_clock: Option<VectorClock>` + `ConflictResolution::VectorClockMerge` | `memory-core/src/episode/structs.rs`, `memory-core/src/sync/conflict.rs` | Existing tests still green; new VC tests |

### Phase D — Gossip Transport (Day 3–4)

| Task | Description | Files | Verification |
|------|-------------|-------|--------------|
| D1 | `MeshFrame` + postcard codec + fuzz harness | `memory-mcp/src/mesh/wire.rs` | `cargo fuzz` 1M iterations green |
| D2 | QUIC server/client via `quinn`, ed25519 self-signed certs | `memory-mcp/src/mesh/transport.rs` | Loopback handshake unit test |
| D3 | Anti-entropy state machine (Plumtree-style, TTL=4, 1Hz heartbeat) | `memory-mcp/src/mesh/peer.rs` | Deterministic state-machine tests |

### Phase E — CLI + Storage (Day 5, parallel)

| Task | Description | Files | Verification |
|------|-------------|-------|--------------|
| E1 | Additive Turso/redb migrations (`peer_episodes`, `peer_keys`, `vector_clocks`) | `memory-storage-turso/migrations/`, redb storage | Migration roundtrip test |
| E2 | `memory mesh init / peer add / peer list / subscribe / publish / status` | `memory-cli/src/commands/mesh/` (new) | CLI snapshot tests (insta) |

### Phase F — Proof-of-Concept Gate (Day 6)

All 8 ADR-055 acceptance criteria must pass.

| Task | Description | Verification |
|------|-------------|--------------|
| F1 | Two-node integration test on loopback | `memory-mcp/tests/mesh_two_node.rs` green |
| F2 | Perf gate: p50 ≤ 2 s pattern propagation; CPU idle ≤ 2%, RSS Δ ≤ 25 MB | Criterion bench + `/proc/self/stat` sampling |
| F3 | Adversarial: tampered frame rejected, vector-clock concurrent → MostRecent fallback converges | Dedicated tampering + concurrency tests |

### Phase G — Documentation + Quality (Day 7)

| Task | Description | Files |
|------|-------------|-------|
| G1 | `agent_docs/federated_memory.md` + `.agents/skills/federated-memory/SKILL.md` | per AGENTS.md skill template |
| G2 | `./scripts/quality-gates.sh` green; coverage ≥90% on new modules; `cargo doc --no-deps` clean | quality gate script |

## Quality Gates (mandatory before any merge attempt)

```bash
./scripts/code-quality.sh fmt
./scripts/code-quality.sh clippy --workspace --all-features
./scripts/build-rust.sh check
cargo nextest run --all --features mesh
cargo nextest run --all                       # mesh OFF — no regressions
cargo test --doc --all-features
cargo doc --no-deps --document-private-items
./scripts/quality-gates.sh                    # coverage ≥90% (QUALITY_GATE_COVERAGE_THRESHOLD)
cargo audit                                   # mandatory for crypto code
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Network test flakiness | High | Medium | `tokio::time::pause()`, loopback only, retry budget = 0 |
| Hot-path regression on Episode (added field) | Medium | High | Field is `Option<_>` + `serde(default)`; benchmark against `<50ms` budget |
| Crypto API misuse | Medium | High | Use `ed25519-dalek` v2 high-level API, no raw curve math; cargo audit gate |
| QUIC port conflicts on CI | Medium | Low | Bind to `127.0.0.1:0`, discover assigned port |
| Feature-flag matrix explosion | Low | Medium | Test only (`default`, `mesh`, `csm,mesh`) — 3 columns max |
| Schema migration breakage | Low | High | Migrations are purely additive; existing tests run with mesh OFF |
| `chaotic_semantic_memory` API drift | Low | Medium | Pin to `=0.3.4`; signing wraps externally, doesn't fork |

## Success Metrics

- ✅ All 8 ADR-055 acceptance criteria green
- ✅ Two `memory-mcp` processes exchange a pattern via `memory mesh subscribe` in <2 s p50
- ✅ Tampered signature rejected with explicit `WARN` + counter increment
- ✅ Concurrent vector-clock edits resolve deterministically
- ✅ Coverage ≥ 90% on all new modules
- ✅ Zero clippy warnings under `--features mesh`
- ✅ Default-feature build size delta ≤ 0 bytes (mesh feature off ⇒ zero impact)

## Truth-Source Doc Updates (per AGENTS.md)

When this plan progresses, update **together**:
- [plans/ROADMAPS/ROADMAP_ACTIVE.md](file:///home/do/rust-self-learning-memory/plans/ROADMAPS/ROADMAP_ACTIVE.md) — add Phase 4: Federation (PoC)
- [plans/GOALS.md](file:///home/do/rust-self-learning-memory/plans/GOALS.md)
- [plans/ACTIONS.md](file:///home/do/rust-self-learning-memory/plans/ACTIONS.md)
- [plans/GOAP_STATE.md](file:///home/do/rust-self-learning-memory/plans/GOAP_STATE.md)
- [plans/STATUS/CURRENT.md](file:///home/do/rust-self-learning-memory/plans/STATUS/CURRENT.md)

## Decision Checkpoint

Before Phase D begins, the architecture WG re-confirms `quinn` vs `libp2p`
selection from a 1-day spike comparing:

- Binary size delta (target ≤ 1.5 MB)
- Startup latency (target ≤ 200 ms cold)
- Test-time port-binding ergonomics

If neither meets the bar, **abort and downgrade to TCP+rustls** before
continuing — the gossip protocol layer is transport-agnostic by design.
