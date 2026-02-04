//! Episode relationship types and data structures.
//!
//! This module provides types for modeling relationships between episodes,
//! enabling hierarchical organization, dependency tracking, and workflow modeling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "proptest-arbitrary")]
use proptest::prelude::{prop_oneof, Arbitrary, BoxedStrategy, Just, Strategy};

/// Type of relationship between two episodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Parent-child hierarchical relationship (e.g., epic → story → subtask)
    ParentChild,
    /// Dependency relationship (from depends on to)
    DependsOn,
    /// Sequential relationship (from follows to)
    Follows,
    /// Loose association between related episodes
    RelatedTo,
    /// Blocking relationship (from blocks to)
    Blocks,
    /// Marks episodes as duplicates
    Duplicates,
    /// General cross-reference
    References,
}

impl RelationshipType {
    /// Check if this relationship type implies directionality
    #[must_use]
    pub fn is_directional(&self) -> bool {
        matches!(
            self,
            Self::ParentChild | Self::DependsOn | Self::Follows | Self::Blocks
        )
    }

    /// Get the inverse relationship type (for bidirectional tracking)
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::ParentChild => Some(Self::ParentChild), // Child knows parent
            Self::DependsOn => Some(Self::DependsOn),     // Reverse dependency
            Self::Follows => Some(Self::Follows),         // Preceded by
            Self::Blocks => Some(Self::Blocks),           // Blocked by
            Self::RelatedTo => None,                      // Symmetric
            Self::Duplicates => None,                     // Symmetric
            Self::References => None,                     // Symmetric
        }
    }

    /// Check if this relationship type should prevent cycles
    #[must_use]
    pub fn requires_acyclic(&self) -> bool {
        matches!(self, Self::DependsOn | Self::ParentChild | Self::Blocks)
    }

    /// Convert to string representation for storage
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ParentChild => "parent_child",
            Self::DependsOn => "depends_on",
            Self::Follows => "follows",
            Self::RelatedTo => "related_to",
            Self::Blocks => "blocks",
            Self::Duplicates => "duplicates",
            Self::References => "references",
        }
    }

    /// Parse from string representation
    ///
    /// # Errors
    ///
    /// Returns an error if the string doesn't match a known relationship type.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "parent_child" => Ok(Self::ParentChild),
            "depends_on" => Ok(Self::DependsOn),
            "follows" => Ok(Self::Follows),
            "related_to" => Ok(Self::RelatedTo),
            "blocks" => Ok(Self::Blocks),
            "duplicates" => Ok(Self::Duplicates),
            "references" => Ok(Self::References),
            _ => Err(format!("Unknown relationship type: {s}")),
        }
    }

    /// Get all relationship types
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::ParentChild,
            Self::DependsOn,
            Self::Follows,
            Self::RelatedTo,
            Self::Blocks,
            Self::Duplicates,
            Self::References,
        ]
    }
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(feature = "proptest-arbitrary")]
impl Arbitrary for RelationshipType {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(Self::ParentChild),
            Just(Self::DependsOn),
            Just(Self::Follows),
            Just(Self::RelatedTo),
            Just(Self::Blocks),
            Just(Self::Duplicates),
            Just(Self::References),
        ]
        .boxed()
    }
}

/// Additional metadata for a relationship.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RelationshipMetadata {
    /// Human-readable reason for the relationship
    pub reason: Option<String>,
    /// Who created this relationship
    pub created_by: Option<String>,
    /// Priority/importance (1-10, higher is more important)
    pub priority: Option<u8>,
    /// Custom fields for extensibility
    #[serde(default)]
    pub custom_fields: HashMap<String, String>,
}

impl RelationshipMetadata {
    /// Create new empty metadata
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create metadata with a reason
    #[must_use]
    pub fn with_reason(reason: String) -> Self {
        Self {
            reason: Some(reason),
            ..Default::default()
        }
    }

    /// Create metadata with reason and `created_by`
    #[must_use]
    pub fn with_creator(reason: String, created_by: String) -> Self {
        Self {
            reason: Some(reason),
            created_by: Some(created_by),
            ..Default::default()
        }
    }

    /// Add or update a custom field
    pub fn set_field(&mut self, key: String, value: String) {
        self.custom_fields.insert(key, value);
    }

    /// Get a custom field value
    #[must_use]
    pub fn get_field(&self, key: &str) -> Option<&String> {
        self.custom_fields.get(key)
    }
}

/// A relationship between two episodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeRelationship {
    /// Unique identifier for this relationship
    pub id: Uuid,
    /// Source episode ID
    pub from_episode_id: Uuid,
    /// Target episode ID
    pub to_episode_id: Uuid,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Additional metadata
    pub metadata: RelationshipMetadata,
    /// When this relationship was created
    pub created_at: DateTime<Utc>,
}

