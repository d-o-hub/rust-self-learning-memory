# YAML Validation Guide

This document outlines the YAML validation strategy for preventing syntax errors in GitHub workflows and configuration files.

## Validation Layers

### 1. Pre-Commit Hook (Local)
**File**: `.claude/hooks/pre-commit-security.sh`

The pre-commit hook now includes YAML validation:
- Checks all staged `.yml` and `.yaml` files
- Uses `yamllint` with 2-space indentation enforcement
- Fails the commit if YAML syntax errors are detected

### 2. Claude Code Hook (Editor-Level)
**File**: `.claude/hooks/yaml-validation.sh`

Validates YAML files when editing:
- Triggered on `Edit|Write` operations for `.yml` and `.yaml` files
- Provides immediate feedback in the editor
- Prevents saving malformed YAML

**To enable**: Add to `.claude/settings.json`:
```json
{
  "name": "Validate YAML Syntax",
  "matcher": "Edit|MultiEdit|Write",
  "hooks": [
    {
      "type": "command",
      "command": "bash .claude/hooks/yaml-validation.sh",
      "timeout": 10
    }
  ]
}
```

### 3. CI Workflow (GitHub Actions)
**File**: `.github/workflows/yaml-lint.yml`

Automated validation on push/PR:
- **yamllint**: Checks YAML syntax and style
- **actionlint**: Validates GitHub Actions workflow semantics
- Runs only when YAML files are modified (path filters)

## Setup Instructions

### Install yamllint (Python)
```bash
# Via pip
pip install yamllint

# Via pipx (isolated)
pipx install yamllint

# Via homebrew (macOS)
brew install yamllint
```

### Install actionlint (GitHub Actions)
```bash
# Via homebrew (macOS/Linux)
brew install actionlint

# Via go
go install github.com/rhysd/actionlint/cmd/actionlint@latest

# Download binary
# See: https://github.com/rhysd/actionlint/releases
```

### Configuration File
**File**: `.yamllint.yml`

Project-wide yamllint configuration:
- **Indentation**: 2 spaces (GitHub Actions standard)
- **Line length**: 120 characters max
- **Truthy values**: Allows `yes/no/on/off`
- **Ignores**: `node_modules/`, `target/`, `.git/`

## YAML Best Practices

### 1. Use Consistent Indentation
✅ **Good** (2 spaces):
```yaml
jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
```

❌ **Bad** (mixed tabs/spaces):
```yaml
jobs:
    test:
	    name: Test  # Tab character here
        runs-on: ubuntu-latest
```

### 2. Quote Special Characters
✅ **Good**:
```yaml
env:
  MESSAGE: "It's a great day!"
  VERSION: "1.0.0"
```

❌ **Bad**:
```yaml
env:
  MESSAGE: It's a great day!  # Unescaped apostrophe
  VERSION: 1.0.0              # Interpreted as number
```

### 3. Validate Before Committing
```bash
# Validate all YAML files
yamllint .github/

# Validate specific file
yamllint .github/workflows/ci.yml

# Validate with custom config
yamllint -d "{extends: default, rules: {line-length: disable}}" file.yml
```

### 4. Test Workflows Locally (act)
```bash
# Install act (GitHub Actions local runner)
brew install act  # macOS
# OR
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Test workflow
act -l                           # List workflows
act push                         # Run push event workflows
act -j test                      # Run specific job
act --dry-run                    # Dry run (no execution)
```

## Common YAML Errors

### 1. Indentation Errors
```yaml
# ERROR: Inconsistent indentation
jobs:
  test:
   steps:  # Only 1 space instead of 2
     - run: echo "hello"
```

### 2. Missing Colons
```yaml
# ERROR: Missing colon after key
env
  VAR: value
```

### 3. Unescaped Special Characters
```yaml
# ERROR: Unescaped @ in string
email: user@example.com  # May be interpreted as reference
# FIX:
email: "user@example.com"
```

### 4. Tab Characters
```yaml
# ERROR: Tab character instead of spaces
jobs:
→test:  # Tab here
  runs-on: ubuntu-latest
```

### 5. Trailing Commas (YAML is not JSON)
```yaml
# ERROR: Trailing comma
matrix:
  os: [ubuntu, macos, windows,]  # Comma after 'windows'
```

## Troubleshooting

### yamllint Warnings
```bash
# See detailed error
yamllint -f parsable .github/workflows/ci.yml

# Show only errors (no warnings)
yamllint --strict .github/workflows/
```

### actionlint Errors
```bash
# Validate all workflows
actionlint

# Check specific file
actionlint .github/workflows/ci.yml

# Verbose output
actionlint -verbose
```

### Disable Specific Rules
In `.yamllint.yml`:
```yaml
rules:
  line-length: disable          # Disable line length check
  comments: disable             # Disable comment spacing
  indentation:
    spaces: 2
    indent-sequences: false     # Don't check list indentation
```

## CI Integration

### Fail on YAML Errors
The `yaml-lint.yml` workflow will:
1. Run on all pushes/PRs that modify YAML files
2. Validate syntax with `yamllint`
3. Validate GitHub Actions semantics with `actionlint`
4. **Fail the build** if any errors are found

### Path Filters
Only runs when YAML files are modified:
```yaml
on:
  push:
    paths:
      - '**.yml'
      - '**.yaml'
      - '.github/workflows/**'
```

## References

- [yamllint documentation](https://yamllint.readthedocs.io/)
- [actionlint documentation](https://github.com/rhysd/actionlint)
- [YAML specification](https://yaml.org/spec/)
- [GitHub Actions syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [act - Local GitHub Actions](https://github.com/nektos/act)

## Summary

**Prevention is better than debugging!**

1. ✅ Install `yamllint` and `actionlint` locally
2. ✅ Enable the Claude Code YAML validation hook
3. ✅ Run `yamllint` before committing
4. ✅ Let CI catch any missed errors
5. ✅ Test workflows with `act` before pushing
