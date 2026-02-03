# Episode Update Command - Quick Reference

## Overview

The `memory episode update` command allows you to modify existing episodes in the memory system. You can update the task description, manage tags, and add metadata.

## Basic Syntax

```bash
memory episode update <EPISODE_ID> [OPTIONS]
```

## Options

| Option | Description | Example |
|--------|-------------|---------|
| `--description <TEXT>` | Update the task description | `--description "Fixed bug"` |
| `--add-tag <TAGS>` | Add tags (comma-separated) | `--add-tag bug,security` |
| `--remove-tag <TAGS>` | Remove tags (comma-separated) | `--remove-tag wip,todo` |
| `--set-tags <TAGS>` | Replace all tags (comma-separated) | `--set-tags done,tested` |
| `--metadata <KEY=VALUE>` | Add metadata (can be used multiple times) | `--metadata priority=high` |
| `--dry-run` | Show what would be done without making changes | `--dry-run` |

## Common Use Cases

### 1. Update Task Description

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --description "Implemented user authentication"
```

### 2. Add Status Tags

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --add-tag in-progress,backend
```

### 3. Mark as Completed

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --add-tag completed \
  --remove-tag wip \
  --metadata completed_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
```

### 4. Change Priority

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --metadata priority=urgent
```

### 5. Replace All Tags

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --set-tags feature,auth,security
```

### 6. Combine Multiple Updates

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --description "Completed OAuth2 implementation" \
  --add-tag done \
  --remove-tag in-progress \
  --metadata complexity=high \
  --metadata reviewed=true
```

### 7. Preview Changes (Dry Run)

```bash
memory episode update 01234567-89ab-cdef-0123-456789abcdef \
  --description "Test update" \
  --add-tag test-tag \
  --dry-run
```

## Tag Management Strategies

### Adding Tags
Use `--add-tag` to append new tags without removing existing ones:

```bash
memory episode update <ID> --add-tag frontend,javascript
```

### Removing Tags
Use `--remove-tag` to delete specific tags:

```bash
memory episode update <ID> --remove-tag todo,wip
```

### Replacing All Tags
Use `--set-tags` to completely replace the tag list:

```bash
memory episode update <ID> --set-tags completed,tested,deployed
```

## Metadata Best Practices

Metadata is useful for storing structured information about episodes:

### Common Metadata Fields

```bash
# Priority levels
--metadata priority=low|medium|high|urgent

# Complexity
--metadata complexity=simple|moderate|complex

# Status tracking
--metadata status=planning|in-progress|review|done

# Assignees
--metadata assignee=username

# Time tracking
--metadata estimated_hours=4
--metadata actual_hours=6

# External references
--metadata issue_id=PROJ-123
--metadata pull_request=456

# Quality indicators
--metadata tested=true
--metadata reviewed=true
--metadata documented=false
```

### Multiple Metadata Fields

You can specify `--metadata` multiple times:

```bash
memory episode update <ID> \
  --metadata priority=high \
  --metadata complexity=complex \
  --metadata assignee=alice \
  --metadata tested=false
```

## Workflow Examples

### Feature Development Workflow

```bash
# Start with create
memory episode create \
  --task "Add user profile page" \
  --context-file feature.json

# Mark as in progress
memory episode update <ID> --add-tag in-progress

# Update description as scope changes
memory episode update <ID> \
  --description "Add user profile page with avatar upload and bio editing"

# Mark as completed
memory episode update <ID> \
  --remove-tag in-progress \
  --add-tag completed \
  --metadata status=done \
  --metadata tested=true
```

### Bug Fix Workflow

```bash
# Start
memory episode create \
  --task "Fix login timeout issue" \
  --context-file bug.json

# Categorize
memory episode update <ID> \
  --add-tag bug,security,urgent \
  --metadata priority=urgent \
  --metadata issue_id=BUG-456

# Update with findings
memory episode update <ID> \
  --description "Fixed race condition in token validation" \
  --metadata root_cause=missing_lock

# Complete
memory episode update <ID> \
  --set-tags bug-fix,security,done \
  --metadata fix_verified=true
```

### Code Review Workflow

```bash
# Track review
memory episode update <ID> \
  --add-tag in-review \
  --metadata reviewer=bob \
  --metadata review_requested_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Request changes
memory episode update <ID> \
  --remove-tag in-review \
  --add-tag needs-changes \
  --metadata review_status=changes_requested

# After rework
memory episode update <ID> \
  --add-tag in-review \
  --remove-tag needs-changes \
  --metadata review_round=2

# Approve
memory episode update <ID> \
  --set-tags reviewed,approved \
  --metadata review_status=approved \
  --metadata approved_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
```

## Tips and Tricks

### 1. Find Episode ID First
```bash
# List recent episodes
memory episode list --limit 10 --sort newest

# Search for specific episode
memory episode list --tags feature --sort newest
```

### 2. Use Short Tags
Keep tags short and meaningful:
- ✅ `auth`, `frontend`, `urgent`
- ❌ `authentication-system`, `frontend-development`, `urgency-high`

### 3. Consistent Metadata
Use consistent metadata keys across episodes:
- Use `priority` not `urgency` or `importance`
- Use `complexity` not `difficulty` or `hardness`

### 4. Batch Updates with Scripts
Update multiple episodes using shell scripts:

```bash
# Mark all feature episodes as reviewed
for id in $(memory episode list --tags feature --format json | jq -r '.[].episode_id'); do
  memory episode update $id --add-tag reviewed
done
```

### 5. Combine with View Command
```bash
# View current state before updating
memory episode view <ID>

# Make updates
memory episode update <ID> --description "New description"

# Verify changes
memory episode view <ID>
```

## Error Messages

### Invalid UUID Format
```
Error: Invalid episode ID format: not-a-uuid
Hint: Episode IDs must be valid UUIDs (e.g., 01234567-89ab-cdef-0123-456789abcdef)
```

### Episode Not Found
```
Error: Failed to update description: Episode not found: 00000000-0000-0000-0000-000000000000
Hint: Use 'memory episode view <ID>' to verify the episode exists
```

### Invalid Metadata Format
```
Error: Invalid metadata format: 'badkey'. Expected 'key=value'
Hint: Use format: --metadata key=value
```

## Integration with MCP

The same functionality is available through the MCP server:

```json
{
  "tool": "update_episode",
  "arguments": {
    "episode_id": "01234567-89ab-cdef-0123-456789abcdef",
    "description": "Updated description",
    "add_tags": ["feature", "backend"],
    "metadata": {
      "priority": "high",
      "complexity": "moderate"
    }
  }
}
```

## See Also

- `memory episode create` - Create new episodes
- `memory episode view` - View episode details
- `memory episode list` - List and filter episodes
- `memory episode delete` - Delete episodes
- `memory tag add` - Alternative tag management
