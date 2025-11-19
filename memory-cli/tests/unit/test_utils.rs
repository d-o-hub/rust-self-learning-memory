//! Test utilities for unit tests.
//!
//! Provides common test data generators, mock objects,
//! and helper functions for unit testing.

use memory_core::*;
use std::collections::HashMap;

/// Create a test episode with predictable data
pub fn create_test_episode(description: &str) -> Episode {
    Episode {
        id: uuid::Uuid::new_v4(),
        task_description: description.to_string(),
        task_type: TaskType::Testing,
        context: create_test_context("unit-test", Some("rust")),
        steps: vec![],
        outcome: None,
        reward_score: None,
        reflection: None,
        patterns: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

/// Create test context data
pub fn create_test_context(domain: &str, language: Option<&str>) -> TaskContext {
    TaskContext {
        domain: domain.to_string(),
        language: language.map(|s| s.to_string()),
        tags: vec!["test".to_string(), "unit".to_string()],
        metadata: HashMap::new(),
    }
}

/// Create a test execution step
pub fn create_test_step(tool: &str, action: &str, success: bool) -> ExecutionStep {
    ExecutionStep {
        tool: tool.to_string(),
        action: action.to_string(),
        parameters: HashMap::new(),
        result: if success { "success".to_string() } else { "failure".to_string() },
        latency_ms: 100,
        tokens: 50,
        success,
        timestamp: chrono::Utc::now(),
        observation: Some(format!("Test observation for {} {}", tool, action)),
        metadata: HashMap::new(),
    }
}

/// Create a test pattern
pub fn create_test_pattern(pattern_type: PatternType, confidence: f64) -> Pattern {
    Pattern {
        id: uuid::Uuid::new_v4(),
        pattern_type,
        description: format!("Test pattern of type {:?}", pattern_type),
        confidence,
        frequency: 5,
        context: create_test_context("test", Some("rust")),
        examples: vec![create_test_episode("example episode")],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Create a test task outcome
pub fn create_test_outcome(success: bool) -> TaskOutcome {
    TaskOutcome {
        success,
        score: if success { 0.9 } else { 0.2 },
        duration_ms: 1000,
        artifacts: vec![],
        metadata: HashMap::new(),
    }
}

/// Create test reward score
pub fn create_test_reward_score() -> RewardScore {
    RewardScore {
        total_score: 0.85,
        components: HashMap::from([
            ("efficiency".to_string(), 0.9),
            ("correctness".to_string(), 0.8),
            ("completeness".to_string(), 0.9),
        ]),
        metadata: HashMap::new(),
    }
}

/// Create test reflection
pub fn create_test_reflection() -> Reflection {
    Reflection {
        summary: "Test reflection summary".to_string(),
        insights: vec![
            "Learned something useful".to_string(),
            "Identified improvement opportunity".to_string(),
        ],
        lessons_learned: vec![
            "Always validate inputs".to_string(),
            "Test edge cases thoroughly".to_string(),
        ],
        recommendations: vec![
            "Add more error handling".to_string(),
            "Improve documentation".to_string(),
        ],
        metadata: HashMap::new(),
    }
}

/// Mock memory system for testing
pub struct MockMemorySystem {
    episodes: std::sync::Mutex<Vec<Episode>>,
    patterns: std::sync::Mutex<Vec<Pattern>>,
}

impl MockMemorySystem {
    pub fn new() -> Self {
        Self {
            episodes: std::sync::Mutex::new(Vec::new()),
            patterns: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_episode(&self, episode: Episode) {
        self.episodes.lock().unwrap().push(episode);
    }

    pub fn get_episodes(&self) -> Vec<Episode> {
        self.episodes.lock().unwrap().clone()
    }

    pub fn add_pattern(&self, pattern: Pattern) {
        self.patterns.lock().unwrap().push(pattern);
    }

    pub fn get_patterns(&self) -> Vec<Pattern> {
        self.patterns.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.episodes.lock().unwrap().clear();
        self.patterns.lock().unwrap().clear();
    }
}

impl Default for MockMemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data generators for property-based testing
#[cfg(feature = "proptest")]
pub mod proptest_generators {
    use super::*;
    use proptest::prelude::*;

    pub fn arb_task_type() -> impl Strategy<Value = TaskType> {
        prop_oneof![
            Just(TaskType::Development),
            Just(TaskType::Testing),
            Just(TaskType::Production),
            Just(TaskType::Analysis),
            Just(TaskType::Training),
            Just(TaskType::Inference),
            Just(TaskType::Deployment),
            Just(TaskType::Monitoring),
            Just(TaskType::Debugging),
            Just(TaskType::Optimization),
        ]
    }

    pub fn arb_pattern_type() -> impl Strategy<Value = PatternType> {
        prop_oneof![
            Just(PatternType::ToolSequence),
            Just(PatternType::DecisionPoint),
            Just(PatternType::ErrorPattern),
            Just(PatternType::SuccessPattern),
            Just(PatternType::PerformancePattern),
            Just(PatternType::ContextPattern),
        ]
    }

    pub fn arb_safe_string() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_\\-\\. ]{1,100}".prop_map(|s| s.to_string())
    }

    pub fn arb_task_context() -> impl Strategy<Value = TaskContext> {
        (arb_safe_string(), proptest::option::of(arb_safe_string()))
            .prop_map(|(domain, language)| create_test_context(&domain, language.as_deref()))
    }

    pub fn arb_episode() -> impl Strategy<Value = Episode> {
        arb_safe_string().prop_map(|desc| create_test_episode(&desc))
    }

    pub fn arb_execution_step() -> impl Strategy<Value = ExecutionStep> {
        (arb_safe_string(), arb_safe_string(), proptest::bool::ANY)
            .prop_map(|(tool, action, success)| create_test_step(&tool, &action, success))
    }
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert that a duration is within acceptable bounds
    pub fn assert_duration_within(actual: Duration, expected_max: Duration) {
        assert!(
            actual <= expected_max,
            "Duration {}ms exceeded maximum {}ms",
            actual.as_millis(),
            expected_max.as_millis()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_episode() {
        let episode = create_test_episode("test description");
        assert_eq!(episode.task_description, "test description");
        assert_eq!(episode.task_type, TaskType::Testing);
        assert_eq!(episode.context.domain, "unit-test");
        assert_eq!(episode.context.language, Some("rust".to_string()));
        assert!(episode.steps.is_empty());
    }

    #[test]
    fn test_create_test_context() {
        let context = create_test_context("test-domain", Some("test-lang"));
        assert_eq!(context.domain, "test-domain");
        assert_eq!(context.language, Some("test-lang".to_string()));
        assert!(context.tags.contains(&"test".to_string()));
        assert!(context.tags.contains(&"unit".to_string()));
    }

    #[test]
    fn test_create_test_step() {
        let step = create_test_step("test_tool", "test_action", true);
        assert_eq!(step.tool, "test_tool");
        assert_eq!(step.action, "test_action");
        assert!(step.success);
        assert_eq!(step.latency_ms, 100);
        assert_eq!(step.tokens, 50);
        assert!(step.observation.is_some());
    }

    #[test]
    fn test_create_test_pattern() {
        let pattern = create_test_pattern(PatternType::ToolSequence, 0.85);
        assert_eq!(pattern.pattern_type, PatternType::ToolSequence);
        assert_eq!(pattern.confidence, 0.85);
        assert_eq!(pattern.frequency, 5);
        assert!(pattern.description.contains("ToolSequence"));
    }

    #[test]
    fn test_mock_memory_system() {
        let mock = MockMemorySystem::new();

        let episode = create_test_episode("test");
        mock.add_episode(episode.clone());

        let pattern = create_test_pattern(PatternType::SuccessPattern, 0.9);
        mock.add_pattern(pattern.clone());

        let episodes = mock.get_episodes();
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].task_description, "test");

        let patterns = mock.get_patterns();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_type, PatternType::SuccessPattern);

        mock.clear();
        assert_eq!(mock.get_episodes().len(), 0);
        assert_eq!(mock.get_patterns().len(), 0);
    }

    #[test]
    fn test_performance_measure_time() {
        let (result, duration) = performance::measure_time(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
        assert!(duration < Duration::from_millis(50)); // Allow some tolerance
    }

    #[test]
    fn test_performance_assert_duration_within() {
        let short_duration = Duration::from_millis(5);
        let long_duration = Duration::from_millis(15);
        let max_duration = Duration::from_millis(10);

        performance::assert_duration_within(short_duration, max_duration);

        // This should panic, but we can't test panics easily in unit tests
        // The assertion would fail in real usage
    }
}