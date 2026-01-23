# Writing Skill Descriptions

## Importance

The description is **critical** - Claude uses it to decide when to invoke the skill.

## Structure Formula

```
[Action verb] [what it does]. Use this when [specific scenarios].
```

## Key Elements

1. **Clear action** (verb start)
2. **What problem it solves**
3. **When to invoke it**
4. **Keywords** for matching

## Good Examples

### Excellent
```yaml
description: Debug and fix failing tests in Rust projects. Use this skill when tests fail and you need to diagnose root causes, fix async/await issues, or handle race conditions.
```

### Excellent
```yaml
description: Implement new features systematically with proper testing and documentation. Use when adding new functionality to the codebase that requires following established patterns.
```

### Good
```yaml
description: Create and manage GitHub Actions workflows for Rust projects. Use when setting up CI/CD pipelines, configuring build caching, or troubleshooting workflow failures.
```

## Bad Examples

### Too Vague
```yaml
description: Helps with testing
```
❌ Missing action, scope, and when-to-use

### Missing Context
```yaml
description: Provides guidance on building APIs
```
❌ No action verb, no specific scenarios

### Too Long
```yaml
description: This skill is designed to help you with all aspects of testing including unit tests integration tests end-to-end tests and also includes debugging capabilities and performance optimization techniques and best practices for writing maintainable tests in Rust projects
```
❌ Exceeds 1024 chars, unfocused

## Quick Reference

| Do | Don't |
|----|-------|
| Start with action verb | Use vague language |
| Include specific scenarios | Skip when-to-use |
| Keep under 1024 chars | Write essays |
| Add keywords | Use jargon without explanation |
| Be specific | Overgeneralize |
