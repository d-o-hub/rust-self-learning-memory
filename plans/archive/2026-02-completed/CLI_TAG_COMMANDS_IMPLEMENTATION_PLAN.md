# CLI Episode Tag Commands - Implementation Plan

**Date**: 2026-02-02  
**Priority**: P0 - Critical  
**Estimated Effort**: 15-20 hours  
**Status**: NOT IMPLEMENTED  

---

## Overview

This plan implements 6 CLI commands for episode tag management. The backend tagging functionality is already implemented and working via MCP, but CLI commands are missing.

**Current Status**: 
- ✅ Backend implemented in `memory-core/src/episodic/tags.rs`
- ✅ MCP tools exist in `memory-mcp/src/mcp/tools/episode_tags/`
- ❌ CLI commands completely missing

---

## Command Specifications

### 1. `memory-cli tag add` (3-4 hours)

**Purpose**: Add one or more tags to an episode

**Usage**:
```bash
memory-cli tag add <EPISODE_ID> <TAG>...

# Examples:
memory-cli tag add abc123 debugging production
memory-cli tag add "Debug Session" bug-fix critical
memory-cli tag add abc123 v1.2.0 --metadata '{"release":"beta"}'
```

**Arguments**:
- `<EPISODE_ID>`: Episode UUID or name
- `<TAG>...`: One or more tags to add

**Options**:
- `--metadata <JSON>`: Additional tag metadata
- `--force`: Overwrite existing tag if present

**Output**:
```
✓ Tags added successfully to "Debug Session" (abc123)
  Added: debugging, production (2 tags)
  Already present: bug-fix (skipped)
  Total tags: 5
```

**Validation**:
- Episode must exist
- Tags normalized (lowercase, trim whitespace, replace spaces with hyphens)
- Duplicate tags silently skipped (unless --force)

---

### 2. `memory-cli tag remove` (2-3 hours)

**Purpose**: Remove tags from an episode

**Usage**:
```bash
memory-cli tag remove <EPISODE_ID> <TAG>...
memory-cli tag remove <EPISODE_ID> --all

# Examples:
memory-cli tag remove abc123 debugging
memory-cli tag remove abc123 v1.0.0 v1.1.0
memory-cli tag remove abc123 --all --confirm
```

**Arguments**:
- `<EPISODE_ID>`: Episode UUID or name
- `<TAG>...`: Tags to remove (optional if --all)

**Options**:
- `--all`: Remove all tags from episode
- `--confirm`: Required when using --all

**Output**:
```
✓ Tags removed successfully from "Debug Session" (abc123)
  Removed: debugging (1 tag)
  Not found: obsolete (skipped)
  Remaining tags: 4
```

---

### 3. `memory-cli tag list` (3-4 hours)

**Purpose**: List tags for episodes or all tags in system

**Usage**:
```bash
memory-cli tag list [EPISODE_ID] [OPTIONS]
memory-cli tag list --all [OPTIONS]

# Examples:
memory-cli tag list abc123
memory-cli tag list --all
memory-cli tag list --all --format json
memory-cli tag list --all --sort count --limit 20
```

**Arguments**:
- `[EPISODE_ID]`: Episode to list tags for (optional)

**Options**:
- `--all`: List all tags in system with usage counts
- `--format <table|json|csv>`: Output format (default: table)
- `--sort <name|count|recent>`: Sort order for --all (default: count)
- `--limit <N>`: Limit results

**Output (for episode)**:
```
Tags for "Debug Session" (abc123):

  • debugging
  • production
  • critical
  • bug-fix
  • v1.2.0

Total: 5 tags
```

**Output (for --all)**:
```
All Tags in System:

┌────────────────┬────────────────┬──────────────────────┐
│ Tag            │ Episode Count  │ Last Used            │
├────────────────┼────────────────┼──────────────────────┤
│ debugging      │ 42             │ 2026-02-02 12:00     │
│ production     │ 38             │ 2026-02-02 11:45     │
│ bug-fix        │ 35             │ 2026-02-01 16:30     │
│ critical       │ 12             │ 2026-02-02 10:15     │
│ performance    │ 8              │ 2026-01-31 14:20     │
└────────────────┴────────────────┴──────────────────────┘

Total: 127 unique tags across 234 episodes
```

---

### 4. `memory-cli tag search` (4-5 hours)

**Purpose**: Search episodes by tags

**Usage**:
```bash
memory-cli tag search <TAG>... [OPTIONS]

# Examples:
memory-cli tag search debugging
memory-cli tag search debugging production --match all
memory-cli tag search bug-fix critical --limit 10 --format json
memory-cli tag search v1.* --pattern
```

**Arguments**:
- `<TAG>...`: Tags to search for

**Options**:
- `--match <any|all>`: Match mode (default: any)
  - `any`: Episodes with ANY of the tags
  - `all`: Episodes with ALL tags
