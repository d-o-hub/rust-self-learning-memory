//! API-call probability heuristics for the cascade retriever.

use super::CascadeRetriever;

impl CascadeRetriever {
    /// Estimate the probability that a query would require an API call.
    ///
    /// Returns a value in [0.0, 1.0] where:
    /// - 0.0 means CPU-local tiers (BM25/HDC/ConceptGraph) are very likely to suffice
    /// - 1.0 means an API embedding call is almost certainly needed
    ///
    /// Heuristic: short keyword-rich queries resolve via BM25 (low probability);
    /// long abstract queries with few known terms need semantic embedding (high probability).
    pub fn estimate_api_call_probability(&self, query: &str) -> f32 {
        let len = query.len() as f32;
        let word_count = query.split_whitespace().count() as f32;

        // Base probability from query length — short queries favor BM25
        let length_factor: f32 = if len < 20.0 {
            0.1
        } else if len < 50.0 {
            0.25
        } else if len < 100.0 {
            0.5
        } else {
            0.7
        };

        // Keyword density — queries with many short words are more BM25-friendly
        let avg_word_len = if word_count > 0.0 {
            len / word_count
        } else {
            10.0
        };
        let keyword_factor: f32 = if avg_word_len < 5.0 {
            0.0 // Short words = good keyword match candidates
        } else if avg_word_len < 8.0 {
            0.15
        } else {
            0.3 // Long words = more semantic, harder for BM25
        };

        // Concept-level boost — code-like tokens (identifiers, paths) are BM25-friendly
        let code_token_count = query
            .split_whitespace()
            .filter(|w| w.contains('_') || w.contains("::") || w.contains('/'))
            .count() as f32;
        let code_factor: f32 = if word_count > 0.0 && code_token_count / word_count > 0.3 {
            -0.15 // Many code tokens boost BM25 relevance
        } else {
            0.0
        };

        (length_factor + keyword_factor + code_factor).clamp(0.0, 1.0)
    }
}
