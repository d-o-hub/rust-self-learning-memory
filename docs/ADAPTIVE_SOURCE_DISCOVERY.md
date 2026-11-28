# Adaptive Source Discovery with Episodic Memory

An external-consumer example: a search application that must decide *how* to discover
sources for a query family, and that uses episodic memory to notice when its default plan
has stopped working.

- Example: [`examples/adaptive_source_discovery.rs`](../examples/adaptive_source_discovery.rs)
- Fixtures: [`examples/fixtures/adaptive-discovery/dataset.json`](../examples/fixtures/adaptive-discovery/dataset.json)
- End-to-end runner: [`scripts/run-adaptive-discovery-example.sh`](../scripts/run-adaptive-discovery-example.sh)

## What the example demonstrates

| Step | What happens |
|------|--------------|
| Observe | Each past attempt becomes one episode: plan, provider, redacted endpoint, outcome class, latency, cost, and final task outcome. |
| Retrieve | Before planning the next search, the application retrieves similar prior episodes for the same query family. |
| Decide | A **bounded** planner either keeps the static baseline or moves to one alternative plan that the evidence already contains. |
| Record | The attempt and its outcome are recorded back into memory, so the loop keeps learning. |
| Verify | Recording and replay fail loudly on incomplete episodes or failed steps. |
| Evaluate | The same deterministic fixtures score the memory-guided plan against a static baseline. |

Memory is advisory throughout. A discovered source is a **candidate**; the consuming
application must verify it before promoting it or taking any side effect.

## Quick start

From a clean checkout:

```bash
# Compile and run the example's unit tests
cargo test -p do-memory-examples --example adaptive_source_discovery

# Run seed -> recommend -> re-seed -> verify across four processes
./scripts/run-adaptive-discovery-example.sh
```

The runner writes everything under `target/adaptive-discovery-example/`:

| Path | Contents |
|------|----------|
| `memory.db` (+ `.cache.redb`) | durable episodes shared by every phase |
| `report/report-recommend.json` | scored comparison plus the decision fingerprint |
| `report/report-verify.json` | the same comparison recomputed by a later process |
| `reseed.log` | the replay's per-family created/replayed/retried counts |
| `traces/*.json` | memory-harness compatible traces for replay |

Manual equivalent, if you want to run the phases yourself:

```bash
cargo run -p do-memory-examples --example adaptive_source_discovery -- \
  --phase seed --db target/ad/memory.db --emit-traces target/ad/traces
cargo run -p do-memory-examples --example adaptive_source_discovery -- \
  --phase recommend --db target/ad/memory.db --report-dir target/ad/report --require-evidence
# Replay the same fixture rows onto the same database.
cargo run -p do-memory-examples --example adaptive_source_discovery -- \
  --phase seed --db target/ad/memory.db
cargo run -p do-memory-examples --example adaptive_source_discovery -- \
  --phase verify --db target/ad/memory.db --report-dir target/ad/report \
  --expect target/ad/report/report-recommend.json
```

`--phase verify` recomputes the decision in a **new process** and refuses to pass when the
fingerprint differs from the report an earlier process wrote. That is the restart check: the
same evidence, recovered from durable storage, must produce the same recommendation. The
runner places a second `--phase seed` before it, so the fingerprint also covers a replay of
the whole fixture.

`--require-evidence` fails the run when a family that *declares* history in the dataset
retrieves no prior episode — the signature of a seed phase that did not persist. Families
with an empty history are expected to retrieve nothing, which is the cold-start case.

`--limit` (default 16) is how many observations the planner uses per family. Retrieval asks
for a wider window — four times the limit, with a floor of 64 — because the ranking is
quality-weighted and a family's blocked attempts rank last; the family's observations are
then truncated back to `--limit`.

## The dataset

`dataset.json` is deterministic and versioned (`version: "1.0"`). Each query family carries
three parts:

```jsonc
{
  "id": "basketball-scores",
  "query": "latest basketball scores and streaming sources for tonight's games",
  "domain": "sports-research",
  "baseline": { "strategy": "keyword", "provider": "alpha" },
  "history": [ /* what a previous session observed; recorded by `seed` */ ],
  "attempts": [ /* ground truth per plan; used only for scoring */ ],
  "truth": { "verified_urls": [ /* recall denominator */ ] }
}
```

