//! Unified learning-cycle coordination.
//!
//! [`LearningOrchestrator`] owns the reward and reflection stages of episode
//! completion and dispatches the pattern-extraction stage to either the
//! background [`PatternExtractionQueue`] or the caller's synchronous path.
//! The storage, summary, event, and audit stages remain in
//! [`SelfLearningMemory::complete_episode_checked`](crate::memory::SelfLearningMemory::complete_episode_checked);
//! this type only removes the duplicated wiring between them.

use std::future::Future;

use uuid::Uuid;

use crate::episode::Episode;
use crate::error::Result;
use crate::learning::queue::PatternExtractionQueue;
use crate::reflection::ReflectionGenerator;
use crate::reward::RewardCalculator;
use crate::types::{Reflection, RewardScore};

/// Reward and reflection produced for one completed episode.
#[derive(Debug, Clone)]
pub struct LearningAnalysis {
    /// Reward score calculated from the episode outcome, efficiency, and quality.
    pub reward: RewardScore,
    /// Reflection generated from the completed episode.
    pub reflection: Reflection,
}

/// How the pattern-extraction stage was dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternDispatch {
    /// The episode was enqueued for asynchronous extraction.
    Enqueued,
    /// The caller's synchronous extraction path was awaited.
    ExtractedSynchronously,
}

/// Coordinates the reward, reflection, and pattern-extraction stages of the
/// learning cycle.
#[derive(Clone)]
pub struct LearningOrchestrator {
    reward_calculator: RewardCalculator,
    reflection_generator: ReflectionGenerator,
}

impl Default for LearningOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningOrchestrator {
    /// Create an orchestrator with the default reward and reflection settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            reward_calculator: RewardCalculator::new(),
            reflection_generator: ReflectionGenerator::new(),
        }
    }

    /// Create an orchestrator with explicitly configured collaborators.
    #[must_use]
    pub fn with_collaborators(
        reward_calculator: RewardCalculator,
        reflection_generator: ReflectionGenerator,
    ) -> Self {
        Self {
            reward_calculator,
            reflection_generator,
        }
    }

    /// Calculate the reward and generate the reflection for a completed episode.
    ///
    /// The two stages are independent; callers that need them concurrently can
    /// still use the underlying calculators directly.
    #[must_use]
    pub fn analyze(&self, episode: &Episode) -> LearningAnalysis {
        LearningAnalysis {
            reward: self.reward_calculator.calculate(episode),
            reflection: self.reflection_generator.generate(episode),
        }
    }

    /// Dispatch pattern extraction through the queue when configured, otherwise
    /// await the caller's synchronous extraction future.
    ///
    /// The synchronous future is created eagerly by the caller but is only
    /// polled when no queue is configured, so the queue path retains its
    /// non-blocking behavior.
    ///
    /// # Errors
    ///
    /// Propagates queue backpressure from
    /// [`PatternExtractionQueue::enqueue_episode`] and any error returned by the
    /// synchronous extraction path.
    pub async fn extract_patterns<F>(
        &self,
        queue: Option<&PatternExtractionQueue>,
        episode_id: Uuid,
        sync_extract: F,
    ) -> Result<PatternDispatch>
    where
        F: Future<Output = Result<()>>,
    {
        if let Some(queue) = queue {
            queue.enqueue_episode(episode_id).await?;
            Ok(PatternDispatch::Enqueued)
        } else {
            sync_extract.await?;
            Ok(PatternDispatch::ExtractedSynchronously)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::learning::queue::QueueConfig;
    use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn completed_episode() -> Episode {
        let mut episode = Episode::new(
            "Release the orchestrator".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );
        let mut step = ExecutionStep::new(1, "test_runner".to_string(), "Run tests".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "all green".to_string(),
        });
        episode.add_step(step);
        episode.complete(TaskOutcome::Success {
            verdict: "done".to_string(),
            artifacts: Vec::new(),
        });
        episode
    }

    #[test]
    fn analyze_matches_the_individual_calculators() {
        let episode = completed_episode();
        let analysis = LearningOrchestrator::new().analyze(&episode);

        assert_eq!(analysis.reward, RewardCalculator::new().calculate(&episode));

        let expected = ReflectionGenerator::new().generate(&episode);
        assert_eq!(analysis.reflection.successes, expected.successes);
        assert_eq!(analysis.reflection.improvements, expected.improvements);
        assert_eq!(analysis.reflection.insights, expected.insights);
    }

    #[tokio::test]
    async fn dispatch_runs_the_sync_path_without_a_queue() {
        let called = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&called);
        let dispatch = LearningOrchestrator::new()
            .extract_patterns(None, Uuid::new_v4(), async move {
                flag.store(true, Ordering::SeqCst);
                Ok(())
            })
            .await
            .expect("sync extraction should succeed");

        assert_eq!(dispatch, PatternDispatch::ExtractedSynchronously);
        assert!(called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn dispatch_enqueues_without_polling_the_sync_path() {
        let memory = Arc::new(crate::memory::SelfLearningMemory::new());
        let queue = PatternExtractionQueue::new(QueueConfig::default(), memory);
        let sync_ran = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&sync_ran);
        let dispatch = LearningOrchestrator::new()
            .extract_patterns(Some(&queue), Uuid::new_v4(), async move {
                flag.store(true, Ordering::SeqCst);
                Ok(())
            })
            .await
            .expect("enqueue should succeed");

        assert_eq!(dispatch, PatternDispatch::Enqueued);
        assert_eq!(queue.queue_size().await, 1);
        assert!(!sync_ran.load(Ordering::SeqCst));
    }
}
