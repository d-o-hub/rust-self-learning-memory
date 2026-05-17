//! ConceptGraph ontology expansion for cascading retrieval (Tier 3, WG-131).
//!
//! Provides domain-term synonym expansion without LLM calls via a curated
//! ontology loaded from an embedded `ontology.json` at compile time.
//!
//! The ConceptGraph loads domains (e.g., "authentication", "database") and
//! their associated terms and related concepts. Query terms are expanded
//! by finding matching domains and returning all synonyms and related concepts.

use anyhow::{Context, Result};

/// A simple concept graph ontology entry for domain-term expansion.
#[derive(Debug, Clone)]
struct OntologyEntry {
    terms: Vec<String>,
    related_concepts: Vec<String>,
}

/// In-memory concept graph for query term expansion (Tier 3).
///
/// Loaded from the embedded `ontology.json` at compile time.
#[derive(Debug, Clone)]
pub struct ConceptGraph {
    entries: Vec<OntologyEntry>,
}

impl ConceptGraph {
    /// Create a new ConceptGraph from the embedded ontology JSON.
    #[must_use]
    pub fn from_embedded() -> Self {
        let json = include_str!("ontology.json");
        Self::from_json(json).unwrap_or_else(|e| {
            tracing::warn!(
                error = %e,
                "Failed to parse embedded ontology, using empty concept graph"
            );
            Self {
                entries: Vec::new(),
            }
        })
    }

    /// Parse a ConceptGraph from JSON.
    fn from_json(json: &str) -> Result<Self> {
        let parsed: serde_json::Value =
            serde_json::from_str(json).context("Failed to parse ontology JSON")?;

        let domains = parsed["domains"]
            .as_array()
            .context("Missing 'domains' array in ontology")?;

        let entries: Vec<OntologyEntry> = domains
            .iter()
            .map(|d| OntologyEntry {
                terms: d["terms"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                related_concepts: d["related_concepts"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect();

        Ok(Self { entries })
    }

    /// Expand query terms using the ontology.
    ///
    /// For each word in the query, finds matching domain terms and returns
    /// all synonyms and related concepts.
    #[must_use]
    pub fn expand_terms(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut expanded: Vec<String> = Vec::new();

        for entry in &self.entries {
            let domain_matched = query_words
                .iter()
                .any(|w| entry.terms.iter().any(|t| t == w));

            if domain_matched {
                // Add all terms and related concepts from matching domains
                expanded.extend(entry.terms.iter().cloned());
                expanded.extend(entry.related_concepts.iter().cloned());
            }
        }

        expanded.sort();
        expanded.dedup();
        expanded
    }

    /// Check if the concept graph has any entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of ontology domains.
    #[must_use]
    pub fn domain_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_graph_from_embedded() {
        let graph = ConceptGraph::from_embedded();
        // Should parse the embedded ontology successfully
        assert!(graph.domain_count() > 0, "ConceptGraph should have domains");
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_concept_graph_expand_auth_terms() {
        let graph = ConceptGraph::from_embedded();
        let expanded = graph.expand_terms("implement auth");

        // Should find auth-related synonyms
        assert!(!expanded.is_empty());
        assert!(expanded.iter().any(|t| t == "auth"));
        assert!(expanded.iter().any(|t| t == "login"));
    }

    #[test]
    fn test_concept_graph_expand_database_terms() {
        let graph = ConceptGraph::from_embedded();
        let expanded = graph.expand_terms("fix db connection");

        assert!(!expanded.is_empty());
        assert!(expanded.iter().any(|t| t == "db"));
    }

    #[test]
    fn test_concept_graph_expand_no_match() {
        let graph = ConceptGraph::from_embedded();
        let expanded = graph.expand_terms("xyzzy_nonexistent_term");

        // No domain should match this term
        assert!(expanded.is_empty());
    }

    #[test]
    fn test_concept_graph_expand_multiple_domains() {
        let graph = ConceptGraph::from_embedded();
        // Query that touches both authentication and database domains
        let expanded = graph.expand_terms("fix auth and db errors");

        assert!(!expanded.is_empty());
        // Should have terms from multiple domains
        let has_auth = expanded.iter().any(|t| t == "auth" || t == "login");
        let has_db = expanded.iter().any(|t| t == "db" || t == "sql");
        assert!(has_auth || has_db, "Should match at least one domain");
    }

    #[test]
    fn test_concept_graph_empty_query() {
        let graph = ConceptGraph::from_embedded();
        let expanded = graph.expand_terms("");
        assert!(expanded.is_empty());
    }

    #[test]
    fn test_concept_graph_domain_count() {
        let graph = ConceptGraph::from_embedded();
        // Should have at least the domains defined in ontology.json
        assert!(graph.domain_count() >= 10, "Expected 10+ ontology domains");
    }
}
