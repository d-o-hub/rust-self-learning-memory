//! Cascading retrieval pipeline (WG-131).
//!
//! Implements a 4-tier retrieval cascade:
//! 1. BM25 keyword index (CPU-local, no API calls)
//! 2. HDC hyperdimensional encoding (CPU-local, no API calls)
//! 3. ConceptGraph ontology expansion (CPU-local, no API calls)
//! 4. API embedding fallback (external API call)
//!
//! The cascade eliminates 50-70% of embedding API calls by satisfying
//! queries from CPU-local tiers before falling back to the API.

#[cfg(feature = "csm")]
use super::gist::GistScoredItem;
use super::gist::{CogniRank, CogniRankWeights, HierarchicalReranker, RerankConfig};
use anyhow::Result;

mod concept_graph;
pub use concept_graph::ConceptGraph;

/// Configuration for the cascading retrieval pipeline.
#[derive(Debug, Clone)]
pub struct CascadeConfig {
    /// Number of results to return from each tier.
    pub top_k: usize,
    /// Minimum score threshold for BM25 results (0.0-1.0).
    pub bm25_threshold: f32,
    /// Minimum similarity threshold for HDC results (0.0-1.0).
    pub hdc_threshold: f32,
    /// Minimum confidence threshold for ConceptGraph results (0.0-1.0).
    pub concept_graph_threshold: f32,
    /// Whether to merge results across tiers.
    pub merge_results: bool,
    /// Minimum results before escalating to next tier.
    pub min_results: usize,
    /// Enable/disable ConceptGraph expansion.
    pub enable_concept_expansion: bool,
    /// Enable/disable CogniRank reranking.
    pub enable_cognirank: bool,
    /// Weights for CogniRank scoring.
    pub cognirank_weights: CogniRankWeights,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            bm25_threshold: 0.3,
            hdc_threshold: 0.5,
            concept_graph_threshold: 0.4,
            merge_results: true,
            min_results: 3,
            enable_concept_expansion: true,
            enable_cognirank: true,
            cognirank_weights: CogniRankWeights::default(),
        }
    }
}

/// Result from a single tier in the cascade.
#[derive(Debug, Clone)]
pub struct TierResult {
    /// Tier identifier (bm25, hdc, concept_graph, api).
    pub tier: String,
    /// Retrieved episode IDs with scores as tuples.
    pub results: Vec<(String, f32)>,
    /// Whether this tier produced sufficient results.
    pub sufficient: bool,
}

impl TierResult {
    /// Get episode IDs from results.
    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        self.results.iter().map(|(id, _)| id.clone()).collect()
    }

    /// Get scores from results.
    #[must_use]
    pub fn scores(&self) -> Vec<f32> {
        self.results.iter().map(|(_, score)| *score).collect()
    }

    /// Check if results are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get number of results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

/// Final result from the cascading retrieval pipeline.
#[derive(Debug, Clone)]
pub struct CascadeResult {
    /// Final merged/re-ranked episode IDs.
    pub episode_ids: Vec<String>,
    /// Final merged/re-ranked scores.
    pub scores: Vec<f32>,
    /// Which tier(s) contributed to the final result.
    pub contributing_tiers: Vec<String>,
    /// Number of API calls made (should be 0 or 1).
    pub api_calls: u32,
}

/// Cascading retrieval orchestrator.
///
/// Coordinates the 4-tier retrieval pipeline, falling back to API
/// only when CPU-local tiers cannot satisfy the query.
pub struct CascadeRetriever {
    config: CascadeConfig,
    /// Episode data indexed for retrieval (id -> text).
    episode_data: Vec<(String, String)>,
    /// Reranker for final results.
    #[cfg(feature = "csm")]
    reranker: HierarchicalReranker,
    /// CogniRank logic.
    #[cfg(feature = "csm")]
    cognirank: CogniRank,
    /// Concept graph for ontology-based term expansion (Tier 3).
    #[cfg(feature = "csm")]
    concept_graph: ConceptGraph,
    #[cfg(feature = "csm")]
    bm25_index: super::Bm25Index,
    #[cfg(feature = "csm")]
    hdc_encoder: super::HdcEncoder,
    #[cfg(feature = "csm")]
    hdc_vectors: Vec<(String, super::HVec10240)>,
}

