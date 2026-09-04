//! Evaluation runner for executing retrieval benchmarks.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::retrieval::cascade::{CascadeConfig, CascadeRetriever, FallbackPolicy};
use crate::search::metrics::{mrr, ndcg_at_k, recall_at_k};

use super::types::{
    BenchmarkMetrics, BenchmarkReport, CostModel, FixtureCorpus, LatencyStats, RetrievalStrategy,
    TierDistribution,
};

/// Evaluator runner that measures quality, latency, tier distribution, and cost.
pub struct RetrievalEvaluator {
    corpus: FixtureCorpus,
    cost_model: CostModel,
}

impl RetrievalEvaluator {
    /// Create a new retrieval evaluator for the given corpus and default cost model.
    #[must_use]
    pub fn new(corpus: FixtureCorpus) -> Self {
        Self {
            corpus,
            cost_model: CostModel::default(),
        }
    }

    /// Set a custom cost model for estimation.
    #[must_use]
    pub fn with_cost_model(mut self, cost_model: CostModel) -> Self {
        self.cost_model = cost_model;
        self
    }

    /// Get corpus version string.
    #[must_use]
    pub fn corpus_version(&self) -> &str {
        &self.corpus.version
    }

    /// Get total corpus items.
    #[must_use]
    pub fn corpus_size(&self) -> usize {
        self.corpus.corpus.len()
    }

    /// Get total benchmark query count.
    #[must_use]
    pub fn query_count(&self) -> usize {
        self.corpus.queries.len()
    }

