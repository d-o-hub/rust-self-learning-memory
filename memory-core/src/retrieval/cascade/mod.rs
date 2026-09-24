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

mod concept_graph;
pub use concept_graph::ConceptGraph;

#[cfg(feature = "csm")]
mod evidence_stage;
mod fallback;
#[cfg(feature = "csm")]
mod finalize;
mod heuristics;
#[cfg(feature = "csm")]
mod pipeline;
pub use fallback::{FallbackDecision, decide_fallback, local_confidence};

mod types;
use std::sync::Arc;
pub use types::{
    CascadeConfig, CascadeError, CascadeResult, FallbackPolicy, FallbackReason, TierResult,
};

use crate::retrieval::{EvidencePolicy, EvidencePolicyError, EvidenceRetrievalResult};

use super::RetrievalJudge;
use super::rerank::{RerankConfigError, SemanticRerankConfig};

/// Cascading retrieval orchestrator.
///
/// Coordinates the 4-tier retrieval pipeline, falling back to API
/// only when CPU-local tiers cannot satisfy the query.
pub struct CascadeRetriever {
    config: CascadeConfig,
    /// Optional semantic judgment provider for candidate evaluation.
    judge: Option<Arc<dyn RetrievalJudge>>,
    /// Optional semantic rerank over the bounded local shortlist (issue #1031).
    ///
    /// Disabled by default: no provider call, no shortlist text index, and the
    /// local tier ordering is returned verbatim.
    semantic_rerank: SemanticRerankConfig,
    /// Optional evidence-classification policy (issue #1032).
    ///
    /// Disabled by default: plain [`CascadeRetriever::retrieve`] ignores this
    /// field entirely, and no candidate is classified unless a caller opts in
    /// with [`CascadeRetriever::with_evidence_policy`] *and* calls
    /// [`CascadeRetriever::retrieve_with_evidence`].
    evidence_policy: Option<EvidencePolicy>,
    /// Episode data indexed for retrieval (id -> text).
    episode_data: Vec<(String, String)>,
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
            judge: None,
            semantic_rerank: SemanticRerankConfig::default(),
            evidence_policy: None,
            episode_data: Vec::new(),
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

    /// Attach an optional semantic judgment provider to the retriever.
    #[must_use]
    pub fn with_judge(mut self, judge: Arc<dyn RetrievalJudge>) -> Self {
        self.judge = Some(judge);
        self
    }

    /// Get reference to the configured semantic judge if present.
    #[must_use]
    pub fn judge(&self) -> Option<&Arc<dyn RetrievalJudge>> {
        self.judge.as_ref()
    }

    /// Attach a semantic rerank configuration (issue #1031).
    ///
    /// The configuration is validated and normalized up front, so an invalid
    /// configuration can never reach the retrieval path. Reranking stays
    /// opt-in: nothing happens unless [`SemanticRerankConfig::enabled`] is set
    /// *and* a judge is attached.
    ///
    /// # Errors
    ///
    /// Returns the first violated rule as [`RerankConfigError`] (for example
    /// `output_k > shortlist_k`); the receiver is dropped unchanged in that
    /// case, so a failed call cannot leave a half-configured retriever behind.
    pub fn with_semantic_rerank(
        mut self,
        config: SemanticRerankConfig,
    ) -> Result<Self, RerankConfigError> {
        self.semantic_rerank = config.validated()?;
        Ok(self)
    }

    /// The validated, normalized semantic rerank configuration in effect.
    #[must_use]
    pub fn semantic_rerank_config(&self) -> &SemanticRerankConfig {
        &self.semantic_rerank
    }

    /// Attach an evidence-classification policy (issue #1032).
    ///
    /// The policy is validated up front, so an invalid policy can never reach
    /// the retrieval path. Classification stays opt-in and separate from
    /// [`Self::retrieve`]: nothing is classified unless
    /// [`Self::retrieve_with_evidence`] is called.
    ///
    /// # Errors
    ///
    /// Returns the first violated rule as [`EvidencePolicyError`] (for example
    /// `candidate_limit == 0`); the receiver is dropped unchanged in that case,
    /// so a failed call cannot leave a half-configured retriever behind.
    pub fn with_evidence_policy(
        mut self,
        policy: EvidencePolicy,
    ) -> Result<Self, EvidencePolicyError> {
        self.evidence_policy = Some(policy.validated()?);
        Ok(self)
    }

