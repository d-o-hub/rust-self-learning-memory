//! Error types for episode relationship operations.
//!
//! This module defines error types for validation failures, removal failures,
//! and graph algorithm errors in the relationship management system.

use std::fmt;
use uuid::Uuid;

use super::RelationshipType;

/// Errors that can occur during relationship validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Attempted to create a self-relationship (episode relating to itself).
    SelfRelationship {
        /// The episode ID that was used for both source and target.
        episode_id: Uuid,
    },
    /// Attempted to create a duplicate relationship.
    DuplicateRelationship {
        /// Source episode ID.
        from: Uuid,
        /// Target episode ID.
        to: Uuid,
        /// Relationship type.
        rel_type: RelationshipType,
    },
    /// A cycle was detected in the relationship graph.
    CycleDetected {
        /// Source episode ID that would create the cycle.
        from: Uuid,
        /// Target episode ID that would create the cycle.
        to: Uuid,
        /// Path of episode IDs forming the cycle.
        cycle_path: Vec<Uuid>,
    },
    /// Invalid priority value provided.
    InvalidPriority {
        /// The priority value that was provided.
        priority: u8,
        /// The valid range for priority values.
        valid_range: (u8, u8),
    },
    /// Referenced episode was not found.
    EpisodeNotFound {
        /// The episode ID that was not found.
        episode_id: Uuid,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfRelationship { episode_id } => {
                write!(f, "Self-relationship not allowed for episode {episode_id}")
            }
            Self::DuplicateRelationship { from, to, rel_type } => {
                write!(f, "Duplicate relationship: {from} -> {to} ({rel_type:?})")
            }
            Self::CycleDetected {
                from,
                to,
                cycle_path,
            } => {
                write!(
                    f,
                    "Cycle detected: {from} -> {to}, cycle path: {cycle_path:?}"
                )
            }
            Self::InvalidPriority {
                priority,
                valid_range,
            } => {
                write!(
                    f,
                    "Invalid priority {priority}, must be in range {valid_range:?}"
                )
            }
            Self::EpisodeNotFound { episode_id } => {
                write!(f, "Episode not found: {episode_id}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Errors that can occur during relationship removal.
#[derive(Debug, Clone, PartialEq)]
pub enum RemovalError {
    /// The relationship to remove was not found.
    NotFound {
        /// The relationship ID that was not found.
        id: Uuid,
    },
}

impl fmt::Display for RemovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { id } => write!(f, "Relationship not found: {id}"),
        }
    }
}

impl std::error::Error for RemovalError {}

/// Errors that can occur during graph algorithm operations.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError {
    /// An error occurred during graph traversal.
    TraversalError {
        /// Description of the error.
        message: String,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TraversalError { message } => write!(f, "Graph traversal error: {message}"),
        }
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let id = Uuid::new_v4();
        let err = ValidationError::SelfRelationship { episode_id: id };
        assert!(err.to_string().contains("Self-relationship"));
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn test_duplicate_relationship_error_display() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let err = ValidationError::DuplicateRelationship {
            from,
            to,
            rel_type: RelationshipType::DependsOn,
        };
        let msg = err.to_string();
        assert!(msg.contains("Duplicate relationship"));
        assert!(msg.contains(&from.to_string()));
        assert!(msg.contains(&to.to_string()));
    }

    #[test]
    fn test_cycle_detected_error_display() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let cycle_path = vec![from, to, from];
        let err = ValidationError::CycleDetected {
            from,
            to,
            cycle_path,
        };
        let msg = err.to_string();
        assert!(msg.contains("Cycle detected"));
        assert!(msg.contains(&from.to_string()));
        assert!(msg.contains(&to.to_string()));
    }

    #[test]
    fn test_invalid_priority_error_display() {
        let err = ValidationError::InvalidPriority {
            priority: 15,
            valid_range: (1, 10),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid priority"));
        assert!(msg.contains("15"));
        assert!(msg.contains("(1, 10)"));
    }

    #[test]
    fn test_episode_not_found_error_display() {
        let id = Uuid::new_v4();
        let err = ValidationError::EpisodeNotFound { episode_id: id };
        assert!(err.to_string().contains("Episode not found"));
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn test_removal_error_display() {
        let id = Uuid::new_v4();
        let err = RemovalError::NotFound { id };
        assert!(err.to_string().contains("Relationship not found"));
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn test_graph_error_display() {
        let err = GraphError::TraversalError {
            message: "Cycle detected".to_string(),
        };
        assert!(err.to_string().contains("Graph traversal error"));
        assert!(err.to_string().contains("Cycle detected"));
    }

    #[test]
    fn test_validation_error_equality() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let err1 = ValidationError::SelfRelationship { episode_id: id1 };
        let err2 = ValidationError::SelfRelationship { episode_id: id1 };
        let err3 = ValidationError::SelfRelationship { episode_id: id2 };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_removal_error_equality() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let err1 = RemovalError::NotFound { id: id1 };
        let err2 = RemovalError::NotFound { id: id1 };
        let err3 = RemovalError::NotFound { id: id2 };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_graph_error_equality() {
        let err1 = GraphError::TraversalError {
            message: "error1".to_string(),
        };
        let err2 = GraphError::TraversalError {
            message: "error1".to_string(),
        };
        let err3 = GraphError::TraversalError {
            message: "error2".to_string(),
        };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(ValidationError::SelfRelationship {
            episode_id: Uuid::new_v4(),
        });
        assert!(err.to_string().contains("Self-relationship"));
    }
}
