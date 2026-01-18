//! Core learning operations for episode completion and pattern processing

use crate::error::{Error, Result};
use crate::pattern::Pattern;
use crate::types::TaskOutcome;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::Super;

pub(super) async fn complete_episode(
    super_ref: &Super,
    episode_id: Uuid,
    outcome: TaskOutcome,
) -> Result<()> {
    if super_ref.config.batch_config.is_some() {
        debug!(episode_id = %episode_id, "Flushing buffered steps before episode completion");
        super_ref.flush_steps(episode_id).await?;
    }

    let episode_arc = {
        let episodes = super_ref.episodes_fallback.read().await;
        episodes
            .get(&episode_id)
            .cloned()
            .ok_or(Error::NotFound(episode_id))?
    };
    let mut episode = (*episode_arc).clone();

    episode.complete(outcome.clone());
    super::validation::validate_episode_size(&episode)?;

    let quality_score = super_ref.quality_assessor.assess_episode(&episode);
    info!(episode_id = %episode_id, quality_score, quality_threshold = super_ref.config.quality_threshold, "Assessed episode quality");

    if quality_score < super_ref.config.quality_threshold {
        warn!(episode_id = %episode_id, quality_score, quality_threshold = super_ref.config.quality_threshold, "Episode rejected: quality score below threshold");
        return Err(Error::ValidationFailed(format!(
            "Episode quality score ({:.2}) below threshold ({:.2})",
            quality_score, super_ref.config.quality_threshold
        )));
    }

    let salient_features = super_ref.salient_extractor.extract(&episode);
    episode.salient_features = Some(salient_features.clone());
    debug!(episode_id = %episode_id, feature_count = salient_features.count(), "Extracted salient features");

    let reward = super_ref.reward_calculator.calculate(&episode);
    episode.reward = Some(reward.clone());
    info!(episode_id = %episode_id, reward_total = reward.total, reward_base = reward.base, reward_efficiency = reward.efficiency, "Calculated reward score");

    let reflection = super_ref.reflection_generator.generate(&episode);
    episode.reflection = Some(reflection.clone());
    debug!(
        successes = reflection.successes.len(),
        improvements = reflection.improvements.len(),
        insights = reflection.insights.len(),
        "Generated reflection"
    );

    let _summary = if let Some(ref summarizer) = super_ref.semantic_summarizer {
        match summarizer.summarize_episode(&episode).await {
            Ok(summary) => {
                info!(episode_id = %episode_id, summary_words = summary.summary_text.split_whitespace().count(), key_concepts = summary.key_concepts.len(), "Generated semantic summary");
                Some(summary)
            }
            Err(e) => {
                warn!("Failed to generate semantic summary: {}", e);
                None
            }
        }
    } else {
        None
    };

    if let Some(ref capacity_mgr) = super_ref.capacity_manager {
        let (current_count, all_episodes) = {
            let eps = super_ref.episodes_fallback.read().await;
            let episodes: Vec<_> = eps
                .iter()
                .filter(|(id, _)| **id != episode_id)
                .map(|(_, ep)| (**ep).clone())
                .collect();
            (episodes.len(), episodes)
        };

        if !capacity_mgr.can_store(current_count) {
            let evicted_ids = capacity_mgr.evict_if_needed(&all_episodes);
            if !evicted_ids.is_empty() {
                info!(episode_id = %episode_id, evicted_count = evicted_ids.len(), "Evicting episodes due to capacity constraints");
                let mut episodes_map = super_ref.episodes_fallback.write().await;
                for evicted_id in &evicted_ids {
                    episodes_map.remove(evicted_id);
                }
                debug!(evicted_ids = ?evicted_ids, "Episodes evicted");

                if let Some(ref index) = super_ref.spatiotemporal_index {
                    if let Ok(mut index_write) = index.try_write() {
                        for evicted_id in &evicted_ids {
                            index_write.remove(*evicted_id);
                        }
                    }
                }
            }
        }
    }

    let episode_ref = &episode;
    if let Some(cache) = &super_ref.cache_storage {
        if let Err(e) = cache.store_episode(episode_ref).await {
            warn!("Failed to store completed episode in cache: {}", e);
        }
    }
    if let Some(turso) = &super_ref.turso_storage {
        if let Err(e) = turso.store_episode(episode_ref).await {
            warn!("Failed to store completed episode in Turso: {}", e);
        }
    }

    if let Some(ref index) = super_ref.spatiotemporal_index {
        if let Ok(mut index_write) = index.try_write() {
            index_write.insert(episode_ref);
            debug!(episode_id = %episode_id, domain = %episode.context.domain, task_type = %episode.task_type, "Inserted episode into spatiotemporal index");
        }
    }

    if let Some(ref semantic) = super_ref.semantic_service {
        if let Err(e) = semantic.embed_episode(episode_ref).await {
            warn!(episode_id = %episode_id, error = %e, "Failed to generate embedding for episode");
        }
    }

    let metrics_before = super_ref.query_cache.metrics();
    super_ref.query_cache.invalidate_all();
    info!(episode_id = %episode_id, invalidated_entries = metrics_before.size, "Invalidated query cache after episode completion");

    let mut episodes = super_ref.episodes_fallback.write().await;
    episodes.insert(episode_id, Arc::new(episode));

    if let Some(queue) = &super_ref.pattern_queue {
        queue.enqueue_episode(episode_id).await?;
        info!(episode_id = %episode_id, "Episode completed, enqueued for async pattern extraction");
    } else {
        extract_patterns_sync(super_ref, episode_id).await?;
        info!(episode_id = %episode_id, "Episode completed and patterns extracted synchronously");
    }

    Ok(())
}