- `--pattern`: Treat tags as glob patterns (supports wildcards)
- `--limit <N>`: Limit results (default: 50)
- `--format <table|json|csv>`: Output format
- `--sort <created|name>`: Sort order (default: created)

**Output**:
```
Episodes matching tags: debugging, production

┌──────────────────────────┬─────────────┬──────────┬──────────────────────┐
│ Episode Name             │ ID          │ Status   │ Created              │
├──────────────────────────┼─────────────┼──────────┼──────────────────────┤
│ Debug Session            │ abc123      │ complete │ 2026-02-02 11:00     │
│ Production Deploy        │ def456      │ complete │ 2026-02-01 15:30     │
│ Hotfix Investigation     │ 789ghi      │ active   │ 2026-02-02 09:15     │
└──────────────────────────┴─────────────┴──────────┴──────────────────────┘

Found 3 episodes with tags: debugging, production
Match mode: any
```

**Advanced Example**:
```bash
# Find all v1.x.x release episodes
memory-cli tag search "v1.*" --pattern

# Find critical bugs in production
memory-cli tag search critical bug-fix production --match all
```

---

### 5. `memory-cli tag rename` (2-3 hours)

**Purpose**: Rename a tag across all episodes

**Usage**:
```bash
memory-cli tag rename <OLD_TAG> <NEW_TAG> [OPTIONS]

# Examples:
memory-cli tag rename bug bugfix
memory-cli tag rename v1.2.0 release-1.2.0 --dry-run
memory-cli tag rename obsolete deprecated --confirm
```

**Arguments**:
- `<OLD_TAG>`: Current tag name
- `<NEW_TAG>`: New tag name

**Options**:
- `--dry-run`: Show what would be changed without applying
- `--confirm`: Required for rename (safety)

**Output**:
```
⚠ Tag Rename Operation

  Old tag: bug
  New tag: bugfix
  Episodes affected: 24

Confirm rename? [y/N]: y

✓ Tag renamed successfully
  Updated: 24 episodes
  Duration: 0.3s
```

**Validation**:
- Old tag must exist
- New tag must not exist (conflict check)
- Requires --confirm flag

---

### 6. `memory-cli tag stats` (3-4 hours)

**Purpose**: Show tag usage statistics and analytics

**Usage**:
```bash
memory-cli tag stats [OPTIONS]

# Examples:
memory-cli tag stats
memory-cli tag stats --top 20
memory-cli tag stats --format json
memory-cli tag stats --chart
```

**Options**:
- `--top <N>`: Show top N tags (default: 10)
- `--format <table|json|chart>`: Output format
- `--chart`: Display ASCII bar chart
- `--include-unused`: Include tags with 0 episodes

**Output (default)**:
```
Tag Usage Statistics

Overall:
  Total unique tags: 127
  Total tagged episodes: 234 (89% of all episodes)
  Average tags per episode: 2.8
  Most used tag: debugging (42 episodes)

Top 10 Tags by Usage:
┌────┬────────────────┬────────────────┬────────────┐
│ #  │ Tag            │ Episode Count  │ Percentage │
├────┼────────────────┼────────────────┼────────────┤
│ 1  │ debugging      │ 42             │ 17.9%      │
│ 2  │ production     │ 38             │ 16.2%      │
│ 3  │ bug-fix        │ 35             │ 15.0%      │
│ 4  │ critical       │ 12             │ 5.1%       │
│ 5  │ performance    │ 8              │ 3.4%       │
│ 6  │ refactoring    │ 7              │ 3.0%       │
│ 7  │ feature        │ 6              │ 2.6%       │
│ 8  │ v1.2.0         │ 5              │ 2.1%       │
│ 9  │ security       │ 4              │ 1.7%       │
│ 10 │ documentation  │ 3              │ 1.3%       │
└────┴────────────────┴────────────────┴────────────┘

Tag Trends (Last 30 Days):
  Most added: debugging (+12)
  Most removed: obsolete (-8)
  Fastest growing: security (+400%)
```

**Output (chart format)**:
```
Tag Usage Distribution:

debugging      ████████████████████████████████████████ 42
production     ████████████████████████████████████ 38
bug-fix        ███████████████████████████████████ 35
critical       ████████████ 12
performance    ████████ 8
refactoring    ███████ 7
feature        ██████ 6
v1.2.0         █████ 5
security       ████ 4
documentation  ███ 3
```

---

## Implementation Structure

### File Organization

```
memory-cli/src/commands/
├── tag.rs                    (NEW - main tag command handler)
├── tag/                      (NEW - submodule)
│   ├── mod.rs                (exports)
│   ├── add.rs                (add command)
│   ├── remove.rs             (remove command)
│   ├── list.rs               (list command)
│   ├── search.rs             (search command)
│   ├── rename.rs             (rename command)
│   └── stats.rs              (statistics command)
└── mod.rs                    (add tag module)
```

