//! Integration test for adaptive reward calibration

use memory_core::{
    AdaptiveRewardCalculator, ComplexityLevel, DomainStatisticsCache, Episode, ExecutionResult,
    ExecutionStep, TaskContext, TaskOutcome, TaskType,
};

#[tokio::test]
async fn test_adaptive_reward_with_domain_statistics() {
    // Create episodes in two different domains
    let mut web_api_episodes = Vec::new();
    let mut data_processing_episodes = Vec::new();

    // Web API domain: Fast episodes (15s, 8 steps)
    for i in 0..10 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("axum".to_string()),
            complexity: ComplexityLevel::Simple,
            domain: "web-api".to_string(),
            tags: vec!["rest".to_string()],
        };

        let mut episode = Episode::new(
            format!("API request {i}"),
            context,
            TaskType::CodeGeneration,
        );

        // Add 8 steps
        for j in 0..8 {
            let mut step = ExecutionStep::new(j + 1, format!("tool_{j}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Simulate 15 second duration
        episode.start_time = chrono::Utc::now() - chrono::Duration::seconds(15);
        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        web_api_episodes.push(episode);
    }

    // Data processing domain: Slow episodes (180s, 25 steps)
    for i in 0..10 {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "data-processing".to_string(),
            tags: vec!["batch".to_string()],
        };

        let mut episode = Episode::new(format!("Process batch {i}"), context, TaskType::Analysis);

        // Add 25 steps
        for j in 0..25 {
            let mut step = ExecutionStep::new(j + 1, format!("tool_{j}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Simulate 180 second duration
        episode.start_time = chrono::Utc::now() - chrono::Duration::seconds(180);
        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        data_processing_episodes.push(episode);
    }

    // Calculate statistics for both domains
    let mut stats_cache = DomainStatisticsCache::new();
    stats_cache.calculate_from_episodes("web-api".to_string(), &web_api_episodes);
    stats_cache.calculate_from_episodes("data-processing".to_string(), &data_processing_episodes);

    // Verify statistics
    let web_api_stats = stats_cache.get("web-api").unwrap();
    assert!(web_api_stats.is_reliable());
    assert_eq!(web_api_stats.episode_count, 10);
    assert!(web_api_stats.p50_step_count <= 10); // Around 8

    let data_stats = stats_cache.get("data-processing").unwrap();
    assert!(data_stats.is_reliable());
    assert_eq!(data_stats.episode_count, 10);
    assert!(data_stats.p50_step_count >= 20); // Around 25

    // Create adaptive calculator
    let calculator = AdaptiveRewardCalculator::new();

    // Test: 30 second episode in web-api (slower than median of 15s)
    let context_web = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Simple,
        domain: "web-api".to_string(),
        tags: vec!["rest".to_string()],
    };
    let mut slow_web_episode = Episode::new(
        "Slow API request".to_string(),
        context_web,
        TaskType::CodeGeneration,
    );

    for j in 0..8 {
        let mut step = ExecutionStep::new(j + 1, format!("tool_{j}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        slow_web_episode.add_step(step);
    }

    slow_web_episode.start_time = chrono::Utc::now() - chrono::Duration::seconds(30);
    slow_web_episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    let web_reward = calculator.calculate(&slow_web_episode, Some(web_api_stats));

    // Test: 30 second episode in data-processing (MUCH faster than median of 180s)
    let context_data = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "data-processing".to_string(),
        tags: vec!["batch".to_string()],
    };
    let mut fast_data_episode = Episode::new(
        "Fast batch process".to_string(),
        context_data,
        TaskType::Analysis,
    );

    for j in 0..25 {
        let mut step = ExecutionStep::new(j + 1, format!("tool_{j}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        fast_data_episode.add_step(step);
    }

    fast_data_episode.start_time = chrono::Utc::now() - chrono::Duration::seconds(30);
    fast_data_episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    let data_reward = calculator.calculate(&fast_data_episode, Some(data_stats));

    // KEY TEST: Same 30-second duration, but different rewards!
    // Data processing episode should have MUCH higher efficiency
    // because 30s << 180s median, while 30s > 15s median for web-api
    println!("Web API reward (30s, median 15s): {:.2}", web_reward.total);
    println!(
        "Data processing reward (30s, median 180s): {:.2}",
        data_reward.total
    );

    // Data processing should have significantly better reward
    assert!(
        data_reward.total > web_reward.total * 1.2,
        "Expected data processing reward ({:.2}) to be >20% higher than web API reward ({:.2})",
        data_reward.total,
        web_reward.total
    );

    // Efficiency should be notably different
    assert!(
        data_reward.efficiency > web_reward.efficiency,
        "Expected data processing efficiency ({:.2}) > web API efficiency ({:.2})",
        data_reward.efficiency,
        web_reward.efficiency
    );
}

#[test]
fn test_domain_statistics_reliability_threshold() {
    let mut stats_cache = DomainStatisticsCache::new();

    // Create 4 episodes (below reliability threshold of 5)
    let mut episodes = Vec::new();
    for i in 0..4 {
        let context = TaskContext {
            domain: "test-domain".to_string(),
            ..Default::default()
        };
        let mut episode = Episode::new(format!("Task {i}"), context, TaskType::Testing);
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
        episodes.push(episode);
    }

    stats_cache.calculate_from_episodes("test-domain".to_string(), &episodes);
    let stats = stats_cache.get("test-domain").unwrap();

    assert_eq!(stats.episode_count, 4);
    assert!(!stats.is_reliable()); // Should be unreliable with <5 episodes

    // Add one more episode
    let context = TaskContext {
        domain: "test-domain".to_string(),
        ..Default::default()
    };
    let mut episode = Episode::new("Task 5".to_string(), context, TaskType::Testing);
    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });
    episodes.push(episode);

    stats_cache.calculate_from_episodes("test-domain".to_string(), &episodes);
    let stats = stats_cache.get("test-domain").unwrap();

    assert_eq!(stats.episode_count, 5);
    assert!(stats.is_reliable()); // Should now be reliable with 5 episodes
}
