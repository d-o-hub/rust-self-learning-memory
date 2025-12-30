# Quick-Start Execution Guide

**Purpose**: Complete Phase 1 benchmarks with minimal friction
**Time Required**: 30-60 minutes
**Prerequisites**: Turso environment (local dev or cloud)

---

## One-Command Execution (If Turso CLI Available)

```bash
cd /workspaces/feat-phase3
./scripts/setup-turso-benchmarks.sh --local --run
```

This command will:
1. Start Turso dev server
2. Verify vector extensions
3. Run all benchmarks
4. Generate performance reports

---

## Option 1: Turso Local Dev Server (RECOMMENDED)

### Step 1: Install Turso CLI (if not installed)

```bash
# Try shell script
curl -sSfL https://get.turso.dev | sh

# Or use Homebrew (macOS)
brew install tursodatabase/tap/turso

# Verify installation
turso --version
```

### Step 2: Start Local Dev Server

```bash
# In one terminal, start server
turso dev --db-file /tmp/turso_benchmark.db

# Expected output:
# sqld listening on port 8080. Use the following URL:
# libsql://127.0.0.1:8080
```

**Keep this terminal running** while executing benchmarks.

### Step 3: Run Benchmarks (in another terminal)

```bash
cd /workspaces/feat-phase3

# Set environment variables
export TURSO_DATABASE_URL="libsql://127.0.0.1:8080"
export TURSO_AUTH_TOKEN=""

# Run benchmarks
cargo bench --bench turso_vector_performance \
  --features memory-storage-turso/turso_multi_dimension
```

### Step 4: Verify Success

Check benchmark output for:
```
Connecting to Turso at: libsql://127.0.0.1:8080
✓ Vector extensions verified: vector32() function available
```

### Step 5: View Results

```bash
# Open HTML report
open target/criterion/report/index.html

# Or view raw results
ls -la target/criterion/turso_vector_performance/
```

---

## Option 2: Turso Cloud Database (FASTEST)

### Step 1: Create Turso Account

1. Visit: https://turso.tech
2. Sign up (free tier available)
3. Create new database:
   - Name: `vector-benchmark-test`
   - Region: Choose closest region

### Step 2: Get Connection Details

```bash
# If Turso CLI is installed:
turso db create vector-benchmark-test
turso db show vector-benchmark-test

# Expected output:
# URL: libsql://<database-id>.turso.io
# Auth token: <auth-token>
```

**Copy URL and auth token.**

### Step 3: Set Environment Variables

```bash
cd /workspaces/feat-phase3

export TURSO_DATABASE_URL="libsql://<database-id>.turso.io"
export TURSO_AUTH_TOKEN="<auth-token>"
```

### Step 4: Run Benchmarks

```bash
cargo bench --bench turso_vector_performance \
  --features memory-storage-turso/turso_multi_dimension
```

### Step 5: Verify Success

Check for:
```
Connecting to Turso at: libsql://<database-id>.turso.io
✓ Vector extensions verified: vector32() function available
```

---

## Option 3: Use Setup Script (AUTOMATED)

### Local Dev Server

```bash
./scripts/setup-turso-benchmarks.sh --local --run
```

### Turso Cloud

```bash
# First, set environment variables
export TURSO_DATABASE_URL="libsql://<database-id>.turso.io"
export TURSO_AUTH_TOKEN="<auth-token>"

# Then run
./scripts/setup-turso-benchmarks.sh --env --run
```

### Interactive Mode

```bash
./scripts/setup-turso-benchmarks.sh
# Follow prompts to choose setup mode
```

---

## Troubleshooting

### Error: "turso: command not found"

**Cause**: Turso CLI not installed

**Fix**: Install Turso CLI (see Option 1, Step 1)

### Error: "connection refused"

**Cause**: Turso dev server not running

**Fix**: Start `turso dev --db-file /tmp/turso_benchmark.db` in a separate terminal

### Error: "Vector extensions NOT available"

**Cause**: Using `file://` URL instead of `libsql://`

**Fix**: Ensure `TURSO_DATABASE_URL` starts with `libsql://`

```bash
# WRONG:
export TURSO_DATABASE_URL="file:///tmp/benchmark.db"

# CORRECT:
export TURSO_DATABASE_URL="libsql://127.0.0.1:8080"
```

### Error: "no such function: vector32"

**Cause**: Not connected to Turso server (still using local SQLite)

**Fix**: Verify environment variables and restart benchmarks

```bash
# Check current values
echo $TURSO_DATABASE_URL
echo $TURSO_AUTH_TOKEN

# Should show libsql:// URL (not file://)
```

### Benchmarks Running Too Slow

**Possible Causes**:
1. Network latency (Turso cloud)
2. Dataset too small (DiskANN overhead > benefit)
3. First query (index not yet built)

**Fixes**:
1. Use local dev server instead of cloud
2. Test with larger datasets (1K, 10K)
3. Run benchmarks twice (second run uses built index)

---

## Expected Timeline

| Task | Time |
|------|------|
| Install Turso CLI (if needed) | 5-10 min |
| Start dev server / configure cloud | 2-5 min |
| Run benchmarks | 20-30 min |
| Verify results | 5-10 min |
| **Total** | **~30-60 min** |

---

## After Benchmarks Complete

### 1. Update Framework Reports

Edit these files with actual measurements:
- `/workspaces/feat-phase3/benchmark_results/phase1_multi_dimension/comparison_against_targets_corrected.md`
- `/workspaces/feat-phase3/benchmark_results/phase1_multi_dimension/final_validated_report.md`

Look for `*PENDING*` markers and fill in:
- Actual query times
- Improvement factors
- Memory usage measurements
- Scaling behavior data

### 2. Validate Vector Index Usage

Connect to Turso and run:

```sql
EXPLAIN QUERY PLAN
SELECT * FROM vector_top_k('idx_episode_embeddings_vector', <vector>, 10);
```

Expected: Shows `vector_top_k` table function usage, not full table scan.

### 3. Generate Final Report

Update `final_validated_report.md`:
- Mark all validation items as complete
- Add EXPLAIN QUERY PLAN outputs
- Document Phase 2 recommendations
- Sign off on Phase 1 completion

### 4. Proceed to Phase 2

Once Phase 1 is validated:
1. Analyze baseline performance
2. Identify optimization opportunities
3. Start Phase 2 (Index Optimization)

---

## Quick Reference

### Environment Variables

| Variable | Default | Purpose |
|-----------|----------|-----------|
| `TURSO_DATABASE_URL` | `libsql://127.0.0.1:8080` | Turso database URL |
| `TURSO_AUTH_TOKEN` | (empty string) | Auth token (cloud only) |

### Connection URLs

| Type | URL Format | Token Required |
|------|------------|---------------|
| Local dev | `libsql://127.0.0.1:8080` | No |
| Cloud | `libsql://<db-id>.turso.io` | Yes |
| Local SQLite | `file:///path/to/db` | No (no vector support) |

### Verification Commands

```bash
# Check vector extensions work
turso db shell libsql://127.0.0.1:8080
> SELECT vector32('0.1,0.2,0.3');

# Verify benchmark code uses libsql://
grep -n "libsql://" benches/turso_vector_performance.rs
```

---

**Guide Created**: 2025-12-30
**Purpose**: Enable quick execution of Phase 1 benchmarks
**Dependencies**: Turso CLI or Turso cloud credentials
**Estimated Time to Complete**: 30-60 minutes
