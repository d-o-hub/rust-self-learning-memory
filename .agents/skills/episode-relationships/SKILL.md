---
name: episode-relationships
description: Create and traverse typed relationships between episodes for dependency tracking and knowledge graphs
---

# Episode Relationships

## When to Use

- Linking related episodes (parent-child, dependencies, sequences)
- Building knowledge graphs of connected work
- Validating relationship integrity (cycle detection)
- Visualizing episode dependency graphs
- Finding related episodes by traversal

## Relationship Types

| Type | Meaning | Example |
|------|---------|---------|
| `parent-child` | Hierarchical containment | Sprint contains tasks |
| `depends-on` | Must complete first | Feature depends on API |
| `follows` | Sequential ordering | Step 2 follows Step 1 |
| `related-to` | Loose association | Similar approach used |
| `blocks` | Prevents progress | Bug blocks release |
| `duplicates` | Same work | Redundant episodes |
| `references` | Informational link | Docs reference impl |

## CLI Commands

| Command | Purpose |
|---------|---------|
| `do-memory-cli relationship add <SRC> <TGT> <TYPE>` | Create relationship |
| `do-memory-cli relationship remove <ID>` | Remove relationship |
| `do-memory-cli relationship list <ID>` | List relationships for episode |
| `do-memory-cli relationship find <ID>` | Find related episodes (traversal) |
| `do-memory-cli relationship info <REL_ID>` | Show relationship details |
| `do-memory-cli relationship graph <ID>` | Generate dependency graph |
| `do-memory-cli relationship validate <ID>` | Check for cycles |

## MCP Tools

| Tool | Parameters | Purpose |
|------|-----------|---------|
| `add_episode_relationship` | source_id, target_id, relationship_type, reason, priority | Create relationship |
| `remove_episode_relationship` | relationship_id | Delete relationship |
| `get_episode_relationships` | episode_id, direction, type_filter | Get relationships |
| `find_related_episodes` | episode_id, max_depth, type_filter | Traverse graph |
| `check_relationship_exists` | source_id, target_id, relationship_type | Check existence |
| `get_dependency_graph` | episode_id, format | Export graph (DOT/JSON/text) |
| `validate_no_cycles` | episode_id | Detect cycles |
| `get_topological_order` | episode_ids | Topological sort |

## Graph Operations

```bash
# Generate Graphviz DOT output
do-memory-cli relationship graph <ID> --format dot > graph.dot
dot -Tpng graph.dot -o graph.png

# JSON graph for programmatic use
do-memory-cli relationship graph <ID> --format json

# Validate no cycles exist
do-memory-cli relationship validate <ID>
```

## Direction Filtering

```bash
# Only outgoing (this episode -> others)
do-memory-cli relationship list <ID> --direction outgoing

# Only incoming (others -> this episode)
do-memory-cli relationship list <ID> --direction incoming

# Both directions (default)
do-memory-cli relationship list <ID> --direction both
```

## Best Practices

- Use `depends-on` for build/execution ordering
- Use `follows` for temporal sequences within a workflow
- Use `related-to` sparingly — prefer specific types
- Run `validate` after adding `depends-on` to catch cycles
- Use `get_topological_order` before executing dependent episodes
