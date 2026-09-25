//! Types for the retrieval quality and cost benchmark harness.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy used for executing retrieval queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStrategy {
    /// Always forces API embedding search (Tier 4).
    AlwaysEmbed,
    /// Uses CPU-local tiers (BM25, HDC, ConceptGraph) only (0 API calls).
    LocalOnly,
    /// Uses CSM cascading retrieval (BM25 -> HDC -> ConceptGraph -> API fallback).
    Adaptive,
}

impl std::fmt::Display for RetrievalStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlwaysEmbed => write!(f, "always_embed"),
            Self::LocalOnly => write!(f, "local_only"),
            Self::Adaptive => write!(f, "adaptive"),
        }
    }
}

impl std::str::FromStr for RetrievalStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always_embed" | "always-embed" | "always" => Ok(Self::AlwaysEmbed),
            "local_only" | "local-only" | "local" => Ok(Self::LocalOnly),
            "adaptive" => Ok(Self::Adaptive),
            _ => Err(format!("Unknown retrieval strategy: '{s}'")),
        }
    }
}

/// Report key of the semantic rerank comparison arm (issue #1031).
///
/// The arm reuses [`RetrievalStrategy::LocalOnly`] with an offline judge and an
/// enabled [`crate::retrieval::rerank::SemanticRerankConfig`]. It is a
/// comparison-only entry: baseline artifacts do not track it, so regression
/// checks skip it when it is missing from the baseline.
pub const RERANK_COMPARISON_STRATEGY: &str = "local_only+rerank";

/// Report key of the evidence-classification comparison arm (issue #1032).
///
/// The arm reuses [`RetrievalStrategy::LocalOnly`] with the deterministic
/// fixture-backed judge and the default [`crate::retrieval::EvidencePolicy`]. It
/// is a comparison-only entry: baseline artifacts do not track it, so regression
/// checks skip it when it is missing from the baseline.
pub const EVIDENCE_COMPARISON_STRATEGY: &str = "local_only+evidence";

/// Cost model for estimating external API embedding usage costs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    /// Cost per 1,000 embedding tokens in USD.
    pub cost_per_1k_tokens: f64,
    /// Flat cost per API call in USD.
    pub cost_per_api_call: f64,
    /// Average tokens per query text for estimation.
    pub default_tokens_per_query: usize,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            // Default model: $0.00002 / 1k tokens (similar to text-embedding-3-small)
            cost_per_1k_tokens: 0.00002,
            cost_per_api_call: 0.00005,
            default_tokens_per_query: 128,
        }
    }
}

/// An item in the ground-truth benchmark corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureItem {
    /// Unique item/episode ID.
    pub id: String,
    /// Item content text (description/code/summary).
    pub text: String,
    /// Optional domain/language context metadata.
    #[serde(default)]
    pub context: Option<crate::types::TaskContext>,
    /// Tags associated with the item.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether this item represents a successful outcome/recommendation.
    #[serde(default)]
    pub is_successful: Option<bool>,
    /// Optional reward score (0.0 to 1.0).
    #[serde(default)]
    pub reward_score: Option<f64>,
}

/// A ground-truth benchmark query entry in the corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkQuery {
    /// Query identifier.
    pub id: String,
    /// Natural language or code search query text.
    pub query: String,
    /// Task context.
    #[serde(default)]
    pub context: Option<crate::types::TaskContext>,
    /// Relevant item IDs expected to be retrieved.
    pub expected_ids: Vec<String>,
    /// Optional tags for filtering query subsets.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Expected top recommendation ID for acceptance testing.
    #[serde(default)]
    pub expected_accepted_id: Option<String>,
    /// Per-candidate evidence ground truth keyed by corpus item ID (issue #1032).
    ///
    /// A candidate with no entry stays unlabelled: it is still classified, but
    /// it never contributes to per-dimension metrics or the false-drop count.
    #[serde(default)]
    pub evidence_labels: HashMap<String, EvidenceFixtureLabel>,
}

