---
name: skill-creator
description: Create new Claude Code skills with proper directory structure, SKILL.md file, and YAML frontmatter. Use this skill when you need to create a new reusable knowledge module for Claude Code.
---

# Skill Creator

Create new Claude Code skills following the official format and best practices.

## When to Use

- Creating a new reusable knowledge module
- Adding specialized guidance for specific tasks
- Building domain-specific expertise into Claude Code
- Need to ensure proper skill format and structure

## Skill Structure

A Claude Code skill consists of:

```
.claude/skills/
└── skill-name/
    └── SKILL.md
```

### SKILL.md Format

```markdown
---
name: skill-name
description: Clear description of what this skill does and when to use it (max 1024 chars)
---

# Skill Title

[Skill content in Markdown]
```

## Naming Requirements

**Skill Name Rules**:
- Lowercase letters only
- Numbers allowed
- Hyphens for word separation (no underscores)
- No spaces
- Max 64 characters
- Descriptive and clear

**Examples**:
- ✅ `episode-management`
- ✅ `test-debugging`
- ✅ `api-integration`
- ✗ `Episode_Management` (no uppercase, no underscores)
- ✗ `test debugging` (no spaces)

## Description Best Practices

The description is **critical** - Claude uses it to decide when to invoke the skill.

**Good Description Structure**:
```
[Action verb] [what it does] [when to use it]
```

**Examples**:

✅ Good:
```yaml
description: Debug and fix failing tests in Rust projects. Use this skill when tests fail and you need to diagnose root causes, fix async/await issues, or handle race conditions.
```

✅ Good:
```yaml
description: Implement new features systematically with proper testing and documentation. Use when adding new functionality to the codebase.
```

✗ Too vague:
```yaml
description: Helps with testing
```

✗ Missing when-to-use:
```yaml
description: Provides guidance on building APIs
```

## Skill Creation Process

### Step 1: Define Purpose

```markdown
What problem does this skill solve?
- Specific task: [e.g., "Deploy to production"]
- Domain: [e.g., "deployment", "testing", "documentation"]
- User need: [e.g., "Ensure safe deployments"]
```

### Step 2: Choose Name

```markdown
Skill name: [lowercase-with-hyphens]
- Descriptive: Clearly indicates purpose
- Concise: Not too long
- Unique: Doesn't conflict with existing skills
```

### Step 3: Write Description

```markdown
description: [Action] [what it does]. Use this when [specific scenarios].

Key elements:
1. Clear action (verb)
2. What problem it solves
3. When to invoke it
4. Keywords Claude can match on
```

### Step 4: Structure Content

**Recommended Sections**:

1. **Introduction**: Brief overview of skill purpose
2. **When to Use**: Specific scenarios for invocation
3. **Core Concepts**: Key knowledge needed
4. **Process/Workflow**: Step-by-step guidance
5. **Examples**: Concrete usage examples
6. **Best Practices**: Do's and don'ts
7. **Integration**: How this works with other skills/agents

**Content Guidelines**:
- Clear, concise language
- Actionable instructions
- Concrete examples
- Code snippets where helpful
- Checklists for processes
- Visual diagrams (ASCII art) for complex flows

### Step 5: Create Files

```bash
# Create directory
mkdir -p .claude/skills/skill-name

# Create SKILL.md with content
cat > .claude/skills/skill-name/SKILL.md << 'EOF'
---
name: skill-name
description: Your description here
---

# Skill Title

[Your skill content]
EOF
```

### Step 6: Test and Validate

**Validation Checklist**:
- [ ] Directory name matches skill name
- [ ] SKILL.md file exists
- [ ] YAML frontmatter is valid
- [ ] Name follows naming rules (lowercase, hyphens)
- [ ] Description is clear and specific (< 1024 chars)
- [ ] Content is well-structured
- [ ] Examples are provided
- [ ] Markdown is properly formatted

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
```

## Skill Maintenance

### Updating Skills

When updating existing skills:
1. Preserve backward compatibility
2. Update description if scope changes
3. Add new sections without removing old ones
4. Update examples to reflect current best practices
5. Maintain clear version history in git

### Deprecating Skills

If a skill becomes obsolete:
1. Update description to indicate deprecation
2. Point to replacement skill
3. Keep file for backward compatibility
4. Consider removing after transition period

## Best Practices Summary

### DO:
✓ Write clear, specific descriptions
✓ Include concrete examples
✓ Structure content logically
✓ Use consistent formatting
✓ Test skill by using it
✓ Update README.md to list new skill
✓ Follow naming conventions

### DON'T:
✗ Use vague or generic descriptions
✗ Skip examples
✗ Make names too long or unclear
✗ Forget YAML frontmatter
✗ Use uppercase or underscores in names
✗ Exceed 1024 chars in description

## Validation Command

After creating a skill, validate it:

```bash
# Check structure
test -f .claude/skills/skill-name/SKILL.md && echo "✓ Structure correct"

# Check YAML frontmatter
head -n 5 .claude/skills/skill-name/SKILL.md | grep "^name:" && echo "✓ YAML valid"

# Check name format
[[ $(grep "^name:" .claude/skills/skill-name/SKILL.md | cut -d' ' -f2) =~ ^[a-z0-9-]+$ ]] && echo "✓ Name format correct"
```

## Summary

Creating effective skills:
1. **Purpose**: Solve specific, well-defined problems
2. **Naming**: Clear, lowercase, hyphenated names
3. **Description**: Specific, actionable, includes when-to-use
4. **Structure**: Well-organized with clear sections
5. **Examples**: Concrete, realistic usage examples
6. **Testing**: Validate structure and use the skill

Skills are the foundation of Claude Code's knowledge. Well-designed skills make Claude more effective at autonomous task execution.
