//! Architecture fitness test — enforce crate layering at test time.
//!
//! The workspace layering contract is:
//!   memory-types → memory-core → memory-storage-{redb,turso} → memory-mcp
//!                                                                    → memory-cli
//!
//! This test uses `cargo_metadata` to resolve the actual dependency graph
//! and fails with a HARNESS VIOLATION message if any forbidden edge exists.

use std::collections::HashMap;
use std::path::Path;

use regex::Regex;

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
                violations.push(format!("  {from} → {to}"));
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

/// Collect workspace-relative `.rs` paths, skipping build output and VCS data.
fn collect_rust_files(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if matches!(name.as_str(), "target" | ".git" | "node_modules") {
                    continue;
                }
                walk(&path, root, out);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            {
                if let Ok(rel) = path.strip_prefix(root) {
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }

    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

/// The `cargo-mutants` config in `Cargo.toml` selects files by regex. A pattern
/// that matches nothing — a renamed module, a plural typo — disables that shard
/// silently: the run reports success while testing none of it. This gate makes
/// such a pattern a hard failure.
#[test]
fn mutation_config_patterns_resolve_to_real_files() {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to read cargo metadata");

    let config = &metadata.workspace_metadata["cargo-mutants"];
    let examine = config["examine_re"]
        .as_array()
        .expect("Cargo.toml is missing [workspace.metadata.cargo-mutants] examine_re");
    let exclude: Vec<Regex> = config["exclude_re"]
        .as_array()
        .expect("exclude_re must be an array")
        .iter()
        .map(|entry| {
            let pattern = entry.as_str().expect("exclude_re entries must be strings");
            Regex::new(pattern).expect("exclude_re entry is not a valid regex")
        })
        .collect();

    let files = collect_rust_files(metadata.workspace_root.as_std_path());
    let mut problems = Vec::new();

    for entry in examine {
        let pattern = entry.as_str().expect("examine_re entries must be strings");
        let regex = Regex::new(pattern).expect("examine_re entry is not a valid regex");
        let selected = files
            .iter()
            .any(|file| regex.is_match(file) && !exclude.iter().any(|ex| ex.is_match(file)));
        if !selected {
            problems.push(format!("  examine_re pattern selects no file: {pattern}"));
        }
    }

    assert!(
        problems.is_empty(),
        "HARNESS VIOLATION: [mutation-config] — cargo-mutants examines nothing for:\n{}\n\
         Fix: point each pattern at an existing module, relative to the workspace root.",
        problems.join("\n")
    );
}