/// Immutable ground-truth benchmark corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureCorpus {
    /// Version identifier.
    #[serde(default)]
    pub version: String,
    /// Description of corpus context and contents.
    #[serde(default)]
    pub description: String,
    /// Corpus items to be indexed and searched over.
    pub corpus: Vec<FixtureItem>,
    /// Queries with expected ground-truth relevance sets.
    pub queries: Vec<BenchmarkQuery>,
}

impl FixtureCorpus {
    /// Load corpus from a JSON string.
    pub fn from_json_str(json: &str) -> anyhow::Result<Self> {
        let corpus: Self = serde_json::from_str(json)?;
        Ok(corpus)
    }

    /// Load corpus from JSONL format where lines are items or queries.
    pub fn from_jsonl_str(jsonl: &str) -> anyhow::Result<Self> {
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum Record {
            #[serde(rename = "item")]
            Item(FixtureItem),
            #[serde(rename = "query")]
            Query(BenchmarkQuery),
            #[serde(rename = "metadata")]
            Meta {
                version: Option<String>,
                description: Option<String>,
            },
        }

        let mut corpus = Vec::new();
        let mut queries = Vec::new();
        let mut version = String::from("1.0.0");
        let mut description = String::from("JSONL fixture dataset");

        for line in jsonl.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }
            let record: Record = serde_json::from_str(line)?;
            match record {
                Record::Item(item) => corpus.push(item),
                Record::Query(query) => queries.push(query),
                Record::Meta {
                    version: v,
                    description: d,
                } => {
                    if let Some(v) = v {
                        version = v;
                    }
                    if let Some(d) = d {
                        description = d;
                    }
                }
            }
        }

        Ok(Self {
            version,
            description,
            corpus,
            queries,
        })
    }
}

/// Latency percentile statistics in microseconds.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyStats {
    /// 50th percentile (median) latency in microseconds.
    pub p50_us: u64,
    /// 95th percentile latency in microseconds.
    pub p95_us: u64,
    /// 99th percentile latency in microseconds.
    pub p99_us: u64,
    /// Mean latency in microseconds.
    pub avg_us: u64,
}

/// Tier resolution distribution breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TierDistribution {
    /// Number of queries resolved at Tier 1 (BM25).
    pub tier1_bm25_count: usize,
    /// Number of queries resolved at Tier 2 (HDC).
    pub tier2_hdc_count: usize,
    /// Number of queries resolved at Tier 3 (ConceptGraph).
    pub tier3_concept_graph_count: usize,
    /// Number of queries requiring Tier 4 (API fallback).
    pub tier4_api_count: usize,
    /// Percentage resolved at Tier 1.
    pub tier1_percentage: f64,
    /// Percentage resolved at Tier 2.
    pub tier2_percentage: f64,
    /// Percentage resolved at Tier 3.
    pub tier3_percentage: f64,
    /// Percentage resolved at Tier 4.
    pub tier4_percentage: f64,
}

/// Per-candidate evidence ground truth for one benchmark query (issue #1032).
///
/// Each boolean is the expected value of one judged dimension, and
/// `expected_disposition` is the disposition the policy should produce for that
/// candidate (`keep`, `demote`, `flag`, or `drop`; empty when unset). A `keep`
/// expectation the evidence stage turns into a drop is the false drop that
/// blocks a release.
// `struct_excessive_bools` targets control-flow state; this struct mirrors the
// documented JSON fixture schema, where each dimension is an independent
// boolean and the field names are the label vocabulary.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFixtureLabel {
    /// Expected query-relevance judgment.
    #[serde(default)]
    pub relevance: bool,
    /// Expected usefulness-as-evidence judgment.
    #[serde(default)]
    pub useful_evidence: bool,
    /// Expected contradiction judgment.
    #[serde(default)]
    pub contradiction: bool,
    /// Expected instruction-likeness judgment.
    #[serde(default)]
    pub instruction_like: bool,
    /// Expected disposition: `keep`, `demote`, `flag`, or `drop`.
    #[serde(default)]
    pub expected_disposition: String,
}

/// Dimension names of [`EvidenceMetrics::per_dimension`], in index order.
pub const EVIDENCE_DIMENSIONS: [&str; 4] = [
    "relevance",
    "useful_evidence",
    "contradiction",
    "instruction_like",
];