pub(super) async fn extract_patterns_sync(super_ref: &Super, episode_id: Uuid) -> Result<()> {
    let episode_arc = {
        let episodes = super_ref.episodes_fallback.read().await;
        episodes
            .get(&episode_id)
            .cloned()
            .ok_or(Error::NotFound(episode_id))?
    };
    let mut episode = (*episode_arc).clone();

    let extracted_patterns = super_ref.pattern_extractor.extract(&episode);
    debug!(
        pattern_count = extracted_patterns.len(),
        "Extracted patterns synchronously"
    );

    let mut patterns = super_ref.patterns_fallback.write().await;
    let mut pattern_ids = Vec::new();

    for pattern in extracted_patterns {
        let pattern_id = pattern.id();
        pattern_ids.push(pattern_id);
        if let Some(cache) = &super_ref.cache_storage {
            if let Err(e) = cache.store_pattern(&pattern).await {
                warn!("Failed to store pattern in cache: {}", e);
            }
        }
        if let Some(turso) = &super_ref.turso_storage {
            if let Err(e) = turso.store_pattern(&pattern).await {
                warn!("Failed to store pattern in Turso: {}", e);
            }
        }
        patterns.insert(pattern_id, pattern);
    }

    episode.patterns = pattern_ids;

    match super_ref.heuristic_extractor.extract(&episode).await {
        Ok(extracted_heuristics) => {
            debug!(
                heuristic_count = extracted_heuristics.len(),
                "Extracted heuristics synchronously"
            );
            let mut heuristic_ids = Vec::new();
            let mut heuristics_map = super_ref.heuristics_fallback.write().await;
            for heuristic in &extracted_heuristics {
                heuristic_ids.push(heuristic.heuristic_id);
                if let Some(cache) = &super_ref.cache_storage {
                    if let Err(e) = cache.store_heuristic(heuristic).await {
                        warn!("Failed to store heuristic in cache: {}", e);
                    }
                }
                if let Some(turso) = &super_ref.turso_storage {
                    if let Err(e) = turso.store_heuristic(heuristic).await {
                        warn!("Failed to store heuristic in Turso: {}", e);
                    }
                }
                heuristics_map.insert(heuristic.heuristic_id, heuristic.clone());
            }
            episode.heuristics = heuristic_ids;
        }
        Err(e) => {
            warn!("Failed to extract heuristics: {}", e);
            episode.heuristics = Vec::new();
        }
    }

    if let Some(cache) = &super_ref.cache_storage {
        if let Err(e) = cache.store_episode(&episode).await {
            warn!("Failed to update episode in cache: {}", e);
        }
    }
    if let Some(turso) = &super_ref.turso_storage {
        if let Err(e) = turso.store_episode(&episode).await {
            warn!("Failed to update episode in Turso: {}", e);
        }
    }

    let mut episodes = super_ref.episodes_fallback.write().await;
    episodes.insert(episode_id, Arc::new(episode));

    Ok(())
}

