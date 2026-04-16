---
name: yaml-validator
description: Validates YAML syntax and structure for .yml/.yaml files. Use when editing, creating, or debugging YAML configuration files, GitHub Actions workflows, or CI/CD configs.
---

# YAML Validator

Validate YAML files for syntax, structure, and schema compliance.

## Common Issues

| Issue | Fix |
|-------|-----|
| Mixed tabs/spaces | Convert to 2-space indentation |
| Missing colon | Add `:` after key |
| Special chars unquoted | Quote strings with `:`, `{`, `}`, `[`, `]` |
| Duplicate keys | Remove or merge |
| Boolean-looking strings | Quote `yes`, `no`, `on`, `off` |
| Trailing spaces | Remove with `sed -i 's/[[:space:]]*$//'` |

## Validation Commands

```bash
# Install yamllint
pip install yamllint

# Validate single file
yamllint file.yml

# Check all YAML files
yamllint .

# Python fallback
python3 -c "import yaml; yaml.safe_load(open('file.yml'))"
```

## Project .yamllint Config

Create `.yamllint.yml` in project root with 2-space indentation, 120 line length max, enable trailing-spaces and key-duplicates rules.

## GitHub Actions Checklist

- Valid `on:` trigger (push, pull_request, schedule)
- Correct `runs-on` value (ubuntu-latest, macos-latest)
- Proper `steps:` list structure
- Modern action versions (v3/v4, not v1/v2)

## Best Practices

### DO
- Use 2 spaces for indentation
- Quote strings with special characters
- Validate before committing
- Keep lines under 120 characters
- End files with newline

### DON'T
- Use tabs for indentation
- Leave trailing spaces
- Use duplicate keys
- Mix YAML 1.1 and 1.2 features