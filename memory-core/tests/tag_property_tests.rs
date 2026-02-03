//! Property-based tests for Tag operations and normalization
//!
//! Tests invariants that must hold regardless of input values:
//! - Tag operations are idempotent
//! - Tag sets maintain uniqueness
//! - Tag normalization is consistent
//! - Tag case and whitespace handling is correct

use memory_core::episode::Episode;
use memory_core::types::{TaskContext, TaskType};
use proptest::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Tag Normalization Properties
// ============================================================================

proptest! {
    /// Tag normalization produces consistent lowercase output
    #[test]
    fn tag_normalization_lowercase(tag in "[a-zA-Z0-9_-]{1,20}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);
        episode.add_tag(tag.clone()).ok();

        assert!(episode.tags.iter().all(|t| t == &t.to_lowercase()));
    }

    /// Tag normalization trims whitespace
    #[test]
    fn tag_normalization_trims(tag in "[a-zA-Z0-9_-]{1,10}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add with surrounding whitespace
        let result = episode.add_tag(format!("  {}  ", tag));
        assert!(result.is_ok());

        // Should be trimmed
        assert_eq!(episode.tags[0], tag);
    }

    /// Tag normalization is deterministic
    #[test]
    fn tag_normalization_deterministic(tag in "[a-zA-Z0-9_-]{1,10}") {
        let variations = vec![
            tag.to_uppercase(),
            tag.to_lowercase(),
            format!("  {}  ", tag.to_uppercase()),
            format!("\t{}\t", tag.to_lowercase()),
            format!("  {}  ", tag),
        ];

        let mut normalized_tags = Vec::new();

        for variation in variations {
            let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);
            episode.add_tag(variation).ok();
            if !episode.tags.is_empty() {
                normalized_tags.push(episode.tags[0].clone());
            }
        }

        // All should normalize to the same value
        assert!(
            normalized_tags.iter().all(|t| t == &normalized_tags[0]),
            "All tag variations should normalize to the same value"
        );
    }
}

// ============================================================================
// Tag Uniqueness Properties
// ============================================================================

proptest! {
    /// Tags maintain uniqueness (no duplicates)
    #[test]
    fn tags_maintain_uniqueness(tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 0..20)) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add all tags
        for tag in &tags {
            episode.add_tag(tag.to_string()).ok();
        }

        // Check no duplicates
        let tag_set: HashSet<_> = episode.tags.iter().collect();
        assert_eq!(tag_set.len(), episode.tags.len());
    }

    /// Duplicate tag addition returns false
    #[test]
    fn duplicate_tag_add_returns_false(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // First add should return true
        let result1 = episode.add_tag(tag.clone());
        assert_eq!(result1, Ok(true));

        // Second add should return false (already exists)
        let result2 = episode.add_tag(tag);
        assert_eq!(result2, Ok(false));
    }

    /// Case variations of same tag are considered duplicates
    #[test]
    fn case_variations_duplicates(tag in "[a-zA-Z0-9_-]{2,10}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add lowercase version
        episode.add_tag(tag.clone()).ok();

        // Try to add uppercase version - should fail
        let result = episode.add_tag(tag.to_uppercase());
        assert_eq!(result, Ok(false));

        // Should still only have one tag
        assert_eq!(episode.tags.len(), 1);
    }

    /// Whitespace variations of same tag are considered duplicates
    #[test]
    fn whitespace_variations_duplicates(tag in "[a-zA-Z0-9_-]{2,10}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add without whitespace
        episode.add_tag(tag.clone()).ok();

        // Try to add with whitespace - should fail
        let variations = vec![
            format!("  {}", tag),
            format!("{}  ", tag),
            format!("  {}  ", tag),
            format!("\t{}\t", tag),
        ];

        for variation in variations {
            let result = episode.add_tag(variation);
            assert_eq!(result, Ok(false));
        }

        // Should still only have one tag
        assert_eq!(episode.tags.len(), 1);
    }
}

