//! Dependency graph for batch operations

use super::types::{BatchOperation, BatchRequest};
use std::collections::{HashMap, HashSet};

/// Dependency graph for batch operations
#[derive(Debug)]
pub struct DependencyGraph {
    /// Operations indexed by ID
    operations: HashMap<String, BatchOperation>,
    /// Operation IDs in insertion order
    operation_order: Vec<String>,
    /// Adjacency list (operation -> dependencies)
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse adjacency list (operation -> dependents)
    dependents: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// Create a new dependency graph from operations
    pub fn new(operations: Vec<BatchOperation>) -> Result<Self, String> {
        let mut graph = Self {
            operations: HashMap::new(),
            operation_order: Vec::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        };

        // Build operation index and preserve order
        for op in operations {
            if graph.operations.contains_key(&op.id) {
                return Err(format!("Duplicate operation ID: {}", op.id));
            }
            graph.operation_order.push(op.id.clone());
            graph.operations.insert(op.id.clone(), op);
        }

        // Build dependency and dependent relationships
        for (id, op) in &graph.operations {
            for dep in &op.depends_on {
                // Validate dependency exists
                if !graph.operations.contains_key(dep) {
                    return Err(format!(
                        "Operation '{}' depends on unknown operation '{}'",
                        id, dep
                    ));
                }

                // Add to dependencies
                graph
                    .dependencies
                    .entry(id.clone())
                    .or_default()
                    .insert(dep.clone());

                // Add to dependents (reverse)
                graph
                    .dependents
                    .entry(dep.clone())
                    .or_default()
                    .insert(id.clone());
            }
        }

        // Validate no cycles
        graph.validate_acyclic()?;

        Ok(graph)
    }

    /// Validate that the graph is acyclic (no circular dependencies)
    fn validate_acyclic(&self) -> Result<(), String> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for id in self.operations.keys() {
            if !visited.contains(id) {
                self.detect_cycle(id, &mut visited, &mut stack)?;
            }
        }

        Ok(())
    }

    /// Detect cycles using DFS
    fn detect_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<(), String> {
        visited.insert(node.to_string());
        stack.insert(node.to_string());

        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.detect_cycle(dep, visited, stack)?;
                } else if stack.contains(dep) {
                    return Err(format!("Circular dependency detected: {} -> {}", node, dep));
                }
            }
        }

        stack.remove(node);
        Ok(())
    }

    /// Get operations that have no pending dependencies (ready to execute)
    pub fn get_ready_operations(&self, completed: &HashSet<String>) -> Vec<BatchOperation> {
        self.operations
            .values()
            .filter(|op| {
                // Check if all dependencies are completed
                self.dependencies
                    .get(&op.id)
                    .map(|deps| deps.iter().all(|dep| completed.contains(dep)))
                    .unwrap_or(true) // No dependencies means ready
                    && !completed.contains(&op.id) // Not already completed
            })
            .cloned()
            .collect()
    }

    /// Get total number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get operations in insertion order
    pub fn operations_in_order(&self) -> Vec<BatchOperation> {
        self.operation_order
            .iter()
            .filter_map(|id| self.operations.get(id).cloned())
            .collect()
    }
}

impl From<BatchRequest> for DependencyGraph {
    fn from(request: BatchRequest) -> Self {
        DependencyGraph::new(request.operations).unwrap_or_else(|e| {
            // In case of error, create an empty graph
            DependencyGraph {
                operations: HashMap::new(),
                operation_order: Vec::new(),
                dependencies: HashMap::new(),
                dependents: HashMap::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_dependency_graph_simple() {
        let ops = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
        ];

        let graph = DependencyGraph::new(ops).unwrap();
        assert_eq!(graph.len(), 2);
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_dependency_graph_with_dependencies() {
        let ops = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec!["op1".to_string()],
            },
        ];

        let graph = DependencyGraph::new(ops).unwrap();
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn test_dependency_graph_cycle() {
        let ops = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec!["op2".to_string()],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec!["op1".to_string()],
            },
        ];

        let result = DependencyGraph::new(ops);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_dependency_graph_unknown_dependency() {
        let ops = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec!["unknown".to_string()],
            },
        ];

        let result = DependencyGraph::new(ops);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown operation"));
    }

    #[test]
    fn test_dependency_graph_duplicate_id() {
        let ops = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
        ];

        let result = DependencyGraph::new(ops);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate operation ID"));
    }

    #[test]
    fn test_get_ready_operations() {
        let ops = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec!["op1".to_string()],
            },
        ];

        let graph = DependencyGraph::new(ops).unwrap();

        // Initially, only op1 should be ready
        let completed = HashSet::new();
        let ready = graph.get_ready_operations(&completed);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "op1");

        // After op1 completes, op2 should be ready
        let mut completed = HashSet::new();
        completed.insert("op1".to_string());
        let ready = graph.get_ready_operations(&completed);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "op2");
    }
}