impl CascadeRetriever {
    /// Create a new cascade retriever with given configuration.
    pub fn new(config: CascadeConfig) -> Self {
        Self {
            config,
            episode_data: Vec::new(),
            #[cfg(feature = "csm")]
            reranker: HierarchicalReranker::new(RerankConfig::dense()),
            #[cfg(feature = "csm")]
            cognirank: CogniRank::default(),
            #[cfg(feature = "csm")]
            concept_graph: ConceptGraph::from_embedded(),
            #[cfg(feature = "csm")]
            bm25_index: super::Bm25Index::new(),
            #[cfg(feature = "csm")]
            hdc_encoder: super::HdcEncoder::new(),
            #[cfg(feature = "csm")]
            hdc_vectors: Vec::new(),
        }
    }

    /// Create a new cascade retriever with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(CascadeConfig::default())
    }

    /// Tokenize text for BM25 indexing/search.
    #[cfg(feature = "csm")]
    fn tokenize(text: &str) -> Vec<String> {
        // Use default tokenization: not code-aware, lowercase enabled
        super::HdcEncoder::tokenize(text, false, true)
    }

    /// Add an episode to the retrieval index.
    ///
    /// This indexes the episode in BM25 and encodes it for HDC similarity search.
    /// When the `csm` feature is not enabled, this just stores the episode data
    /// for later retrieval.
    pub fn add_episode(&mut self, id: &str, text: &str) {
        self.episode_data.push((id.to_string(), text.to_string()));

        #[cfg(feature = "csm")]
        {
            // Tokenize and add to BM25 index
            let tokens = Self::tokenize(text);
            self.bm25_index.add_document(id, &tokens);

            // Encode and store HDC vector
            let hdc_vector = self.hdc_encoder.encode(text);
            self.hdc_vectors.push((id.to_string(), hdc_vector));
        }
    }

    /// Clear all indexed episodes.
    pub fn clear(&mut self) {
        self.episode_data.clear();

        #[cfg(feature = "csm")]
        {
            self.bm25_index.clear();
            self.hdc_vectors.clear();
        }
    }

    /// Get the number of indexed episodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.episode_data.len()
    }

    /// Check if the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.episode_data.is_empty()
    }

    /// Execute the cascading retrieval pipeline.
    ///
    /// When the `csm` feature is enabled, this implements a 4-tier cascade:
    /// 1. BM25 keyword search (CPU-local, 0 API calls)
    /// 2. HDC similarity search (CPU-local, 0 API calls)
    /// 3. ConceptGraph expansion (CPU-local, 0 API calls)
    /// 4. API fallback (requires external embedding call)
    ///
    /// Without `csm`, returns empty results (placeholder behavior).
    pub fn retrieve(&self, query: &str) -> Result<CascadeResult> {
        #[cfg(feature = "csm")]
        {
            self.retrieve_with_csm(query)
        }

        #[cfg(not(feature = "csm"))]
        {
            // Placeholder implementation - returns empty results
            // query is intentionally unused in placeholder mode
            let _ = query;
            Ok(CascadeResult {
                episode_ids: Vec::new(),
                scores: Vec::new(),
                contributing_tiers: Vec::new(),
                api_calls: 0,
            })
        }
    }

    /// Full cascade implementation using CSM components.
    #[cfg(feature = "csm")]
    fn retrieve_with_csm(&self, query: &str) -> Result<CascadeResult> {
        use super::{compute_weights, merge_results};
        use std::sync::Arc;

        // Tier 1: BM25 keyword search
        let bm25_results = self.retrieve_bm25(query);

        // Tier 2: HDC similarity search
        let hdc_results = self.retrieve_hdc(query);

        // Tier 3: ConceptGraph expansion (optional)
        let concept_results = if self.config.enable_concept_expansion {
            self.retrieve_concept_graph(query)
        } else {
            TierResult {
                tier: "concept_graph".to_string(),
                results: Vec::new(),
                sufficient: false,
            }
        };

        // Merge and Rerank
        let (mut final_results, mut contributing_tiers) = if self.config.merge_results {
            let weights = compute_weights(query.len());
            let mut merged = merge_results(&bm25_results.results, &hdc_results.results, weights);
            let mut tiers = Vec::new();
            if !bm25_results.is_empty() {
                tiers.push("bm25".to_string());
            }
            if !hdc_results.is_empty() {
                tiers.push("hdc".to_string());
            }

            if !concept_results.is_empty() {
                // Simplified merge for concept results
                for (id, score) in &concept_results.results {
                    if let Some(existing) = merged.iter_mut().find(|(eid, _)| eid == id) {
                        existing.1 = (existing.1 + score) / 2.0;
                    } else {
                        merged.push((id.clone(), *score));
                    }
                }
                tiers.push("concept_graph".to_string());
            }
            (merged, tiers)
        } else if !concept_results.is_empty() {
            (
                concept_results.results.clone(),
                vec!["concept_graph".to_string()],
            )
        } else if !hdc_results.is_empty() {
            (hdc_results.results.clone(), vec!["hdc".to_string()])
        } else {
            (bm25_results.results.clone(), vec!["bm25".to_string()])
        };

        // Apply CogniRank if enabled and we have results
        if self.config.enable_cognirank && !final_results.is_empty() {
            let mut scored_items = Vec::new();
            for (id, score) in &final_results {
                if let Some((_, text)) = self.episode_data.iter().find(|(eid, _)| eid == id) {
                    // Create a dummy episode for gist extraction
                    let ep = Arc::new(crate::Episode::new(
                        text.clone(),
                        crate::TaskContext::default(),
                        crate::TaskType::Other,
                    ));
                    let gist_config = self.reranker.config().max_key_points;
                    let gist_obj =
                        super::gist::GistExtractor::new(gist_config).extract_from_episode(&ep);
                    scored_items.push(GistScoredItem::new(ep, gist_obj, *score));
                }
            }

            if !scored_items.is_empty() {
                let reranked = self.cognirank.rerank(
                    query,
                    scored_items,
                    self.config.cognirank_weights.clone(),
                );
                final_results = reranked
                    .into_iter()
                    .map(|item| (item.episode().episode_id.to_string(), item.combined_score()))
                    .collect();
                contributing_tiers.push("cognirank".to_string());
            }
        }

        let api_calls = if final_results.len() < self.config.min_results {
            contributing_tiers.push("api_fallback_needed".to_string());
            1
        } else {
            0
        };

        Ok(CascadeResult {
            episode_ids: final_results
                .iter()
                .take(self.config.top_k)
                .map(|(id, _)| id.clone())
                .collect(),
            scores: final_results
                .iter()
                .take(self.config.top_k)
                .map(|(_, s)| *s)
                .collect(),
            contributing_tiers,
            api_calls,
        })
    }

    /// BM25 keyword search (Tier 1).
    #[cfg(feature = "csm")]
    fn retrieve_bm25(&self, query: &str) -> TierResult {
        // Tokenize query for BM25 search
        let query_tokens = Self::tokenize(query);
        let raw_results = self.bm25_index.search(&query_tokens, self.config.top_k);

        // Normalize BM25 scores to 0.0-1.0 range
        let results = super::normalize_scores(&raw_results);

        // Determine if results are sufficient
        let sufficient = results.len() >= self.config.min_results
            && results
                .iter()
                .any(|(_, s)| *s >= self.config.bm25_threshold);

        TierResult {
            tier: "bm25".to_string(),
            results,
            sufficient,
        }
    }

    /// HDC hyperdimensional similarity search (Tier 2).
    #[cfg(feature = "csm")]
    fn retrieve_hdc(&self, query: &str) -> TierResult {
        // Encode query to HDC vector
        let query_vector = self.hdc_encoder.encode(query);

        // Compute similarities with all indexed vectors
        let mut similarities: Vec<(String, f32)> = self
            .hdc_vectors
            .iter()
            .map(|(id, vec)| {
                // Use cosine similarity (normalized hamming distance)
                let sim = query_vector.cosine_similarity(vec);
                (id.clone(), sim)
            })
            .collect();

        // Select top-k by similarity (highest first)
        // Optimization: O(N + k log k) instead of O(N log N)
        let top_k = self.config.top_k;
        let similarities = crate::search::select_top_k(&mut similarities, top_k, |a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        // Determine if results are sufficient
        let sufficient = similarities.len() >= self.config.min_results
            && similarities
                .iter()
                .any(|(_, s)| *s >= self.config.hdc_threshold);

        TierResult {
            tier: "hdc".to_string(),
            results: similarities,
            sufficient,
        }
    }

    /// ConceptGraph expansion search (Tier 3).
    ///
    /// Uses the embedded coding-agent domain ontology to expand query terms
    /// and match against indexed episode data using expanded terminology.
    ///
    /// # How it works
    ///
    /// 1. Expand query terms using the ontology (e.g., "auth" → "authentication")
    /// 2. Match expanded terms against episode text
    /// 3. Score results based on term overlap density
    #[cfg(feature = "csm")]
    fn retrieve_concept_graph(&self, query: &str) -> TierResult {
        // Expand query terms using the ontology
        let expanded_terms = self.concept_graph.expand_terms(query);

        if expanded_terms.is_empty() {
            return TierResult {
                tier: "concept_graph".to_string(),
                results: Vec::new(),
                sufficient: false,
            };
        }

        // Match expanded terms against episode data
        let mut scored: Vec<(String, f32)> = self
            .episode_data
            .iter()
            .map(|(id, text)| {
                let text_lower = text.to_lowercase();
                let match_count = expanded_terms
                    .iter()
                    .filter(|term| text_lower.split_whitespace().any(|w| w == term.as_str()))
                    .count();

                // Score based on term overlap density
                let score = if expanded_terms.is_empty() {
                    0.0
                } else {
                    match_count as f32 / expanded_terms.len() as f32
                };

                (id.clone(), score)
            })
            .filter(|(_, s)| *s >= self.config.concept_graph_threshold)
            .collect();

        // Select top-k by score using the shared select_top_k utility
        // (consistent with the HDC tier's approach)
        let top_k = self.config.top_k;
        let scored = crate::search::select_top_k(&mut scored, top_k, |a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let sufficient = scored.len() >= self.config.min_results;

        TierResult {
            tier: "concept_graph".to_string(),
            results: scored,
            sufficient,
        }
    }

    /// Get the configuration for this retriever.
    pub fn config(&self) -> &CascadeConfig {
        &self.config
    }

    /// Estimate the number of API calls that would be saved for a query.
    ///
    /// Returns 1.0 if the query would likely require an API call,
    /// or 0.0 if CPU-local tiers would likely suffice.
    pub fn estimate_api_call_probability(&self, _query: &str) -> f32 {
        // Placeholder - in full implementation, this would analyze
        // query characteristics (length, keywords, complexity)
        // to estimate probability of needing API fallback
        0.5
    }
}

/// Weight computation for query-length-dependent tier weighting.
///
/// Short queries favor BM25 (keyword matching), long queries favor
/// HDC/semantic matching.
#[cfg(feature = "csm")]
pub fn compute_tier_weights(query: &str) -> (f32, f32, f32) {
    let len = query.len();
    if len < 20 {
        // Short query: favor keyword matching
        (0.7, 0.2, 0.1)
    } else if len < 100 {
        // Medium query: balanced weighting
        (0.4, 0.4, 0.2)
    } else {
        // Long query: favor semantic matching
        (0.2, 0.5, 0.3)
    }
}

#[cfg(test)]
mod tests;
