# Dead Code Analysis Report

**Generated**: 2026-03-07
**Total dead_code annotations**: 134 (in source files, excluding target/)

## Categorization

### High-Impact Files (Top 10)

| File | Count | Category | Action |
|------|-------|----------|--------|
| `memory-mcp/src/bin/server_impl/types.rs` | 13 | MCP types | Review for feature flags |
| `memory-core/src/embeddings/real_model/model.rs` | 10 | Embeddings | Review for unused model types |
| `memory-cli/src/errors.rs` | 10 | CLI errors | Error variants may be used |
| `memory-core/tests/common/helpers.rs` | 7 | Test helpers | Acceptable |
| `memory-storage-redb/src/tables.rs` | 6 | Storage | Review for feature flags |
| `memory-mcp/src/bin/server_impl/oauth.rs` | 5 | OAuth | Feature-gated code |
| `memory-core/src/memory/types.rs` | 5 | Memory types | Review for usage |
| `memory-core/src/memory/learning_ops.rs` | 5 | Learning | Review for usage |
| `memory-core/src/memory/core/struct_priv.rs` | 5 | Memory core | Review for usage |
| `memory-core/src/embeddings/openai/utils.rs` | 5 | OpenAI | Review for usage |

### Categories

1. **Test Helpers** (~25 annotations): Acceptable - test infrastructure
2. **Error Variants** (~15 annotations): May be used in pattern matching
3. **Feature-Gated Code** (~30 annotations): Should have proper `#[cfg]` attributes
4. **Genuinely Unused** (~20 annotations): Should be removed
5. **API Types** (~44 annotations): May be used by consumers

## Recommended Actions

1. **Remove genuinely unused code** (~20 annotations)
2. **Add proper cfg attributes** for feature-gated code (~30 annotations)
3. **Keep** test helpers, error variants, and API types (~84 annotations)

## Target

- Current: 134 annotations
- Target: ≤10 annotations (removing genuinely unused only)
- Acceptable: ~84 annotations (test helpers, errors, API types)