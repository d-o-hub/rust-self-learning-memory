use super::*;
use crate::episode::{Episode, ExecutionStep};
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

fn create_test_episode() -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "testing".to_string(),
        tags: vec!["async".to_string()],
    };
    Episode::new("Test task".to_string(), context, TaskType::Testing)
}

fn complete_episode(episode: &mut Episode) {
    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });
}

#[tokio::test]
async fn test_decision_point_keywords_coverage() {
    let extractor = DecisionPointExtractor::new();
    let keywords = vec!["if condition", "when event", "ensure state", "decide path", "determine value"];
    
    let mut episode = create_test_episode();
    for (i, kw) in keywords.iter().enumerate() {
        let mut step = ExecutionStep::new(i+1, "tool".to_string(), kw.to_string());
        step.result = Some(ExecutionResult::Success { output: "ok".to_string() });
        step.latency_ms = 10;
        episode.add_step(step);
    }
    complete_episode(&mut episode);
    
    let patterns = extractor.extract(&episode).await.unwrap();
    assert_eq!(patterns.len(), keywords.len());
}

#[tokio::test]
async fn test_extractors_incomplete_episode() {
    let episode = create_test_episode(); // Not completed
    
    let dp_extractor = DecisionPointExtractor::new();
    assert!(dp_extractor.extract(&episode).await.unwrap().is_empty());
    
    let ts_extractor = ToolSequenceExtractor::new();
    assert!(ts_extractor.extract(&episode).await.unwrap().is_empty());
}

#[tokio::test]
async fn test_tool_sequence_success_rate_threshold() {
    let extractor = ToolSequenceExtractor::new(); // default threshold 0.7
    let mut episode = create_test_episode();
    
    // 2 steps, 1 success, 1 failure -> 0.5 success rate (< 0.7)
    let mut s1 = ExecutionStep::new(1, "t".to_string(), "a".to_string());
    s1.result = Some(ExecutionResult::Success { output: "ok".to_string() });
    s1.latency_ms = 10;
    
    let mut s2 = ExecutionStep::new(2, "t".to_string(), "a".to_string());
    s2.result = Some(ExecutionResult::Error { message: "fail".to_string() });
    s2.latency_ms = 10;
    
    episode.add_step(s1);
    episode.add_step(s2);
    complete_episode(&mut episode);
    
    let patterns = extractor.extract(&episode).await.unwrap();
    assert!(patterns.is_empty());
}
