# Skill Creation Examples

## Example 1: Deployment Skill

```bash
# 1. Create directory
mkdir -p .claude/skills/production-deploy

# 2. Create SKILL.md
cat > .claude/skills/production-deploy/SKILL.md << 'EOF'
---
name: production-deploy
description: Deploy Rust applications to production safely with pre-deployment checks, rollback procedures, and monitoring. Use when deploying to production environments.
---

# Production Deployment

Guide for safe production deployments of Rust applications.

## When to Use

- Deploying new releases to production
- Updating production systems
- Rolling back problematic deployments

## Pre-Deployment Checklist

- [ ] All tests passing
- [ ] Code reviewed and approved
- [ ] Changelog updated
- [ ] Database migrations tested
- [ ] Rollback plan prepared

## Deployment Process

### Step 1: Pre-Deployment Checks
```bash
cargo test --all
cargo clippy -- -D warnings
cargo build --release
```

### Step 2: Deploy
```bash
# Deploy to staging first
./deploy.sh staging

# Verify staging
./verify.sh staging

# Deploy to production
./deploy.sh production
```

### Step 3: Post-Deployment Verification
- Monitor error rates
- Check key metrics
- Verify functionality

## Rollback Procedure

If deployment fails:
```bash
./rollback.sh production
```

## Best Practices

✓ Always deploy to staging first
✓ Monitor during and after deployment
✓ Have rollback plan ready
✗ Never skip pre-deployment checks
✗ Don't deploy on Friday afternoon
EOF
```

## Example 2: Property Testing Skill

```bash
mkdir -p .claude/skills/property-testing

cat > .claude/skills/property-testing/SKILL.md << 'EOF'
---
name: property-testing
description: Write property-based tests using QuickCheck or proptest for Rust code. Use when you need to test properties that should hold for many inputs rather than specific examples.
---

# Property-Based Testing

Guide for writing effective property-based tests in Rust.

## When to Use

- Testing properties that should hold universally
- Discovering edge cases automatically
- Testing complex logic with many input combinations
- Replacing large test suites with property tests

## Core Concepts

### Properties vs Examples

**Example-based test**:
```rust
assert_eq!(reverse(vec![1, 2, 3]), vec![3, 2, 1]);
```

**Property-based test**:
```rust
// Property: reverse(reverse(x)) == x
proptest! {
    fn reverse_involution(vec: Vec<i32>) {
        let reversed_twice = reverse(reverse(vec.clone()));
        assert_eq!(vec, reversed_twice);
    }
}
```

## Implementation

### Setup
```toml
[dev-dependencies]
proptest = "1.0"
```

### Writing Properties
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_name(input: InputType) {
        // Test property
        prop_assert!(condition);
    }
}
```

## Common Properties

1. **Idempotence**: `f(f(x)) == f(x)`
2. **Involution**: `f(f(x)) == x`
3. **Commutativity**: `f(x, y) == f(y, x)`
4. **Associativity**: `f(f(x, y), z) == f(x, f(y, z))`

## Best Practices

✓ Test properties, not implementations
✓ Use shrinking to find minimal failing cases
✓ Combine with example-based tests
✗ Don't test trivial properties
✗ Don't make properties too specific
EOF
```

## Example 3: Minimal Skill

```bash
mkdir -p .claude/skills/quick-reference

cat > .claude/skills/quick-reference/SKILL.md << 'EOF'
---
name: quick-reference
description: Quick reference for common commands and patterns. Use when you need a fast lookup for syntax or command examples.
---

# Quick Reference

Fast lookup for common Rust commands and patterns.

## Build Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Debug build |
| `cargo build --release` | Optimized build |
| `cargo check` | Type check only |
| `cargo clippy` | Linting |

## Test Commands

| Command | Purpose |
|---------|---------|
| `cargo test` | Run all tests |
| `cargo test --lib` | Library tests only |
| `cargo test --doc` | Doc tests only |

## Common Patterns

### Error Handling
```rust
fn example() -> anyhow::Result<T> {
    // Use anyhow for application errors
}
```

### Async Runtime
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Tokio runtime
}
```
EOF
```
