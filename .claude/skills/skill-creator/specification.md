# agentskills.io Specification

Complete specification for Agent Skills format from https://agentskills.io/specification

## Overview

Agent Skills are reusable knowledge modules that help AI agents perform specific tasks. They follow a standardized format for portability across different agent implementations.

## Directory Structure

### Minimal Structure

A skill requires at minimum a `SKILL.md` file:

```
skill-name/
└── SKILL.md          # Required
```

### Optional Directories

```
skill-name/
├── SKILL.md          # Required
├── scripts/          # Executable code
├── references/       # Additional documentation
└── assets/          # Static resources
```

## SKILL.md Format

The `SKILL.md` file must contain YAML frontmatter followed by Markdown content.

### Frontmatter (Required)

```yaml
---
name: skill-name
description: A description of what this skill does and when to use it.
---
```

### Frontmatter Fields

| Field | Required | Constraints |
|-------|----------|-------------|
| `name` | Yes | Max 64 chars. Lowercase letters, numbers, hyphens only. No leading/trailing hyphen. No consecutive hyphens. Must match directory name. |
| `description` | Yes | Max 1024 chars. Non-empty. Describes what the skill does AND when to use it. |
| `license` | No | License name or reference to bundled license file. |
| `compatibility` | No | Max 500 chars. Environment requirements (intended product, system packages, network access). |
| `metadata` | No | Arbitrary key-value mapping for additional metadata. |
| `allowed-tools` | No | Space-delimited list of pre-approved tools (experimental). |

### Name Field Rules

**Valid:**
- `pdf-processing`
- `data-analysis`
- `code-review`

**Invalid:**
- `PDF-Processing` (uppercase)
- `-pdf` (leading hyphen)
- `pdf--processing` (consecutive hyphens)
- `pdf-` (trailing hyphen)

### Description Field Best Practices

**Good:**
```yaml
description: Extracts text and tables from PDF files, fills PDF forms, and merges multiple PDFs. Use when working with PDF documents or when the user mentions PDFs, forms, or document extraction.
```

**Poor:**
```yaml
description: Helps with PDFs.
```

## Optional Directories

### scripts/

Contains executable code:
- Be self-contained or document dependencies
- Include helpful error messages
- Handle edge cases gracefully

Supported languages depend on agent (Python, Bash, JavaScript common).

### references/

Additional documentation loaded on demand:
- `REFERENCE.md` - Detailed technical reference
- `FORMS.md` - Form templates
- Domain-specific files

Keep files focused - agents load on demand.

### assets/

Static resources:
- Templates
- Images
- Data files
- Schemas

## Progressive Disclosure

Skills structured for efficient context usage:

| Level | Content | Size | Loaded |
|-------|---------|------|--------|
| 1 | Metadata | ~100 tokens | Startup |
| 2 | Instructions | < 5000 tokens (~250 lines) | Skill activation |
| 3 | Resources | As needed | On-demand |

Keep `SKILL.md` under 250 lines. Move detailed reference material to separate files.

## File References

Use relative paths from skill root:

```markdown
See [the reference guide](references/REFERENCE.md) for details.

Run the extraction script:
scripts/extract.py
```

Keep file references one level deep from `SKILL.md`. Avoid deeply nested chains.

## Validation

Use skills-ref reference library:

```bash
skills-ref validate ./my-skill
```

Checks:
- Valid YAML frontmatter
- Name follows conventions
- Description present and valid
- File structure correct

## Example Complete Skill

```
pdf-processor/
├── SKILL.md
├── scripts/
│   └── extract.py
└── references/
    └── SUPPORTED_FORMATS.md
```

**SKILL.md:**
```yaml
---
name: pdf-processor
description: Extracts text and tables from PDF files, fills PDF forms, and merges multiple PDFs. Use when working with PDF documents or when the user mentions PDFs, forms, or document extraction.
license: MIT
metadata:
  author: example-org
  version: "1.0"
---

# PDF Processor

Extract and manipulate PDF documents.

## Quick Start

Run [extract script](scripts/extract.py) to extract text:

```bash
python scripts/extract.py input.pdf
```

See [supported formats](references/SUPPORTED_FORMATS.md) for details.
```

## Compliance Checklist

- [ ] `name` is 1-64 chars, lowercase, alphanumeric + hyphens
- [ ] `name` has no leading/trailing hyphens
- [ ] `name` has no consecutive hyphens
- [ ] `name` matches directory name
- [ ] `description` is 1-1024 chars
- [ ] `description` explains what AND when to use
- [ ] SKILL.md is under 250 lines
- [ ] Valid YAML frontmatter syntax
- [ ] Markdown body follows frontmatter
- [ ] File references use relative paths
- [ ] References are one level deep max
