//! Tests for reconstructive retrieval windows (WG-120).

use super::*;
use uuid::Uuid;

#[test]
fn test_window_config_default() {
    let config = WindowConfig::default();
    assert_eq!(config.backward_steps, 3);
    assert_eq!(config.forward_steps, 2);
    assert_eq!(config.max_window_size, 20);
    assert!(config.include_episode_context);
    assert!((config.min_score_threshold - 0.3).abs() < 0.01);
}

#[test]
fn test_window_config_compact() {
    let config = WindowConfig::compact();
    assert_eq!(config.backward_steps, 1);
    assert_eq!(config.forward_steps, 1);
    assert_eq!(config.max_window_size, 10);
    assert!(!config.include_episode_context);
}

#[test]
fn test_window_config_comprehensive() {
    let config = WindowConfig::comprehensive();
    assert_eq!(config.backward_steps, 5);
    assert_eq!(config.forward_steps, 5);
    assert_eq!(config.max_window_size, 50);
    assert!(config.include_episode_context);
}

#[test]
fn test_retrieval_hit_creation() {
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::new(episode_id, 0.85, "bm25".to_string());
    assert_eq!(hit.episode_id, episode_id);
    assert_eq!(hit.score, 0.85);
    assert_eq!(hit.source_tier, "bm25");
    assert!(hit.step_number.is_none());
}

#[test]
fn test_retrieval_hit_with_step() {
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::with_step(episode_id, 5, 0.75, "hdc".to_string());
    assert_eq!(hit.episode_id, episode_id);
    assert_eq!(hit.step_number, Some(5));
    assert_eq!(hit.score, 0.75);
}

#[test]
fn test_retrieval_hit_threshold() {
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::new(episode_id, 0.4, "bm25".to_string());
    assert!(hit.meets_threshold(0.3));
    assert!(!hit.meets_threshold(0.5));
}

#[test]
fn test_window_expander_creation() {
    let config = WindowConfig::default();
    let expander = WindowExpander::new(config);
    assert_eq!(expander.config().backward_steps, 3);
}

#[test]
fn test_window_expander_default() {
    let expander = WindowExpander::default_config();
    assert_eq!(expander.config().backward_steps, 3);
}

#[test]
fn test_expand_empty_hits() {
    let expander = WindowExpander::default_config();
    let result = expander.expand(&[], &[]);
    assert!(result.is_empty());
    assert_eq!(result.original_hit_count, 0);
}

#[test]
fn test_expand_single_hit() {
    let expander = WindowExpander::default_config();
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::with_step(episode_id, 10, 0.8, "bm25".to_string());
    let step_counts = [(episode_id, 20)];

    let result = expander.expand(&[hit], &step_counts);

    assert_eq!(result.len(), 1);
    assert_eq!(result.original_hit_count, 1);
    assert!(!result.truncated);

    let window = &result.windows[0];
    assert_eq!(window.episode_id, episode_id);
    // Window should be centered at step 10: 7-12 (backward 3, forward 2)
    assert_eq!(window.start_step, 7);
    assert_eq!(window.end_step, 12);
    assert_eq!(window.step_count, 6);
}

#[test]
fn test_expand_near_episode_start() {
    let expander = WindowExpander::default_config();
    let episode_id = Uuid::new_v4();
    // Hit at step 2, backward_steps=3 would go to -1, saturating_sub to 0
    let hit = RetrievalHit::with_step(episode_id, 2, 0.8, "bm25".to_string());
    let step_counts = [(episode_id, 10)];

    let result = expander.expand(&[hit], &step_counts);

    let window = &result.windows[0];
    // start_step should be 0 (saturating_sub)
    assert_eq!(window.start_step, 0);
    assert_eq!(window.end_step, 4); // 2 + 2 forward
}

#[test]
fn test_expand_near_episode_end() {
    let expander = WindowExpander::default_config();
    let episode_id = Uuid::new_v4();
    // Hit at step 18, episode has 20 steps
    let hit = RetrievalHit::with_step(episode_id, 18, 0.8, "bm25".to_string());
    let step_counts = [(episode_id, 20)];

    let result = expander.expand(&[hit], &step_counts);

    let window = &result.windows[0];
    assert_eq!(window.start_step, 15); // 18 - 3
    assert_eq!(window.end_step, 20); // min(20, 20) = 20
}

#[test]
fn test_expand_truncation() {
    let config = WindowConfig {
        backward_steps: 10,
        forward_steps: 10,
        max_window_size: 5,
        include_episode_context: true,
        min_score_threshold: 0.3,
    };
    let expander = WindowExpander::new(config);
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::with_step(episode_id, 15, 0.8, "bm25".to_string());
    let step_counts = [(episode_id, 30)];

    let result = expander.expand(&[hit], &step_counts);

    assert!(result.truncated);
    assert!(result.windows[0].step_count <= 5);
}

#[test]
fn test_expand_below_threshold() {
    let expander = WindowExpander::default_config();
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::new(episode_id, 0.1, "bm25".to_string()); // Below 0.3 threshold
    let step_counts = [(episode_id, 10)];

    let result = expander.expand(&[hit], &step_counts);

    // Hit should be filtered out
    assert!(result.is_empty());
}

#[test]
fn test_expand_multiple_hits() {
    let expander = WindowExpander::default_config();
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();

    let hits = [
        RetrievalHit::with_step(ep1, 5, 0.8, "bm25".to_string()),
        RetrievalHit::with_step(ep2, 10, 0.7, "hdc".to_string()),
    ];
    let step_counts = [(ep1, 10), (ep2, 15)];

    let result = expander.expand(&hits, &step_counts);

    assert_eq!(result.len(), 2);
    assert_eq!(result.original_hit_count, 2);
}