impl EpisodeRelationship {
    /// Create a new relationship
    #[must_use]
    pub fn new(
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
        metadata: RelationshipMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_episode_id,
            to_episode_id,
            relationship_type,
            metadata,
            created_at: Utc::now(),
        }
    }

    /// Create a simple relationship with just a reason
    #[must_use]
    pub fn with_reason(
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
        reason: String,
    ) -> Self {
        Self::new(
            from_episode_id,
            to_episode_id,
            relationship_type,
            RelationshipMetadata::with_reason(reason),
        )
    }

    /// Check if this relationship is directional
    #[must_use]
    pub fn is_directional(&self) -> bool {
        self.relationship_type.is_directional()
    }

    /// Get the inverse of this relationship (swap from/to)
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        self.relationship_type.inverse().map(|inv_type| Self {
            id: Uuid::new_v4(),
            from_episode_id: self.to_episode_id,
            to_episode_id: self.from_episode_id,
            relationship_type: inv_type,
            metadata: self.metadata.clone(),
            created_at: self.created_at,
        })
    }
}

/// Direction for querying relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Outgoing relationships (this episode → others)
    Outgoing,
    /// Incoming relationships (others → this episode)
    Incoming,
    /// Both directions
    Both,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_directional() {
        assert!(RelationshipType::ParentChild.is_directional());
        assert!(RelationshipType::DependsOn.is_directional());
        assert!(RelationshipType::Follows.is_directional());
        assert!(RelationshipType::Blocks.is_directional());
        assert!(!RelationshipType::RelatedTo.is_directional());
        assert!(!RelationshipType::Duplicates.is_directional());
        assert!(!RelationshipType::References.is_directional());
    }

    #[test]
    fn test_relationship_type_acyclic() {
        assert!(RelationshipType::DependsOn.requires_acyclic());
        assert!(RelationshipType::ParentChild.requires_acyclic());
        assert!(RelationshipType::Blocks.requires_acyclic());
        assert!(!RelationshipType::Follows.requires_acyclic());
        assert!(!RelationshipType::RelatedTo.requires_acyclic());
    }

    #[test]
    fn test_relationship_type_str_conversion() {
        for rel_type in RelationshipType::all() {
            let s = rel_type.as_str();
            let parsed = RelationshipType::parse(s).unwrap();
            assert_eq!(rel_type, parsed);
        }
    }

    #[test]
    fn test_relationship_type_from_str_invalid() {
        let result = RelationshipType::parse("invalid_type");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown relationship type"));
    }

    #[test]
    fn test_relationship_creation() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let metadata = RelationshipMetadata::with_reason("Subtask of parent".to_string());

        let rel = EpisodeRelationship::new(
            from_id,
            to_id,
            RelationshipType::ParentChild,
            metadata.clone(),
        );

        assert_eq!(rel.from_episode_id, from_id);
        assert_eq!(rel.to_episode_id, to_id);
        assert_eq!(rel.relationship_type, RelationshipType::ParentChild);
        assert_eq!(rel.metadata.reason, Some("Subtask of parent".to_string()));
    }

    #[test]
    fn test_relationship_with_reason() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = EpisodeRelationship::with_reason(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            "Requires API design".to_string(),
        );

        assert_eq!(rel.from_episode_id, from_id);
        assert_eq!(rel.to_episode_id, to_id);
        assert_eq!(rel.metadata.reason, Some("Requires API design".to_string()));
    }

    #[test]
    fn test_relationship_inverse() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = EpisodeRelationship::with_reason(
            from_id,
            to_id,
            RelationshipType::ParentChild,
            "Child task".to_string(),
        );

        let inverse = rel.inverse().expect("Should have inverse");
        assert_eq!(inverse.from_episode_id, to_id);
        assert_eq!(inverse.to_episode_id, from_id);
        assert_eq!(inverse.relationship_type, RelationshipType::ParentChild);
    }

    #[test]
    fn test_relationship_metadata() {
        let mut metadata = RelationshipMetadata::new();
        assert!(metadata.reason.is_none());
        assert!(metadata.created_by.is_none());
        assert!(metadata.priority.is_none());

        metadata.set_field("project".to_string(), "memory-system".to_string());
        assert_eq!(
            metadata.get_field("project"),
            Some(&"memory-system".to_string())
        );
    }

    #[test]
    fn test_relationship_metadata_with_creator() {
        let metadata =
            RelationshipMetadata::with_creator("Bug fix".to_string(), "alice".to_string());

        assert_eq!(metadata.reason, Some("Bug fix".to_string()));
        assert_eq!(metadata.created_by, Some("alice".to_string()));
    }

    #[test]
    fn test_relationship_serialization() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let rel = EpisodeRelationship::with_reason(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            "Test reason".to_string(),
        );

        let json = serde_json::to_string(&rel).unwrap();
        let deserialized: EpisodeRelationship = serde_json::from_str(&json).unwrap();

        assert_eq!(rel.from_episode_id, deserialized.from_episode_id);
        assert_eq!(rel.to_episode_id, deserialized.to_episode_id);
        assert_eq!(rel.relationship_type, deserialized.relationship_type);
        assert_eq!(rel.metadata.reason, deserialized.metadata.reason);
    }

    #[test]
    fn test_direction_enum() {
        // Just ensure the Direction enum variants compile
        let _ = Direction::Outgoing;
        let _ = Direction::Incoming;
        let _ = Direction::Both;
    }
}
