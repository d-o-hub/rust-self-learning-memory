use super::*;
use crate::episode::Episode;
use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
use crate::ExecutionStep;

// Helper function to create a test episode
fn create_test_episode(
    description: &str,
    task_type: TaskType,
    steps: Vec<ExecutionStep>,
    outcome: Option<TaskOutcome>,
) -> Episode {
    let mut episode = Episode::new(description.to_string(), TaskContext::default(), task_type);
    for step in steps {
        episode.add_step(step);
    }
    if let Some(outcome) = outcome {
        episode.complete(outcome);
    }
    episode
}

fn successful_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Success".to_string(),
    });
    step.latency_ms = 100;
    step.tokens_used = Some(50);
    step
}

fn failed_step(step_number: usize, tool: &str, action: &str, error_msg: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Error {
        message: error_msg.to_string(),
    });
    step.latency_ms = 200;
    step.tokens_used = Some(25);
    step
}

#[test]
fn test_success_analyzer_edge_cases() {
    // Case 1: Partial success with no completed subtasks
    let steps = vec![successful_step(1, "tool", "action")];
    let outcome = TaskOutcome::PartialSuccess {
        verdict: "Partial".to_string(),
        completed: vec![],
        failed: vec!["something".to_string()],
    };
    let episode = create_test_episode("Edge 1", TaskType::Testing, steps.clone(), Some(outcome));
    let successes = success_analyzer::identify_successes(&episode, 5);
    // Should contain verdict but NOT "Completed X subtask(s)"
    assert!(successes.iter().any(|s| s.contains("Partial success: Partial")));
    assert!(!successes.iter().any(|s| s.contains("Completed")));

    // Case 2: Success with no artifacts
    let outcome = TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    };
    let episode = create_test_episode("Edge 2", TaskType::Testing, steps.clone(), Some(outcome));
    let successes = success_analyzer::identify_successes(&episode, 5);
    // Should contain verdict but NOT "Generated X artifact(s)"
    assert!(successes.iter().any(|s| s.contains("Successfully completed task")));
    assert!(!successes.iter().any(|s| s.contains("Generated")));

    // Case 3: Success rate just below 0.8 (e.g., 3 success, 1 fail = 0.75)
    let mut steps = vec![];
    for i in 1..=3 { steps.push(successful_step(i, "tool", "action")); }
    steps.push(failed_step(4, "tool", "action", "fail"));
    
    let outcome = TaskOutcome::PartialSuccess { verdict: "p".to_string(), completed: vec![], failed: vec![] };
    let episode = create_test_episode("Edge 3", TaskType::Testing, steps, Some(outcome));
    let successes = success_analyzer::identify_successes(&episode, 5);
    // Should NOT contain high execution success rate
    assert!(!successes.iter().any(|s| s.contains("High execution success rate")));
}

#[test]
fn test_success_patterns_edge_cases() {
    // Case 1: Tool sequence < 2 steps
    let steps = vec![successful_step(1, "tool1", "action1")];
    let outcome = TaskOutcome::Success { verdict: "ok".to_string(), artifacts: vec![] };
    let episode = create_test_episode("Pattern 1", TaskType::Testing, steps, Some(outcome.clone()));
    let patterns = success_analyzer::analyze_success_patterns(&episode);
    // Should NOT contain tool sequence (requires >= 3 steps)
    assert!(!patterns.iter().any(|p| p.contains("Effective tool sequence")));

    // Case 2: Focused tool strategy (repetition)
    let steps = vec![
        successful_step(1, "tool1", "action1"),
        successful_step(2, "tool1", "action2"),
        successful_step(3, "tool1", "action3"),
    ];
    let episode = create_test_episode("Pattern 2", TaskType::Testing, steps, Some(outcome.clone()));
    let patterns = success_analyzer::analyze_success_patterns(&episode);
    assert!(patterns.iter().any(|p| p.contains("focused tool strategy")));

    // Case 3: Context success factors - language set but few steps
    let context = TaskContext {
        language: Some("Rust".to_string()),
        domain: "coding".to_string(),
        ..Default::default()
    };
    let steps = vec![successful_step(1, "tool", "action")];
    let mut episode = Episode::new("Pattern 3".to_string(), context, TaskType::CodeGeneration);
    episode.add_step(steps[0].clone());
    episode.complete(outcome);
    let patterns = success_analyzer::analyze_success_patterns(&episode);
    // Should NOT contain language specific success factor (needs > 5 steps)
    assert!(!patterns.iter().any(|p| p.contains("Successfully leveraged Rust-specific")));
}

