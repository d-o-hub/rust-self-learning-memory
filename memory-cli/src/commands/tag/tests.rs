//! Tag command tests

use super::types::{
    TagAddResult, TagListResult, TagRemoveResult, TagSearchEpisode, TagSearchResult, TagSetResult,
    TagShowResult,
};
use crate::output::{Output, OutputFormat};

#[test]
fn test_tag_add_result_output() {
    let result = TagAddResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        tags_added: 2,
        current_tags: vec!["tag1".to_string(), "tag2".to_string()],
        success: true,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Added 2 tag(s)"));
    assert!(output.contains("tag1"));
    assert!(output.contains("tag2"));
}

#[test]
fn test_tag_add_result_json_output() {
    let result = TagAddResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        tags_added: 2,
        current_tags: vec!["tag1".to_string(), "tag2".to_string()],
        success: true,
    };

    let mut buffer = Vec::new();
    result.write_json(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["tags_added"], 2);
    assert_eq!(parsed["success"], true);
}

#[test]
fn test_tag_remove_result_output() {
    let result = TagRemoveResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        tags_removed: 1,
        current_tags: vec!["tag1".to_string()],
        success: true,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Removed 1 tag(s)"));
    assert!(output.contains("tag1"));
}

#[test]
fn test_tag_set_result_output() {
    let result = TagSetResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        tags_set: 3,
        current_tags: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        success: true,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Set 3 tag(s)"));
    assert!(output.contains("a"));
    assert!(output.contains("b"));
    assert!(output.contains("c"));
}

#[test]
fn test_tag_list_result_output() {
    let result = TagListResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        count: 2,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Tags for episode"));
    assert!(output.contains("Total: 2 tag(s)"));
    assert!(output.contains("tag1"));
    assert!(output.contains("tag2"));
}

#[test]
fn test_tag_list_result_empty_output() {
    let result = TagListResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        count: 0,
        tags: vec![],
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Tags for episode"));
    assert!(output.contains("Total: 0 tag(s)"));
}

#[test]
fn test_tag_search_result_output() {
    let episodes = vec![
        TagSearchEpisode {
            episode_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            task_description: "Test task 1".to_string(),
            task_type: "CodeGeneration".to_string(),
            tags: vec!["bug".to_string(), "critical".to_string()],
            start_time: 1234567890,
            end_time: Some(1234567900),
            outcome: Some("Success".to_string()),
        },
        TagSearchEpisode {
            episode_id: "550e8400-e29b-41d4-a716-446655440002".to_string(),
            task_description: "Test task 2".to_string(),
            task_type: "Refactoring".to_string(),
            tags: vec!["bug".to_string()],
            start_time: 1234567891,
            end_time: None,
            outcome: None,
        },
    ];

    let result = TagSearchResult {
        count: 2,
        episodes,
        search_criteria: "Any of: [bug]".to_string(),
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Found 2 episode(s)"));
    assert!(output.contains("Test task 1"));
    assert!(output.contains("Test task 2"));
    assert!(output.contains("[2 tags]"));
    assert!(output.contains("[1 tags]"));
}

#[test]
fn test_tag_show_result_output() {
    let result = TagShowResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        task_description: "Test task".to_string(),
        status: "completed".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        completed_at: Some("2023-01-01T01:00:00Z".to_string()),
        duration_ms: Some(3600000),
        outcome: Some("Success".to_string()),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        tags_count: 2,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Episode Details"));
    assert!(output.contains("Test task"));
    assert!(output.contains("completed"));
    assert!(output.contains("Tags:"));
    assert!(output.contains("tag1"));
    assert!(output.contains("tag2"));
}

#[test]
fn test_tag_show_result_in_progress_output() {
    let result = TagShowResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        task_description: "Test task".to_string(),
        status: "in_progress".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        completed_at: None,
        duration_ms: None,
        outcome: None,
        tags: vec![],
        tags_count: 0,
    };

    let mut buffer = Vec::new();
    result.write_human(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("Episode Details"));
    assert!(output.contains("in_progress"));
    assert!(output.contains("(none)"));
}

#[test]
fn test_tag_search_result_json_output() {
    let episodes = vec![TagSearchEpisode {
        episode_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
        task_description: "Test task".to_string(),
        task_type: "CodeGeneration".to_string(),
        tags: vec!["bug".to_string()],
        start_time: 1234567890,
        end_time: Some(1234567900),
        outcome: Some("Success".to_string()),
    }];

    let result = TagSearchResult {
        count: 1,
        episodes,
        search_criteria: "Any of: [bug]".to_string(),
    };

    let mut buffer = Vec::new();
    result.write_json(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["count"], 1);
    assert!(parsed["episodes"].as_array().unwrap().len() > 0);
}

#[test]
fn test_tag_show_result_json_output() {
    let result = TagShowResult {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        task_description: "Test task".to_string(),
        status: "completed".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        completed_at: Some("2023-01-01T01:00:00Z".to_string()),
        duration_ms: Some(3600000),
        outcome: Some("Success".to_string()),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        tags_count: 2,
    };

    let mut buffer = Vec::new();
    result.write_json(&mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["episode_id"], "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(parsed["tags_count"], 2);
}