    /// The validated evidence-classification policy in effect, if any.
    #[must_use]
    pub fn evidence_policy(&self) -> Option<&EvidencePolicy> {
        self.evidence_policy.as_ref()
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
    /// Without `csm`, this method returns `Err(CascadeError::CapabilityUnavailable)`
    /// rather than empty results.
    ///
    /// Every local result set is finalized through one rerank-aware path: when
    /// semantic reranking is configured (see [`Self::with_semantic_rerank`])
    /// the bounded local shortlist is reranked exactly once per call. The
    /// default configuration is disabled, which leaves ids, scores, and
    /// accounting byte-identical to the unreranked pipeline.
    pub fn retrieve(&self, query: &str) -> Result<CascadeResult, CascadeError> {
        #[cfg(feature = "csm")]
        {
            let start = std::time::Instant::now();
            let result = self.retrieve_with_csm(query);
            // No query text, IDs, or scores cross into telemetry: the record
            // call reads only tier attributions, counts, and the fallback
            // marker (redaction contract, issue #962).
            crate::monitoring::metrics::global_retrieval_metrics().record_cascade(
                start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                &result,
            );
            Ok(result)
        }

        #[cfg(not(feature = "csm"))]
        {
            tracing::warn!(
                "CSM feature not enabled; cascade retrieval is unavailable. \
                 Enable the `csm` feature for BM25/HDC/ConceptGraph retrieval."
            );
            let _ = query;
            Err(CascadeError::CapabilityUnavailable)
        }
    }

    /// Execute the cascade and classify every candidate as evidence (issue #1032).
    ///
    /// Runs the same local cascade as [`Self::retrieve`] and returns its result
    /// unchanged as [`EvidenceRetrievalResult::base`]; `hits` is the evidence
    /// view over that ranking and `status` is the bounded outcome of the
    /// evidence stage.
    ///
    /// The stage is opt-in. Without a policy (see
    /// [`Self::with_evidence_policy`]) the status is
    /// [`EvidenceStatus::Disabled`](crate::monitoring::metrics::EvidenceStatus::Disabled), no
    /// candidate is classified, and the result is the plain cascade result plus
    /// one evidence hit per candidate.
    ///
    /// At most one provider call is made: the rerank stage and the evidence
    /// stage share a single judgment batch (see `retrieve_evidence_with_csm`).
    ///
    /// Without the `csm` feature this returns
    /// `Err(CascadeError::CapabilityUnavailable)` exactly like [`Self::retrieve`].
    pub fn retrieve_with_evidence(
        &self,
        query: &str,
    ) -> Result<EvidenceRetrievalResult, CascadeError> {
        #[cfg(feature = "csm")]
        {
            let start = std::time::Instant::now();
            let result = self.retrieve_evidence_with_csm(query);
            // The returned base is a cascade result, so cascade telemetry is
            // recorded here exactly as `retrieve()` does (bounded fields only:
            // tiers, counts, fallback marker — redaction contract, issue #962).
            crate::monitoring::metrics::global_retrieval_metrics().record_cascade(
                start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                &result.base,
            );
            Ok(result)
        }

        #[cfg(not(feature = "csm"))]
        {
            tracing::warn!(
                "CSM feature not enabled; evidence-aware cascade retrieval is unavailable. \
                 Enable the `csm` feature for BM25/HDC/ConceptGraph retrieval."
            );
            let _ = query;
            Err(CascadeError::CapabilityUnavailable)
        }
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
                // (`expanded_terms` is non-empty here; an empty expansion is
                // handled by the early return above)
                let score = match_count as f32 / expanded_terms.len() as f32;

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
}

#[cfg(test)]
mod tests;
#[cfg(feature = "csm")]
pub mod weights;
#[cfg(feature = "csm")]
pub use weights::compute_tier_weights;
