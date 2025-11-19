//! Test fixtures for episodes and patterns.
//!
//! Provides realistic test data for integration testing.

use memory_core::*;
use std::collections::HashMap;

/// Test fixture for creating realistic episodes
pub struct EpisodeFixture {
    pub episode: Episode,
}

impl EpisodeFixture {
    /// Create a basic development episode
    pub fn development_task(description: &str) -> Self {
        let mut episode = Episode::new(
            description.to_string(),
            TaskType::Development,
            TaskContext {
                domain: "software".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["feature".to_string(), "backend".to_string()],
                metadata: HashMap::new(),
            },
        );

        // Add some realistic steps
        episode.steps = vec![
            ExecutionStep {
                tool: "code_editor".to_string(),
                action: "write_function".to_string(),
                parameters: HashMap::from([
                    ("function_name".to_string(), "process_data".to_string()),
                    ("language".to_string(), "rust".to_string()),
                ]),
                result: "Function implemented successfully".to_string(),
                latency_ms: 2500,
                tokens: 150,
                success: true,
                timestamp: chrono::Utc::now(),
                observation: Some("Implemented data processing function with error handling".to_string()),
                metadata: HashMap::new(),
            },
            ExecutionStep {
                tool: "compiler".to_string(),
                action: "check_compilation".to_string(),
                parameters: HashMap::from([
                    ("target".to_string(), "debug".to_string()),
                ]),
                result: "Compilation successful".to_string(),
                latency_ms: 1200,
                tokens: 0,
                success: true,
                timestamp: chrono::Utc::now(),
                observation: Some("Code compiles without errors".to_string()),
                metadata: HashMap::new(),
            },
        ];

        episode.outcome = Some(TaskOutcome {
            success: true,
            score: 0.95,
            duration_ms: 5000,
            artifacts: vec!["src/lib.rs".to_string()],
            metadata: HashMap::new(),
        });

        episode.reward_score = Some(RewardScore {
            total_score: 0.92,
            components: HashMap::from([
                ("correctness".to_string(), 0.95),
                ("efficiency".to_string(), 0.90),
                ("maintainability".to_string(), 0.91),
            ]),
            metadata: HashMap::new(),
        });

        Self { episode }
    }

    /// Create a testing episode
    pub fn testing_task(description: &str) -> Self {
        let mut episode = Episode::new(
            description.to_string(),
            TaskType::Testing,
            TaskContext {
                domain: "quality".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["test".to_string(), "unit".to_string()],
                metadata: HashMap::new(),
            },
        );

        episode.steps = vec![
            ExecutionStep {
                tool: "test_runner".to_string(),
                action: "run_unit_tests".to_string(),
                parameters: HashMap::from([
                    ("test_pattern".to_string(), "*".to_string()),
                ]),
                result: "All tests passed".to_string(),
                latency_ms: 800,
                tokens: 0,
                success: true,
                timestamp: chrono::Utc::now(),
                observation: Some("Unit test suite completed successfully".to_string()),
                metadata: HashMap::new(),
            },
        ];

        episode.outcome = Some(TaskOutcome {
            success: true,
            score: 1.0,
            duration_ms: 1000,
            artifacts: vec![],
            metadata: HashMap::new(),
        });

        Self { episode }
    }

    /// Create a failed episode
    pub fn failed_task(description: &str) -> Self {
        let mut episode = Episode::new(
            description.to_string(),
            TaskType::Development,
            TaskContext {
                domain: "software".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["bug".to_string(), "error".to_string()],
                metadata: HashMap::new(),
            },
        );

        episode.steps = vec![
            ExecutionStep {
                tool: "code_editor".to_string(),
                action: "write_function".to_string(),
                parameters: HashMap::from([
                    ("function_name".to_string(), "buggy_function".to_string()),
                ]),
                result: "Function implemented but has bugs".to_string(),
                latency_ms: 1800,
                tokens: 120,
                success: false,
                timestamp: chrono::Utc::now(),
                observation: Some("Function compiles but fails at runtime".to_string()),
                metadata: HashMap::new(),
            },
            ExecutionStep {
                tool: "debugger".to_string(),
                action: "analyze_error".to_string(),
                parameters: HashMap::from([
                    ("error_type".to_string(), "panic".to_string()),
                ]),
                result: "Found null pointer dereference".to_string(),
                latency_ms: 500,
                tokens: 30,
                success: true,
                timestamp: chrono::Utc::now(),
                observation: Some("Identified the root cause of the panic".to_string()),
                metadata: HashMap::new(),
            },
        ];

        episode.outcome = Some(TaskOutcome {
            success: false,
            score: 0.3,
            duration_ms: 3000,
            artifacts: vec![],
            metadata: HashMap::new(),
        });

        Self { episode }
    }