pub(super) async fn store_patterns(
    super_ref: &Super,
    episode_id: Uuid,
    extracted_patterns: Vec<Pattern>,
) -> Result<()> {
    let episode_arc = {
        let episodes = super_ref.episodes_fallback.read().await;
        episodes
            .get(&episode_id)
            .cloned()
            .ok_or(Error::NotFound(episode_id))?
    };
    let mut episode = (*episode_arc).clone();

    let mut patterns = super_ref.patterns_fallback.write().await;
    let mut pattern_ids = Vec::new();

    for pattern in extracted_patterns {
        let pattern_id = pattern.id();
        pattern_ids.push(pattern_id);
        if let Some(cache) = &super_ref.cache_storage {
            if let Err(e) = cache.store_pattern(&pattern).await {
                warn!("Failed to store pattern in cache: {}", e);
            }
        }
        if let Some(turso) = &super_ref.turso_storage {
            if let Err(e) = turso.store_pattern(&pattern).await {
                warn!("Failed to store pattern in Turso: {}", e);
            }
        }
        patterns.insert(pattern_id, pattern);
    }

    episode.patterns = pattern_ids;

    if let Some(cache) = &super_ref.cache_storage {
        if let Err(e) = cache.store_episode(&episode).await {
            warn!("Failed to update episode with patterns in cache: {}", e);
        }
    }
    if let Some(turso) = &super_ref.turso_storage {
        if let Err(e) = turso.store_episode(&episode).await {
            warn!("Failed to update episode with patterns in Turso: {}", e);
        }
    }

    let mut episodes = super_ref.episodes_fallback.write().await;
    episodes.insert(episode_id, Arc::new(episode));

    Ok(())
}

pub(super) async fn get_queue_stats(
    super_ref: &Super,
) -> Option<crate::learning::queue::QueueStats> {
    if let Some(queue) = &super_ref.pattern_queue {
        Some(queue.get_stats().await)
    } else {
        None
    }
}

pub(super) async fn update_heuristic_confidence(
    super_ref: &Super,
    heuristic_id: Uuid,
    episode_id: Uuid,
    outcome: TaskOutcome,
) -> Result<()> {
    let mut heuristics = super_ref.heuristics_fallback.write().await;
    let heuristic = heuristics
        .get_mut(&heuristic_id)
        .ok_or(Error::NotFound(heuristic_id))?;

    let is_success = matches!(
        outcome,
        TaskOutcome::Success { .. } | TaskOutcome::PartialSuccess { .. }
    );
    debug!(heuristic_id = %heuristic_id, episode_id = %episode_id, is_success, "Updating heuristic confidence");

    heuristic.update_evidence(episode_id, is_success);
    let new_confidence =
        heuristic.evidence.success_rate * (heuristic.evidence.sample_size as f32).sqrt();
    heuristic.confidence = new_confidence;
    info!(heuristic_id = %heuristic_id, new_confidence, "Updated heuristic confidence");

    if let Some(cache) = &super_ref.cache_storage {
        if let Err(e) = cache.store_heuristic(heuristic).await {
            warn!("Failed to store updated heuristic in cache: {}", e);
        }
    }
    if let Some(turso) = &super_ref.turso_storage {
        if let Err(e) = turso.store_heuristic(heuristic).await {
            warn!("Failed to store updated heuristic in Turso: {}", e);
        }
    }

    Ok(())
}
