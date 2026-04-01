# CI Issue: Semver Check Timeout

**Date**: 2026-04-01
**Issue**: Semver Check job timing out at 15 minutes during build phase
**Severity**: P1 - Blocking automerge

## Problem

The `Semver Check` job in `.github/workflows/ci.yml` exceeds the 15-minute timeout while building rustdocs for the workspace. The job was canceled while building `do-memory-mcp` crate.

### Root Cause

1. **Timeout too short**: 15 minutes insufficient for workspace-wide semver checks
2. **No baseline caching**: Each run rebuilds rustdocs from scratch
3. **Building all crates**: Checks entire workspace even for minor changes

## Solution

Implement timeout increase + baseline caching:

### Changes to `.github/workflows/ci.yml`

1. **Increase timeout** from 15 to 30 minutes
2. **Add baseline caching** to reuse rustdocs from main branch
3. **Use `--baseline-branch main`** for PR checks

### Implementation

```yaml
semver-check:
  name: Semver Check
  runs-on: ubuntu-latest
  timeout-minutes: 30  # Increased from default 15
  if: github.actor != 'dependabot[bot]'
  steps:
    - uses: actions/checkout@v6
      with:
        fetch-depth: 0  # Required for baseline-branch

    - name: Setup isolated target dir
      run: |
        chmod +x scripts/setup-target-dir.sh
        ./scripts/setup-target-dir.sh ci-semver-check

    - uses: dtolnay/rust-toolchain@stable

    - uses: Swatinem/rust-cache@v2.9.1
      with:
        save-if: ${{ github.ref == 'refs/heads/main' }}

    - name: Cache semver baseline
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/semver-checks
          target/semver-checks
        key: semver-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-${{ github.base_ref || 'main' }}
        restore-keys: |
          semver-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-
          semver-${{ runner.os }}-

    - name: Install cargo-semver-checks
      run: cargo install --locked cargo-semver-checks

    - name: Check semver compatibility
      run: |
        if [ "${{ github.event_name }}" == "pull_request" ]; then
          cargo semver-checks check-release --workspace --baseline-branch main
        else
          cargo semver-checks check-release --workspace
        fi
      continue-on-error: true
```

## Status

- [ ] Implement fix in `.github/workflows/ci.yml`
- [ ] Test on new PR
- [ ] Document in memory if successful

## References

- cargo-semver-checks: https://github.com/obi1kenobi/cargo-semver-checks
- Baseline caching docs: https://github.com/obi1kenobi/cargo-semver-checks# caching