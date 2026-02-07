---
name: skill-creator
description: >
  Create new Claude Code skills following the agentskills.io specification.
  Use when creating reusable knowledge modules, adding specialized guidance,
  or building domain-specific expertise. Ensures proper YAML frontmatter,
  directory structure, and progressive disclosure. Validates against
  agentskills.io specification.
---

# Skill Creator

Create Claude Code skills following the official agentskills.io specification.

## Quick Reference

| Resource | Purpose |
|----------|---------|
| **[Specification](specification.md)** | agentskills.io spec compliance |
| **[Structure Guide](structure.md)** | Directory format and organization |
| **[Naming Rules](naming.md)** | Skill naming requirements |
| **[Description Guide](description.md)** | Writing effective descriptions |
| **[Templates](templates.md)** | Ready-to-use skill templates |
| **[Examples](examples.md)** | Complete walkthroughs |
| **[Validation](validation.md)** | Validation commands |

## When to Use

- Creating new reusable knowledge modules
- Adding specialized task guidance
- Building domain-specific expertise
- Ensuring spec compliance
- Validating existing skills

## Quick Start

### 1. Create Directory Structure

```bash
mkdir -p .claude/skills/my-skill/{rules,references,scripts}
```

### 2. Write SKILL.md

```markdown
---
name: my-skill
description: >
  What this skill does and when to use it.
  Include trigger terms for matching.
---

# My Skill

## Overview
Brief description...

## Quick Reference
- **[Rules](rules/)** - Detailed rules
- **[Reference](references/)** - Technical details

## Usage
When to invoke this skill...
```

### 3. Validate

```bash
# Check line count (< 250 lines)
wc -l .claude/skills/my-skill/SKILL.md

# Check frontmatter syntax
head -5 .claude/skills/my-skill/SKILL.md
```

## Required SKILL.md Format

### YAML Frontmatter (Line 1)

```yaml
---
name: skill-name                    # Required
description: >                     # Required (max 1024 chars)
  Multi-line description of what
  this skill does and when to use it.
license: MIT                        # Optional
metadata:                           # Optional
  author: your-name
  version: "1.0.0"
  source: https://github.com/...
---
```

### Field Specifications

| Field | Required | Constraints |
|-------|----------|-------------|
| `name` | Yes | 1-64 chars, lowercase, alphanumeric + hyphens, no leading/trailing hyphen, no consecutive hyphens, matches directory name |
| `description` | Yes | 1-1024 chars, describes what AND when to use, include trigger terms |
| `license` | No | Short license name or file reference |
| `compatibility` | No | Max 500 chars, environment requirements |
| `metadata` | No | Key-value mapping for extra properties |
| `allowed-tools` | No | Space-delimited pre-approved tools (experimental) |

### Validation Rules

**Name:** lowercase, alphanumeric + hyphens, 1-64 chars, no leading/trailing/consecutive hyphens

**Description:** 1-1024 chars, explains what AND when to use, include trigger terms

## Directory Structure

### Minimal Structure
```
skill-name/
└── SKILL.md              # Required
```

### Recommended Structure
```
skill-name/
├── SKILL.md              # Overview & navigation (< 250 lines)
├── rules/                # Detailed rule files
│   ├── category1.md
│   └── category2.md
├── references/           # Technical references
│   ├── REFERENCE.md
│   └── api.md
├── scripts/              # Executable scripts
│   └── validate.sh
└── assets/               # Static resources
    └── diagram.png
```

## Progressive Disclosure

Structure skills for efficient context usage:

| Level | Content | Size | When Loaded |
|-------|---------|------|-------------|
| 1 | Metadata | ~100 tokens | Startup |
| 2 | Instructions | < 5000 tokens (~250 lines) | Skill activation |
| 3 | Resources | As needed | On-demand |

### SKILL.md Length Limits

- **Maximum:** 250 lines
- **Recommended:** 150-200 lines
- **Goal:** Comprehensive but concise

### Reference File Guidelines

- Load on demand via markdown links
- Keep files focused and small
- Avoid deeply nested references
- Maximum one level deep from SKILL.md

## File References

Use relative paths from skill root:

```markdown
See [detailed rules](rules/category.md) for specifics.

Run [validation script](scripts/validate.sh).

Check [API reference](references/api.md).
```

## Validation Checklist

Before using a skill, verify:

- [ ] `name` matches directory name
- [ ] `name` is 1-64 lowercase alphanumeric + hyphens
- [ ] No leading/trailing hyphens in name
- [ ] No consecutive hyphens in name
- [ ] `description` is 1-1024 characters
- [ ] `description` includes trigger terms
- [ ] SKILL.md is under 250 lines
- [ ] YAML frontmatter is valid
- [ ] Markdown body follows frontmatter
- [ ] Relative links work correctly

## Example: Complete Skill

```
rust-testing/
├── SKILL.md
├── rules/
│   ├── unit-tests.md
│   ├── integration-tests.md
│   └── mocking.md
└── references/
    └── cargo-commands.md
```

**SKILL.md:**
```markdown
---
name: rust-testing
description: >
  Rust testing patterns and best practices.
  Use when writing tests, debugging failures,
  or improving test coverage.
---

# Rust Testing

## Quick Reference

| Category | Rules |
|----------|-------|
| Unit Tests | [6 rules](rules/unit-tests.md) |
| Integration | [5 rules](rules/integration-tests.md) |
| Mocking | [4 rules](rules/mocking.md) |

## Basic Usage

```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}
```

See [cargo commands](references/cargo-commands.md) for details.
```

## See Also

- **[agentskills.io/specification](https://agentskills.io/specification)** - Official spec
- **[Structure Guide](structure.md)** - Detailed directory layout
- **[Templates](templates.md)** - Copy-paste templates
- **[Validation](validation.md)** - Validation commands