A `history` row carries an optional `id`, the attempt's stable identity in the source
record:

```jsonc
{ "id": "run-2026-11-17-1", "plan": { … }, "outcome": "verified", … }
```

Seeding is a replay, so the identity decides which episode a row resolves to. Supply `id`
when your source record has a durable identity of its own; without it the row's position is
used (`<family>-h<index>`), which is stable for an unchanged dataset but shifts when a row is
inserted above it. Identities must be unique and tag-safe (alphanumeric, `-`, `_`), and
`dataset.validate()` rejects a duplicate or unusable one instead of silently collapsing two
attempts into one episode.

`expectations` makes the example self-checking: it names the plan and decision reason each
family must produce and the minimum aggregate recall gain. A regression fails the run
instead of printing a plausible report.

The fixture set covers the cases that matter:

| Family | Baseline evidence | Expected decision |
|--------|-------------------|-------------------|
| `basketball-scores` | paywalled baseline, alternative verified | switch (`blocked_alternative`) |
| `rust-async-docs` | baseline works | keep baseline (`baseline_supported`) |
| `local-weather` | baseline transport-errors, alternative verified | switch |
| `arxiv-papers` | baseline irrelevant, alternative verified | switch |
| `kubernetes-release-notes` | no history at all | keep baseline (`no_evidence`) |

## Seeding is a replay