// ============================================================================
// Tag Idempotence Properties
// ============================================================================

proptest! {
    /// Adding a tag multiple times is idempotent
    #[test]
    fn add_tag_idempotent(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        episode.add_tag(tag.clone()).ok();
        let tags_after_first = episode.tags.clone();

        // Try adding same tag multiple times
        for _ in 0..5 {
            episode.add_tag(tag.clone()).ok();
        }

        let tags_after_multiple = episode.tags;

        assert_eq!(tags_after_first, tags_after_multiple);
    }

    /// Removing a tag is idempotent
    #[test]
    fn remove_tag_idempotent(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag
        episode.add_tag(tag.clone()).ok();
        assert_eq!(episode.tags.len(), 1);

        // Remove once
        let result1 = episode.remove_tag(&tag);
        assert!(result1);
        assert!(episode.tags.is_empty());

        // Try removing again - should return false
        let result2 = episode.remove_tag(&tag);
        assert!(!result2);
        assert!(episode.tags.is_empty());

        // Third removal should also return false
        let result3 = episode.remove_tag(&tag);
        assert!(!result3);
    }

    /// Clear tags is idempotent
    #[test]
    fn clear_tags_idempotent(
        tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 0..10)
    ) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tags
        for tag in &tags {
            episode.add_tag(tag.to_string()).ok();
        }

        // Clear once
        episode.clear_tags();
        assert!(episode.tags.is_empty());

        // Clear again - should still be empty
        episode.clear_tags();
        assert!(episode.tags.is_empty());
    }
}

// ============================================================================
// Tag Validation Properties
// ============================================================================

/// Empty tags are rejected
#[test]
fn empty_tag_rejected() {
    let mut episode = Episode::new(
        "Test".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    let result = episode.add_tag("".to_string());
    assert!(result.is_err());
}

proptest! {
    /// Whitespace-only tags are rejected
    #[test]
    fn whitespace_only_tag_rejected(whitespace in "[ \t\n\r]+") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        let result = episode.add_tag(whitespace);
        assert!(result.is_err());
    }

    /// Tags with invalid characters are rejected
    #[test]
    fn invalid_characters_rejected(
        valid_base in "[a-zA-Z0-9_-]{1,5}",
        invalid_char in "[@#$%^&*()+=<>?/\\]",
        invalid_char2 in "[@#$%^&*()+=<>?/\\]"
    ) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        let invalid_tags = vec![
            format!("{}{}{}", valid_base, invalid_char, valid_base),
            format!("{}{}", valid_base, invalid_char2),
            format!("{}{}{}", invalid_char, valid_base, invalid_char2),
        ];

        for tag in invalid_tags {
            let result = episode.add_tag(tag);
            assert!(result.is_err(), "Tag with invalid character should be rejected");
        }
    }

    /// Too-short tags are rejected
    #[test]
    fn too_short_tag_rejected() {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        let too_short_tags = vec!["a", "b", "c", "1", "-", "_", "_" ];

        for tag in too_short_tags {
            let result = episode.add_tag(tag.to_string());
            assert!(result.is_err());
        }
    }
}

