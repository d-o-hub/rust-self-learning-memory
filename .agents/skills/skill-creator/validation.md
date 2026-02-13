# Skill Validation Commands

## Structure Validation

```bash
# Check skill directory exists
test -d .claude/skills/skill-name && echo "✓ Directory exists"

# Check SKILL.md exists
test -f .claude/skills/skill-name/SKILL.md && echo "✓ SKILL.md exists"
```

## YAML Frontmatter Validation

```bash
# Check name field exists
grep "^name:" .claude/skills/skill-name/SKILL.md && echo "✓ Name field found"

# Check description field exists
grep "^description:" .claude/skills/skill-name/SKILL.md && echo "✓ Description field found"
```

## Name Format Validation

```bash
# Extract and validate name
SKILL_NAME=$(grep "^name:" .claude/skills/skill-name/SKILL.md | cut -d' ' -f2)

if [[ "$SKILL_NAME" =~ ^[a-z0-9-]+$ ]]; then
    echo "✓ Name format correct: $SKILL_NAME"
else
    echo "✗ Invalid name format: $SKILL_NAME"
    echo "  Must use lowercase letters, numbers, and hyphens only"
fi
```

## Description Validation

```bash
# Check description length
DESC_LENGTH=$(grep "^description:" .claude/skills/skill-name/SKILL.md | cut -d' ' -f2- | wc -c)

if [ "$DESC_LENGTH" -lt 1024 ]; then
    echo "✓ Description length OK ($DESC_LENGTH chars)"
else
    echo "✗ Description too long ($DESC_LENGTH chars, max 1024)"
fi

# Check description has action verb
DESCRIPTION=$(grep "^description:" .claude/skills/skill-name/SKILL.md | cut -d' ' -f2-)

if echo "$DESCRIPTION" | grep -qE "^(Debug|Implement|Test|Build|Run|Create|Manage|Optimize|Analyze|Deploy|Fix|Review|Validate)"; then
    echo "✓ Description starts with action verb"
else
    echo "⚠ Description may benefit from starting with action verb"
fi
```

## Complete Validation Script

```bash
#!/bin/bash
# validate-skill.sh

SKILL_DIR=".claude/skills/$1"

if [ -z "$1" ]; then
    echo "Usage: ./validate-skill.sh skill-name"
    exit 1
fi

echo "Validating skill: $1"
echo "================================"

# Check directory
if [ -d "$SKILL_DIR" ]; then
    echo "✓ Directory exists"
else
    echo "✗ Directory missing: $SKILL_DIR"
    exit 1
fi

# Check SKILL.md
if [ -f "$SKILL_DIR/SKILL.md" ]; then
    echo "✓ SKILL.md exists"
else
    echo "✗ SKILL.md missing"
    exit 1
fi

# Check YAML frontmatter
if head -n 5 "$SKILL_DIR/SKILL.md" | grep -q "^name:"; then
    echo "✓ YAML frontmatter has name"
else
    echo "✗ Missing name in YAML frontmatter"
fi

if head -n 5 "$SKILL_DIR/SKILL.md" | grep -q "^description:"; then
    echo "✓ YAML frontmatter has description"
else
    echo "✗ Missing description in YAML frontmatter"
fi

# Validate name format
SKILL_NAME=$(grep "^name:" "$SKILL_DIR/SKILL.md" | cut -d' ' -f2)
if [[ "$SKILL_NAME" =~ ^[a-z0-9-]+$ ]]; then
    echo "✓ Name format valid: $SKILL_NAME"
else
    echo "✗ Invalid name format: $SKILL_NAME"
fi

echo "================================"
echo "Validation complete"
```

## Pre-Commit Hook Integration

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Validating new skills..."

for skill_dir in .claude/skills/*/; do
    skill_name=$(basename "$skill_dir")
    ./scripts/validate-skill.sh "$skill_name" || exit 1
done

echo "✓ All skills validated"
```

## Markdown Formatting Validation

```bash
# Check for broken links
grep -rE "\[.*\]\(.*\)" .claude/skills/ | while read -r line; do
    link=$(echo "$line" | sed -n 's/.*\[\([^]]*\)\](\([^)]*\)).*/\1 \2/p')
    echo "Link: $link"
done

# Check heading hierarchy (H1 -> H2 -> H3, no skips)
awk '/^#+ / {level = length($1); print level": "$0}' .claude/skills/*/SKILL.md
```