/// Counts and derived rates for one evidence dimension.
///
/// Predicted positives are the judged values at or above that dimension's policy
/// threshold, compared against the fixture labels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    /// Labelled positives the judge also scored at or above the threshold.
    pub tp: u64,
    /// Labelled negatives the judge scored at or above the threshold.
    pub fp: u64,
    /// Labelled positives the judge scored below the threshold.
    #[serde(default, rename = "fn")]
    pub fn_: u64,
    /// Labelled negatives the judge scored below the threshold.
    pub tn: u64,
    /// `tp / (tp + fp)`, `0.0` when nothing was predicted positive.
    pub precision: f64,
    /// `tp / (tp + fn)`, `0.0` when nothing was labelled positive.
    pub recall: f64,
}

impl ClassificationMetrics {
    /// Derive precision and recall from raw confusion counts.
    #[must_use]
    pub fn from_counts(tp: u64, fp: u64, fn_: u64, tn: u64) -> Self {
        Self {
            tp,
            fp,
            fn_,
            tn,
            precision: count_ratio(tp, tp + fp),
            recall: count_ratio(tp, tp + fn_),
        }
    }
}

/// Disposition distribution over one evidence comparison run.
///
/// Counts are in [`crate::retrieval::EvidenceDisposition`] rank order and
/// `total` is their sum; percentages are shares of `total`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DispositionDistribution {
    /// Candidates kept unchanged.
    pub keep: u64,
    /// Candidates kept but flagged as contradictory or instruction-like.
    pub flag: u64,
    /// Candidates kept at a demoted score.
    pub demote: u64,
    /// Candidates removed from the result.
    pub drop: u64,
    /// Sum of the four counts.
    pub total: u64,
    /// `keep` as a percentage of `total`.
    pub keep_percentage: f64,
    /// `flag` as a percentage of `total`.
    pub flag_percentage: f64,
    /// `demote` as a percentage of `total`.
    pub demote_percentage: f64,
    /// `drop` as a percentage of `total`.
    pub drop_percentage: f64,
}

impl DispositionDistribution {
    /// Build the distribution from counts in rank order: keep, flag, demote, drop.
    #[must_use]
    pub fn from_counts(counts: [u64; 4]) -> Self {
        let [keep, flag, demote, dropped] = counts;
        let total = counts.iter().sum();
        let percentage = |count: u64| count_ratio(count, total) * 100.0;
        Self {
            keep,
            flag,
            demote,
            drop: dropped,
            total,
            keep_percentage: percentage(keep),
            flag_percentage: percentage(flag),
            demote_percentage: percentage(demote),
            drop_percentage: percentage(dropped),
        }
    }
}

/// Evidence-classification metrics for one `local_only+evidence` run (issue #1032).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceMetrics {
    /// Per-dimension counts in [`EVIDENCE_DIMENSIONS`] order.
    #[serde(default)]
    pub per_dimension: [ClassificationMetrics; 4],
    /// Disposition counts over every classified candidate.
    #[serde(default)]
    pub dispositions: DispositionDistribution,
    /// Labelled candidates expected to be kept that the evidence stage dropped.
    ///
    /// Release-blocking: any non-zero value fails the regression check.
    #[serde(default)]
    pub false_drop_count: u64,
    /// `false_drop_count` as a share of the classified labelled candidates.
    #[serde(default)]
    pub false_drop_rate: f64,
    /// Judge invocations per evaluated query.
    #[serde(default)]
    pub judge_calls_per_query: f64,
    /// Median per-query time spent inside the judge, in microseconds.
    #[serde(default)]
    pub added_latency_p50_us: u64,
    /// 95th percentile per-query time spent inside the judge, in microseconds.
    #[serde(default)]
    pub added_latency_p95_us: u64,
}

