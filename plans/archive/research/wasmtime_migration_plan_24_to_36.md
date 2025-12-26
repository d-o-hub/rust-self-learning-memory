# Wasmtime Migration Plan: 24.0.5 → 36.0.3

## Overview
Major version upgrade from wasmtime 24.0.5 to 36.0.3. This is a significant jump that requires careful migration planning and testing.

## Breaking Changes Summary
Based on research findings, the key breaking changes include:

### 1. **Rust Version Requirement**
- **Before**: Requires Rust 1.78.0+
- **After**: Requires Rust 1.89.0+
- **Impact**: Need to update CI/CD pipelines and developer environments

### 2. **WASI HTTP Header Handling**
- **Before**: Simple string comparison for forbidden headers
- **After**: Uses `DEFAULT_FORBIDDEN_HEADERS.contains(name)` check
- **Code Change Required**:
```rust
// Before
if name == "custom-forbidden-header" {
    // handle forbidden header
}

// After
if DEFAULT_FORBIDDEN_HEADERS.contains(name) || name == "custom-forbidden-header" {
    // handle forbidden header
}
```

### 3. **Component Model Modifications**
- Breaking changes in component model API
- May affect WASI integration if used

### 4. **Multiple Returns**
- Multiple return values are now gated by default
- May require explicit feature flags for certain APIs

## Migration Steps

### Phase 1: Environment Preparation
1. **Update Rust Toolchain**
   ```bash
   rustup update
   rustup default stable
   ```

2. **Update CI/CD Pipelines**
   - Check `.github/workflows/` for Rust version requirements
   - Update to Rust 1.89.0+ in GitHub Actions

### Phase 2: Dependency Updates
1. **Update Cargo.toml Files**
   ```toml
   # Before
   wasmtime = "24.0.5"
   wasmtime-wasi = "24.0.5"
   
   # After  
   wasmtime = "36.0.3"
   wasmtime-wasi = "36.0.3"
   ```

2. **Run Dependency Updates**
   ```bash
   cargo update
   cargo tree -p wasmtime
   cargo tree -p wasmtime-wasi
   ```

### Phase 3: Code Updates
1. **WASI HTTP Header Handling**
   - Review all WASI HTTP header handling code
   - Update to use `DEFAULT_FORBIDDEN_HEADERS.contains(name)`
   - Test custom forbidden header handling

2. **Component Model Updates**
   - Review component model usage
   - Update any deprecated API calls
   - Test component model functionality

3. **Multiple Returns Handling**
   - Review functions with multiple return values
   - Add explicit feature flags if needed
   - Test all affected functionality

### Phase 4: Testing & Validation
1. **Unit Tests**
   ```bash
   cargo test -p wasmtime-related
   ```

2. **Integration Tests**
   ```bash
   cargo test --all
   ```

3. **WASI Functionality Tests**
   - Test all WASI operations
   - Verify HTTP header handling
   - Check component model integration

## Risk Assessment
- **Risk Level**: 🟡 MEDIUM
- **Migration Effort**: 8-16 hours
- **Breaking Changes**: Moderate
- **Testing Required**: Extensive

## Files Likely Affected
Based on initial codebase scan:
- `memory-mcp/` - May use wasmtime for WASM execution
- Any WASI-related code
- Component model usage (if any)

## Rollback Plan
If migration fails:
1. Revert to wasmtime 24.0.5 in Cargo.toml
2. Run `cargo update` to restore old versions
3. Document failure reasons for future attempts

## Success Criteria
- [ ] All tests pass with wasmtime 36.0.3
- [ ] WASI functionality works correctly
- [ ] HTTP header handling works as expected
- [ ] No component model regressions
- [ ] CI/CD pipelines pass with new Rust version

## Timeline
- **Phase 1**: 1-2 hours (Environment setup)
- **Phase 2**: 1 hour (Dependencies)
- **Phase 3**: 4-8 hours (Code updates)
- **Phase 4**: 2-4 hours (Testing)
- **Total**: 8-16 hours estimated

## References
- Wasmtime 36.0 release notes
- WASI HTTP specification updates
- Component model documentation
- Migration guides from bytecodealliance/wasmtime
