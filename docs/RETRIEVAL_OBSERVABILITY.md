# Retrieval Observability

**Issue**: #962 (retrieval-plane telemetry)
**Last Updated**: 2026-09-05
**Registry**: `do_memory_core::monitoring::metrics::global_retrieval_metrics()`

---

## Overview

Every retrieval request, cache lookup, embedding call, Tier-4 fallback, and
recommendation feedback signal is recorded into a process-global
`RetrievalMetrics` registry. The same registry backs three instrumentation
surfaces, so the CLI, MCP, and Prometheus always agree:

| Surface | Access | Format |
|---------|--------|--------|
| MCP tool | `get_metrics` with `metric_type: "retrieval"` | JSON snapshot |
| CLI | `do-memory-cli monitor retrieval` | JSON snapshot or Prometheus text |
| Prometheus scrape | `MetricsRegistry::export_metrics()` / `MetricsHttpServer` (`do_memory_core::monitoring::metrics`) | Text exposition |
| Rust API | `global_retrieval_metrics()` -> `snapshot()` / `export_prometheus()` | JSON / text |

The registry is process-local: counters reset on restart and are not shared
across processes. For durable, cross-process history, scrape the exposition
endpoint into Prometheus.

---

## Metric Families

| Family | Type | Labels | Description |
|--------|------|--------|-------------|
| `memory_retrieval_requests_total` | counter | `operation`, `tier`, `outcome` | Retrieval requests served, by operation, serving tier, and hit/miss outcome |
| `memory_retrieval_duration_seconds` | summary | `operation`, `tier`, `quantile` | End-to-end retrieval latency per (operation, tier); quantiles 0.5, 0.95, 0.99 |
| `memory_retrieval_candidates_sum` | counter | `stage` | Cumulative candidate-set size entering each pipeline stage |
| `memory_retrieval_candidates_count` | counter | `stage` | Observations per stage; divide `sum` by `count` for the average candidate-set size |
| `memory_cache_requests_total` | counter | `layer`, `result` | Query-cache lookups; `result` is `hit` or `miss` (currently `layer="query"`) |
| `memory_embedding_requests_total` | counter | `provider`, `result` | Embedding calls by provider and `ok`/`error` result |
| `memory_embedding_duration_seconds` | summary | `provider`, `quantile` | Embedding call latency per provider; quantiles 0.5, 0.95, 0.99 |
| `memory_retrieval_fallback_total` | counter | `reason` | Tier-4 (remote embedding) fallback decisions by reason |
| `memory_recommendation_feedback_total` | counter | `signal` | Recorded recommendation feedback by outcome signal |

`memory_retrieval_candidates_sum` and `_count` are the `sum`/`count` pair of a
single logical summary; HELP/TYPE lines are emitted once per family.

---

## Label Vocabularies (Bounded Enums)

Labels are drawn from fixed enums only. Unknown values cannot be recorded, so
the series count is bounded at compile time.

| Label | Values |
|-------|--------|
| `operation` | `query`, `cascade` |
| `tier` | `cache`, `hybrid`, `semantic`, `hierarchical`, `keyword`, `bm25`, `hdc`, `concept_graph`, `api`, `blended`, `none` |
| `outcome` (requests) | `hit`, `miss` |
| `stage` | `cascade`, `scored` |
| `layer` | `query` |
| `result` (cache) | `hit`, `miss` |
| `provider` | `local`, `openai`, `mistral`, `azure_openai`, `custom` |
| `result` (embeddings) | `ok`, `error` |
| `reason` (fallback) | `local_tier_sufficient`, `local_confident`, `insufficient_confidence`, `no_local_results`, `always_embed_policy`, `local_only_policy` |
| `signal` (feedback) | `success`, `partial`, `failure`, `abstained` |

Tier markers that describe pipeline internals (for example `api_fallback_needed`)
never become labels; they are folded into `tier` values or fallback reasons.

---

## PromQL / Dashboard Examples

The four panels required by #962:

**Tier distribution** (requests/sec served by each tier):

```promql
sum by (tier) (rate(memory_retrieval_requests_total[5m]))
```

**P95 retrieval latency** (seconds, per operation and tier):

```promql
memory_retrieval_duration_seconds{quantile="0.95"}
```

**Cache hit rate** (0..1):

```promql
sum(rate(memory_cache_requests_total{result="hit"}[5m]))
  /
sum(rate(memory_cache_requests_total[5m]))
```