/// Zero-safe ratio behind every count-derived rate in this module.
pub(super) fn count_ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// Aggregated metrics for a single retrieval evaluation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Strategy evaluated.
    pub strategy: RetrievalStrategy,
    /// Total queries evaluated.
    pub total_queries: usize,
    /// Recall at k=1, 3, 5, 10.
    pub recall_at_1: f64,
    pub recall_at_3: f64,
    pub recall_at_5: f64,
    pub recall_at_10: f64,
    /// Mean Reciprocal Rank.
    pub mrr: f64,
    /// NDCG at k=1, 3, 5, 10.
    pub ndcg_at_1: f64,
    pub ndcg_at_3: f64,
    pub ndcg_at_5: f64,
    pub ndcg_at_10: f64,
    /// Recommendation acceptance/success proxy rate (0.0 to 1.0).
    pub recommendation_success_rate: f64,
    /// Breakdown of tier resolution counts and percentages.
    pub tier_distribution: TierDistribution,
    /// Average external embedding API calls per query.
    pub embedding_calls_per_query: f64,
    /// Local search latency percentiles (microseconds).
    pub local_latency: LatencyStats,
    /// End-to-end retrieval latency percentiles (microseconds).
    pub end_to_end_latency: LatencyStats,
    /// Cache hit rate (0.0 to 1.0).
    pub cache_hit_rate: f64,
    /// Average candidate set size prior to reranking.
    pub avg_candidate_set_before_ranking: f64,
    /// Average candidate set size after reranking/filtering.
    pub avg_candidate_set_after_ranking: f64,
    /// Estimated cost per query in USD.
    pub estimated_cost_per_query: f64,
    /// Estimated cost per successful recommendation in USD.
    pub estimated_cost_per_successful_rec: f64,
    /// Average semantic judge invocations per query (issue #1031).
    ///
    /// `0.0` unless the run exercised the semantic rerank comparison arm, and
    /// `0.0` for queries where the rerank stage short-circuited before calling
    /// the judge (disabled, no judge, empty shortlist, provider skipped).
    #[serde(default)]
    pub judge_calls_per_query: f64,
    /// Average number of candidates submitted to the semantic judge per query.
    ///
    /// Equals the judged shortlist size, so it never exceeds
    /// `SemanticRerankConfig::shortlist_k` for reranked queries and is `0.0`
    /// for queries the judge never saw.
    #[serde(default)]
    pub rerank_candidates_per_query: f64,
    /// Fraction of evaluated queries whose top-1 result changed after rerank.
    ///
    /// Derived without a second retrieval: see
    /// `RetrievalEvaluator::evaluate_strategy_with_rerank`.
    #[serde(default)]
    pub top1_changed_rate: f64,
    /// Evidence-classification metrics (issue #1032).
    ///
    /// `None` unless the run exercised the `local_only+evidence` comparison arm;
    /// baseline artifacts predate the key and keep loading unchanged.
    #[serde(default)]
    pub evidence_metrics: Option<EvidenceMetrics>,
}

/// Threshold limits for detecting statistical quality or cost regressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionThresholds {
    /// Maximum allowed drop in Recall@5 (e.g., 0.05 for 5%).
    pub max_recall_drop: f64,
    /// Maximum allowed drop in MRR.
    pub max_mrr_drop: f64,
    /// Maximum allowed drop in NDCG@5.
    pub max_ndcg_drop: f64,
    /// Maximum allowed relative increase in P95 latency (e.g., 0.50 for 50%).
    pub max_latency_increase_ratio: f64,
    /// Maximum allowed relative increase in estimated cost (e.g., 0.20 for 20%).
    pub max_cost_increase_ratio: f64,
}

impl Default for RegressionThresholds {
    fn default() -> Self {
        Self {
            max_recall_drop: 0.05,
            max_mrr_drop: 0.05,
            max_ndcg_drop: 0.05,
            max_latency_increase_ratio: 0.50,
            max_cost_increase_ratio: 0.20,
        }
    }
}

/// Complete benchmark report containing metrics across strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Timestamp ISO-8601 string.
    pub timestamp: String,
    /// Version of corpus evaluated.
    pub corpus_version: String,
    /// Total corpus items.
    pub corpus_size: usize,
    /// Total queries evaluated.
    pub query_count: usize,
    /// Metrics by retrieval strategy name.
    pub strategies: HashMap<String, BenchmarkMetrics>,
}
