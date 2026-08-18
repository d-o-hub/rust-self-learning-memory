# Reproducible Retrieval Quality-and-Cost Evaluation Guide

## Overview

The retrieval evaluation benchmark harness provides a shared evidence base for evaluating performance, accuracy, tier distribution, latency, and external API embedding cost across retrieval strategies in `rust-self-learning-memory`.

Evaluating latency alone is insufficient because product quality (Recall@k, MRR, NDCG@k, recommendation acceptance) and API consumption (cost per query / successful recommendation) are primary system metrics.

---

## Quick Start (Local Execution Without Credentials)

Run the retrieval benchmark locally without external credentials:

```bash
cargo run -p do-memory-cli -- eval benchmark
```

To enable cascading retrieval with CSM components:

```bash
cargo run -p do-memory-cli --features do-memory-core/csm -- eval benchmark
```

---

## Retrieval Strategies Compared

The harness evaluates and compares three primary strategies:

1. **`always_embed`**: Forces Tier 4 external API embedding search for all queries (1.00 API calls/query).
2. **`local_only`**: Forces CPU-local search tiers (BM25, HDC, ConceptGraph) without external API calls (0.00 API calls/query).
3. **`adaptive`**: CSM cascading pipeline (BM25 $\rightarrow$ HDC $\rightarrow$ ConceptGraph $\rightarrow$ API fallback). CPU-local tiers satisfy queries first, falling back to API only when confidence/result count is insufficient.

### Comparing Strategies

To evaluate a single strategy or all strategies:

```bash
# Evaluate all strategies (default)
cargo run -p do-memory-cli -- eval benchmark --strategy all

# Evaluate only adaptive cascading retrieval
cargo run -p do-memory-cli -- eval benchmark --strategy adaptive
```

---

## Metrics Reported

The benchmark produces both machine-readable JSON and concise Markdown reports detailing:

- **Quality & Accuracy**:
  - `Recall@k` (k=1, 3, 5, 10): Fraction of expected items retrieved.
  - `MRR`: Mean Reciprocal Rank of first relevant item.
  - `NDCG@k` (k=1, 3, 5, 10): Normalized Discounted Cumulative Gain.
  - `Rec Acceptance`: Recommendation success proxy rate (% queries matching top expected/successful outcome).
- **Tier Distribution & API Usage**:
  - % queries resolved at Tier 1 (BM25), Tier 2 (HDC), Tier 3 (ConceptGraph), Tier 4 (API fallback).
  - External embedding API calls per query.
- **Performance & Cost**:
  - `Local Latency` P50 / P95 / P99 in microseconds.
  - `End-to-End Latency` P50 / P95 / P99 in microseconds.
  - Candidate set size before and after ranking.
  - Cache hit rate.
  - Estimated embedding cost per query ($) and per successful recommendation ($).

---

## Fixture Corpus & Baseline Artifacts

Corpus fixtures and baselines are located in `benches/fixtures/`:

- `benches/fixtures/retrieval_benchmark_corpus.json`: Immutable ground-truth corpus containing items and test queries.
- `benches/fixtures/retrieval_baseline.json`: Versioned baseline metrics artifact used for regression detection.

### Fixture Format (JSON / JSONL)

A JSON fixture corpus consists of `corpus` items and `queries`:

```json
{
  "version": "1.0.0",
  "description": "Ground-truth retrieval benchmark corpus",
  "corpus": [
    {
      "id": "item-auth-01",
      "text": "OAuth2 JWT token authentication middleware in Rust tokio",
      "context": {
        "domain": "security",
        "language": "rust",
        "framework": "tokio",
        "complexity": "Moderate",
        "tags": ["auth"]
      },
      "tags": ["auth", "jwt"],
      "is_successful": true,
      "reward_score": 0.95
    }
  ],
  "queries": [
    {
      "id": "q-01",
      "query": "OAuth2 JWT token authentication in Rust tokio",
      "context": {
        "domain": "security",
        "complexity": "Moderate"
      },
      "expected_ids": ["item-auth-01"],
      "tags": ["auth"],
      "expected_accepted_id": "item-auth-01"
    }
  ]
}
```

---

## Corpus Updates & Anti-Overfitting Rules

To maintain scientific integrity and prevent over-fitting retrieval parameters (such as cascade thresholds or ranking weights) to specific test queries:

1. **Train/Test Query Separation**: Never tune cascade thresholds or scoring parameters directly against test queries in `retrieval_benchmark_corpus.json`. Use synthetic training queries during development.
2. **Immutable Test Queries**: Test queries in a released corpus version (e.g. `1.0.0`) are frozen. Do not modify expected relevance sets to mask retrieval failures.
3. **Corpus Expansion Guidelines**:
   - Adding new queries requires cross-validation: verify that new queries cover realistic agent tasks and domain variations.
   - When introducing new domains, increment corpus `version` (e.g., `1.1.0`) and regenerate `retrieval_baseline.json` via a formal PR.
4. **Deterministic Seed & Local Execution**: All local benchmark runs must be deterministic and runnable without external API credentials or secret keys.

---

## CI Thresholds & Regression Checking

In CI, the evaluation harness compares current runs against `benches/fixtures/retrieval_baseline.json`:

```bash
cargo run -p do-memory-cli -- eval benchmark \
  --fixture benches/fixtures/retrieval_benchmark_corpus.json \
  --baseline benches/fixtures/retrieval_baseline.json \
  --fail-on-regression \
  --max-recall-drop 0.05 \
  --max-latency-increase 0.50 \
  --max-cost-increase 0.20 \
  --output-json benchmark_results/retrieval_eval.json \
  --output-markdown benchmark_results/retrieval_eval.md
```

### Configurable Regression Thresholds

- `--max-recall-drop`: Maximum allowed drop in Recall@5 (default: 0.05 / 5%).
- `--max-mrr-drop`: Maximum allowed drop in MRR (default: 0.05).
- `--max-ndcg-drop`: Maximum allowed drop in NDCG@5 (default: 0.05).
- `--max-latency-increase`: Maximum allowed increase in P95 latency ratio (default: 0.50 / 50%).
- `--max-cost-increase`: Maximum allowed increase in cost ratio (default: 0.20 / 20%).

If any threshold is exceeded, the CLI exits with non-zero status code and lists all violations.
