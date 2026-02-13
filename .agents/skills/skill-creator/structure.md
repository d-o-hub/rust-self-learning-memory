# Skill Structure Guide

## Directory Format

```
.claude/skills/
└── skill-name/
    ├── SKILL.md           # Required - main entry point
    ├── additional-file.md # Optional - extended content
    └── templates/         # Optional - code/templates
```

## SKILL.md Requirements

Must have valid YAML frontmatter:

```markdown
---
name: skill-name
description: Clear description of what this skill does (max 1024 chars)
---

# Skill Title

[Content]
```

## File Organization

| Component | Required | Purpose |
|-----------|----------|---------|
| SKILL.md | Yes | Main skill entry point |
| *.md | No | Extended documentation |
| templates/ | No | Code templates |
| scripts/ | No | Helper scripts |
| resources/ | No | Reference materials |

## Subdirectory Patterns

**For large skills:**

```
skill-name/
├── SKILL.md              # Overview + links
├── guide.md              # Full guide
├── reference.md          # API/reference
├── templates/            # Code templates
└── examples/             # Example files
```

**For medium skills:**

```
skill-name/
├── SKILL.md              # Core content
└── advanced.md           # Extended topics
```

**For simple skills:**

```
skill-name/
└── SKILL.md              # Self-contained
```