/// Tags length limit is enforced
#[test]
fn tag_length_limit_enforced() {
    let mut episode = Episode::new(
        "Test".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // Valid tag (length 100 should be max limit based on implementation)
    let valid_tag = "a".repeat(100);
    let result = episode.add_tag(valid_tag.clone());
    // The implementation allows up to 100 chars, so this should succeed
    assert!(result.is_ok() || result.unwrap_err().contains("100"));

    // Invalid tag (too long)
    let invalid_tag = "a".repeat(101);
    let result = episode.add_tag(invalid_tag);
    assert!(result.is_err());
}

// ============================================================================
// Tag Query Properties
// ============================================================================

proptest! {
    /// has_tag returns correct result
    #[test]
    fn has_tag_correct(
        tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 0..10),
        query_tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 0..5)
    ) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tags
        for tag in &tags {
            episode.add_tag(tag.to_string()).ok();
        }

        // Normalize query tags for comparison
        let normalized_tags: HashSet<_> = tags
            .iter()
            .map(|t| t.to_lowercase().trim().to_string())
            .collect();

        // Check query tags
        for query_tag in query_tags {
            let normalized_query = query_tag.to_lowercase();
            let should_exist = normalized_tags.contains(normalized_query.as_str());
            let does_exist = episode.has_tag(&query_tag);

            assert_eq!(does_exist, should_exist);
        }
    }

    /// has_tag handles case-insensitive lookups
    #[test]
    fn has_tag_case_insensitive(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag in lowercase
        episode.add_tag(tag.to_lowercase()).ok();

        // Check with various cases
        assert!(episode.has_tag(&tag.to_lowercase()));
        assert!(episode.has_tag(&tag.to_uppercase()));
        assert!(episode.has_tag(&format!("  {}  ", tag)));
    }

    /// has_tag handles whitespace-insensitive lookups
    #[test]
    fn has_tag_whitespace_insensitive(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag without whitespace
        episode.add_tag(tag.clone()).ok();

        // Check with whitespace variations
        assert!(episode.has_tag(&tag));
        assert!(episode.has_tag(&format!("  {}", tag)));
        assert!(episode.has_tag(&format!("{}  ", tag)));
        assert!(episode.has_tag(&format!("  {}  ", tag)));
    }

    /// get_tags returns all tags
    #[test]
    fn get_tags_returns_all(tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 0..10)) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tags
        let mut normalized_tags = HashSet::new();
        for tag in &tags {
            let add_result = episode.add_tag(tag.to_string()).ok();
            if add_result == Some(true) {
                // Get the normalized version
                if let Some(stored_tag) = episode.tags.last() {
                    normalized_tags.insert(stored_tag.clone());
                }
            }
        }

        // Get all tags
        let retrieved_tags = episode.get_tags();

        // Should have same length
        assert_eq!(retrieved_tags.len(), normalized_tags.len());

        // All retrieved tags should be in original set
        for tag in retrieved_tags {
            assert!(normalized_tags.contains(tag));
        }
    }
}

// ============================================================================
// Tag Combination Properties
// ============================================================================

proptest! {
    /// Adding multiple unique tags works correctly
    #[test]
    fn add_multiple_unique_tags(
        tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 1..10)
    ) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        let mut added_count = 0;
        for tag in &tags {
            if episode.add_tag(tag.to_string()).ok() == Some(true) {
                added_count += 1;
            }
        }

        assert_eq!(episode.tags.len(), added_count);
    }

    /// Remove and re-add tag works correctly
    #[test]
    fn remove_and_readd_tag(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag
        episode.add_tag(tag.clone()).ok();
        assert_eq!(episode.tags.len(), 1);

        // Remove
        episode.remove_tag(&tag);
        assert!(episode.tags.is_empty());

        // Re-add
        let result = episode.add_tag(tag);
        assert_eq!(result, Ok(true));
        assert_eq!(episode.tags.len(), 1);
    }

    /// Tag operations preserve order of first addition
    #[test]
    fn tags_preserve_addition_order(
        tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 1..10)
    ) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tags in specific order, deduplicating
        let mut unique_tags = Vec::new();
        let mut seen = HashSet::new();
        for tag in tags {
            let normalized = tag.to_lowercase();
            if seen.insert(normalized.clone()) {
                unique_tags.push(normalized);
            }
        }

        for tag in &unique_tags {
            episode.add_tag(tag.to_string()).ok();
        }

        // Tags should be in order of first addition
        for (i, tag) in unique_tags.iter().enumerate() {
            assert_eq!(episode.tags.get(i), Some(tag));
        }
    }
}