`seed` is not a one-shot import. Every attempt carries a stable identity, and the identity
decides which episode it resolves to, so re-running the same dataset updates the episodes
that already record those rows instead of adding a second observation per row. That matters
because `MIN_ALTERNATIVE_OBSERVATIONS` counts observations: a replay that minted fresh
episodes would manufacture exactly the "independent" evidence a bounded planner uses to
switch plans (issue #1302).

The identity is stored as an `attempt_` episode tag. For every row, `seed`:

| Store state for the identity | What `seed` does | Reported as |
|------------------------------|------------------|-------------|
| A completed episode | Leaves it untouched, and re-emits its trace | `replayed` |
| A partial episode (tagged, never completed) | Deletes it, then records the row again | `retried` |
| Nothing | Records the row | `created` |

An interrupted import therefore resolves to one completed episode rather than two, and the
counts are printed per family. The runner treats a non-zero `created` count on the replay as
a failure.

A database seeded *before* attempts had identities cannot be replayed: there is nothing to
resolve a fixture row against, so `seed` refuses with an error naming the number of untagged
episodes it found for the family. Re-seed such a database into a fresh `--db`; replaying onto
it would count those rows twice.

## Bounded recommendation rules

The planner cannot roam. It is limited by three constants in
[`planner.rs`](../examples/adaptive_discovery/planner.rs):

| Constant | Value | Meaning |
|----------|-------|---------|
| `MIN_BASELINE_USEFUL_RATE` | `0.5` | A baseline at or above this rate is left alone. |
| `MIN_ALTERNATIVE_OBSERVATIONS` | `2` | An alternative needs at least two observations before it can be used. |
| `MIN_USEFUL_RATE_GAIN` | `0.2` | An alternative must beat the baseline by this margin. |

Consequences worth keeping:

- No evidence means **no change**. A cold database recommends the baseline.
- A single lucky observation cannot flip a plan.
- Only plans that already appear in the evidence are eligible; the planner never invents a
  provider or strategy.
- Decisions are deterministic: ties break by useful rate, then observation count, then plan
  name.

## Redaction contract

Only `scheme://host/path` reaches memory, traces, or reports. `RedactedEndpoint` drops URL
userinfo, query strings, and fragments, so an API key in a query parameter or a session
token in a URL can never be persisted. Request headers are never copied into step
parameters at all.

Fixtures deliberately contain userinfo and tracking parameters so the unit tests can prove
the projection; the serialised trace test asserts those substrings are absent.

## Scoring

Both plans are scored against the same fixture ground truth:

- **Recall** — verified sources found ÷ verified sources the family actually has.
- **Precision** — verified sources ÷ all candidates the plan returned.
- **Diversity** — distinct hosts among the verified sources.
- **Latency / cost** — declared per attempt in the fixture.

A source only counts as a hit when the application marked it `verified` **and** the family's
verified list contains it. The memory-guided plan is never scored against its own claims.

## Measured result

`./scripts/run-adaptive-discovery-example.sh` on the committed fixture (recorded
2026-11-17; 20 prior attempts across 5 families, dataset `version: 1.0`):

| Metric | Static baseline | Memory-guided | Delta |
|--------|-----------------|---------------|-------|
| Recall | 0.33 | 0.87 | **+0.53** |
| Precision | 0.23 | 0.70 | **+0.47** |
| Diversity (hosts) | 0.60 | 1.80 | **+1.20** |
| Latency | 3130 ms | 2560 ms | −570 ms |
| Cost | $0.0090 | $0.0140 | +$0.0050 |

| Family | Baseline | Memory-guided | Recall | Precision | Decision |
|--------|----------|---------------|--------|-----------|----------|
| `basketball-scores` | keyword@alpha | keyword@beta | 0.00 → 0.67 | 0.00 → 0.67 | `blocked_alternative` |
| `rust-async-docs` | keyword@alpha | keyword@alpha | 0.67 → 0.67 | 0.67 → 0.67 | `baseline_supported` |
| `local-weather` | semantic@alpha | keyword@gamma | 0.00 → 1.00 | 0.00 → 1.00 | `blocked_alternative` |
| `arxiv-papers` | semantic@alpha | keyword@beta | 0.00 → 1.00 | 0.00 → 0.67 | `blocked_alternative` |
| `kubernetes-release-notes` | keyword@alpha | keyword@alpha | 1.00 → 1.00 | 0.50 → 0.50 | `no_evidence` |

Both the `recommend` and the `verify` process reported the same decision fingerprint
(`56e59a953ba85e6f`), and every dataset expectation passed. The `kubernetes-release-notes`
row is the cold-start case: it has no history, so the planner keeps the baseline, and the
runner does not treat that as missing evidence.

Improvements are not free: the guided run costs about half a cent more, and the report
carries both numbers rather than claiming a free win.

## Replaying the recorded attempts

`seed --emit-traces` writes memory-harness traces, so the recorded attempts can be replayed
through the CLI instead of a bespoke format:

```bash
cargo build -p do-memory-cli
MEMORY_DB_PATH=target/ad/replay.db DO_MEMORY_CLI=target/debug/do-memory-cli \
  bash .agents/skills/memory-harness/replay-trace.sh \
  target/ad/traces/basketball-scores-keyword-beta.json
```

The runner also proves the fail-loud contract: it rewrites a trace's outcome to
`in_progress` and requires the replay path to reject it.

## Adapting it to a real application

1. Replace the fixture loader with your own attempt recorder; keep the tags
   (`family_`, `strategy_`, `provider_`, `outcome_`, `attempt_`) so retrieval stays
   explainable and a replay can resolve an attempt to the episode that already records it.
2. Record only outcome classes your application can defend. "Verified" must mean the
   application verified the source itself, not that the provider claimed relevance.
3. Do not let the default `quality_threshold` (`0.70`) discard your failure episodes. A
   blocked or transport-error attempt scores far below it, and that is exactly the evidence
   that a plan has stopped working. The example sets `quality_threshold: 0.0` on purpose: an
   application that stores only its successes cannot learn to abandon a failing strategy.
4. Ask for a wider retrieval window than the number of observations you intend to use.
   Retrieval ranks by a quality-weighted relevance score, so blocked attempts rank last; a
   consumer that asks for exactly `N` candidates can be starved by other families in a busy
   store and then wrongly conclude it has no evidence. The example requests four times the
   window (with a floor of 64) and truncates the family's observations back to `--limit`.
5. Keep the bounds. Loosening `MIN_ALTERNATIVE_OBSERVATIONS` or dropping the gain margin is
   how a memory system starts oscillating between plans.
6. Treat the recommendation as one input to routing, never as a source of truth, and never
   as a licence to skip verification.

## Not a persistence-API change

The example uses only the existing public surface: `SelfLearningMemory::start_episode`,
`log_step`, `add_episode_tags`, `complete_episode`, `list_episodes_by_tags`,
`delete_episode`, `retrieve_relevant_context`, and `recommend_patterns_for_task`. No
production interface was added or changed for it.