#[test]
fn test_expand_with_context() {
    let expander = WindowExpander::default_config();
    let episode_id = Uuid::new_v4();
    let hit = RetrievalHit::with_step(episode_id, 5, 0.8, "bm25".to_string());
    let episode_data = [(episode_id, 10, "Fix authentication bug".to_string())];

    let result = expander.expand_with_context(&[hit], &episode_data);

    assert_eq!(result.len(), 1);
    assert!(result.windows[0].episode_context.is_some());
    assert_eq!(
        result.windows[0].episode_context.as_ref().unwrap(),
        "Fix authentication bug"
    );
}

#[test]
fn test_context_window_helpers() {
    let episode_id = Uuid::new_v4();
    let window = ContextWindow {
        hit: RetrievalHit::new(episode_id, 0.8, "bm25".to_string()),
        episode_id,
        start_step: 5,
        end_step: 10,
        episode_context: Some("Test episode".to_string()),
        window_score: 0.8,
        step_count: 6,
    };

    assert_eq!(window.size(), 6);
    assert!(!window.is_empty());
    assert_eq!(window.range(), (5, 10));
}

#[test]
fn test_window_expansion_result_helpers() {
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();
    let result = WindowExpansionResult {
        windows: vec![
            ContextWindow {
                hit: RetrievalHit::new(ep1, 0.8, "bm25".to_string()),
                episode_id: ep1,
                start_step: 1,
                end_step: 5,
                episode_context: None,
                window_score: 0.8,
                step_count: 5,
            },
            ContextWindow {
                hit: RetrievalHit::new(ep2, 0.7, "hdc".to_string()),
                episode_id: ep2,
                start_step: 3,
                end_step: 8,
                episode_context: None,
                window_score: 0.7,
                step_count: 6,
            },
        ],
        total_steps: 11,
        original_hit_count: 2,
        truncated: false,
    };

    assert_eq!(result.episode_ids(), vec![ep1, ep2]);
    assert_eq!(result.len(), 2);
    assert!(!result.is_empty());
}

#[test]
fn test_merge_single_window() {
    let episode_id = Uuid::new_v4();
    let window = ContextWindow {
        hit: RetrievalHit::new(episode_id, 0.8, "bm25".to_string()),
        episode_id,
        start_step: 5,
        end_step: 10,
        episode_context: None,
        window_score: 0.8,
        step_count: 6,
    };

    let merged = merge_overlapping_windows(std::slice::from_ref(&window));
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].episode_id, episode_id);
}

#[test]
fn test_merge_overlapping_same_episode() {
    let episode_id = Uuid::new_v4();
    let windows = [
        ContextWindow {
            hit: RetrievalHit::with_step(episode_id, 5, 0.8, "bm25".to_string()),
            episode_id,
            start_step: 2,
            end_step: 7,
            episode_context: Some("Test".to_string()),
            window_score: 0.8,
            step_count: 6,
        },
        ContextWindow {
            hit: RetrievalHit::with_step(episode_id, 8, 0.7, "hdc".to_string()),
            episode_id,
            start_step: 5,
            end_step: 10,
            episode_context: None,
            window_score: 0.7,
            step_count: 6,
        },
    ];

    let merged = merge_overlapping_windows(&windows);

    // Should merge into one window covering both ranges
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].start_step, 2); // min of both
    assert_eq!(merged[0].end_step, 10); // max of both
    // Should use best score
    assert!((merged[0].window_score - 0.8).abs() < 0.01);
}

#[test]
fn test_merge_separate_episodes() {
    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();

    let windows = [
        ContextWindow {
            hit: RetrievalHit::new(ep1, 0.8, "bm25".to_string()),
            episode_id: ep1,
            start_step: 1,
            end_step: 5,
            episode_context: None,
            window_score: 0.8,
            step_count: 5,
        },
        ContextWindow {
            hit: RetrievalHit::new(ep2, 0.7, "hdc".to_string()),
            episode_id: ep2,
            start_step: 3,
            end_step: 8,
            episode_context: None,
            window_score: 0.7,
            step_count: 6,
        },
    ];

    let merged = merge_overlapping_windows(&windows);

    // Should keep separate windows for different episodes
    assert_eq!(merged.len(), 2);
    // Should be sorted by score
    assert!((merged[0].window_score - 0.8).abs() < 0.01);
    assert!((merged[1].window_score - 0.7).abs() < 0.01);
}

#[test]
fn test_estimate_token_savings() {
    let expander = WindowExpander::default_config();

    // Full episode has 100 steps, window has 20 steps
    let savings = expander.estimate_token_savings(100, 20);
    assert!((savings - 80.0).abs() < 1.0); // 80% savings

    // Full episode has 50 steps, window has 10 steps
    let savings = expander.estimate_token_savings(50, 10);
    assert!((savings - 80.0).abs() < 1.0);

    // No savings when window equals full episode
    let savings = expander.estimate_token_savings(20, 20);
    assert!((savings - 0.0).abs() < 1.0);
}

#[test]
fn test_empty_result_helpers() {
    let result = WindowExpansionResult::empty();
    assert!(result.is_empty());
    assert_eq!(result.len(), 0);
    assert!(result.episode_ids().is_empty());
}
