## Skill Templates

### Template 1: Process Skill

```markdown
---
name: process-name
description: [Action] [what] [when to use]
---

# Process Name

Brief description of what this process achieves.

## When to Use

- Scenario 1
- Scenario 2

## Prerequisites

- Requirement 1
- Requirement 2

## Process Steps

### Step 1: [Action]
Instructions for step 1

### Step 2: [Action]
Instructions for step 2

## Quality Checklist

- [ ] Check 1
- [ ] Check 2

## Examples

### Example 1: [Scenario]
[Concrete example]

## Best Practices

✓ Do this
✗ Don't do this

## Integration

How this skill works with other skills/agents.
```

### Template 2: Knowledge Skill

```markdown
---
name: domain-knowledge
description: [Topic] knowledge and guidance for [use case]
---

# Domain Knowledge

Overview of knowledge domain.

## Core Concepts

### Concept 1
Explanation

### Concept 2
Explanation

## Guidelines

### Guideline 1
Details

## Patterns

### Pattern 1: [Name]
**Use When**: [scenario]
**Implementation**: [how-to]

## Anti-Patterns

### Anti-Pattern 1: [Name]
**Problem**: [what's wrong]
**Solution**: [correct approach]

## References

- Related skill 1
- Related skill 2
```

### Template 3: Tool Skill

```markdown
---
name: tool-usage
description: Use [tool] for [purpose]. Invoke when [scenarios]
---

# Tool Usage: [Tool Name]

Guide for effectively using [tool].

## When to Use

- Use case 1
- Use case 2

## Basic Usage

### Command Structure
```
tool [options] [arguments]
```

### Common Operations

#### Operation 1
```
tool command1
```

#### Operation 2
```
tool command2
```

## Advanced Usage

### Advanced Operation 1
Details and examples

## Troubleshooting

### Issue 1
**Symptom**: [what happens]
**Solution**: [how to fix]

## Best Practices

✓ Recommendation 1
✓ Recommendation 2
```

## Integration with Agent Creator

When creating skills that work with agents:

1. **Reference agents in skill**: Mention which agents use this skill
2. **Skill-agent coordination**: Ensure skill complements agent capabilities
3. **Invocation clarity**: Make clear when skill vs agent is appropriate

## Project-Specific Considerations

### For Rust Self-Learning Memory Project

**Domain-Specific Skills**:
- Episode management (start, log, complete)
- Pattern extraction and storage
- Memory retrieval optimization
- Turso/redb synchronization
- Async/Tokio patterns

**Skill Naming Convention**:
- `episode-[operation]` for episode-related skills
- `storage-[operation]` for storage operations
- `pattern-[operation]` for pattern handling
- `memory-[operation]` for memory operations

**Integration Requirements**:
- Reference AGENTS.md standards
- Include examples using project structure
- Consider self-learning memory tracking

## Examples

### Example 1: Creating a Deployment Skill

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

### Example 2: Creating a Testing Skill

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