**Embedding-call rate** (calls/sec by provider and result):

```promql
sum by (provider, result) (rate(memory_embedding_requests_total[5m]))
```

Additional useful panels:

**Fallback decisions by reason** (share of escalations to remote embeddings):

```promql
sum by (reason) (rate(memory_retrieval_fallback_total[5m]))
```

**Average candidate-set size** per stage:

```promql
memory_retrieval_candidates_sum{stage="cascade"} / memory_retrieval_candidates_count{stage="cascade"}
```

**Embedding error ratio**:

```promql
sum by (provider) (rate(memory_embedding_requests_total{result="error"}[5m]))
  /
sum by (provider) (rate(memory_embedding_requests_total[5m]))
```

Suggested Grafana panel types: tier distribution and fallback reasons as
time-series (stacked), P95 latency as time-series per tier, cache hit rate and
embedding error ratio as stat/gauge panels.

---

## MCP Usage

Call the `get_metrics` tool with `metric_type` set to `"retrieval"`:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_metrics",
    "arguments": {
      "metric_type": "retrieval"
    }
  }
}
```

Sample result (shape only — zero-valued entries are omitted in real output):

```json
{
  "retrieval_metrics": {
    "requests": [
      {
        "operation": "cascade",
        "tier": "bm25",
        "outcome": "hit",
        "count": 128,
        "latency_ms": { "p50": 1.8, "p95": 6.2, "p99": 11.4, "avg": 2.4 }
      }
    ],
    "fallbacks": { "insufficient_confidence": 9 },
    "feedback": { "success": 41, "partial": 3 },
    "cache": { "hit": 210, "miss": 64 },
    "embeddings": { "local": { "ok": 64, "error": 1 } },
    "candidates": {
      "cascade": { "observations": 64, "total": 1583 },
      "scored": { "observations": 64, "total": 320 }
    }
  },
  "timestamp": 1757068800
}
```

Snapshot keys: `requests` (per-series counts plus `latency_ms` percentiles),
`fallbacks` (reason -> count), `feedback` (signal -> count), `cache`
(hit/miss -> count), `embeddings` (provider -> `{ok, error}`), `candidates`
(stage -> `{observations, total}`).

---

## CLI Usage

```console
# JSON snapshot (default) — same shape as the MCP view above, without the envelope
do-memory-cli monitor retrieval

# Prometheus text exposition
do-memory-cli monitor retrieval --format prometheus
```

The command reads the in-process registry directly (no network, no storage
round-trips) and respects the global `--output human|json|yaml` formatting
flag. To scrape the same families over HTTP, run a `MetricsHttpServer` on a
`MetricsRegistry` and point Prometheus at its `/metrics` endpoint.

---

## Cardinality and Redaction Contract

**Cardinality** — every label is a bounded enum, so the worst-case series count
is fixed regardless of traffic or data:

| Family | Max series |
|--------|------------|
| `memory_retrieval_requests_total` | 2 x 11 x 2 = 44 |
| `memory_retrieval_duration_seconds` | 2 x 11 x 3 = 66 |
| `memory_retrieval_candidates_sum` / `_count` | 2 stages x 2 = 4 |
| `memory_cache_requests_total` | 1 x 2 = 2 |
| `memory_embedding_requests_total` | 5 x 2 = 10 |
| `memory_embedding_duration_seconds` | 5 x 3 = 15 |
| `memory_retrieval_fallback_total` | 6 |
| `memory_recommendation_feedback_total` | 4 |
| **Total** | **151** |

Zero-valued series are omitted from both the JSON snapshot and the Prometheus
exposition, so an idle process exports almost nothing. A latency summary series
is emitted once per (operation, tier) — not per outcome — and each metric
family's HELP/TYPE header appears exactly once, keeping the exposition valid
for strict scrapers.

**Redaction** — telemetry is safe to expose by construction:

- Labels come only from the bounded enums above; there is no free-form label
  input path.
- Query text, episode/pattern IDs, tags, and error strings can never appear as
  label values, series names, or HELP text.
- The JSON snapshot carries the same aggregates only; it never echoes request
  payloads.

---

## See Also

- [API Reference](./API_REFERENCE.md) — MCP tool contract (`get_metrics`)
- <https://prometheus.io/docs/prometheus/latest/querying/basics/> — PromQL basics
- <https://grafana.com/docs/grafana/latest/panels-visualizations/> — dashboard panels
