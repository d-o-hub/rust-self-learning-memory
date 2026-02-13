# Validation Workflow

## Phase 1: Plan Discovery

```bash
# Find all plan files
ls -1 plans/*.md

# Read plan index
cat plans/README.md
```

**Output**: List of all plan files to analyze

## Phase 2: Architecture Extraction

```bash
# Extract components/crates
grep -rh "crate\|component\|module" plans/ | sort -u

# Extract dependencies
grep -rh "depend\|flow\|import" plans/ | sort -u

# Extract performance targets
grep -rh "target\|metric\|<.*ms\|P[0-9]" plans/ | sort -u

# Extract security requirements
grep -rh "security\|threat\|attack" plans/ -i | sort -u

# Extract data models
grep -rh "struct\|enum\|type\|schema" plans/ | sort -u
```

**Output**: Structured list of architectural elements

## Phase 3: Codebase Analysis

```bash
# Analyze project structure
find . -name "Cargo.toml" -not -path "*/target/*"
tree -L 2 -I target

# Analyze dependencies
cargo tree --depth 1
cargo tree --duplicates

# Analyze code
rg "pub (async )?fn|pub struct|pub enum" --type rust
```

**Output**: Actual implementation state

## Phase 4: Compliance Validation

For each discovered architectural element:
1. Check if it exists in codebase
2. Validate it matches specification
3. Assess compliance level
4. Document findings

**Output**: Compliance matrix

## Phase 5: Gap Analysis

Identify:
- **Missing**: Planned but not implemented
- **Drift**: Implemented differently than planned
- **Extra**: Implemented but not documented

**Output**: Gap report with priorities

## Phase 6: Report Generation

Generate comprehensive report with:
- Executive summary
- Detailed findings per dimension
- Specific recommendations
- Action items with priorities

## Validation Commands

### Discovery
```bash
ls -1 plans/*.md | wc -l
head -20 plans/README.md
```

### Extraction
```bash
for file in plans/*.md; do
  echo "=== $file ==="
  grep -i "decision:\|requirement:\|target:\|constraint:" "$file"
done
```

### Analysis
```bash
echo "Planned crates:" && grep -rh "crate" plans/ | wc -l
echo "Actual crates:" && find . -name "Cargo.toml" | wc -l
```

### Validation
```bash
grep -r "requirement X" plans/
rg "implementation of X" --type rust
```

## Best Practices

1. **Start with README**: Read `plans/README.md` first
2. **Read all plans**: Don't skip any plan files
3. **Extract systematically**: Use consistent patterns
4. **Be specific**: Reference exact file and line numbers
5. **Assess impact**: Explain why violations matter
6. **Provide solutions**: Give clear remediation steps
7. **Track evolution**: Compare current vs previous validations

## Edge Cases

- **No plans folder**: Report validation cannot proceed
- **Empty plans**: Report insufficient documentation
- **Conflicting plans**: Flag conflicts for resolution
- **Outdated plans**: Note discrepancies and suggest updates
- **Ambiguous plans**: Request clarification
- **Multiple architectures**: Validate each separately
