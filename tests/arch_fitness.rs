//! Architecture fitness test — enforce crate layering at test time.
//!
//! The workspace layering contract is:
//!   memory-types → memory-core → memory-storage-{redb,turso} → memory-mcp
//!                                                                    → memory-cli
//!
//! This test uses `cargo_metadata` to resolve the actual dependency graph
//! and fails with a HARNESS VIOLATION message if any forbidden edge exists.

use std::collections::HashMap;

/// Forbidden dependency edges: (from, to)
/// The "from" crate must NOT depend on the "to" crate.
const FORBIDDEN_EDGES: &[(&str, &str)] = &[
    // Core must not depend on CLI or MCP
    ("do-memory-core", "do-memory-cli"),
    ("do-memory-core", "do-memory-mcp"),
    // Storage crates must not depend on CLI or MCP
    ("do-memory-storage-redb", "do-memory-cli"),
    ("do-memory-storage-redb", "do-memory-mcp"),
    ("do-memory-storage-turso", "do-memory-cli"),
    ("do-memory-storage-turso", "do-memory-mcp"),
    // MCP and CLI must not depend on each other
    ("do-memory-mcp", "do-memory-cli"),
    ("do-memory-cli", "do-memory-mcp"),
];

#[test]
#[expect(clippy::panic, clippy::manual_assert)]
fn enforce_crate_layering() {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to read cargo metadata");

    // Build a map: package_name -> Vec<dependency_name>
    let mut dep_graph: HashMap<String, Vec<String>> = HashMap::new();

    for package in &metadata.packages {
        let deps: Vec<String> = package
            .dependencies
            .iter()
            .filter(|dep| {
                // Only consider workspace members (path dependencies)
                dep.path.is_some()
            })
            .map(|dep| dep.name.clone())
            .collect();
        // cargo_metadata 0.23: Package::name is PackageName, not String.
        dep_graph.insert(package.name.to_string(), deps);
    }

    let mut violations = Vec::new();

    for &(from, to) in FORBIDDEN_EDGES {
        if let Some(deps) = dep_graph.get(from) {
            if deps.contains(&to.to_string()) {
                violations.push(format!("  {} → {}", from, to));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "HARNESS VIOLATION: [architecture] — Forbidden crate dependencies detected:\n{}\n\
             Fix: Remove the forbidden dependency from the crate's Cargo.toml.\n\
             Layering contract: memory-types → memory-core → memory-storage-* → memory-mcp/memory-cli",
            violations.join("\n")
        );
    }
}
