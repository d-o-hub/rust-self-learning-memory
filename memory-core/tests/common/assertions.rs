//! Assertion helpers for common test validations

use memory_core::{Episode, RewardScore};

/// Assert that an episode has been completed
///
/// # Panics
///
/// Panics if the episode is not complete or missing completion data
///
/// # Examples
///
/// ```ignore
/// let episode = memory.get_episode(episode_id).await.unwrap();
/// assert_episode_completed(&episode);
/// ```
pub fn assert_episode_completed(episode: &Episode) {
    assert!(
        episode.is_complete(),
        "Episode {} should be marked as complete",
        episode.episode_id
    );
    assert!(
        episode.end_time.is_some(),
        "Episode {} should have end_time set",
        episode.episode_id
    );
    assert!(
        episode.outcome.is_some(),
        "Episode {} should have outcome set",
        episode.episode_id
    );
    assert!(
        episode.reward.is_some(),
        "Episode {} should have reward calculated",
        episode.episode_id
    );
    assert!(
        episode.reflection.is_some(),
        "Episode {} should have reflection generated",
        episode.episode_id
    );
}

/// Assert that a reward score falls within an expected range
///
/// # Panics
///
/// Panics if the reward total is outside the specified range
///
/// # Examples
///
/// ```ignore
/// let reward = episode.reward.unwrap();
/// assert_reward_in_range(&reward, 0.8, 1.2);
/// ```
pub fn assert_reward_in_range(reward: &RewardScore, min: f32, max: f32) {
    assert!(
        reward.total >= min && reward.total <= max,
        "Reward total {} is outside expected range [{}, {}]",
        reward.total,
        min,
        max
    );
}

/// Assert that a reward score is for a successful outcome
///
/// # Panics
///
/// Panics if the reward base score is not 1.0 (success)
///
/// # Examples
///
/// ```ignore
/// let reward = episode.reward.unwrap();
/// assert_reward_is_success(&reward);
/// ```
pub fn assert_reward_is_success(reward: &RewardScore) {
    assert_eq!(
        reward.base, 1.0,
        "Reward base score should be 1.0 for success, got {}",
        reward.base
    );
}

/// Assert that a reward score is for a failed outcome
///
/// # Panics
///
/// Panics if the reward base score is not 0.0 (failure)
///
/// # Examples
///
/// ```ignore
/// let reward = episode.reward.unwrap();
/// assert_reward_is_failure(&reward);
/// ```
pub fn assert_reward_is_failure(reward: &RewardScore) {
    assert_eq!(
        reward.base, 0.0,
        "Reward base score should be 0.0 for failure, got {}",
        reward.base
    );
}

/// Assert that a reward score is for a partial success
///
/// # Panics
///
/// Panics if the reward base score is not between 0.0 and 1.0 (exclusive)
///
/// # Examples
///
/// ```ignore
/// let reward = episode.reward.unwrap();
/// assert_reward_is_partial(&reward);
/// ```
#[allow(dead_code)]
pub fn assert_reward_is_partial(reward: &RewardScore) {
    assert!(
        reward.base > 0.0 && reward.base < 1.0,
        "Reward base score should be between 0.0 and 1.0 for partial success, got {}",
        reward.base
    );
}

/// Assert that an episode has extracted patterns
///
/// # Panics
///
/// Panics if the episode has no patterns extracted
///
/// # Examples
///
/// ```ignore
/// let episode = memory.get_episode(episode_id).await.unwrap();
/// assert_has_patterns(&episode);
/// ```
#[allow(dead_code)]
pub fn assert_has_patterns(episode: &Episode) {
    assert!(
        !episode.patterns.is_empty(),
        "Episode {} should have extracted patterns",
        episode.episode_id
    );
}

/// Assert that an episode has a minimum number of patterns
///
/// # Panics
///
/// Panics if the episode has fewer than the minimum number of patterns
///
/// # Examples
///
/// ```ignore
/// let episode = memory.get_episode(episode_id).await.unwrap();
/// assert_min_patterns(&episode, 2);
/// ```
#[allow(dead_code)]
pub fn assert_min_patterns(episode: &Episode, min_count: usize) {
    assert!(
        episode.patterns.len() >= min_count,
        "Episode {} should have at least {} patterns, found {}",
        episode.episode_id,
        min_count,
        episode.patterns.len()
    );
}

/// Assert that an episode has a specific number of steps
///
/// # Panics
///
/// Panics if the episode doesn't have exactly the expected number of steps
///
/// # Examples
///
/// ```ignore
/// let episode = memory.get_episode(episode_id).await.unwrap();
/// assert_step_count(&episode, 5);
/// ```
pub fn assert_step_count(episode: &Episode, expected_count: usize) {
    assert_eq!(
        episode.steps.len(),
        expected_count,
        "Episode {} should have {} steps, found {}",
        episode.episode_id,
        expected_count,
        episode.steps.len()
    );
}

/// Assert that an episode has at least a minimum number of steps
///
/// # Panics
///
/// Panics if the episode has fewer than the minimum number of steps
///
/// # Examples
///
/// ```ignore
/// let episode = memory.get_episode(episode_id).await.unwrap();
/// assert_min_steps(&episode, 3);
/// ```
pub fn assert_min_steps(episode: &Episode, min_count: usize) {
    assert!(
        episode.steps.len() >= min_count,
        "Episode {} should have at least {} steps, found {}",
        episode.episode_id,
        min_count,
        episode.steps.len()
    );
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::test_context;
    use super::*;
    use memory_core::{Episode, TaskType};

    #[tokio::test]
    async fn test_assert_episode_completed() {
        use super::super::helpers::setup_test_memory;

        let memory = setup_test_memory();

        let episode_id = memory
            .start_episode("test".to_string(), test_context(), TaskType::Testing)
            .await
            .unwrap();

        memory
            .complete_episode(
                episode_id,
                memory_core::TaskOutcome::Success {
                    verdict: "done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_episode_completed(&episode);
    }

    #[test]
    fn test_assert_reward_is_success() {
        let reward = RewardScore {
            base: 1.0,
            complexity_bonus: 1.1,
            efficiency: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
            total: 1.1,
        };

        assert_reward_is_success(&reward);
    }

    #[test]
    fn test_assert_reward_is_failure() {
        let reward = RewardScore {
            base: 0.0,
            complexity_bonus: 1.0,
            efficiency: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
            total: 0.0,
        };

        assert_reward_is_failure(&reward);
    }

    #[test]
    fn test_assert_reward_in_range() {
        let reward = RewardScore {
            base: 1.0,
            complexity_bonus: 1.1,
            efficiency: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
            total: 1.1,
        };

        assert_reward_in_range(&reward, 1.0, 1.5);
    }

    #[test]
    fn test_assert_step_count() {
        let mut episode = Episode::new("test".to_string(), test_context(), TaskType::Testing);

        episode.add_step(super::super::helpers::create_test_step(1));
        episode.add_step(super::super::helpers::create_test_step(2));

        assert_step_count(&episode, 2);
        assert_min_steps(&episode, 1);
    }
}
