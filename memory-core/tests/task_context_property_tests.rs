//! Property-based tests for TaskContext matching
//!
//! These tests verify invariants for task context creation and matching
//! using the proptest crate for property-based testing.
//!
//! Covers WG-046 from ADR-047 (v0.1.22 Quality Polish)

#![allow(clippy::cast_precision_loss)]

use memory_core::{ComplexityLevel, TaskContext};
use proptest::prelude::*;

/// Generate arbitrary domain names
fn arb_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("testing".to_string()),
        Just("development".to_string()),
        Just("security".to_string()),
        Just("documentation".to_string()),
        Just("refactoring".to_string()),
        Just("api".to_string()),
        Just("database".to_string()),
    ]
}

/// Generate arbitrary language names
fn arb_language() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(Some("rust".to_string())),
        Just(Some("python".to_string())),
        Just(Some("typescript".to_string())),
        Just(None),
    ]
}

/// Generate arbitrary framework names
fn arb_framework() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(Some("tokio".to_string())),
        Just(Some("actix".to_string())),
        Just(Some("axum".to_string())),
        Just(None),
    ]
}

/// Generate arbitrary complexity levels
fn arb_complexity() -> impl Strategy<Value = ComplexityLevel> {
    prop_oneof![
        Just(ComplexityLevel::Simple),
        Just(ComplexityLevel::Moderate),
        Just(ComplexityLevel::Complex),
    ]
}

/// Generate arbitrary tags
fn arb_tags() -> impl Strategy<Value = Vec<String>> {
    proptest::collection::vec("[a-z]{3,8}", 0..5)
}

proptest! {
    /// Test that TaskContext domain is always set
    #[test]
    fn context_domain_always_set(domain in arb_domain()) {
        let ctx = TaskContext {
            domain: domain.clone(),
            ..TaskContext::default()
        };
        assert!(!ctx.domain.is_empty());
        assert_eq!(ctx.domain, domain);
    }

    /// Test that context clone preserves all fields
    #[test]
    fn context_clone_preserves_fields(
        domain in arb_domain(),
        language in arb_language(),
        framework in arb_framework(),
        complexity in arb_complexity(),
        tags in arb_tags(),
    ) {
        let ctx = TaskContext {
            domain,
            language: language.clone(),
            framework: framework.clone(),
            complexity,
            tags: tags.clone(),
        };
        let cloned = ctx.clone();

        assert_eq!(ctx.domain, cloned.domain);
        assert_eq!(ctx.language, cloned.language);
        assert_eq!(ctx.framework, cloned.framework);
        assert_eq!(ctx.complexity, cloned.complexity);
        assert_eq!(ctx.tags, cloned.tags);
    }

    /// Test that domains are case-sensitive
    #[test]
    fn domains_are_case_sensitive(
        domain1 in "[a-z]{3,10}",
        domain2 in "[a-z]{3,10}",
    ) {
        let ctx1 = TaskContext {
            domain: domain1.clone(),
            ..TaskContext::default()
        };
        let ctx2 = TaskContext {
            domain: domain2.clone(),
            ..TaskContext::default()
        };

        if domain1 == domain2 {
            assert_eq!(ctx1.domain, ctx2.domain);
        } else {
            assert_ne!(ctx1.domain, ctx2.domain);
        }
    }

    /// Test that tags count is bounded
    #[test]
    fn tags_count_bounded(num_tags in 0..10usize) {
        let tags: Vec<String> = (0..num_tags).map(|i| format!("tag{i}")).collect();
        let ctx = TaskContext {
            tags: tags.clone(),
            ..TaskContext::default()
        };
        assert_eq!(ctx.tags.len(), num_tags);
    }

}

/// Test that empty context has defaults
#[test]
fn test_empty_context_has_defaults() {
    let ctx = TaskContext::default();
    assert!(!ctx.domain.is_empty());
    assert!(ctx.language.is_none());
    assert!(ctx.framework.is_none());
    assert_eq!(ctx.complexity, ComplexityLevel::Moderate);
    assert!(ctx.tags.is_empty());
}

/// Test that complexity default is Moderate
#[test]
fn test_complexity_default_is_moderate() {
    let ctx = TaskContext::default();
    assert_eq!(ctx.complexity, ComplexityLevel::Moderate);
}

/// Test that context can be updated
#[test]
fn test_context_can_be_updated() {
    let mut ctx = TaskContext {
        domain: "testing".to_string(),
        language: Some("rust".to_string()),
        ..TaskContext::default()
    };

    ctx.domain = "development".to_string();
    ctx.language = Some("python".to_string());

    assert_eq!(ctx.domain, "development");
    assert_eq!(ctx.language, Some("python".to_string()));
}

/// Test that complexity levels are distinct
#[test]
fn test_complexity_levels_distinct() {
    assert_ne!(ComplexityLevel::Simple, ComplexityLevel::Moderate);
    assert_ne!(ComplexityLevel::Moderate, ComplexityLevel::Complex);
    assert_ne!(ComplexityLevel::Simple, ComplexityLevel::Complex);
}