### Integration Points

**File**: `memory-cli/src/commands/mod.rs`

Add:
```rust
pub mod tag;

pub enum Command {
    // ... existing commands ...
    Tag(tag::TagCommand),
}
```

**File**: `memory-cli/src/main.rs`

Add to CLI parser:
```rust
#[derive(Debug, Subcommand)]
pub enum Commands {
    // ... existing commands ...
    /// Manage episode tags
    Tag(TagArgs),
}
```

---

## Dependencies

### Crate Dependencies

All dependencies likely already present:
- `comfy-table` - Table rendering (already used)
- `serde_json` - JSON parsing (already used)
- `glob` - Pattern matching for tag search (may need to add)

**Add to Cargo.toml if needed**:
```toml
glob = "0.3"
```

### Storage Layer

Uses existing:
- `EpisodicStorage::add_tag()`
- `EpisodicStorage::remove_tag()`
- `EpisodicStorage::get_tags()`
- `EpisodicStorage::search_by_tags()`

---

## Testing Strategy

### Unit Tests (per command)
```rust
#[test]
fn test_tag_add() {
    // Create episode
    // Add tags via CLI
    // Verify tags stored
    // Check output formatting
}

#[test]
fn test_tag_search_pattern() {
    // Create episodes with version tags
    // Search with pattern "v1.*"
    // Verify matches
}

#[test]
fn test_tag_rename() {
    // Tag multiple episodes
    // Rename tag
    // Verify all episodes updated
}
```

### Integration Tests
```rust
#[test]
fn test_tag_workflow() {
    // 1. Create episode
    // 2. Add tags
    // 3. Search by tags
    // 4. List tags
    // 5. Rename tag
    // 6. Verify stats
    // 7. Remove tags
}
```

### Edge Cases
- Empty tag names
- Very long tag names (>100 chars)
- Special characters in tags
- Unicode tags
- Concurrent tag operations
- 1000+ tags on single episode

---

## Implementation Phases

### Phase 1: Core Commands (Days 1-2)
**Effort**: 8-10 hours

1. Implement `tag add` (3-4h)
2. Implement `tag remove` (2-3h)
3. Implement `tag list` (3-4h)
4. Basic unit tests (2h)

**Deliverable**: Basic tag CRUD working

---

### Phase 2: Search & Query (Day 3)
**Effort**: 4-5 hours

1. Implement `tag search` (4-5h)
   - Include pattern matching
   - Match mode (any/all)

**Deliverable**: Tag search functional

---

### Phase 3: Advanced Features (Day 4)
**Effort**: 5-7 hours

1. Implement `tag rename` (2-3h)
2. Implement `tag stats` (3-4h)
   - Include chart visualization

**Deliverable**: Full feature set complete

---

### Phase 4: Polish & Testing (Day 5)
**Effort**: 3-4 hours

1. Integration tests (2h)
2. Documentation (1h)
3. Error message improvements (1h)

**Deliverable**: Production-ready CLI

---

## Success Criteria

- [ ] All 6 commands implemented and tested
- [ ] Pattern matching working (glob syntax)
- [ ] Tag normalization consistent
- [ ] Statistics accurate and helpful
- [ ] Chart visualization working
- [ ] Integration tests pass
- [ ] Documentation complete

---

## User Experience Features

### Tag Normalization
Auto-normalize tags for consistency:
- `Debugging` → `debugging` (lowercase)
- `bug fix` → `bug-fix` (replace spaces)
- `  v1.0  ` → `v1.0` (trim whitespace)

### Smart Suggestions
When tag not found, suggest similar:
```bash
$ memory-cli tag search debuging
✗ No episodes found with tag: debuging

Did you mean?
  • debugging (42 episodes)
  • debug (8 episodes)
```

### Batch Operations
Allow file input for bulk tagging:
```bash
$ memory-cli tag add abc123 @tags.txt
✓ Added 10 tags from tags.txt
```

### Export/Import
```bash
# Export all tags
memory-cli tag list --all --format json > tags.json

# Import tags
memory-cli tag import tags.json
```

---

## Performance Considerations

### Large Tag Sets
- Pagination for `tag list --all`
- Caching for `tag stats`
- Index-based search

### Concurrent Operations
- Tag operations should be atomic
- Handle concurrent add/remove gracefully

---

## Next Steps

1. Review and approve this plan
2. Create feature branch: `feat/cli-tag-commands`
3. Start with Phase 1 (core commands)
4. Daily progress updates in plans folder

---

**Total Effort**: 15-20 hours (2-3 business days)  
**Priority**: P0 - Required for complete CLI experience  
**Dependencies**: None (backend already implemented)  
**Blocker**: No - can implement immediately
