# Session State Preservation

When compacting context, ALWAYS preserve:

| What to Preserve | Why |
|------------------|-----|
| Last passing/failing test names + output | Reproducibility, regression detection |
| Most recent build status (success/error + message) | Know if you're in broken state |
| Files created/modified this session (path + why) | Track what changed, enables rollback |
| Open TODOs with current state | Prevents losing work-in-progress |
| Env vars/config set this session | Re-establish environment |
| Decisions made (WHY, not WHAT) | Enables contextual future decisions |

## DO NOT Compress

- Exact file paths
- Error messages (verbatim)
- Test names
- Numeric results (counts, sizes, checksums)

## Rationale

Context compaction risks losing debugging clues. Preserving test names, build status, and file changes maintains the "breadcrumb trail" needed to understand what happened in this session without forcing full re-execution or re-discovery.

Example preservation:
```
Tests: 2 fail (test_embed_vector, test_turso_insert)
Build: error - "missing trait impl Write for &str"
Files: do-memory-core/src/embed.rs (added async embed)
Env: TURSO_DATABASE_URL=libsql://...
```

## See Also
- Use `context-compaction` skill for detailed guidance