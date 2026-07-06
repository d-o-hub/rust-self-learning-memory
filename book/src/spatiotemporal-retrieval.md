# Spatiotemporal Retrieval Cookbook

Spatiotemporal retrieval is a structured approach to finding relevant episodic memories by leveraging domain knowledge, task classification, and temporal context.

## Core Concepts

Unlike traditional flat semantic search, spatiotemporal retrieval uses a 4-level hierarchy to filter and rank memories:

1.  **Domain**: Filters by high-level logical context (e.g., `web-api`, `infrastructure`).
2.  **Task Type**: Filters by the nature of the work (e.g., `CodeGeneration`, `Debugging`).
3.  **Temporal Clusters**: Prioritizes recent episodes using time-based bucketing.
4.  **Semantic Similarity**: Performs fine-grained vector comparison on the remaining candidates.

## Common Patterns

### 1. Recent Similar Task Lookup

When starting a new task, it's often useful to see how similar tasks were handled recently.

```rust
let query = RetrievalQuery {
    query_text: "Implement OAuth2 flow".to_string(),
    domain: Some("auth-service".to_string()),
    task_type: Some(TaskType::CodeGeneration),
    limit: 3,
    ..Default::default()
};

// HierarchicalRetriever will prioritize 'auth-service' and 'CodeGeneration'
// from the last few weeks.
let matches = retriever.retrieve(&query, &history).await?;
```

### 2. Locality-Aware Recall

If you're working within a specific domain, you can restrict search to that domain to avoid "cross-talk" from unrelated areas that might share similar keywords but different implementations.

### 3. Time-Window Constrained Retrieval

You can use the `SpatiotemporalIndex` directly to find all episodes in a specific domain within a time range.

```rust
let start = Utc::now() - Duration::days(7);
let end = Utc::now();
let episode_ids = index.query("frontend", Some(TaskType::Testing), Some(start), Some(end), 10);
```

## Ranking Composition

The final relevance score is calculated as follows:

`Relevance = 0.3 * DomainMatch + 0.3 * TaskTypeMatch + W * Recency + (0.4 - W) * Similarity`

*   **DomainMatch**: 1.0 if domain matches exactly, 0.0 otherwise (0.5 if no domain specified).
*   **TaskTypeMatch**: 1.0 if task type matches exactly, 0.0 otherwise (0.5 if no task type specified).
*   **Recency**: A score from 1.0 (brand new) to 0.0 (older than 30 days).
*   **Similarity**: Cosine similarity of embeddings or word-overlap score.
*   **W**: The `temporal_bias_weight` (default 0.3).

> **Note on Reward**: While reward scores are used in capacity management and eviction policies, they are not currently part of the core hierarchical retrieval scoring. This ensures that even "failed" tasks can be retrieved for guidance (to avoid repeating mistakes), provided they are contextually relevant.
