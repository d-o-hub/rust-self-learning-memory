// ============================================================================
// Audit Helper Functions
// ============================================================================
//!
//! Convenience functions for creating common audit entries.

use super::AuditContext;
use super::types::{AuditEntry, AuditEventType, AuditLogLevel, AuditResult};
use uuid::Uuid;

/// Create an audit entry for episode creation.
#[must_use]
pub fn episode_created(
    context: &AuditContext,
    episode_id: Uuid,
    task_description: &str,
    task_type: &str,
) -> AuditEntry {
    AuditEntry::new(AuditEventType::EpisodeCreated, context.actor.clone())
        .with_level(AuditLogLevel::Info)
        .with_resource_id(episode_id.to_string())
        .with_detail("task_description", task_description)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::EpisodeCreated, context.actor.clone()))
        .with_detail("task_type", task_type)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::EpisodeCreated, context.actor.clone()))
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for episode completion.
#[must_use]
pub fn episode_completed(
    context: &AuditContext,
    episode_id: Uuid,
    outcome: &str,
    success: bool,
) -> AuditEntry {
    let level = if success {
        AuditLogLevel::Info
    } else {
        AuditLogLevel::Warn
    };

    AuditEntry::new(AuditEventType::EpisodeCompleted, context.actor.clone())
        .with_level(level)
        .with_resource_id(episode_id.to_string())
        .with_detail("outcome", outcome)
        .unwrap_or_else(|_| {
            AuditEntry::new(AuditEventType::EpisodeCompleted, context.actor.clone())
        })
        .with_detail("success", success)
        .unwrap_or_else(|_| {
            AuditEntry::new(AuditEventType::EpisodeCompleted, context.actor.clone())
        })
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for episode deletion.
#[must_use]
pub fn episode_deleted(context: &AuditContext, episode_id: Uuid) -> AuditEntry {
    AuditEntry::new(AuditEventType::EpisodeDeleted, context.actor.clone())
        .with_level(AuditLogLevel::Warn)
        .with_resource_id(episode_id.to_string())
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for relationship addition.
#[must_use]
pub fn relationship_added(
    context: &AuditContext,
    relationship_id: Uuid,
    from_episode: Uuid,
    to_episode: Uuid,
    relationship_type: &str,
) -> AuditEntry {
    AuditEntry::new(AuditEventType::RelationshipAdded, context.actor.clone())
        .with_level(AuditLogLevel::Info)
        .with_resource_id(relationship_id.to_string())
        .with_detail("from_episode", from_episode.to_string())
        .unwrap_or_else(|_| {
            AuditEntry::new(AuditEventType::RelationshipAdded, context.actor.clone())
        })
        .with_detail("to_episode", to_episode.to_string())
        .unwrap_or_else(|_| {
            AuditEntry::new(AuditEventType::RelationshipAdded, context.actor.clone())
        })
        .with_detail("relationship_type", relationship_type)
        .unwrap_or_else(|_| {
            AuditEntry::new(AuditEventType::RelationshipAdded, context.actor.clone())
        })
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for relationship removal.
#[must_use]
pub fn relationship_removed(context: &AuditContext, relationship_id: Uuid) -> AuditEntry {
    AuditEntry::new(AuditEventType::RelationshipRemoved, context.actor.clone())
        .with_level(AuditLogLevel::Warn)
        .with_resource_id(relationship_id.to_string())
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for tag modification.
#[must_use]
pub fn tags_modified(
    context: &AuditContext,
    episode_id: Uuid,
    action: &str,
    tags: &[String],
) -> AuditEntry {
    let event_type = match action {
        "added" => AuditEventType::TagsAdded,
        "removed" => AuditEventType::TagsRemoved,
        "set" => AuditEventType::TagsSet,
        _ => AuditEventType::TagsAdded,
    };

    AuditEntry::new(event_type, context.actor.clone())
        .with_level(AuditLogLevel::Info)
        .with_resource_id(episode_id.to_string())
        .with_detail("action", action)
        .unwrap_or_else(|_| AuditEntry::new(event_type, context.actor.clone()))
        .with_detail("tags", tags)
        .unwrap_or_else(|_| AuditEntry::new(event_type, context.actor.clone()))
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for access denial.
#[must_use]
pub fn access_denied(
    context: &AuditContext,
    resource: &str,
    action: &str,
    reason: &str,
) -> AuditEntry {
    AuditEntry::new(AuditEventType::AccessDenied, context.actor.clone())
        .with_level(AuditLogLevel::Critical)
        .with_resource_id(resource)
        .with_detail("action", action)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::AccessDenied, context.actor.clone()))
        .with_detail("reason", reason)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::AccessDenied, context.actor.clone()))
        .with_result(AuditResult::Denied {
            reason: reason.to_string(),
        })
        .with_session_id(context.session_id.clone().unwrap_or_default())
}

/// Create an audit entry for configuration changes.
#[must_use]
pub fn config_changed(
    context: &AuditContext,
    config_key: &str,
    old_value: &str,
    new_value: &str,
) -> AuditEntry {
    AuditEntry::new(AuditEventType::ConfigChanged, context.actor.clone())
        .with_level(AuditLogLevel::Warn)
        .with_resource_id(config_key)
        .with_detail("config_key", config_key)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::ConfigChanged, context.actor.clone()))
        .with_detail("old_value", old_value)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::ConfigChanged, context.actor.clone()))
        .with_detail("new_value", new_value)
        .unwrap_or_else(|_| AuditEntry::new(AuditEventType::ConfigChanged, context.actor.clone()))
        .with_session_id(context.session_id.clone().unwrap_or_default())
}
