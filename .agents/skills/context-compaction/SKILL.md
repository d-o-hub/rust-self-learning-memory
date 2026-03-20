---
name: context-compaction
description: Preserve critical session state when compacting context. Use when context window is filling up and you need to summarize/reduce while keeping essential debugging information.
---

# Context Compaction

Preserve essential state when context window fills up.

## Always Preserve

| Item | Why |
|------|-----|
| Test names + output (pass/fail) | Regression detection, reproducibility |
| Build status (success/error + msg) | Know if in broken state |
| Files modified (path + why) | Track changes, enable rollback |
| Open TODOs with state | Prevents losing WIP |
| Env vars set this session | Re-establish environment |
| Decisions made (WHY) | Enables contextual future decisions |

## DO NOT Compress

- Exact file paths
- Error messages (verbatim)
- Test names
- Numeric results (counts, sizes, checksums)

## Example Output

```
Tests: 2 fail (test_embed_vector, test_turso_insert)
Build: error - "missing trait impl Write for &str"  
Files: memory-core/src/embed.rs (added async embed)
TODOs: [in_progress] refactor embed.rs - 60% done
Env: TURSO_DATABASE_URL=libsql://...
Decision: switched to async embed to fix blocking in hot path
```

## See Also

- [Session State Preservation](../../../agent_docs/session_state_preservation.md) - Full guidelines
- [Token Efficiency](../../../agent_docs/token_efficiency.md) - Tool selection priority