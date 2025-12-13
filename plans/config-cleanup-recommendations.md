# Config Files Cleanup Recommendations

**Date**: 2025-12-11
**Analysis Method**: GOAP-orchestrated comprehensive audit
**Status**: Ready for review and implementation

---

## Executive Summary

After a comprehensive GOAP-orchestrated analysis of all configuration files in the project:
- **Total configs found**: 41+ files (23 TOML, 12 YAML, 6 JSON, 2 ENV)
- **Build/Cargo configs**: 13 Cargo.toml files (all necessary for Rust workspace)
- **Production configs**: 6 actively used files
- **Orphaned/duplicate**: 7 files recommended for cleanup
- **Development configs**: 6 IDE/tooling configs (keep as-is)

---

## Priority 1: Safe to Remove (ORPHANED)

These files are NOT referenced anywhere in code, tests, docs, or scripts:

### 1. `/workspaces/feat-phase3/scripts/test-config.toml` ❌ REMOVE
- **Status**: Untracked (not in git)
- **Usage**: No references found
- **Reason**: Appears to be a leftover from when test configs were in root
- **Action**: `rm scripts/test-config.toml`

### 2. `/workspaces/feat-phase3/scripts/test-config-ep.toml` ❌ REMOVE
- **Status**: Untracked (not in git)
- **Usage**: No references found
- **Reason**: Orphaned test config, never used
- **Action**: `rm scripts/test-config-ep.toml`

### 3. `/workspaces/feat-phase3/scripts/test-config-mem.toml` ❌ REMOVE
- **Status**: Untracked (not in git)
- **Usage**: No references found
- **Reason**: Orphaned test config, never used
- **Action**: `rm scripts/test-config-mem.toml`

### 4. Root test configs (already deleted from git) ✅ ALREADY REMOVED
These show as deleted in git status - ensure they stay deleted:
- `test-config.toml`
- `test-config-ep.toml`
- `test-config-mem.toml`

**Impact**: ZERO - These files are not referenced anywhere
**Risk**: NONE

---

## Priority 2: Duplicates to Consolidate

### 5. `/workspaces/feat-phase3/memory-cli/test-config.toml` ❌ REMOVE (Duplicate)
- **Status**: Tracked, 30 lines
- **Duplicate of**: `/workspaces/feat-phase3/memory-cli/config/test-config.toml` (identical content)
- **Usage**: No direct references; tests create configs dynamically via `test_utils.rs`
- **Reason**: Exact duplicate; keep canonical version in `config/` subdirectory
- **Action**:
  ```bash
  rm memory-cli/test-config.toml
  # Keep: memory-cli/config/test-config.toml
  ```

**Impact**: ZERO - No code references the root-level copy
**Risk**: NONE - Tests use dynamic config creation

---

## Priority 3: Investigate Further

### 6. `/workspaces/feat-phase3/data/test-cli.toml` ⚠️ INVESTIGATE
- **Status**: Tracked, 15 lines
- **Usage**: No direct references found in code or docs
- **Observation**: Test utilities create similar configs dynamically
- **Recommendation**:
  1. Verify no manual testing scripts use this file
  2. Check if any local development workflows depend on it
  3. If unused, consider removing; if used, document it
- **Action**:
  ```bash
  # Search for any overlooked references:
  grep -r "data/test-cli.toml" . --exclude-dir=target --exclude-dir=.git

  # If no results, safe to remove:
  # rm data/test-cli.toml
  ```

**Impact**: LOW - Likely unused, but verify first
**Risk**: LOW - Can be recreated if needed

---

## Keep As-Is: Production & Essential Configs

### Production Application Configs ✅ KEEP

#### 7. `/workspaces/feat-phase3/memory-cli/config/memory-cli.toml` ✅ KEEP
- **Type**: Production template
- **Usage**: Referenced in Docker, setup scripts, CLI loader, extensive documentation
- **Locations**:
  - Code: `memory-cli/src/config.rs` (default search path)
  - Docker: `memory-cli/docker/Dockerfile`, `docker-compose.yml`
  - Scripts: `scripts/setup-local-db.sh`
  - Docs: 5+ documentation files
- **Reason**: PRIMARY production configuration template

#### 8. `/workspaces/feat-phase3/mcp-config-memory.json` ✅ KEEP
- **Type**: Production MCP server template
- **Usage**: Referenced in MCP production readiness docs
- **Reason**: Template for MCP server configuration

### IDE & Development Configs ✅ KEEP

#### 9. `/workspaces/feat-phase3/.mcp.json` ✅ KEEP
- **Type**: IDE MCP stdio configuration (untracked - intentional)
- **Usage**: Local development MCP server config
- **Reason**: Active development tool configuration
- **Note**: Untracked status is intentional for local customization

#### 10. `/workspaces/feat-phase3/opencode.json` ✅ KEEP
- **Type**: OpenCode IDE configuration (tracked, modified)
- **Usage**: AI provider + MCP server configuration for OpenCode IDE
- **Reason**: Active development environment config
- **Note**: Consider `.gitignore` if purely local, or commit if team-shared

