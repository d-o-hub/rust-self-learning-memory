//! Tests for relationship command result types and their `Output` rendering.
//!
//! Declared from `relationships/mod.rs` as `#[cfg(test)] mod types_tests;` so
//! the source file `types.rs` stays within the 500-LOC gate.

use super::types::InfoResult;
use crate::output::Output;

fn render_human(result: &InfoResult) -> String {
    let mut buf = Vec::new();
    result.write_human(&mut buf).unwrap();
    // Strip ANSI escape sequences (ESC [ ... m) so assertions are color-independent.
    let raw = String::from_utf8_lossy(&buf);
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn base_result() -> InfoResult {
    InfoResult {
        relationship_id: "rel-111".to_string(),
        relationship_type: "DependsOn".to_string(),
        source_episode_id: "src-aaa".to_string(),
        target_episode_id: "tgt-bbb".to_string(),
        source_task: "Build the widget".to_string(),
        target_task: "Test the widget".to_string(),
        priority: None,
        reason: None,
        created_by: None,
        created_at: None,
        custom_fields: vec![],
    }
}

#[test]
fn test_info_result_human_contains_ids_and_type() {
    let result = base_result();
    let output = render_human(&result);
    assert!(output.contains("rel-111"));
    assert!(output.contains("DependsOn"));
    assert!(output.contains("src-aaa"));
    assert!(output.contains("tgt-bbb"));
    assert!(output.contains("Build the widget"));
    assert!(output.contains("Test the widget"));
}

#[test]
fn test_info_result_human_no_metadata_section_when_empty() {
    let result = base_result();
    let output = render_human(&result);
    assert!(!output.contains("Metadata"));
}

#[test]
fn test_info_result_human_metadata_section_with_all_fields() {
    let mut result = base_result();
    result.priority = Some(7);
    result.reason = Some("needed for deploy".to_string());
    result.created_by = Some("alice".to_string());
    result.created_at = Some("2026-07-24T00:00:00Z".to_string());
    result.custom_fields = vec![("env".to_string(), "prod".to_string())];

    let output = render_human(&result);
    assert!(output.contains("Metadata"));
    assert!(output.contains("7"));
    assert!(output.contains("needed for deploy"));
    assert!(output.contains("alice"));
    assert!(output.contains("2026-07-24T00:00:00Z"));
    assert!(output.contains("env"));
    assert!(output.contains("prod"));
}

#[test]
fn test_info_result_human_empty_source_task_shows_placeholder() {
    let mut result = base_result();
    result.source_task = String::new();
    let output = render_human(&result);
    assert!(output.contains("(no description)"));
}

#[test]
fn test_info_result_human_empty_target_task_shows_placeholder() {
    let mut result = base_result();
    result.target_task = String::new();
    let output = render_human(&result);
    assert!(output.contains("(no description)"));
}

#[test]
fn test_info_result_human_long_source_task_is_truncated() {
    let mut result = base_result();
    result.source_task = "a".repeat(60);
    let output = render_human(&result);
    assert!(output.contains('…'));
}

#[test]
fn test_info_result_human_long_target_task_is_truncated() {
    let mut result = base_result();
    result.target_task = "b".repeat(60);
    let output = render_human(&result);
    assert!(output.contains('…'));
}

#[test]
fn test_info_result_human_metadata_only_priority() {
    let mut result = base_result();
    result.priority = Some(3);
    let output = render_human(&result);
    assert!(output.contains("Metadata"));
    assert!(output.contains("3"));
    assert!(!output.contains("Reason"));
}

#[test]
fn test_info_result_human_box_frame_present() {
    let result = base_result();
    let output = render_human(&result);
    assert!(output.contains("┌─"));
    assert!(output.contains("└─"));
    assert!(output.contains("├─"));
}
