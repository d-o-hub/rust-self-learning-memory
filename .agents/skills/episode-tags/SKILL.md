---
name: episode-tags
description: Manage episode tags for organization, filtering, and retrieval across the memory system
---

# Episode Tags

## When to Use

- Organizing episodes by topic, project, or workflow
- Searching episodes by tag combinations
- Analyzing tag usage patterns
- Renaming tags for consistency

## CLI Commands

| Command | Purpose |
|---------|---------|
| `do-memory-cli tag add <ID> <TAG...>` | Add tags to an episode |
| `do-memory-cli tag remove <ID> <TAG...>` | Remove tags from an episode |
| `do-memory-cli tag set <ID> <TAG...>` | Replace all tags on an episode |
| `do-memory-cli tag list --episode <ID>` | List tags for one episode |
| `do-memory-cli tag list` | List all tags system-wide |
| `do-memory-cli tag search <TAG...>` | Find episodes by tags |
| `do-memory-cli tag show <ID>` | Show episode details with tags |
| `do-memory-cli tag rename <OLD> <NEW>` | Rename tag across all episodes |
| `do-memory-cli tag stats` | Tag usage statistics |

## MCP Tools

| Tool | Parameters | Purpose |
|------|-----------|---------|
| `add_episode_tags` | episode_id, tags | Add tags to episode |
| `remove_episode_tags` | episode_id, tags | Remove specific tags |
| `set_episode_tags` | episode_id, tags | Replace all tags |
| `get_episode_tags` | episode_id | Get tags for episode |
| `search_episodes_by_tags` | tags, match_all, limit | Find episodes by tags |

## Search Options

```bash
# OR logic (any tag matches)
do-memory-cli tag search rust async

# AND logic (all tags must match)
do-memory-cli tag search rust async --all

# Partial matching
do-memory-cli tag search "debug" --partial

# Case-sensitive
do-memory-cli tag search "API" --case-sensitive

# Limit results
do-memory-cli tag search performance --limit 5
```

## Tag Conventions

- Use lowercase kebab-case: `code-review`, `bug-fix`, `performance`
- Use domain prefixes for project scoping: `web/auth`, `cli/parsing`
- Color option available: `--color red` (for visual grouping)

## Stats & Maintenance

```bash
# Top 10 most-used tags
do-memory-cli tag stats --top 10

# Sort by recency
do-memory-cli tag stats --sort recent

# Dry-run rename before committing
do-memory-cli tag rename old-name new-name --dry-run
```