    /// Create an episode with many steps
    pub fn complex_task(description: &str, num_steps: usize) -> Self {
        let mut episode = Episode::new(
            description.to_string(),
            TaskType::Analysis,
            TaskContext {
                domain: "data".to_string(),
                language: Some("python".to_string()),
                tags: vec!["analysis".to_string(), "complex".to_string()],
                metadata: HashMap::new(),
            },
        );

        episode.steps = (0..num_steps)
            .map(|i| ExecutionStep {
                tool: format!("tool_{}", i),
                action: format!("action_{}", i),
                parameters: HashMap::from([
                    ("step".to_string(), i.to_string()),
                ]),
                result: format!("Step {} completed", i),
                latency_ms: 100 + (i as u64 * 50),
                tokens: 10 + (i as u32 * 5),
                success: i % 10 != 0, // Every 10th step fails
                timestamp: chrono::Utc::now(),
                observation: Some(format!("Observation for step {}", i)),
                metadata: HashMap::new(),
            })
            .collect();

        let success_count = episode.steps.iter().filter(|s| s.success).count();
        let success_rate = success_count as f64 / num_steps as f64;

        episode.outcome = Some(TaskOutcome {
            success: success_rate > 0.8,
            score: success_rate,
            duration_ms: episode.steps.iter().map(|s| s.latency_ms).sum(),
            artifacts: vec![format!("result_{}.json", num_steps)],
            metadata: HashMap::new(),
        });

        Self { episode }
    }
}

/// Test fixture for creating realistic patterns
pub struct PatternFixture {
    pub pattern: Pattern,
}

