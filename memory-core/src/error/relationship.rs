//! Error types for relationship operations.
//!
//! This module defines specific error types for relationship management,
//! providing detailed context for failures during relationship operations.

use std::fmt;
use uuid::Uuid;

use crate::episode::RelationshipType;

/// Errors that can occur during relationship operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipError {
    /// Attempted to create a self-referencing relationship.
    SelfReference {
        /// The episode ID that was used for both source and target.
        episode_id: Uuid,
    },
    /// Attempted to create a duplicate relationship.
    Duplicate {
        /// Source episode ID.
        from: Uuid,
        /// Target episode ID.
        to: Uuid,
        /// Relationship type.
        rel_type: RelationshipType,
    },
    /// A cycle was detected in the relationship graph.
    CycleDetected {
        /// Path of episode IDs forming the cycle.
        path: Vec<Uuid>,
    },
    /// The requested relationship was not found.
    NotFound {
        /// The relationship ID that was not found.
        relationship_id: Uuid,
    },
    /// Invalid relationship type was specified.
    InvalidType {
        /// The invalid type name that was provided.
        type_name: String,
    },
    /// General validation failure.
    ValidationFailed {
        /// The reason for validation failure.
        reason: String,
    },
}

impl fmt::Display for RelationshipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfReference { episode_id } => {
                write!(
                    f,
                    "Cannot create self-referencing relationship for episode {episode_id}"
                )
            }
            Self::Duplicate { from, to, rel_type } => {
                write!(
                    f,
                    "Duplicate relationship: {from} -> {to} (type: {rel_type})"
                )
            }
            Self::CycleDetected { path } => {
                write!(f, "Cycle detected in relationship graph: ")?;
                for (i, id) in path.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{id}")?;
                }
                Ok(())
            }
            Self::NotFound { relationship_id } => {
                write!(f, "Relationship not found: {relationship_id}")
            }
            Self::InvalidType { type_name } => {
                write!(f, "Invalid relationship type: {type_name}")
            }
            Self::ValidationFailed { reason } => {
                write!(f, "Relationship validation failed: {reason}")
            }
        }
    }
}

impl std::error::Error for RelationshipError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_reference_error() {
        let id = Uuid::new_v4();
        let err = RelationshipError::SelfReference { episode_id: id };
        let msg = err.to_string();
        assert!(msg.contains("self-referencing"));
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn test_duplicate_error() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let err = RelationshipError::Duplicate {
            from,
            to,
            rel_type: RelationshipType::DependsOn,
        };
        let msg = err.to_string();
        assert!(msg.contains("Duplicate relationship"));
        assert!(msg.contains(&from.to_string()));
        assert!(msg.contains(&to.to_string()));
        assert!(msg.contains("depends_on"));
    }

    #[test]
    fn test_cycle_detected_error() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let err = RelationshipError::CycleDetected {
            path: vec![id1, id2, id3, id1],
        };
        let msg = err.to_string();
        assert!(msg.contains("Cycle detected"));
        assert!(msg.contains(&id1.to_string()));
        assert!(msg.contains(&id2.to_string()));
        assert!(msg.contains(&id3.to_string()));
    }

    #[test]
    fn test_not_found_error() {
        let id = Uuid::new_v4();
        let err = RelationshipError::NotFound {
            relationship_id: id,
        };
        let msg = err.to_string();
        assert!(msg.contains("Relationship not found"));
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn test_invalid_type_error() {
        let err = RelationshipError::InvalidType {
            type_name: "invalid_type".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid relationship type"));
        assert!(msg.contains("invalid_type"));
    }

    #[test]
    fn test_validation_failed_error() {
        let err = RelationshipError::ValidationFailed {
            reason: "Priority out of range".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("validation failed"));
        assert!(msg.contains("Priority out of range"));
    }

    #[test]
    fn test_error_equality() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let err1 = RelationshipError::SelfReference { episode_id: id1 };
        let err2 = RelationshipError::SelfReference { episode_id: id1 };
        let err3 = RelationshipError::SelfReference { episode_id: id2 };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(RelationshipError::SelfReference {
            episode_id: Uuid::new_v4(),
        });
        assert!(err.to_string().contains("self-referencing"));
    }
}
