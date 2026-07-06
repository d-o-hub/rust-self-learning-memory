//! Core types for advanced search functionality
//!
//! This module defines the types used for fuzzy search, regex matching,
//! multi-field search, and result ranking.

use serde::{Deserialize, Serialize};

/// Search mode for text matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum SearchMode {
    /// Exact substring match (case-insensitive)
    /// This is the default behavior and maintains backward compatibility
    #[default]
    Exact,

    /// Fuzzy match with similarity threshold
    /// The threshold is a value between 0.0 and 1.0, where:
    /// - 1.0 = exact match required
    /// - 0.8 = 80% similarity (recommended default)
    /// - 0.6 = 60% similarity (more lenient)
    Fuzzy { threshold: f64 },

    /// Regular expression pattern matching
    /// Note: Patterns are validated before execution to prevent `ReDoS` attacks
    Regex,
}

/// Fields to search within episodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SearchField {
    /// Episode task description
    Description,

    /// Episode step content (actions and observations)
    Steps,

    /// Episode outcome message
    Outcome,

    /// Episode tags
    Tags,

    /// Episode domain
    Domain,

    /// Search all fields
    All,
}

impl SearchField {
    /// Get the relative weight of this field for ranking
    /// Higher weight means matches in this field are more relevant
    #[must_use]
    pub const fn weight(&self) -> f64 {
        match self {
            Self::Description => 1.0, // Most important
            Self::Outcome => 0.8,     // Very relevant
            Self::Steps => 0.6,       // Moderately relevant
            Self::Tags => 0.5,        // Context clues
            Self::Domain => 0.4,      // General categorization
            Self::All => 0.7,         // Average weight
        }
    }
}

/// A search result with scoring information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult<T> {
    /// The matched item
    pub item: T,

    /// Overall relevance score (0.0 to 1.0)
    pub score: f64,

    /// Details about which fields matched
    pub matches: Vec<FieldMatch>,
}

/// Details about a field match
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldMatch {
    /// Which field matched
    pub field: SearchField,

    /// The matched text snippet
    pub matched_text: String,

    /// Match quality score (0.0 to 1.0)
    pub similarity: f64,

    /// Position in the text where match was found
    pub position: usize,
}

impl SearchMode {
    /// Validate the search mode configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Fuzzy threshold is not between 0.0 and 1.0
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Fuzzy { threshold } => {
                if *threshold < 0.0 || *threshold > 1.0 {
                    return Err(format!(
                        "Fuzzy threshold must be between 0.0 and 1.0, got {threshold}"
                    ));
                }
                Ok(())
            }
            Self::Exact | Self::Regex => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_mode_default() {
        assert_eq!(SearchMode::default(), SearchMode::Exact);
    }

    #[test]
    fn test_search_mode_validation() {
        // Valid thresholds
        assert!(SearchMode::Fuzzy { threshold: 0.0 }.validate().is_ok());
        assert!(SearchMode::Fuzzy { threshold: 0.5 }.validate().is_ok());
        assert!(SearchMode::Fuzzy { threshold: 1.0 }.validate().is_ok());

        // Invalid thresholds
        assert!(SearchMode::Fuzzy { threshold: -0.1 }.validate().is_err());
        assert!(SearchMode::Fuzzy { threshold: 1.1 }.validate().is_err());

        // Other modes always valid
        assert!(SearchMode::Exact.validate().is_ok());
        assert!(SearchMode::Regex.validate().is_ok());
    }

    #[test]
    fn test_search_field_weights() {
        assert_eq!(SearchField::Description.weight(), 1.0);
        assert_eq!(SearchField::Outcome.weight(), 0.8);
        assert_eq!(SearchField::Steps.weight(), 0.6);
        assert_eq!(SearchField::Tags.weight(), 0.5);
        assert_eq!(SearchField::Domain.weight(), 0.4);
        assert_eq!(SearchField::All.weight(), 0.7);
    }

    #[test]
    fn test_field_match_creation() {
        let field_match = FieldMatch {
            field: SearchField::Description,
            matched_text: "test query".to_string(),
            similarity: 0.95,
            position: 10,
        };

        assert_eq!(field_match.field, SearchField::Description);
        assert_eq!(field_match.matched_text, "test query");
        assert_eq!(field_match.similarity, 0.95);
        assert_eq!(field_match.position, 10);
    }

    #[test]
    fn test_search_result_creation() {
        let matches = vec![FieldMatch {
            field: SearchField::Description,
            matched_text: "test".to_string(),
            similarity: 0.9,
            position: 0,
        }];

        let result = SearchResult {
            item: "test_episode",
            score: 0.85,
            matches,
        };

        assert_eq!(result.item, "test_episode");
        assert_eq!(result.score, 0.85);
        assert_eq!(result.matches.len(), 1);
    }
}