impl PatternFixture {
    /// Create a tool sequence pattern
    pub fn tool_sequence_pattern(description: &str, confidence: f64) -> Self {
        let pattern = Pattern {
            id: uuid::Uuid::new_v4(),
            pattern_type: PatternType::ToolSequence,
            description: description.to_string(),
            confidence,
            frequency: 5,
            context: TaskContext {
                domain: "development".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["workflow".to_string(), "efficiency".to_string()],
                metadata: HashMap::new(),
            },
            examples: vec![
                EpisodeFixture::development_task("Implement feature X").episode,
                EpisodeFixture::development_task("Refactor module Y").episode,
            ],
            metadata: HashMap::from([
                ("sequence".to_string(), "edit->compile->test".to_string()),
                ("avg_duration".to_string(), "4500".to_string()),
            ]),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Self { pattern }
    }

    /// Create a success pattern
    pub fn success_pattern(description: &str, confidence: f64) -> Self {
        let pattern = Pattern {
            id: uuid::Uuid::new_v4(),
            pattern_type: PatternType::SuccessPattern,
            description: description.to_string(),
            confidence,
            frequency: 8,
            context: TaskContext {
                domain: "testing".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["quality".to_string(), "success".to_string()],
                metadata: HashMap::new(),
            },
            examples: vec![
                EpisodeFixture::testing_task("Run test suite").episode,
                EpisodeFixture::testing_task("Validate implementation").episode,
            ],
            metadata: HashMap::from([
                ("success_rate".to_string(), "0.95".to_string()),
                ("common_factors".to_string(), "good_test_coverage,early_validation".to_string()),
            ]),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Self { pattern }
    }

    /// Create an error pattern
    pub fn error_pattern(description: &str, confidence: f64) -> Self {
        let pattern = Pattern {
            id: uuid::Uuid::new_v4(),
            pattern_type: PatternType::ErrorPattern,
            description: description.to_string(),
            confidence,
            frequency: 3,
            context: TaskContext {
                domain: "debugging".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["error".to_string(), "bug".to_string()],
                metadata: HashMap::new(),
            },
            examples: vec![
                EpisodeFixture::failed_task("Fix null pointer bug").episode,
            ],
            metadata: HashMap::from([
                ("error_type".to_string(), "panic".to_string()),
                ("common_cause".to_string(), "missing_null_check".to_string()),
                ("solution".to_string(), "add_proper_validation".to_string()),
            ]),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Self { pattern }
    }

    /// Create a performance pattern
    pub fn performance_pattern(description: &str, confidence: f64) -> Self {
        let pattern = Pattern {
            id: uuid::Uuid::new_v4(),
            pattern_type: PatternType::PerformancePattern,
            description: description.to_string(),
            confidence,
            frequency: 4,
            context: TaskContext {
                domain: "optimization".to_string(),
                language: Some("rust".to_string()),
                tags: vec!["performance".to_string(), "optimization".to_string()],
                metadata: HashMap::new(),
            },
            examples: vec![
                EpisodeFixture::complex_task("Optimize algorithm", 20).episode,
            ],
            metadata: HashMap::from([
                ("avg_improvement".to_string(), "35%".to_string()),
                ("bottleneck_type".to_string(), "algorithm_complexity".to_string()),
            ]),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Self { pattern }
    }
}

/// Collection of test fixtures for comprehensive testing
pub struct TestFixtureCollection {
    pub episodes: Vec<Episode>,
    pub patterns: Vec<Pattern>,
}

impl TestFixtureCollection {
    /// Create a comprehensive test dataset
    pub fn comprehensive() -> Self {
        let episodes = vec![
            EpisodeFixture::development_task("Implement user authentication").episode,
            EpisodeFixture::development_task("Add database integration").episode,
            EpisodeFixture::testing_task("Validate user authentication").episode,
            EpisodeFixture::testing_task("Test database operations").episode,
            EpisodeFixture::failed_task("Fix authentication bug").episode,
            EpisodeFixture::complex_task("Implement complex business logic", 15).episode,
        ];

        let patterns = vec![
            PatternFixture::tool_sequence_pattern("Standard development workflow", 0.85).pattern,
            PatternFixture::success_pattern("Successful testing patterns", 0.92).pattern,
            PatternFixture::error_pattern("Common error patterns", 0.78).pattern,
            PatternFixture::performance_pattern("Performance optimization patterns", 0.88).pattern,
        ];

        Self { episodes, patterns }
    }

    /// Create a small test dataset for quick testing
    pub fn minimal() -> Self {
        let episodes = vec![
            EpisodeFixture::development_task("Simple feature").episode,
            EpisodeFixture::testing_task("Basic test").episode,
        ];

        let patterns = vec![
            PatternFixture::tool_sequence_pattern("Basic workflow", 0.8).pattern,
        ];

        Self { episodes, patterns }
    }

    /// Create a dataset focused on failures
    pub fn failure_focused() -> Self {
        let episodes = vec![
            EpisodeFixture::failed_task("Bug in feature A").episode,
            EpisodeFixture::failed_task("Issue with integration").episode,
            EpisodeFixture::failed_task("Performance problem").episode,
        ];

        let patterns = vec![
            PatternFixture::error_pattern("Recurring bugs", 0.9).pattern,
        ];

        Self { episodes, patterns }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_fixture_creation() {
        let fixture = EpisodeFixture::development_task("Test task");
        assert_eq!(fixture.episode.task_description, "Test task");
        assert_eq!(fixture.episode.task_type, TaskType::Development);
        assert_eq!(fixture.episode.context.domain, "software");
        assert!(!fixture.episode.steps.is_empty());
        assert!(fixture.episode.outcome.is_some());
    }

    #[test]
    fn test_failed_episode_fixture() {
        let fixture = EpisodeFixture::failed_task("Failed task");
        assert_eq!(fixture.episode.outcome.as_ref().unwrap().success, false);
        assert!(fixture.episode.outcome.as_ref().unwrap().score < 0.5);
    }

    #[test]
    fn test_complex_episode_fixture() {
        let fixture = EpisodeFixture::complex_task("Complex task", 10);
        assert_eq!(fixture.episode.steps.len(), 10);
        assert!(fixture.episode.outcome.is_some());
    }

    #[test]
    fn test_pattern_fixture_creation() {
        let fixture = PatternFixture::tool_sequence_pattern("Test pattern", 0.8);
        assert_eq!(fixture.pattern.pattern_type, PatternType::ToolSequence);
        assert_eq!(fixture.pattern.confidence, 0.8);
        assert!(!fixture.pattern.examples.is_empty());
    }

    #[test]
    fn test_comprehensive_fixture_collection() {
        let collection = TestFixtureCollection::comprehensive();
        assert!(!collection.episodes.is_empty());
        assert!(!collection.patterns.is_empty());

        // Check that we have different types of episodes
        let has_development = collection.episodes.iter().any(|e| e.task_type == TaskType::Development);
        let has_testing = collection.episodes.iter().any(|e| e.task_type == TaskType::Testing);
        assert!(has_development);
        assert!(has_testing);
    }

    #[test]
    fn test_minimal_fixture_collection() {
        let collection = TestFixtureCollection::minimal();
        assert_eq!(collection.episodes.len(), 2);
        assert_eq!(collection.patterns.len(), 1);
    }

    #[test]
    fn test_failure_focused_collection() {
        let collection = TestFixtureCollection::failure_focused();
        assert!(!collection.episodes.is_empty());

        // All episodes should have failed outcomes
        for episode in &collection.episodes {
            assert!(!episode.outcome.as_ref().unwrap().success);
        }
    }
}