#### 11. `/workspaces/feat-phase3/.devcontainer.json` ✅ KEEP
- **Type**: VS Code Dev Container config (untracked - intentional)
- **Usage**: Development container configuration
- **Reason**: Supports containerized development workflow
- **Note**: Untracked status is intentional for local customization

#### 12. `/workspaces/feat-phase3/memory-cli/config/test-config.toml` ✅ KEEP
- **Type**: Test configuration template
- **Usage**: Used by test utilities as template
- **Reason**: Canonical test config location (in config/ subdirectory)

### Rust Tooling Configs ✅ KEEP (All necessary)

- `.cargo/config.toml` - Cargo build configuration
- `.clippy.toml` - Clippy linter rules
- `.test-quality.toml` - Test quality thresholds
- `rust-toolchain.toml` - Rust version specification
- `rustfmt.toml` - Code formatting rules
- `deny.toml` - Dependency security/license checks
- All `Cargo.toml` files (13 total) - Workspace and crate manifests

### CI/CD & GitHub Configs ✅ KEEP (All necessary)

- `.codecov.yml` - Code coverage reporting
- `.yamllint.yml` - YAML linting rules
- `.github/workflows/*.yml` (6 files) - GitHub Actions CI/CD
- `.github/dependabot.yml` - Dependency updates

### Environment Files ✅ KEEP

- `/workspaces/feat-phase3/.env` - Local environment variables (tracked, but should be in `.gitignore`)
- `/workspaces/feat-phase3/memory-cli/.env.example` - Environment template (tracked)

---

## Implementation Plan

### Step 1: Remove Orphaned Files (Priority 1)
```bash
# Remove untracked orphaned test configs from scripts/
rm scripts/test-config.toml
rm scripts/test-config-ep.toml
rm scripts/test-config-mem.toml

# Verify root test configs remain deleted (they are)
git status | grep -E "test-config.*\.toml"
```

### Step 2: Remove Duplicate (Priority 2)
```bash
# Remove duplicate test config from memory-cli root
rm memory-cli/test-config.toml

# Verify canonical version remains
ls -la memory-cli/config/test-config.toml
```

### Step 3: Investigate data/test-cli.toml (Priority 3)
```bash
# Search for any references
grep -r "data/test-cli.toml" . --exclude-dir=target --exclude-dir=.git

# If no results, remove:
rm data/test-cli.toml

# If references found, document them
```

### Step 4: Verify No Breakage
```bash
# Run full test suite
cargo test --all

# Run CLI tests specifically
cd memory-cli && cargo test

# Verify build succeeds
cargo build --all --release
```

### Step 5: Update Documentation (if needed)
- No documentation updates needed (removed files weren't documented)
- Consider adding note to README about config file locations

---

## Summary of Actions

| File | Action | Priority | Risk |
|------|--------|----------|------|
| `scripts/test-config.toml` | DELETE | P1 | None |
| `scripts/test-config-ep.toml` | DELETE | P1 | None |
| `scripts/test-config-mem.toml` | DELETE | P1 | None |
| `memory-cli/test-config.toml` | DELETE (duplicate) | P2 | None |
| `data/test-cli.toml` | INVESTIGATE then likely DELETE | P3 | Low |
| All production configs | KEEP | - | - |
| All IDE configs | KEEP | - | - |
| All Rust tooling configs | KEEP | - | - |
| All CI/CD configs | KEEP | - | - |

**Total files to remove**: 4-5 (depending on investigation of data/test-cli.toml)
**Total files to keep**: 36+

---

## Validation Checklist

Before finalizing cleanup:
- [ ] Verify all orphaned files have no hidden references
- [ ] Confirm duplicate removal doesn't break tests
- [ ] Investigate data/test-cli.toml usage
- [ ] Run full test suite
- [ ] Build project successfully
- [ ] Update .gitignore if needed (for .env files)
- [ ] Commit cleanup with clear message

---

## Notes for Future Maintenance

1. **Test configs**: Tests create configs dynamically via `test_utils.rs` - no static test configs needed in root
2. **Config search order**: `config.rs` searches for configs in specific order (see config.rs:91-96)
3. **IDE configs**: `.mcp.json`, `opencode.json`, `.devcontainer.json` are intentionally untracked for local customization
4. **Production template**: `memory-cli/config/memory-cli.toml` is the canonical production config template
5. **Environment files**: Consider adding `.env` to `.gitignore` to prevent accidental secret commits

---

## Recommended Commands

```bash
# One-liner to clean up all orphaned/duplicate configs
rm -f scripts/test-config*.toml memory-cli/test-config.toml

# After investigation, optionally remove data/test-cli.toml
# rm -f data/test-cli.toml

# Verify cleanup
cargo test --all && cargo build --all --release
```

---

**End of Recommendations**