    /// Run evaluation for all three strategies (`AlwaysEmbed`, `LocalOnly`, `Adaptive`).
    pub fn evaluate_all(&self) -> anyhow::Result<BenchmarkReport> {
        let mut strategies = HashMap::new();

        for strategy in [
            RetrievalStrategy::Adaptive,
            RetrievalStrategy::LocalOnly,
            RetrievalStrategy::AlwaysEmbed,
        ] {
            let metrics = self.evaluate_strategy(strategy)?;
            strategies.insert(strategy.to_string(), metrics);
        }

        Ok(BenchmarkReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            corpus_version: self.corpus.version.clone(),
            corpus_size: self.corpus.corpus.len(),
            query_count: self.corpus.queries.len(),
            strategies,
        })
    }

    /// Run evaluation for a specific retrieval strategy.
    #[allow(clippy::too_many_lines)]
    pub fn evaluate_strategy(
        &self,
        strategy: RetrievalStrategy,
    ) -> anyhow::Result<BenchmarkMetrics> {
        let (id_to_index, successful_item_ids) = self.build_item_indexes();

        // Each strategy drives the cascade's Tier 4 fallback policy directly
        // (issue #968), so `Adaptive` numbers reflect confidence gating while
        // `AlwaysEmbed` / `LocalOnly` provide the cost/quality baselines.
        let fallback_policy = match strategy {
            RetrievalStrategy::AlwaysEmbed => FallbackPolicy::AlwaysEmbed,
            RetrievalStrategy::LocalOnly => FallbackPolicy::LocalOnly,
            RetrievalStrategy::Adaptive => FallbackPolicy::Adaptive,
        };
        let config = CascadeConfig {
            top_k: 10,
            min_results: 1,
            fallback_policy,
            ..CascadeConfig::default()
        };

        let mut retriever = CascadeRetriever::new(config);
        for item in &self.corpus.corpus {
            retriever.add_episode(&item.id, &item.text);
        }

        let mut local_latencies = Vec::with_capacity(self.corpus.queries.len());
        let mut e2e_latencies = Vec::with_capacity(self.corpus.queries.len());

        let mut retrieved_string_lists = Vec::with_capacity(self.corpus.queries.len());
        let mut expected_string_sets = Vec::with_capacity(self.corpus.queries.len());
        let mut relevance_maps = Vec::with_capacity(self.corpus.queries.len());

        let (mut tier1_count, mut tier2_count, mut tier3_count, mut tier4_count) = (0, 0, 0, 0);
        let mut total_embedding_calls = 0u32;
        let mut candidate_sizes_before = Vec::new();
        let mut candidate_sizes_after = Vec::new();

        let mut rec_successes = 0usize;
        let mut cache_hits = 0usize;
        let mut query_cache: HashMap<String, Vec<String>> = HashMap::new();

        for query_entry in &self.corpus.queries {
            let start_e2e = Instant::now();

            let cached_results = query_cache.get(&query_entry.query).cloned();
            let is_cache_hit = cached_results.is_some();
            if is_cache_hit {
                cache_hits += 1;
            }

            let start_local = Instant::now();
            let cascade_res = retriever.retrieve(&query_entry.query);
            let local_duration_us = start_local.elapsed().as_micros() as u64;

            let (retrieved_ids, api_calls, contributing_tiers) =
                self.resolve_query_strategy(&cascade_res, &query_entry.query, strategy);

            let e2e_duration_us = start_e2e.elapsed().as_micros() as u64;

            local_latencies.push(local_duration_us);
            e2e_latencies.push(e2e_duration_us);

            if !is_cache_hit {
                query_cache.insert(query_entry.query.clone(), retrieved_ids.clone());
            }

            total_embedding_calls += api_calls;

            if contributing_tiers.contains(&"bm25".to_string()) {
                tier1_count += 1;
            } else if contributing_tiers.contains(&"hdc".to_string()) {
                tier2_count += 1;
            } else if contributing_tiers.contains(&"concept_graph".to_string()) {
                tier3_count += 1;
            } else {
                tier4_count += 1;
            }

            candidate_sizes_before.push(retriever.len() as f64);
            candidate_sizes_after.push(retrieved_ids.len() as f64);

            if let Some(top_id) = retrieved_ids.first() {
                let matches_expected = query_entry
                    .expected_accepted_id
                    .as_ref()
                    .is_some_and(|expected| top_id == expected);
                let matches_successful = query_entry.expected_accepted_id.is_none()
                    && successful_item_ids.contains(top_id);

                if matches_expected || matches_successful {
                    rec_successes += 1;
                }
            }

            let expected_set: HashSet<String> = query_entry.expected_ids.iter().cloned().collect();
            let mut rel_map = HashMap::new();
            for expected_id in &query_entry.expected_ids {
                if let Some(&idx) = id_to_index.get(expected_id) {
                    rel_map.insert(idx, 1.0);
                }
            }

            retrieved_string_lists.push(retrieved_ids);
            expected_string_sets.push(expected_set);
            relevance_maps.push(rel_map);
        }

        let total_q = self.corpus.queries.len();
        if total_q == 0 {
            anyhow::bail!("Fixture corpus has no queries");
        }

        let (retrieved_idx_lists, expected_idx_sets) = self.convert_ids_to_indices(
            &id_to_index,
            &retrieved_string_lists,
            &expected_string_sets,
        );

        let (recall_at_1, recall_at_3, recall_at_5, recall_at_10) =
            compute_recalls(&retrieved_idx_lists, &expected_idx_sets, total_q);
        let (ndcg_at_1, ndcg_at_3, ndcg_at_5, ndcg_at_10) =
            compute_ndcgs(&retrieved_idx_lists, &relevance_maps, total_q);

        let mrr_score = mrr(&retrieved_idx_lists, &expected_idx_sets);
        let total_q_f = total_q as f64;

        let embedding_calls_per_query = f64::from(total_embedding_calls) / total_q_f;

        let cost_per_query = (self.cost_model.cost_per_api_call
            + (self.cost_model.default_tokens_per_query as f64 / 1000.0)
                * self.cost_model.cost_per_1k_tokens)
            * embedding_calls_per_query;

        let cost_per_rec = if rec_successes > 0 {
            (cost_per_query * total_q_f) / rec_successes as f64
        } else {
            cost_per_query * total_q_f
        };

        Ok(BenchmarkMetrics {
            strategy,
            total_queries: total_q,
            recall_at_1,
            recall_at_3,
            recall_at_5,
            recall_at_10,
            mrr: mrr_score,
            ndcg_at_1,
            ndcg_at_3,
            ndcg_at_5,
            ndcg_at_10,
            recommendation_success_rate: rec_successes as f64 / total_q_f,
            tier_distribution: TierDistribution {
                tier1_bm25_count: tier1_count,
                tier2_hdc_count: tier2_count,
                tier3_concept_graph_count: tier3_count,
                tier4_api_count: tier4_count,
                tier1_percentage: (tier1_count as f64 / total_q_f) * 100.0,
                tier2_percentage: (tier2_count as f64 / total_q_f) * 100.0,
                tier3_percentage: (tier3_count as f64 / total_q_f) * 100.0,
                tier4_percentage: (tier4_count as f64 / total_q_f) * 100.0,
            },
            embedding_calls_per_query,
            local_latency: compute_percentiles(&local_latencies),
            end_to_end_latency: compute_percentiles(&e2e_latencies),
            cache_hit_rate: cache_hits as f64 / total_q_f,
            avg_candidate_set_before_ranking: candidate_sizes_before.iter().sum::<f64>()
                / candidate_sizes_before.len() as f64,
            avg_candidate_set_after_ranking: candidate_sizes_after.iter().sum::<f64>()
                / candidate_sizes_after.len() as f64,
            estimated_cost_per_query: cost_per_query,
            estimated_cost_per_successful_rec: cost_per_rec,
        })
    }

    fn build_item_indexes(&self) -> (HashMap<String, usize>, HashSet<String>) {
        let mut id_to_index = HashMap::new();
        let mut successful_item_ids = HashSet::new();

        for (idx, item) in self.corpus.corpus.iter().enumerate() {
            id_to_index.insert(item.id.clone(), idx);
            if item.is_successful.unwrap_or(false) {
                successful_item_ids.insert(item.id.clone());
            }
        }

        (id_to_index, successful_item_ids)
    }

    fn resolve_query_strategy(
        &self,
        cascade_res: &Result<
            crate::retrieval::cascade::CascadeResult,
            crate::retrieval::cascade::CascadeError,
        >,
        query: &str,
        strategy: RetrievalStrategy,
    ) -> (Vec<String>, u32, Vec<String>) {
        if let Ok(r) = cascade_res {
            return match strategy {
                RetrievalStrategy::AlwaysEmbed => {
                    (r.episode_ids.clone(), 1, vec!["api".to_string()])
                }
                RetrievalStrategy::LocalOnly => {
                    (r.episode_ids.clone(), 0, r.contributing_tiers.clone())
                }
                RetrievalStrategy::Adaptive => (
                    r.episode_ids.clone(),
                    // Policy-enforced inside the cascade (see evaluate_strategy):
                    // confident local results report 0, the rest report 1.
                    r.api_calls,
                    r.contributing_tiers.clone(),
                ),
            };
        }

        let fallback_ids = self.keyword_fallback_search(query, 10);
        let has_good_match = !fallback_ids.is_empty();

        match strategy {
            RetrievalStrategy::AlwaysEmbed => (fallback_ids, 1, vec!["api".to_string()]),
            RetrievalStrategy::LocalOnly => (fallback_ids, 0, vec!["bm25".to_string()]),
            RetrievalStrategy::Adaptive => {
                if has_good_match {
                    (fallback_ids, 0, vec!["bm25".to_string()])
                } else {
                    (fallback_ids, 1, vec!["api_fallback_needed".to_string()])
                }
            }
        }
    }

    fn convert_ids_to_indices(
        &self,
        id_to_index: &HashMap<String, usize>,
        retrieved_string_lists: &[Vec<String>],
        expected_string_sets: &[HashSet<String>],
    ) -> (Vec<Vec<usize>>, Vec<HashSet<usize>>) {
        let mut retrieved_idx_lists = Vec::with_capacity(retrieved_string_lists.len());
        let mut expected_idx_sets = Vec::with_capacity(expected_string_sets.len());

        for (retrieved_strings, expected_strings) in retrieved_string_lists
            .iter()
            .zip(expected_string_sets.iter())
        {
            let mut r_idxs = Vec::new();
            for s in retrieved_strings {
                if let Some(&idx) = id_to_index.get(s) {
                    r_idxs.push(idx);
                }
            }
            let mut e_idxs = HashSet::new();
            for s in expected_strings {
                if let Some(&idx) = id_to_index.get(s) {
                    e_idxs.insert(idx);
                }
            }
            retrieved_idx_lists.push(r_idxs);
            expected_idx_sets.push(e_idxs);
        }

        (retrieved_idx_lists, expected_idx_sets)
    }

    fn keyword_fallback_search(&self, query: &str, top_k: usize) -> Vec<String> {
        let q_tokens: HashSet<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if q_tokens.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(String, f32)> = self
            .corpus
            .corpus
            .iter()
            .map(|item| {
                let text_lower = item.text.to_lowercase();
                let matches = q_tokens
                    .iter()
                    .filter(|token| text_lower.contains(token.as_str()))
                    .count();
                let score = matches as f32 / q_tokens.len() as f32;
                (item.id.clone(), score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        scored.into_iter().map(|(id, _)| id).collect()
    }
}

fn compute_recalls(
    retrieved: &[Vec<usize>],
    expected: &[HashSet<usize>],
    total_q: usize,
) -> (f64, f64, f64, f64) {
    let mut sum_1 = 0.0;
    let mut sum_3 = 0.0;
    let mut sum_5 = 0.0;
    let mut sum_10 = 0.0;

    for (r, e) in retrieved.iter().zip(expected.iter()) {
        sum_1 += recall_at_k(r, e, 1);
        sum_3 += recall_at_k(r, e, 3);
        sum_5 += recall_at_k(r, e, 5);
        sum_10 += recall_at_k(r, e, 10);
    }

    let q_f = total_q as f64;
    (sum_1 / q_f, sum_3 / q_f, sum_5 / q_f, sum_10 / q_f)
}

fn compute_ndcgs(
    retrieved: &[Vec<usize>],
    relevance_maps: &[HashMap<usize, f64>],
    total_q: usize,
) -> (f64, f64, f64, f64) {
    let mut sum_1 = 0.0;
    let mut sum_3 = 0.0;
    let mut sum_5 = 0.0;
    let mut sum_10 = 0.0;

    for (r, rel) in retrieved.iter().zip(relevance_maps.iter()) {
        sum_1 += ndcg_at_k(r, rel, 1);
        sum_3 += ndcg_at_k(r, rel, 3);
        sum_5 += ndcg_at_k(r, rel, 5);
        sum_10 += ndcg_at_k(r, rel, 10);
    }

    let q_f = total_q as f64;
    (sum_1 / q_f, sum_3 / q_f, sum_5 / q_f, sum_10 / q_f)
}

fn compute_percentiles(latencies: &[u64]) -> LatencyStats {
    if latencies.is_empty() {
        return LatencyStats::default();
    }
    let mut sorted = latencies.to_vec();
    sorted.sort_unstable();

    let len = sorted.len();
    let p50_idx = (len * 50 / 100).min(len - 1);
    let p95_idx = (len * 95 / 100).min(len - 1);
    let p99_idx = (len * 99 / 100).min(len - 1);

    let sum: u64 = sorted.iter().sum();

    LatencyStats {
        p50_us: sorted[p50_idx],
        p95_us: sorted[p95_idx],
        p99_us: sorted[p99_idx],
        avg_us: sum / len as u64,
    }
}