#[test]
fn test_improvement_analyzer_edge_cases() {
    // Case 1: Duration between 30s and 300s (no improvement needed)
    let steps = vec![successful_step(1, "tool", "action")];
    let outcome = TaskOutcome::Success { verdict: "ok".to_string(), artifacts: vec![] };
    let mut episode = create_test_episode("Imp 1", TaskType::Testing, steps, Some(outcome));
    episode.end_time = Some(episode.start_time + chrono::Duration::seconds(100));
    
    let improvements = improvement_analyzer::identify_improvements(&episode, 5);
    assert!(!improvements.iter().any(|i| i.contains("Optimize execution time")));

    // Case 2: Failed steps but no repeated tool failure (count < 2 for each tool)
    let steps = vec![
        failed_step(1, "tool1", "action", "fail"),
        failed_step(2, "tool2", "action", "fail"),
    ];
    let outcome = TaskOutcome::Failure { reason: "f".to_string(), error_details: None };
    let episode = create_test_episode("Imp 2", TaskType::Testing, steps, Some(outcome));
    let improvements = improvement_analyzer::identify_improvements(&episode, 5);
    // Should have "Reduce failed execution steps" but NO "Tool 'X' failed Y times"
    assert!(improvements.iter().any(|i| i.contains("Reduce failed execution steps")));
    assert!(!improvements.iter().any(|i| i.contains("failed 2 times")));
}

#[test]
fn test_improvement_opportunities_edge_cases() {
    // Case 1: Bottleneck threshold (max < 3 * avg)
    let mut s1 = successful_step(1, "tool", "action"); s1.latency_ms = 100;
    let mut s2 = successful_step(2, "tool", "action"); s2.latency_ms = 200; // avg = 150. max = 200. 200 < 450.
    let steps = vec![s1, s2];
    let outcome = TaskOutcome::Success { verdict: "ok".to_string(), artifacts: vec![] };
    let episode = create_test_episode("Opp 1", TaskType::Testing, steps, Some(outcome.clone()));
    let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);
    assert!(!opportunities.iter().any(|o| o.contains("Performance bottleneck")));

    // Case 2: Redundancy just below threshold (count = 4)
    let steps: Vec<_> = (0..4).map(|i| successful_step(i, "tool", "action")).collect();
    let episode = create_test_episode("Opp 2", TaskType::Testing, steps, Some(outcome.clone()));
    let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);
    assert!(!opportunities.iter().any(|o| o.contains("High repetition")));

    // Case 3: Optimization opportunities (consecutive same type < 2 pairs)
    let steps = vec![
        successful_step(1, "tool1", "action"),
        successful_step(2, "tool2", "action"),
        successful_step(3, "tool3", "action"),
    ];
    let episode = create_test_episode("Opp 3", TaskType::Testing, steps, Some(outcome.clone()));
    let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);
    assert!(!opportunities.iter().any(|o| o.contains("Potential for parallelization")));

    // Case 4: Resource utilization normal (tokens between 1000 and 10000)
    let mut s1 = successful_step(1, "tool", "action"); s1.tokens_used = Some(5000);
    let steps = vec![s1];
    let episode = create_test_episode("Opp 4", TaskType::Testing, steps, Some(outcome));
    let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);
    assert!(!opportunities.iter().any(|o| o.contains("token usage")));
}
